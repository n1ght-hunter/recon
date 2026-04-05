//! Guest-side bindings for the `recon:event-bus/bus` WIT interface.

wit_bindgen::generate!({
    path: "wit",
    world: "bus-world",
});

pub use recon::event_bus::bus::{EventMessage, publish, subscribe};
