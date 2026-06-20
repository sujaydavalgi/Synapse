//! Declarative event dispatch for Spanda programs.
//!
//! Maps event names declared in source to handler statement blocks. The runtime
//! [`crate::runtime::Interpreter`] registers handlers at load time and dispatches
//! them when triggers or explicit `emit` statements fire matching events.

use crate::ast::Stmt;
use std::collections::HashMap;
/// Event bus mapping declared events to handler bodies.
///
/// Stores a name-to-statements table populated from `on event` declarations in
/// a Spanda program. Handlers are executed by the interpreter when a matching
/// event is dispatched.
///
/// # Parameters
///
/// None — construct with [`EventBus::new`].
///
/// # Returns
///
/// N/A (type definition).
///
/// # Options
///
/// None.
///
/// # Example
///
/// ```
/// use spanda_core::ast::Stmt;
/// use spanda_core::events::EventBus;
///
/// let mut bus = EventBus::new();
/// bus.register("obstacle_detected".into(), vec![]);
/// assert!(bus.handler_body("obstacle_detected").is_some());
/// ```
pub struct EventBus {
    handlers: HashMap<String, Vec<Stmt>>,
}

impl EventBus {
    pub fn new() -> Self {
        // Creates an empty event bus with no registered handlers.
        //
        // Parameters:
        //
        // None.
        //
        // Returns:
        //
        // A new [`EventBus`] ready for [`Self::register`] calls.
        //
        // Options:
        //
        // None.
        //
        // Example:
        //
        // use spanda_core::events::EventBus;
        // let bus = EventBus::new();

        // assert!(bus.handler_body("any").is_none());
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, event: String, body: Vec<Stmt>) {
        // Registers or replaces the handler body for an event name.
        //
        // Parameters:
        //
        // * `event` — Declared event identifier (e.g. `"emergency_stop"`).
        // * `body` — Statement block to execute when the event fires.
        //
        // Returns:
        //
        // Nothing; overwrites any prior handler for the same name.
        //
        // Options:
        //
        // None.
        //
        // Example:
        //
        // use spanda_core::events::EventBus;
        // let mut bus = EventBus::new();

        // bus.register("tick".into(), vec![]);
        self.handlers.insert(event, body);
    }

    pub fn handler_body(&self, event: &str) -> Option<&[Stmt]> {
        // Returns the handler statement slice for an event, if registered.
        //
        // Parameters:
        //
        // * `event` — Event name to look up.
        //
        // Returns:
        //
        // `Some` slice of handler [`Stmt`]s, or `None` when the event is unknown.
        //
        // Options:
        //
        // None.
        //
        // Example:
        //
        // use spanda_core::events::EventBus;
        // let bus = EventBus::new();

        // assert!(bus.handler_body("missing").is_none());
        self.handlers.get(event).map(|v| v.as_slice())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::events::default();

        // Build the result via new.
        Self::new()
    }
}
