//! Message envelope type carried through the bus.

use std::{sync::Arc, time::Instant};

/// A message on the bus.
///
/// Both `topic` and `payload` are `Arc<str>`, so cloning an envelope
/// to fan out to multiple subscribers costs only two atomic increments.
#[derive(Debug, Clone)]
pub struct Envelope {
    pub topic: Arc<str>,
    pub payload: Arc<str>,
    pub timestamp: Instant,
}

impl Envelope {
    pub fn new(topic: Arc<str>, payload: Arc<str>) -> Self {
        Self {
            topic,
            payload,
            timestamp: Instant::now(),
        }
    }

    /// Deserialize the JSON payload into `T`.
    #[cfg(feature = "serde")]
    pub fn deserialize<T: ::serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.payload)
    }
}
