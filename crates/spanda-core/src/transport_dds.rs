//! Compatibility shim: DDS live transport moved to `spanda-transport-dds`.
//!
use crate::runtime::RuntimeValue;

/// Live DDS bridge with Spanda runtime value conversion (compatibility shim).
#[derive(Debug, Default)]
pub struct LiveDdsBridge {
    inner: spanda_transport_dds::LiveDdsBridge,
}

impl LiveDdsBridge {
    pub fn connect(domain_id: u32) -> Result<Self, String> {
        Ok(Self {
            inner: spanda_transport_dds::LiveDdsBridge::connect(domain_id)?,
        })
    }

    pub fn publish(&self, topic: &str, payload: &str) -> Result<(), String> {
        self.inner.publish(topic, payload)
    }

    pub fn subscribe(&self, topic: &str) -> Result<(), String> {
        self.inner.subscribe(topic)
    }

    pub fn receive(&self, topic: &str) -> Option<RuntimeValue> {
        self.inner
            .receive(topic)
            .map(|value| RuntimeValue::String { value })
    }
}
