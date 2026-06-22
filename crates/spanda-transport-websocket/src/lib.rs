//! WebSocket transport backend extracted from Spanda core for lean-core package architecture.
//!
pub mod adapter;

#[cfg(feature = "live")]
mod live;

pub use adapter::{WebsocketTransportAdapter, WebsocketTransportAdapterLive};

/// Live WebSocket bridge handle over a tungstenite session.
#[derive(Debug, Default)]
pub struct LiveWebsocketBridge {
    #[cfg(feature = "live")]
    inner: Option<live::LiveWebsocketBridge>,
}

impl LiveWebsocketBridge {
    pub fn connect(broker_url: &str) -> Result<Self, String> {
        #[cfg(feature = "live")]
        {
            return Ok(Self {
                inner: Some(live::LiveWebsocketBridge::connect(broker_url)?),
            });
        }
        #[cfg(not(feature = "live"))]
        {
            let _ = broker_url;
            Err(
                "live WebSocket support not enabled (build spanda-transport-websocket with --features live)"
                    .into(),
            )
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
