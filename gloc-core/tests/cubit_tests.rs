//! Integration tests for the [`Cubit`] trait and [`CubitBase`] implementation.
//!
//! Tests are grouped into logical modules that each cover one concern:
//! - `state_trait`   — the `State` blanket implementation and type bounds
//! - `cubit_base`    — `CubitBase` behaviour (emit, change-detection, etc.)
//! - `custom_cubit`  — verifies that user-defined cubits integrate correctly
//! - `injection`     — demonstrates Dependency Inversion via trait objects
//! - `edge_cases`    — boundary conditions and type variety

use gloc_core::{Cubit, CubitBase, State};

// ---------------------------------------------------------------------------
// Shared fixtures
// ---------------------------------------------------------------------------

/// A minimal state type used across multiple test modules.
#[derive(Clone, PartialEq, Debug)]
struct CounterState {
    count: i32,
}

impl CounterState {
    fn new(count: i32) -> Self {
        Self { count }
    }
}

/// A concrete `Cubit` implementation wrapping `CounterState`.
/// Demonstrates the pattern users will follow in real code.
struct CounterCubit {
    state: CounterState,
}

impl CounterCubit {
    fn new(initial: i32) -> Self {
        Self {
            state: CounterState::new(initial),
        }
    }

    fn increment(&mut self) {
        let next = self.state().count + 1;
        self.emit(CounterState::new(next));
    }

    fn decrement(&mut self) {
        let next = self.state().count - 1;
        self.emit(CounterState::new(next));
    }

    fn reset(&mut self) {
        self.emit(CounterState::new(0));
    }

    fn add(&mut self, delta: i32) {
        let next = self.state().count + delta;
        self.emit(CounterState::new(next));
    }
}

impl Cubit for CounterCubit {
    type State = CounterState;

    fn state(&self) -> &CounterState {
        &self.state
    }

    fn emit(&mut self, next: CounterState) {
        if next != self.state {
            self.state = next;
        }
    }
}

// ---------------------------------------------------------------------------
// Module: state_trait
// ---------------------------------------------------------------------------

mod state_trait {
    use super::*;

    /// Primitive types satisfy the `State` bound via the blanket impl.
    #[test]
    fn primitives_implement_state() {
        fn assert_state<S: State>() {}
        assert_state::<i32>();
        assert_state::<u64>();
        assert_state::<f64>();
        assert_state::<bool>();
        assert_state::<&str>();
        assert_state::<String>();
    }

    /// Tuple types composed of `State` types are also `State`.
    #[test]
    fn tuple_implements_state() {
        fn assert_state<S: State>() {}
        assert_state::<(i32, String)>();
    }

    /// Unit type is a valid (if trivial) `State`.
    #[test]
    fn unit_is_state() {
        fn assert_state<S: State>() {}
        assert_state::<()>();
    }

    /// A user-defined struct with the required derives is a `State`.
    #[test]
    fn custom_struct_implements_state() {
        fn assert_state<S: State>() {}
        assert_state::<CounterState>();
    }

    /// Option<State> is itself a valid State (useful for nullable values).
    #[test]
    fn option_of_state_is_state() {
        fn assert_state<S: State>() {}
        assert_state::<Option<i32>>();
        assert_state::<Option<CounterState>>();
    }
}

// ---------------------------------------------------------------------------
// Module: cubit_base
// ---------------------------------------------------------------------------

mod cubit_base {
    use super::*;

    /// `new()` stores the initial state exactly as provided.
    #[test]
    fn new_stores_initial_state() {
        let cubit = CubitBase::new(CounterState::new(10));
        assert_eq!(*cubit.state(), CounterState::new(10));
    }

    /// Emitting a different value replaces the current state.
    #[test]
    fn emit_updates_state() {
        let mut cubit = CubitBase::new(CounterState::new(0));
        cubit.emit(CounterState::new(5));
        assert_eq!(cubit.state().count, 5);
    }

