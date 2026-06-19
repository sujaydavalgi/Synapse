use crate::ast::Stmt;
use std::collections::HashMap;

/// Event bus mapping declared events to handler bodies.
pub struct EventBus {
    handlers: HashMap<String, Vec<Stmt>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, event: String, body: Vec<Stmt>) {
        self.handlers.insert(event, body);
    }

    pub fn handler_body(&self, event: &str) -> Option<&[Stmt]> {
        self.handlers.get(event).map(|v| v.as_slice())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
