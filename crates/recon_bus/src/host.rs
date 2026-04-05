//! Wasmtime host bindings for the `recon:event-bus/bus` WIT interface.
//!
//! `subscribe` is `async func` in the WIT because wasmtime's bindgen only
//! provides store access (via `Accessor`) for async functions. Creating a
//! `StreamReader` requires store access, which sync `func_wrap` doesn't
//! expose to the `Host` trait.
//!
//! See wasmtime-internal-wit-bindgen `config.rs` lines 77-88: only
//! `AsyncFreestanding` gets the `STORE` flag that triggers
//! `func_wrap_concurrent` with `Accessor<T>`.
//! Ref: <https://github.com/bytecodealliance/wasmtime/blob/v43.0.0/crates/wit-bindgen/src/config.rs#L77-L88>

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use wasmtime::{
    StoreContextMut,
    component::{Destination, HasData, StreamProducer, StreamReader, StreamResult, VecBuffer},
};

use crate::Subscription;

wasmtime::component::bindgen!({
    path: "wit",
    world: "bus-world",
});

pub use recon::event_bus::bus::{EventMessage, Host, HostWithStore};

/// Marker type for the event bus host capability.
///
/// Used as the `D` parameter in `add_to_linker::<T, EventBus>`.
/// Implement [`EventBusView`] on your store data type to provide access.
pub struct EventBus;

impl HasData for EventBus {
    type Data<'a> = EventBusCtx<'a>;
}

/// View into event bus state, projected from the store.
pub struct EventBusCtx<'a> {
    pub bus: &'a crate::Bus,
}

/// Implement this on your store data type to provide event bus access.
pub trait EventBusView: Send {
    fn event_bus(&mut self) -> EventBusCtx<'_>;
}

impl Host for EventBusCtx<'_> {
    fn publish(&mut self, topic: String, payload: String) -> Result<u64, String> {
        match self.bus.publish(&*topic, payload) {
            Ok(count) => Ok(count as u64),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl HostWithStore for EventBus {
    async fn subscribe<S: Send>(
        accessor: &wasmtime::component::Accessor<S, Self>,
        filter: String,
    ) -> Result<StreamReader<EventMessage>, String> {
        accessor.with(|mut access| {
            let bus = access.get().bus;
            let sub = bus.subscribe(&*filter).map_err(|e| e.to_string())?;
            subscribe_stream(&mut access, sub).map_err(|e| e.to_string())
        })
    }
}

/// Create a `StreamReader<EventMessage>` from a `Subscription`.
pub fn subscribe_stream<S: wasmtime::AsContextMut>(
    store: &mut S,
    sub: Subscription,
) -> wasmtime::Result<StreamReader<EventMessage>> {
    StreamReader::new(store, SubscriptionProducer::new(sub))
}

type WatchReceiver = tokio::sync::watch::Receiver<Option<crate::Envelope>>;
type ChangedFut =
    Pin<Box<dyn std::future::Future<Output = (WatchReceiver, Option<crate::Envelope>)> + Send>>;

struct SubscriptionProducer {
    _sub: Subscription,
    fut: ChangedFut,
}

fn make_changed_fut(mut receiver: WatchReceiver) -> ChangedFut {
    Box::pin(async move {
        let result = match receiver.changed().await {
            Ok(()) => receiver.borrow_and_update().clone(),
            Err(_) => None,
        };
        (receiver, result)
    })
}

impl SubscriptionProducer {
    fn new(sub: Subscription) -> Self {
        let fut = make_changed_fut(sub.clone_receiver());
        Self { _sub: sub, fut }
    }
}

impl<D> StreamProducer<D> for SubscriptionProducer {
    type Item = EventMessage;
    type Buffer = VecBuffer<EventMessage>;

    fn poll_produce<'a>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        _store: StoreContextMut<'a, D>,
        mut dst: Destination<'a, Self::Item, Self::Buffer>,
        finish: bool,
    ) -> Poll<wasmtime::Result<StreamResult>> {
        if finish {
            return Poll::Ready(Ok(StreamResult::Cancelled));
        }

        let this = self.get_mut();

        match this.fut.as_mut().poll(cx) {
            Poll::Ready((receiver, Some(envelope))) => {
                let msg = EventMessage {
                    topic: envelope.topic.to_string(),
                    payload: envelope.payload.to_string(),
                };
                dst.set_buffer(vec![msg].into());
                this.fut = make_changed_fut(receiver);
                Poll::Ready(Ok(StreamResult::Completed))
            }
            Poll::Ready((_, None)) => Poll::Ready(Ok(StreamResult::Dropped)),
            Poll::Pending => Poll::Pending,
        }
    }
}
