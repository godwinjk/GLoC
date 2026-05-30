//! `CounterCubit` — business logic for a simple increment/decrement counter.
//!
//! This module is entirely UI-framework-agnostic. It knows nothing about
//! Dioxus, signals, or rendering; it only knows about `Cubit` and `State`.
//! Swapping the UI layer (e.g. from Dioxus to Bevy or Axum) requires zero
//! changes here — that is the core promise of GLOC.

use gloc::Cubit;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// A snapshot of the counter's data at a single point in time.
///
/// Implements [`State`] automatically through the blanket impl because it
/// satisfies `Clone + PartialEq + Debug`.
#[derive(Clone, PartialEq, Debug)]
pub struct CounterState {
    /// The current counter value. Can be negative (no lower bound is enforced
    /// at the state level — callers add their own guards if required).
    pub count: i32,

    /// Human-readable label that reflects the magnitude of the counter.
    /// Updated on every transition by [`CounterCubit`].
    pub label: String,
}

impl CounterState {
    /// Constructs a new `CounterState` and derives its label automatically.
    ///
    /// # Parameters
    ///
    /// - `count` — the numeric value to store.
    pub fn new(count: i32) -> Self {
        Self {
            label: Self::label_for(count),
            count,
        }
    }

    /// Maps a count to a descriptive label.
    ///
    /// | Range          | Label      |
    /// |----------------|------------|
    /// | count < 0      | "Negative" |
    /// | count == 0     | "Zero"     |
    /// | 1 ..= 9        | "Low"      |
    /// | 10 ..= 99      | "Medium"   |
    /// | 100+           | "High"     |
    fn label_for(count: i32) -> String {
        match count {
            i32::MIN..=-1 => "Negative".into(),
            0 => "Zero".into(),
            1..=9 => "Low".into(),
            10..=99 => "Medium".into(),
            _ => "High".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Cubit
// ---------------------------------------------------------------------------

/// Manages the counter's state and exposes all domain operations.
///
/// Consumers call methods like [`increment`](CounterCubit::increment) or
/// [`reset`](CounterCubit::reset); internally the cubit calls `emit()` to
/// produce the next [`CounterState`]. The UI layer observes state via
/// [`Cubit::state`] — it never mutates state directly.
///
/// # SOLID alignment
///
/// - **Single Responsibility** — owns only counter logic; no rendering code.
/// - **Dependency Inversion**  — callers depend on `dyn Cubit<State = CounterState>`,
///   making it trivial to inject a mock in tests.
pub struct CounterCubit {
    state: CounterState,
}

impl CounterCubit {
    /// Creates a new `CounterCubit` starting at the given `initial` count.
    ///
    /// # Parameters
    ///
    /// - `initial` — the starting value. Pass `0` for the most common case.
    pub fn new(initial: i32) -> Self {
        Self {
            state: CounterState::new(initial),
        }
    }

    /// Increases the count by `1` and emits the resulting state.
    ///
    /// This is a saturating-safe operation on `i32`; it will not panic even
    /// when the counter is at `i32::MAX` (overflow wraps — add a guard here
    /// if your domain requires it).
    pub fn increment(&mut self) {
        self.emit(CounterState::new(self.state.count + 1));
    }

    /// Decreases the count by `1` and emits the resulting state.
    pub fn decrement(&mut self) {
        self.emit(CounterState::new(self.state.count - 1));
    }

    /// Resets the counter to `0`, regardless of its current value.
    ///
    /// If the current count is already `0`, this is a **no-op** because
    /// [`Cubit::emit`] performs change-detection before updating state.
    pub fn reset(&mut self) {
        self.emit(CounterState::new(0));
    }

    /// Sets the counter to an arbitrary `value` and emits the resulting state.
    #[allow(dead_code)]
    ///
    /// # Parameters
    ///
    /// - `value` — the exact count to set. This bypasses increment/decrement
    ///   and is intended for initialisation, deep-linking, and testing.
    pub fn set(&mut self, value: i32) {
        self.emit(CounterState::new(value));
    }
}

impl Cubit for CounterCubit {
    type State = CounterState;

    /// Returns a reference to the cubit's current [`CounterState`].
    fn state(&self) -> &CounterState {
        &self.state
    }

    /// Replaces the current state with `next` if and only if it differs.
    ///
    /// Change-detection prevents unnecessary UI re-renders when the value has
    /// not logically changed (e.g. calling `reset()` when count is already 0).
    fn emit(&mut self, next: CounterState) {
        if next != self.state {
            self.state = next;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use gloc::Cubit;

    #[test]
    fn initial_state_is_zero_with_label_zero() {
        let c = CounterCubit::new(0);
        assert_eq!(c.state().count, 0);
        assert_eq!(c.state().label, "Zero");
    }

    #[test]
    fn increment_updates_count_and_label() {
        let mut c = CounterCubit::new(0);
        c.increment();
        assert_eq!(c.state().count, 1);
        assert_eq!(c.state().label, "Low");
    }

    #[test]
    fn decrement_from_zero_produces_negative_label() {
        let mut c = CounterCubit::new(0);
        c.decrement();
        assert_eq!(c.state().count, -1);
        assert_eq!(c.state().label, "Negative");
    }

    #[test]
    fn reset_returns_to_zero() {
        let mut c = CounterCubit::new(50);
        c.reset();
        assert_eq!(c.state().count, 0);
        assert_eq!(c.state().label, "Zero");
    }

    #[test]
    fn reset_on_zero_is_noop() {
        let mut c = CounterCubit::new(0);
        let before = c.state().clone();
        c.reset();
        assert_eq!(c.state(), &before);
    }

    #[test]
    fn set_to_ten_gives_medium_label() {
        let mut c = CounterCubit::new(0);
        c.set(10);
        assert_eq!(c.state().count, 10);
        assert_eq!(c.state().label, "Medium");
    }

    #[test]
    fn set_to_hundred_gives_high_label() {
        let mut c = CounterCubit::new(0);
        c.set(100);
        assert_eq!(c.state().count, 100);
        assert_eq!(c.state().label, "High");
    }

    #[test]
    fn label_boundaries_are_correct() {
        let cases = [
            (-1, "Negative"),
            (0, "Zero"),
            (1, "Low"),
            (9, "Low"),
            (10, "Medium"),
            (99, "Medium"),
            (100, "High"),
        ];
        for (count, expected_label) in cases {
            let s = CounterState::new(count);
            assert_eq!(s.label, expected_label, "count={count}");
        }
    }

    /// Demonstrates Dependency Inversion: this helper only knows `dyn Cubit`.
    fn drive_to_five(cubit: &mut dyn Cubit<State = CounterState>) {
        for _ in 0..5 {
            let next = cubit.state().count + 1;
            cubit.emit(CounterState::new(next));
        }
    }

    #[test]
    fn trait_object_injection_works() {
        let mut c = CounterCubit::new(0);
        drive_to_five(&mut c);
        assert_eq!(c.state().count, 5);
    }
}
