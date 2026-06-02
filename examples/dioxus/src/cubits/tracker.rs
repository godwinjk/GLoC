//! `ClickTrackerReactor` — tracks total button clicks across the app.
//!
//! Demonstrates Mode B (generated state) with multiple state fields.
//! This cubit observes clicks regardless of which button was pressed.
use gloc::reactor;
use gloc::Reactor;
// ---------------------------------------------------------------------------
// Cubit — Mode B: macro generates ClickTrackerReactorState
// ---------------------------------------------------------------------------
/// Tracks total clicks and the name of the last button pressed.
///
/// Uses Mode B — `#[state]` fields are collected into a generated
/// `ClickTrackerReactorState { total: u32, last_action: String }`.
#[reactor]
pub struct ClickTrackerReactor {
    #[state]
    pub total: u32,
    #[state]
    pub last_action: String,
}
impl ClickTrackerReactor {
    /// Records a click with the name of the action that triggered it.
    ///
    /// # Parameters
    /// - `action` — a human-readable label e.g. `"increment"`, `"reset"`
    pub fn record(&mut self, action: &str) {
        self.emit(ClickTrackerReactorState {
            total: self.state().total + 1,
            last_action: action.to_string(),
        });
    }
    /// Resets the tracker back to zero.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.emit(ClickTrackerReactorState {
            total: 0,
            last_action: "reset tracker".to_string(),
        });
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use gloc_test::{reactor_test, ReactorTester};

    use super::*;

    fn tracker() -> ClickTrackerReactor {
        ClickTrackerReactor::new(ClickTrackerReactorState {
            total: 0,
            last_action: String::new(),
        })
    }

    // ---- happy path ----

    #[test]
    fn record_increments_total() {
        reactor_test! {
            build: tracker(),
            acts: [|r| r.record("increment")],
            expect_states: [ClickTrackerReactorState {
                total: 1,
                last_action: "increment".into(),
            }],
        }
    }

    #[test]
    fn record_updates_last_action() {
        let tester = ReactorTester::new(tracker());
        tester.act(|r| r.record("foo"));
        tester.act(|r| r.record("bar"));
        assert_eq!(tester.state().last_action, "bar");
    }

    #[test]
    fn record_accumulates_total_across_calls() {
        let tester = ReactorTester::new(tracker());
        tester.act(|r| r.record("a"));
        tester.act(|r| r.record("b"));
        tester.act(|r| r.record("c"));
        assert_eq!(tester.state().total, 3);
    }

    #[test]
    fn reset_clears_total_and_last_action() {
        reactor_test! {
            build: tracker(),
            acts: [
                |r| r.record("some action"),
                |r| r.reset(),
            ],
            expect_states: [
                ClickTrackerReactorState { total: 1, last_action: "some action".into() },
                ClickTrackerReactorState { total: 0, last_action: "reset tracker".into() },
            ],
        }
    }

    // ---- edge case ----

    #[test]
    fn record_sequence_each_step_is_captured() {
        reactor_test! {
            build: tracker(),
            acts: [
                |r| r.record("click-1"),
                |r| r.record("click-2"),
            ],
            expect_states: [
                ClickTrackerReactorState { total: 1, last_action: "click-1".into() },
                ClickTrackerReactorState { total: 2, last_action: "click-2".into() },
            ],
        }
    }

    // ---- boundary ----

    #[test]
    fn tracker_starts_at_zero_with_no_emissions() {
        reactor_test! {
            build: tracker(),
            expect_no_emissions: true,
        }
    }

    #[test]
    fn record_empty_string_action_is_valid() {
        let tester = ReactorTester::new(tracker());
        tester.act(|r| r.record(""));
        assert_eq!(tester.state().total, 1);
        assert_eq!(tester.state().last_action, "");
    }
}
