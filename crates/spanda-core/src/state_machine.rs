//! Runtime execution of declarative state machines.
//!
//! Tracks the active state of a `state_machine` block and enforces allowed
//! `(from, to)` transitions declared in source. Used by the interpreter for
//! `enter` statements and state-entry/exit triggers.
/// Runtime state for a declared state machine with validated transitions.
///
/// Holds the machine name, current state, the full state list, and the
/// directed transition graph from the AST.
///
/// # Parameters
///
/// None — construct with [`StateMachineRuntime::new`].
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
/// use spanda_core::state_machine::StateMachineRuntime;
///
/// let mut sm = StateMachineRuntime::new(
///     "Nav".into(),
///     vec!["Idle".into(), "Moving".into()],
///     vec![("Idle".into(), "Moving".into())],
/// );
/// assert_eq!(sm.current, "Idle");
/// ```
pub struct StateMachineRuntime {
    pub name: String,
    pub current: String,
    states: Vec<String>,
    transitions: Vec<(String, String)>,
}

impl StateMachineRuntime {
    pub fn new(name: String, states: Vec<String>, transitions: Vec<(String, String)>) -> Self {
        // Builds a state machine runtime starting in the first declared state.
        //
        // Parameters:
        //
        // * `name` — Machine identifier from the declaration.
        // * `states` — Ordered list of valid state names.
        // * `transitions` — Allowed `(from, to)` edges.
        //
        // Returns:
        //
        // A new runtime whose [`Self::current`] is the first state, or empty if
        // `states` is empty.
        //
        // Options:
        //
        // None.
        //
        // Example:
        //
        // use spanda_core::state_machine::StateMachineRuntime;
        // let sm = StateMachineRuntime::new(
        // "Flow".into(),
        // vec!["A".into(), "B".into()],
        // vec![],
        // );
        // assert_eq!(sm.current, "A");
        let current = states.first().cloned().unwrap_or_default();
        Self {
            name,
            current,
            states,
            transitions,
        }
    }

    pub fn try_enter(&mut self, target: &str) -> Option<String> {
        // Attempts a transition to `target` when allowed from the current state.
        //
        // Parameters:
        //
        // * `target` — Desired next state name.
        //
        // Returns:
        //
        // `Some(previous_state)` on success after updating [`Self::current`], or
        // `None` if `target` is unknown or the transition is not declared.
        //
        // Options:
        //
        // None.
        //
        // Example:
        //
        // use spanda_core::state_machine::StateMachineRuntime;
        // let mut sm = StateMachineRuntime::new(
        // "Flow".into(),
        // vec!["Idle".into(), "Loading".into()],
        // vec![("Idle".into(), "Loading".into())],
        // );
        // assert_eq!(sm.try_enter("Loading"), Some("Idle".into()));
        // assert_eq!(sm.current, "Loading");
        if !self.states.iter().any(|s| s == target) {
            return None;
        }
        let allowed = self
            .transitions
            .iter()
            .any(|(from, to)| from == &self.current && to == target);

        // Take the branch when allowed is false.
        if !allowed {
            return None;
        }
        let previous = self.current.clone();
        self.current = target.to_string();
        Some(previous)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_enter_follows_declared_transitions() {
        // Try enter follows declared transitions.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::state_machine::try_enter_follows_declared_transitions();

        let mut sm = StateMachineRuntime::new(
            "Flow".into(),
            vec!["Idle".into(), "Loading".into()],
            vec![("Idle".into(), "Loading".into())],
        );
        assert_eq!(sm.current, "Idle");
        assert_eq!(sm.try_enter("Loading"), Some("Idle".into()));
        assert_eq!(sm.current, "Loading");
        assert_eq!(sm.try_enter("Idle"), None);
    }
}
