//! In-process async topic-based pub/sub event bus with wildcard matching.

mod envelope;
mod topic;
mod trie;

use std::sync::{
    Arc, RwLock,
    atomic::{AtomicU64, Ordering},
};

use dashmap::DashMap;
pub use envelope::Envelope;
use tokio::sync::watch;
pub use topic::{Topic, TopicError, topic_matches};
use trie::{SubscriberId, TopicTrie};

#[derive(Debug)]
pub enum BusError {
    Topic(TopicError),
    #[cfg(feature = "serde")]
    Serialize(serde_json::Error),
}

impl std::fmt::Display for BusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Topic(e) => write!(f, "{e}"),
            #[cfg(feature = "serde")]
            Self::Serialize(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for BusError {}

impl From<TopicError> for BusError {
    fn from(e: TopicError) -> Self {
        Self::Topic(e)
    }
}

#[cfg(feature = "serde")]
impl From<serde_json::Error> for BusError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialize(e)
    }
}

struct Subscriber {
    sender: watch::Sender<Option<Envelope>>,
}

struct BusInner {
    trie: RwLock<TopicTrie>,
    subscribers: DashMap<SubscriberId, Subscriber>,
    next_id: AtomicU64,
}

/// The event bus. Clone to share across threads.
#[derive(Clone)]
pub struct Bus {
    inner: Arc<BusInner>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(BusInner {
                trie: RwLock::new(TopicTrie::new()),
                subscribers: DashMap::new(),
                next_id: AtomicU64::new(0),
            }),
        }
    }

    /// Publish a string payload to a topic.
    pub fn publish(
        &self,
        topic: impl TryInto<Topic, Error = TopicError>,
        payload: impl Into<String>,
    ) -> Result<usize, BusError> {
        let topic = topic.try_into()?;
        let payload: Arc<str> = Arc::from(payload.into());
        let envelope = Envelope::new(topic.as_raw().clone(), payload);
        Ok(self.deliver(&topic, envelope))
    }

    /// Publish a serializable value as JSON to a topic.
    #[cfg(feature = "serde")]
    pub fn publish_serde(
        &self,
        topic: impl TryInto<Topic, Error = TopicError>,
        value: &(impl ::serde::Serialize + ?Sized),
    ) -> Result<usize, BusError> {
        let topic = topic.try_into()?;
        let payload: Arc<str> = Arc::from(serde_json::to_string(value)?);
        let envelope = Envelope::new(topic.as_raw().clone(), payload);
        Ok(self.deliver(&topic, envelope))
    }

    fn deliver(&self, topic: &Topic, envelope: Envelope) -> usize {
        let matching = self
            .inner
            .trie
            .read()
            .expect("trie lock poisoned")
            .matching(topic);

        let mut delivered = 0;
        matching.iter().for_each(|id| {
            if let Some(sub) = self.inner.subscribers.get(id) {
                let _ = sub.sender.send(Some(envelope.clone()));
                delivered += 1;
            }
        });

        delivered
    }

    /// Subscribe to a topic pattern (may contain `*` and `**` wildcards).
    pub fn subscribe(
        &self,
        filter: impl TryInto<Topic, Error = TopicError>,
    ) -> Result<Subscription, BusError> {
        let filter = filter.try_into()?;
        let id = SubscriberId(self.inner.next_id.fetch_add(1, Ordering::Relaxed));
        let (tx, rx) = watch::channel(None);

        self.inner.subscribers.insert(id, Subscriber { sender: tx });

        self.inner
            .trie
            .write()
            .expect("trie lock poisoned")
            .insert(&filter, id);

        Ok(Subscription {
            id,
            filter,
            bus: Arc::clone(&self.inner),
            receiver: rx,
        })
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

/// A subscription handle. Dropping it unsubscribes automatically.
pub struct Subscription {
    id: SubscriberId,
    filter: Topic,
    bus: Arc<BusInner>,
    receiver: watch::Receiver<Option<Envelope>>,
}

impl Subscription {
    /// Wait for the next value change and return the envelope.
    ///
    /// Returns `None` if the bus is dropped.
    pub async fn recv(&mut self) -> Option<Envelope> {
        self.receiver.changed().await.ok()?;
        self.receiver.borrow_and_update().clone()
    }

    /// Read the current latest value without waiting.
    pub fn get(&self) -> Option<Envelope> {
        self.receiver.borrow().clone()
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.bus.subscribers.remove(&self.id);
        if let Ok(mut trie) = self.bus.trie.write() {
            trie.remove(&self.filter, self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publish_and_receive() {
        let bus = Bus::new();
        let mut sub = bus.subscribe("test/topic").unwrap();

        let delivered = bus.publish("test/topic", "hello").unwrap();
        assert_eq!(delivered, 1);

        let envelope = sub.recv().await.unwrap();
        assert_eq!(&*envelope.topic, "test/topic");
    }

    #[tokio::test]
    async fn wildcard_subscription() {
        let bus = Bus::new();
        let mut sub = bus.subscribe("game/*/status").unwrap();

        bus.publish("game/valorant/status", "online").unwrap();
        assert!(sub.recv().await.is_some());
    }

    #[tokio::test]
    async fn latest_value_semantics() {
        let bus = Bus::new();
        let mut sub = bus.subscribe("counter").unwrap();

        bus.publish("counter", "1").unwrap();
        bus.publish("counter", "2").unwrap();
        bus.publish("counter", "3").unwrap();

        let envelope = sub.recv().await.unwrap();
        assert!(envelope.payload.contains('3'));
    }

    #[tokio::test]
    async fn get_current_value() {
        let bus = Bus::new();
        let sub = bus.subscribe("data").unwrap();
        assert!(sub.get().is_none());

        bus.publish("data", "value").unwrap();
        assert!(sub.get().is_some());
    }

    #[tokio::test]
    async fn multi_wildcard_subscription() {
        let bus = Bus::new();
        let mut sub = bus.subscribe("game/**").unwrap();

        bus.publish("game/valorant/player/health", "100").unwrap();
        assert!(sub.recv().await.is_some());
    }

    #[tokio::test]
    async fn unsubscribe_on_drop() {
        let bus = Bus::new();
        let sub = bus.subscribe("ephemeral").unwrap();
        drop(sub);

        let delivered = bus.publish("ephemeral", "nobody home").unwrap();
        assert_eq!(delivered, 0);
    }

    #[tokio::test]
    async fn multiple_subscribers() {
        let bus = Bus::new();
        let mut sub1 = bus.subscribe("shared").unwrap();
        let mut sub2 = bus.subscribe("shared").unwrap();

        let delivered = bus.publish("shared", "broadcast").unwrap();
        assert_eq!(delivered, 2);

        assert!(sub1.recv().await.is_some());
        assert!(sub2.recv().await.is_some());
    }

    #[tokio::test]
    async fn no_match_no_delivery() {
        let bus = Bus::new();
        let _sub = bus.subscribe("a/b").unwrap();

        let delivered = bus.publish("x/y", "miss").unwrap();
        assert_eq!(delivered, 0);
    }

    #[tokio::test]
    async fn invalid_topic_returns_error() {
        let bus = Bus::new();
        assert!(bus.publish("", "test").is_err());
        assert!(bus.subscribe("").is_err());
    }

    #[cfg(feature = "serde")]
    #[tokio::test]
    async fn serde_struct_roundtrip() {
        #[derive(::serde::Serialize, ::serde::Deserialize, Debug, PartialEq)]
        struct GameState {
            health: u32,
            score: u64,
        }

        let bus = Bus::new();
        let mut sub = bus.subscribe("game/state").unwrap();

        let state = GameState {
            health: 100,
            score: 42000,
        };
        bus.publish_serde("game/state", &state).unwrap();

        let envelope = sub.recv().await.unwrap();
        let received: GameState = envelope.deserialize().unwrap();
        assert_eq!(received, state);
    }

    #[cfg(feature = "serde")]
    #[tokio::test]
    async fn serde_string_roundtrip() {
        let bus = Bus::new();
        let mut sub = bus.subscribe("msg").unwrap();

        bus.publish_serde("msg", "hello").unwrap();

        let envelope = sub.recv().await.unwrap();
        let received: String = envelope.deserialize().unwrap();
        assert_eq!(received, "hello");
    }
}
