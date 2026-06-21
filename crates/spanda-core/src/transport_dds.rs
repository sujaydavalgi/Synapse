//! Optional live DDS domain integration via UDP multicast.

use crate::runtime::RuntimeValue;

#[cfg(feature = "live-dds")]
use std::collections::{HashMap, VecDeque};
#[cfg(feature = "live-dds")]
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
#[cfg(feature = "live-dds")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "live-dds")]
use std::thread;

#[cfg(feature = "live-dds")]
#[derive(serde::Serialize, serde::Deserialize)]
struct DdsWireEnvelope {
    topic: String,
    payload: String,
}

#[cfg(feature = "live-dds")]
struct LiveDdsInner {
    socket: UdpSocket,
    group: Ipv4Addr,
    port: u16,
    inbound: Arc<Mutex<HashMap<String, VecDeque<RuntimeValue>>>>,
}

#[cfg(feature = "live-dds")]
impl LiveDdsInner {
    fn connect(domain_id: u32) -> Result<Self, String> {
        let port = 7400_u16.saturating_add(domain_id as u16);
        let octet = domain_id.min(255) as u8;
        let group = Ipv4Addr::new(239, 255, 0, octet);
        let socket = UdpSocket::bind(format!("0.0.0.0:{port}"))
            .map_err(|e| format!("dds bind failed: {e}"))?;
        socket
            .join_multicast_v4(&group, &Ipv4Addr::UNSPECIFIED)
            .map_err(|e| format!("dds join multicast failed: {e}"))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| format!("dds nonblocking failed: {e}"))?;
        let inbound: Arc<Mutex<HashMap<String, VecDeque<RuntimeValue>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let inbound_poll = Arc::clone(&inbound);
        let reader = socket
            .try_clone()
            .map_err(|e| format!("dds clone socket: {e}"))?;
        thread::spawn(move || {
            let mut buf = [0_u8; 65507];
            loop {
                match reader.recv_from(&mut buf) {
                    Ok((len, _src)) => {
                        if let Ok(text) = std::str::from_utf8(&buf[..len]) {
                            if let Ok(frame) = serde_json::from_str::<DdsWireEnvelope>(text) {
                                if let Ok(mut map) = inbound_poll.lock() {
                                    map.entry(frame.topic).or_default().push_back(
                                        RuntimeValue::String {
                                            value: frame.payload,
                                        },
                                    );
                                }
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            socket,
            group,
            port,
            inbound,
        })
    }

    fn publish(&self, topic: &str, payload: &str) -> Result<(), String> {
        let envelope = DdsWireEnvelope {
            topic: topic.to_string(),
            payload: payload.to_string(),
        };
        let bytes =
            serde_json::to_vec(&envelope).map_err(|e| format!("dds serialize failed: {e}"))?;
        let dest = SocketAddr::from((self.group, self.port));
        self.socket
            .send_to(&bytes, dest)
            .map_err(|e| format!("dds publish failed: {e}"))?;
        Ok(())
    }

    fn receive(&self, topic: &str) -> Option<RuntimeValue> {
        let mut map = self.inbound.lock().ok()?;
        map.get_mut(topic).and_then(|q| q.pop_front())
    }
}

/// Live DDS bridge handle over UDP multicast.
#[derive(Debug, Default)]
pub struct LiveDdsBridge {
    #[cfg(feature = "live-dds")]
    inner: Option<LiveDdsInner>,
}

impl LiveDdsBridge {
    pub fn connect(domain_id: u32) -> Result<Self, String> {
        #[cfg(feature = "live-dds")]
        {
            return Ok(Self {
                inner: Some(LiveDdsInner::connect(domain_id)?),
            });
        }
        #[cfg(not(feature = "live-dds"))]
        {
            let _ = domain_id;
            Err("live DDS support not enabled (build with --features live-dds)".into())
        }
    }

    pub fn publish(&self, topic: &str, payload: &str) -> Result<(), String> {
        #[cfg(feature = "live-dds")]
        if let Some(inner) = &self.inner {
            return inner.publish(topic, payload);
        }
        let _ = (topic, payload);
        Ok(())
    }

    pub fn subscribe(&self, _topic: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn receive(&self, topic: &str) -> Option<RuntimeValue> {
        #[cfg(feature = "live-dds")]
        if let Some(inner) = &self.inner {
            return inner.receive(topic);
        }
        let _ = topic;
        None
    }
}
