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