    /// Multiple sequential emissions each take effect.
    #[test]
    fn sequential_emissions_are_applied_in_order() {
        let mut cubit = CubitBase::new(CounterState::new(0));
        cubit.emit(CounterState::new(1));
        cubit.emit(CounterState::new(2));
        cubit.emit(CounterState::new(3));
        assert_eq!(cubit.state().count, 3);
    }

    /// Emitting a value equal to the current state is a no-op (change-detection).
    #[test]
    fn emit_same_state_is_noop() {
        let mut cubit = CubitBase::new(42_i32);
        cubit.emit(42);
        assert_eq!(*cubit.state(), 42);

        // Confirm we can still emit a different value afterward.
        cubit.emit(43);
        assert_eq!(*cubit.state(), 43);
    }

    /// Works correctly with a primitive `i32` state.
    #[test]
    fn works_with_primitive_i32() {
        let mut cubit = CubitBase::new(0_i32);
        cubit.emit(100);
        assert_eq!(*cubit.state(), 100);
    }

    /// Works correctly with `String` state.
    #[test]
    fn works_with_string_state() {
        let mut cubit = CubitBase::new(String::from("idle"));
        cubit.emit(String::from("loading"));
        assert_eq!(cubit.state(), "loading");
        cubit.emit(String::from("success"));
        assert_eq!(cubit.state(), "success");
    }

    /// Works with `bool` state — useful for toggle cubits.
    #[test]
    fn works_with_bool_state() {
        let mut cubit = CubitBase::new(false);
        cubit.emit(true);
        assert!(*cubit.state());
        cubit.emit(true); // no-op
        assert!(*cubit.state());
        cubit.emit(false);
        assert!(!cubit.state());
    }

    /// Works with `Option<T>` state — nullable/optional patterns.
    #[test]
    fn works_with_option_state() {
        let mut cubit: CubitBase<Option<String>> = CubitBase::new(None);
        assert_eq!(*cubit.state(), None);
        cubit.emit(Some(String::from("value")));
        assert_eq!(cubit.state(), &Some(String::from("value")));
        cubit.emit(None);
        assert_eq!(*cubit.state(), None);
    }

    /// `Debug` implementation is reachable (required by `State` bounds).
    #[test]
    fn debug_output_is_accessible() {
        let cubit = CubitBase::new(CounterState::new(7));
        let debug_str = format!("{:?}", cubit);
        assert!(debug_str.contains("7"));
    }
}

// ---------------------------------------------------------------------------
// Module: custom_cubit
// ---------------------------------------------------------------------------

mod custom_cubit {
    use super::*;

    /// Initial state is returned before any mutation.
    #[test]
    fn initial_state_is_correct() {
        let cubit = CounterCubit::new(0);
        assert_eq!(cubit.state().count, 0);
    }

    /// Custom initial value is respected.
    #[test]
    fn custom_initial_value_is_respected() {
        let cubit = CounterCubit::new(99);
        assert_eq!(cubit.state().count, 99);
    }

    /// `increment` increases the count by 1.
    #[test]
    fn increment_increases_count_by_one() {
        let mut cubit = CounterCubit::new(0);
        cubit.increment();
        assert_eq!(cubit.state().count, 1);
    }

    /// Multiple increments accumulate correctly.
    #[test]
    fn multiple_increments_accumulate() {
        let mut cubit = CounterCubit::new(0);
        for _ in 0..5 {
            cubit.increment();
        }
        assert_eq!(cubit.state().count, 5);
    }

    /// `decrement` decreases the count by 1.
    #[test]
    fn decrement_decreases_count_by_one() {
        let mut cubit = CounterCubit::new(10);
        cubit.decrement();
        assert_eq!(cubit.state().count, 9);
    }

    /// `decrement` can produce negative values (no clamping in base trait).
    #[test]
    fn decrement_below_zero_is_allowed() {
        let mut cubit = CounterCubit::new(0);
        cubit.decrement();
        assert_eq!(cubit.state().count, -1);
    }

