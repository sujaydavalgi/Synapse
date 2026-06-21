//! Optional live MQTT broker integration via rumqttc.

use crate::runtime::RuntimeValue;

#[cfg(feature = "live-mqtt")]
mod live {
    use super::*;
    use rumqttc::{Client, Event, Incoming, MqttOptions, QoS};
    use std::collections::{HashMap, VecDeque};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    pub struct LiveMqttBridge {
        client: Client,
        inbound: Arc<Mutex<HashMap<String, VecDeque<RuntimeValue>>>>,
    }

    impl LiveMqttBridge {
        pub fn connect(broker_url: &str, client_id: &str) -> Result<Self, String> {
            let (host, port) = parse_broker_url(broker_url)?;
            let mut options = MqttOptions::new(client_id, host, port);
            options.set_keep_alive(Duration::from_secs(5));
            let (client, mut connection) = Client::new(options, 10);
            let inbound: Arc<Mutex<HashMap<String, VecDeque<RuntimeValue>>>> =
                Arc::new(Mutex::new(HashMap::new()));
            let inbound_poll = Arc::clone(&inbound);
            thread::spawn(move || loop {
                match connection.iter().next() {
                    Some(Ok(Event::Incoming(Incoming::Publish(packet)))) => {
                        let payload = String::from_utf8_lossy(&packet.payload).to_string();
                        if let Ok(mut map) = inbound_poll.lock() {
                            map.entry(packet.topic)
                                .or_default()
                                .push_back(RuntimeValue::String { value: payload });
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => {
                        eprintln!("live mqtt connection error: {e}");
                        break;
                    }
                    None => break,
                }
            });
            Ok(Self { client, inbound })
        }

        pub fn publish(&self, topic: &str, payload: &str) -> Result<(), String> {
            self.client
                .publish(topic, QoS::AtMostOnce, false, payload.as_bytes())
                .map_err(|e| format!("mqtt publish failed: {e}"))
        }

        pub fn subscribe(&self, topic: &str) -> Result<(), String> {
            self.client
                .subscribe(topic, QoS::AtMostOnce)
                .map_err(|e| format!("mqtt subscribe failed: {e}"))
        }

        pub fn receive(&self, topic: &str) -> Option<RuntimeValue> {
            let mut map = self.inbound.lock().ok()?;
            map.get_mut(topic).and_then(|q| q.pop_front())
        }
    }

    fn parse_broker_url(url: &str) -> Result<(String, u16), String> {
        let stripped = url
            .trim_start_matches("mqtts://")
            .trim_start_matches("mqtt://")
            .trim_start_matches("ssl://");
        let (host, port) = stripped
            .split_once(':')
            .map(|(h, p)| (h.to_string(), p.parse().unwrap_or(1883)))
            .unwrap_or((stripped.to_string(), 1883));
        Ok((host, port))
    }
}

/// Live MQTT bridge handle; inactive unless built with `live-mqtt` and connected.
#[derive(Debug, Default)]
pub struct LiveMqttBridge {
    #[cfg(feature = "live-mqtt")]
    inner: Option<live::LiveMqttBridge>,
}

impl LiveMqttBridge {
    pub fn connect(broker_url: &str, client_id: &str) -> Result<Self, String> {
        #[cfg(feature = "live-mqtt")]
        {
            return Ok(Self {
                inner: Some(live::LiveMqttBridge::connect(broker_url, client_id)?),
            });
        }
        #[cfg(not(feature = "live-mqtt"))]
        {
            let _ = (broker_url, client_id);
            Err("live MQTT support not enabled (build with --features live-mqtt)".into())
        }
    }

    pub fn publish(&self, topic: &str, payload: &str) -> Result<(), String> {
        #[cfg(feature = "live-mqtt")]
        if let Some(inner) = &self.inner {
            return inner.publish(topic, payload);
        }
        let _ = (topic, payload);
        Ok(())
    }

    pub fn subscribe(&self, topic: &str) -> Result<(), String> {
        #[cfg(feature = "live-mqtt")]
        if let Some(inner) = &self.inner {
            return inner.subscribe(topic);
        }
        let _ = topic;
        Ok(())
    }

    pub fn receive(&self, topic: &str) -> Option<RuntimeValue> {
        #[cfg(feature = "live-mqtt")]
        if let Some(inner) = &self.inner {
            return inner.receive(topic);
        }
        let _ = topic;
        None
    }
}
