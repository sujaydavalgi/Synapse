//! DDS transport backend extracted from Spanda core for lean-core package architecture.
//!
pub mod adapter;

#[cfg(feature = "live")]
mod live;

pub use adapter::{DdsTransportAdapter, DdsTransportAdapterLive};

/// Live DDS bridge handle over UDP multicast.
#[derive(Debug, Default)]
pub struct LiveDdsBridge {
    #[cfg(feature = "live")]
    inner: Option<live::LiveDdsBridge>,
}

impl LiveDdsBridge {
    pub fn connect(domain_id: u32) -> Result<Self, String> {
        #[cfg(feature = "live")]
        {
            return Ok(Self {
                inner: Some(live::LiveDdsBridge::connect(domain_id)?),
            });
        }
        #[cfg(not(feature = "live"))]
        {
            let _ = domain_id;
            Err("live DDS support not enabled (build spanda-transport-dds with --features live)".into())
        }
    }

    pub fn publish(&self, topic: &str, payload: &str) -> Result<(), String> {
        #[cfg(feature = "live")]
        if let Some(inner) = &self.inner {
            return inner.publish(topic, payload);
        }
        let _ = (topic, payload);
        Ok(())
    }

    pub fn subscribe(&self, topic: &str) -> Result<(), String> {
        #[cfg(feature = "live")]
        if let Some(inner) = &self.inner {
            return inner.subscribe(topic);
        }
        let _ = topic;
        Ok(())
    }

    pub fn receive(&self, topic: &str) -> Option<String> {
        #[cfg(feature = "live")]
        if let Some(inner) = &self.inner {
            return inner.receive(topic);
        }
        let _ = topic;
        None
    }
}