    /// `reset` always returns the count to zero regardless of current value.
    #[test]
    fn reset_returns_count_to_zero() {
        let mut cubit = CounterCubit::new(0);
        cubit.increment();
        cubit.increment();
        cubit.increment();
        cubit.reset();
        assert_eq!(cubit.state().count, 0);
    }

    /// `reset` on an already-zero state is a no-op (change-detection).
    #[test]
    fn reset_on_zero_state_is_noop() {
        let mut cubit = CounterCubit::new(0);
        cubit.reset();
        assert_eq!(cubit.state().count, 0);
    }

    /// `add` applies an arbitrary positive delta.
    #[test]
    fn add_applies_positive_delta() {
        let mut cubit = CounterCubit::new(0);
        cubit.add(10);
        assert_eq!(cubit.state().count, 10);
    }

    /// `add` applies an arbitrary negative delta.
    #[test]
    fn add_applies_negative_delta() {
        let mut cubit = CounterCubit::new(100);
        cubit.add(-40);
        assert_eq!(cubit.state().count, 60);
    }

    /// Mixed operations produce the correct final state.
    #[test]
    fn mixed_operations_produce_correct_state() {
        let mut cubit = CounterCubit::new(0);
        cubit.increment(); // 1
        cubit.increment(); // 2
        cubit.decrement(); // 1
        cubit.add(9); // 10
        cubit.reset(); // 0
        cubit.add(5); // 5
        assert_eq!(cubit.state().count, 5);
    }

    /// Emitting a state equal to the current state does not change anything.
    #[test]
    fn emit_same_state_is_noop() {
        let mut cubit = CounterCubit::new(5);
        cubit.emit(CounterState::new(5));
        assert_eq!(cubit.state().count, 5);
    }
}

// ---------------------------------------------------------------------------
// Module: injection — Dependency Inversion tests
// ---------------------------------------------------------------------------
//
// These tests demonstrate that callers depend only on the `Cubit` trait, not
// on concrete types. This is the Dependency Inversion Principle (SOLID-D):
// high-level policy code accepts `&mut dyn Cubit<State = …>` and is therefore
// trivially testable with any implementation — real or mock.

mod injection {
    use super::*;

    /// A helper that operates through the `Cubit` trait alone.
    /// It does not know or care what concrete type it receives.
    fn apply_ten_increments(cubit: &mut dyn Cubit<State = CounterState>) {
        for _ in 0..10 {
            let next = cubit.state().count + 1;
            cubit.emit(CounterState::new(next));
        }
    }

    /// The same helper can accept a custom `CounterCubit`.
    #[test]
    fn trait_object_accepts_custom_cubit() {
        let mut cubit = CounterCubit::new(0);
        apply_ten_increments(&mut cubit);
        assert_eq!(cubit.state().count, 10);
    }

    /// The same helper can accept a `CubitBase`.
    #[test]
    fn trait_object_accepts_cubit_base() {
        let mut cubit = CubitBase::new(CounterState::new(0));
        apply_ten_increments(&mut cubit);
        assert_eq!(cubit.state().count, 10);
    }

    /// A mock cubit that records every emitted state — useful in unit tests
    /// for verifying that the correct sequence of states was produced.
    struct RecordingCubit {
        state: CounterState,
        history: Vec<CounterState>,
    }

    impl RecordingCubit {
        fn new(initial: i32) -> Self {
            Self {
                state: CounterState::new(initial),
                history: vec![CounterState::new(initial)],
            }
        }
    }

    impl Cubit for RecordingCubit {
        type State = CounterState;

        fn state(&self) -> &CounterState {
            &self.state
        }

        fn emit(&mut self, next: CounterState) {
            if next != self.state {
                self.state = next.clone();
                self.history.push(next);
            }
        }
    }

    /// `RecordingCubit` captures the full emission history.
    #[test]
    fn recording_cubit_captures_history() {
        let mut cubit = RecordingCubit::new(0);
        apply_ten_increments(&mut cubit);

        // history starts with the initial state (0) then grows to 11 entries.
        assert_eq!(cubit.history.len(), 11);
        assert_eq!(cubit.history[0].count, 0);
        assert_eq!(cubit.history[10].count, 10);
    }

