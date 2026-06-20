//! Cooperative channels, spawn queue, and select for concurrent Spanda tasks.

use crate::error::SpandaError;
use crate::runtime::{RuntimeError, RuntimeValue};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

pub type ChannelHandle = Rc<RefCell<VecDeque<RuntimeValue>>>;

#[derive(Debug, Clone)]
pub struct ConcurrencyRuntime {
    next_channel_id: u64,
    channels: HashMap<u64, ChannelHandle>,
    spawn_queue: Vec<SpawnJob>,
}

#[derive(Debug, Clone)]
pub struct SpawnJob {
    pub func_name: String,
    pub args: Vec<RuntimeValue>,
}

impl Default for ConcurrencyRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ConcurrencyRuntime {
    pub fn new() -> Self {
        Self {
            next_channel_id: 1,
            channels: HashMap::new(),
            spawn_queue: Vec::new(),
        }
    }

    pub fn create_channel(&mut self) -> RuntimeValue {
        let id = self.next_channel_id;
        self.next_channel_id += 1;
        let handle = Rc::new(RefCell::new(VecDeque::new()));
        self.channels.insert(id, handle);
        RuntimeValue::Channel { id }
    }

    pub fn send(
        &self,
        channel: &RuntimeValue,
        value: RuntimeValue,
        line: u32,
    ) -> Result<(), SpandaError> {
        let RuntimeValue::Channel { id } = channel else {
            return Err(RuntimeError::new("send requires a channel", line).into_spanda());
        };
        let handle = self.channels.get(id).ok_or_else(|| {
            RuntimeError::new(format!("Unknown channel id {id}"), line).into_spanda()
        })?;
        handle.borrow_mut().push_back(value);
        Ok(())
    }

    pub fn try_recv(
        &self,
        channel: &RuntimeValue,
        line: u32,
    ) -> Result<Option<RuntimeValue>, SpandaError> {
        let RuntimeValue::Channel { id } = channel else {
            return Err(RuntimeError::new("recv requires a channel", line).into_spanda());
        };
        let handle = self.channels.get(id).ok_or_else(|| {
            RuntimeError::new(format!("Unknown channel id {id}"), line).into_spanda()
        })?;
        Ok(handle.borrow_mut().pop_front())
    }

    pub fn queue_spawn(&mut self, func_name: String, args: Vec<RuntimeValue>) {
        self.spawn_queue.push(SpawnJob { func_name, args });
    }

    pub fn drain_spawn_queue(&mut self) -> Vec<SpawnJob> {
        std::mem::take(&mut self.spawn_queue)
    }
}
