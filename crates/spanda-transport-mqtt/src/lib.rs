//! MQTT transport backend extracted from Spanda core for lean-core package architecture.
//!
//! Used by `spanda-mqtt` and wired through `spanda-core` compatibility shims until
//! all callers migrate to package-scoped provider registration.
//!
mod python_bridge;

#[cfg(feature = "live")]
mod live;

/// Live MQTT bridge handle; inactive unless built with the `live` feature.
#[derive(Debug, Default)]
pub struct LiveMqttBridge {
    #[cfg(feature = "live")]
    inner: Option<live::LiveMqttBridge>,
}

impl LiveMqttBridge {
    pub fn connect(broker_url: &str, client_id: &str) -> Result<Self, String> {
        // Connect to a live MQTT broker when the live feature is enabled.
        //
        // Parameters:
        // - `broker_url` — broker URL (`mqtt://host:port`)
        // - `client_id` — MQTT client identifier
        //
        // Returns:
        // Connected bridge, or an error when live support is disabled.
        //
        // Options:
        // None.
        //
        // Example:
        // let bridge = LiveMqttBridge::connect("mqtt://localhost:1883", "spanda")?;

        #[cfg(feature = "live")]
        {
            return Ok(Self {
                inner: Some(live::LiveMqttBridge::connect(broker_url, client_id)?),
            });
        }
        #[cfg(not(feature = "live"))]
        {
            let _ = (broker_url, client_id);
            Err("live MQTT support not enabled (build spanda-transport-mqtt with --features live)".into())
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

pub use python_bridge::{mqtt_live_enabled, try_mqtt_publish};
