/// Runtime state for a declared state machine with validated transitions.
pub struct StateMachineRuntime {
    pub name: String,
    pub current: String,
    states: Vec<String>,
    transitions: Vec<(String, String)>,
}

impl StateMachineRuntime {
    pub fn new(name: String, states: Vec<String>, transitions: Vec<(String, String)>) -> Self {
        let current = states.first().cloned().unwrap_or_default();
        Self {
            name,
            current,
            states,
            transitions,
        }
    }

    /// Returns the previous state when a declared transition to `target` succeeds.
    pub fn try_enter(&mut self, target: &str) -> Option<String> {
        if !self.states.iter().any(|s| s == target) {
            return None;
        }
        let allowed = self
            .transitions
            .iter()
            .any(|(from, to)| from == &self.current && to == target);
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