    /// `RecordingCubit` does not record duplicate states.
    #[test]
    fn recording_cubit_skips_duplicate_emissions() {
        let mut cubit = RecordingCubit::new(5);
        cubit.emit(CounterState::new(5)); // duplicate — ignored
        cubit.emit(CounterState::new(5)); // duplicate — ignored
        cubit.emit(CounterState::new(6)); // new — recorded
                                          // history: [5, 6]
        assert_eq!(cubit.history.len(), 2);
        assert_eq!(cubit.history[1].count, 6);
    }

    /// A helper that reads state through a shared reference.
    fn read_count(cubit: &dyn Cubit<State = CounterState>) -> i32 {
        cubit.state().count
    }

    /// Shared reference to trait object is sufficient for read-only access.
    #[test]
    fn shared_reference_trait_object_for_reads() {
        let cubit = CounterCubit::new(42);
        assert_eq!(read_count(&cubit), 42);
    }
}

// ---------------------------------------------------------------------------
// Module: edge_cases
// ---------------------------------------------------------------------------

mod edge_cases {
    use super::*;

    /// State with maximum `i32` value does not panic.
    #[test]
    fn emit_max_i32_does_not_panic() {
        let mut cubit = CubitBase::new(i32::MAX - 1);
        cubit.emit(i32::MAX);
        assert_eq!(*cubit.state(), i32::MAX);
    }

    /// State with minimum `i32` value does not panic.
    #[test]
    fn emit_min_i32_does_not_panic() {
        let mut cubit = CubitBase::new(i32::MIN + 1);
        cubit.emit(i32::MIN);
        assert_eq!(*cubit.state(), i32::MIN);
    }

    /// An empty `Vec` is a valid initial state.
    #[test]
    fn empty_vec_as_initial_state() {
        let mut cubit: CubitBase<Vec<i32>> = CubitBase::new(vec![]);
        assert!(cubit.state().is_empty());
        cubit.emit(vec![1, 2, 3]);
        assert_eq!(*cubit.state(), vec![1, 2, 3]);
    }

    /// Rapid alternation between two states converges on the final one.
    #[test]
    fn rapid_alternation_converges() {
        let mut cubit = CubitBase::new(false);
        for i in 0..1000 {
            cubit.emit(i % 2 == 0);
        }
        // 1000 iterations: last is i=999, 999 % 2 == 1, so false
        assert!(!cubit.state());
    }

    /// A unit state `()` is valid; emit is always a no-op since `() == ()`.
    #[test]
    fn unit_state_emit_is_always_noop() {
        let mut cubit = CubitBase::new(());
        cubit.emit(());
        assert_eq!(*cubit.state(), ());
    }

    /// `CubitBase<String>` with an empty string initial state.
    #[test]
    fn empty_string_initial_state() {
        let mut cubit = CubitBase::new(String::new());
        assert!(cubit.state().is_empty());
        cubit.emit(String::from("non-empty"));
        assert_eq!(cubit.state(), "non-empty");
    }

    /// Emitting the same value twice only "sticks" the first time.
    #[test]
    fn emit_same_value_twice_no_double_update() {
        let mut cubit = CubitBase::new(0_i32);
        cubit.emit(1); // applied
        cubit.emit(1); // no-op
        assert_eq!(*cubit.state(), 1);
    }

    /// A large struct as state is cloned correctly.
    #[test]
    fn large_struct_state_is_cloned_correctly() {
        #[derive(Clone, PartialEq, Debug)]
        struct BigState {
            data: Vec<String>,
            label: String,
        }

        let initial = BigState {
            data: vec!["a".into(), "b".into()],
            label: "first".into(),
        };
        let next = BigState {
            data: vec!["x".into(), "y".into(), "z".into()],
            label: "second".into(),
        };

        let mut cubit = CubitBase::new(initial);
        cubit.emit(next.clone());
        assert_eq!(cubit.state(), &next);
    }
}
