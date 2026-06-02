//! Integration tests for the [`Reactor`] trait and [`ReactorBase`] implementation.
//!
//! Tests are grouped into logical modules that each cover one concern:
//! - `neutron_trait`   — the `Neutron` blanket implementation and type bounds
//! - `state_trait`     — the `State` blanket implementation and type bounds
//! - `reactor_base`    — `ReactorBase` behaviour (emit, change-detection, etc.)
//! - `custom_reactor`  — verifies that user-defined reactors integrate correctly
//! - `injection`       — demonstrates Dependency Inversion via trait objects
//! - `edge_cases`      — boundary conditions and type variety

use gloc_core::{Neutron, Reactor, ReactorBase, State};

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

/// A concrete `Reactor` implementation wrapping `CounterState`.
/// Demonstrates the pattern users will follow in real code.
struct CounterReactor {
    state: CounterState,
}

impl CounterReactor {
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

impl Reactor for CounterReactor {
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
// Module: neutron_trait
// ---------------------------------------------------------------------------

mod neutron_trait {
    use super::*;

    /// Primitive types satisfy `Neutron` via the blanket impl.
    #[test]
    fn primitives_implement_neutron() {
        fn assert_neutron<N: Neutron>() {}
        assert_neutron::<i32>();
        assert_neutron::<String>();
        assert_neutron::<bool>();
    }

    /// Enums with Debug + Send + 'static are Neutrons.
    #[test]
    fn enum_implements_neutron() {
        #[derive(Debug)]
        #[allow(dead_code)]
        enum CounterEvent {
            Increment,
            Decrement,
            Reset,
            AddBy(i32),
        }

        fn assert_neutron<N: Neutron>() {}
        assert_neutron::<CounterEvent>();
    }

    /// Structs with Debug + Send + 'static are Neutrons.
    #[test]
    fn struct_implements_neutron() {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct SetValue(i32);

        fn assert_neutron<N: Neutron>() {}
        assert_neutron::<SetValue>();
    }

    /// Neutrons do NOT require Clone or PartialEq — unlike State.
    #[test]
    fn neutron_does_not_require_clone_or_partialeq() {
        // This type has no Clone or PartialEq — must still be a Neutron.
        #[derive(Debug)]
        #[allow(dead_code)]
        struct NonCloneNeutron {
            payload: String,
        }

        fn assert_neutron<N: Neutron>() {}
        assert_neutron::<NonCloneNeutron>();
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
// Module: reactor_base
// ---------------------------------------------------------------------------

mod reactor_base {
    use super::*;

    /// `new()` stores the initial state exactly as provided.
    #[test]
    fn new_stores_initial_state() {
        let reactor = ReactorBase::new(CounterState::new(10));
        assert_eq!(*reactor.state(), CounterState::new(10));
    }

    /// Emitting a different value replaces the current state.
    #[test]
    fn emit_updates_state() {
        let mut reactor = ReactorBase::new(CounterState::new(0));
        reactor.emit(CounterState::new(5));
        assert_eq!(reactor.state().count, 5);
    }

    /// Multiple sequential emissions each take effect.
    #[test]
    fn sequential_emissions_are_applied_in_order() {
        let mut reactor = ReactorBase::new(CounterState::new(0));
        reactor.emit(CounterState::new(1));
        reactor.emit(CounterState::new(2));
        reactor.emit(CounterState::new(3));
        assert_eq!(reactor.state().count, 3);
    }

    /// Emitting a value equal to the current state is a no-op (change-detection).
    #[test]
    fn emit_same_state_is_noop() {
        let mut reactor = ReactorBase::new(42_i32);
        reactor.emit(42);
        assert_eq!(*reactor.state(), 42);

        // Confirm we can still emit a different value afterward.
        reactor.emit(43);
        assert_eq!(*reactor.state(), 43);
    }

    /// Works correctly with a primitive `i32` state.
    #[test]
    fn works_with_primitive_i32() {
        let mut reactor = ReactorBase::new(0_i32);
        reactor.emit(100);
        assert_eq!(*reactor.state(), 100);
    }

    /// Works correctly with `String` state.
    #[test]
    fn works_with_string_state() {
        let mut reactor = ReactorBase::new(String::from("idle"));
        reactor.emit(String::from("loading"));
        assert_eq!(reactor.state(), "loading");
        reactor.emit(String::from("success"));
        assert_eq!(reactor.state(), "success");
    }

    /// Works with `bool` state — useful for toggle reactors.
    #[test]
    fn works_with_bool_state() {
        let mut reactor = ReactorBase::new(false);
        reactor.emit(true);
        assert!(*reactor.state());
        reactor.emit(true); // no-op
        assert!(*reactor.state());
        reactor.emit(false);
        assert!(!reactor.state());
    }

    /// Works with `Option<T>` state — nullable/optional patterns.
    #[test]
    fn works_with_option_state() {
        let mut reactor: ReactorBase<Option<String>> = ReactorBase::new(None);
        assert_eq!(*reactor.state(), None);
        reactor.emit(Some(String::from("value")));
        assert_eq!(reactor.state(), &Some(String::from("value")));
        reactor.emit(None);
        assert_eq!(*reactor.state(), None);
    }

    /// `Debug` implementation is reachable (required by `State` bounds).
    #[test]
    fn debug_output_is_accessible() {
        let reactor = ReactorBase::new(CounterState::new(7));
        let debug_str = format!("{:?}", reactor);
        assert!(debug_str.contains("7"));
    }
}

// ---------------------------------------------------------------------------
// Module: custom_reactor
// ---------------------------------------------------------------------------

mod custom_reactor {
    use super::*;

    /// Initial state is returned before any mutation.
    #[test]
    fn initial_state_is_correct() {
        let reactor = CounterReactor::new(0);
        assert_eq!(reactor.state().count, 0);
    }

    /// Custom initial value is respected.
    #[test]
    fn custom_initial_value_is_respected() {
        let reactor = CounterReactor::new(99);
        assert_eq!(reactor.state().count, 99);
    }

    /// `increment` increases the count by 1.
    #[test]
    fn increment_increases_count_by_one() {
        let mut reactor = CounterReactor::new(0);
        reactor.increment();
        assert_eq!(reactor.state().count, 1);
    }

    /// Multiple increments accumulate correctly.
    #[test]
    fn multiple_increments_accumulate() {
        let mut reactor = CounterReactor::new(0);
        for _ in 0..5 {
            reactor.increment();
        }
        assert_eq!(reactor.state().count, 5);
    }

    /// `decrement` decreases the count by 1.
    #[test]
    fn decrement_decreases_count_by_one() {
        let mut reactor = CounterReactor::new(10);
        reactor.decrement();
        assert_eq!(reactor.state().count, 9);
    }

    /// `decrement` can produce negative values (no clamping in base trait).
    #[test]
    fn decrement_below_zero_is_allowed() {
        let mut reactor = CounterReactor::new(0);
        reactor.decrement();
        assert_eq!(reactor.state().count, -1);
    }

    /// `reset` always returns the count to zero regardless of current value.
    #[test]
    fn reset_returns_count_to_zero() {
        let mut reactor = CounterReactor::new(0);
        reactor.increment();
        reactor.increment();
        reactor.increment();
        reactor.reset();
        assert_eq!(reactor.state().count, 0);
    }

    /// `reset` on an already-zero state is a no-op (change-detection).
    #[test]
    fn reset_on_zero_state_is_noop() {
        let mut reactor = CounterReactor::new(0);
        reactor.reset();
        assert_eq!(reactor.state().count, 0);
    }

    /// `add` applies an arbitrary positive delta.
    #[test]
    fn add_applies_positive_delta() {
        let mut reactor = CounterReactor::new(0);
        reactor.add(10);
        assert_eq!(reactor.state().count, 10);
    }

    /// `add` applies an arbitrary negative delta.
    #[test]
    fn add_applies_negative_delta() {
        let mut reactor = CounterReactor::new(100);
        reactor.add(-40);
        assert_eq!(reactor.state().count, 60);
    }

    /// Mixed operations produce the correct final state.
    #[test]
    fn mixed_operations_produce_correct_state() {
        let mut reactor = CounterReactor::new(0);
        reactor.increment(); // 1
        reactor.increment(); // 2
        reactor.decrement(); // 1
        reactor.add(9); // 10
        reactor.reset(); // 0
        reactor.add(5); // 5
        assert_eq!(reactor.state().count, 5);
    }

    /// Emitting a state equal to the current state does not change anything.
    #[test]
    fn emit_same_state_is_noop() {
        let mut reactor = CounterReactor::new(5);
        reactor.emit(CounterState::new(5));
        assert_eq!(reactor.state().count, 5);
    }
}

// ---------------------------------------------------------------------------
// Module: injection — Dependency Inversion tests
// ---------------------------------------------------------------------------
//
// These tests demonstrate that callers depend only on the `Reactor` trait, not
// on concrete types. This is the Dependency Inversion Principle (SOLID-D):
// high-level policy code accepts `&mut dyn Reactor<State = …>` and is therefore
// trivially testable with any implementation — real or mock.

mod injection {
    use super::*;

    /// A helper that operates through the `Reactor` trait alone.
    /// It does not know or care what concrete type it receives.
    fn apply_ten_increments(reactor: &mut dyn Reactor<State = CounterState>) {
        for _ in 0..10 {
            let next = reactor.state().count + 1;
            reactor.emit(CounterState::new(next));
        }
    }

    /// The same helper can accept a custom `CounterReactor`.
    #[test]
    fn trait_object_accepts_custom_reactor() {
        let mut reactor = CounterReactor::new(0);
        apply_ten_increments(&mut reactor);
        assert_eq!(reactor.state().count, 10);
    }

    /// The same helper can accept a `ReactorBase`.
    #[test]
    fn trait_object_accepts_reactor_base() {
        let mut reactor = ReactorBase::new(CounterState::new(0));
        apply_ten_increments(&mut reactor);
        assert_eq!(reactor.state().count, 10);
    }

    /// A mock reactor that records every emitted state — useful in unit tests
    /// for verifying that the correct sequence of states was produced.
    struct RecordingReactor {
        state: CounterState,
        history: Vec<CounterState>,
    }

    impl RecordingReactor {
        fn new(initial: i32) -> Self {
            Self {
                state: CounterState::new(initial),
                history: vec![CounterState::new(initial)],
            }
        }
    }

    impl Reactor for RecordingReactor {
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

    /// `RecordingReactor` captures the full emission history.
    #[test]
    fn recording_reactor_captures_history() {
        let mut reactor = RecordingReactor::new(0);
        apply_ten_increments(&mut reactor);

        // history starts with the initial state (0) then grows to 11 entries.
        assert_eq!(reactor.history.len(), 11);
        assert_eq!(reactor.history[0].count, 0);
        assert_eq!(reactor.history[10].count, 10);
    }

    /// `RecordingReactor` does not record duplicate states.
    #[test]
    fn recording_reactor_skips_duplicate_emissions() {
        let mut reactor = RecordingReactor::new(5);
        reactor.emit(CounterState::new(5)); // duplicate — ignored
        reactor.emit(CounterState::new(5)); // duplicate — ignored
        reactor.emit(CounterState::new(6)); // new — recorded
                                            // history: [5, 6]
        assert_eq!(reactor.history.len(), 2);
        assert_eq!(reactor.history[1].count, 6);
    }

    /// A helper that reads state through a shared reference.
    fn read_count(reactor: &dyn Reactor<State = CounterState>) -> i32 {
        reactor.state().count
    }

    /// Shared reference to trait object is sufficient for read-only access.
    #[test]
    fn shared_reference_trait_object_for_reads() {
        let reactor = CounterReactor::new(42);
        assert_eq!(read_count(&reactor), 42);
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
        let mut reactor = ReactorBase::new(i32::MAX - 1);
        reactor.emit(i32::MAX);
        assert_eq!(*reactor.state(), i32::MAX);
    }

    /// State with minimum `i32` value does not panic.
    #[test]
    fn emit_min_i32_does_not_panic() {
        let mut reactor = ReactorBase::new(i32::MIN + 1);
        reactor.emit(i32::MIN);
        assert_eq!(*reactor.state(), i32::MIN);
    }

    /// An empty `Vec` is a valid initial state.
    #[test]
    fn empty_vec_as_initial_state() {
        let mut reactor: ReactorBase<Vec<i32>> = ReactorBase::new(vec![]);
        assert!(reactor.state().is_empty());
        reactor.emit(vec![1, 2, 3]);
        assert_eq!(*reactor.state(), vec![1, 2, 3]);
    }

    /// Rapid alternation between two states converges on the final one.
    #[test]
    fn rapid_alternation_converges() {
        let mut reactor = ReactorBase::new(false);
        for i in 0..1000 {
            reactor.emit(i % 2 == 0);
        }
        // 1000 iterations: last is i=999, 999 % 2 == 1, so false
        assert!(!reactor.state());
    }

    /// A unit state `()` is valid; emit is always a no-op since `() == ()`.
    #[test]
    fn unit_state_emit_is_always_noop() {
        let mut reactor = ReactorBase::new(());
        reactor.emit(());
        assert_eq!(*reactor.state(), ());
    }

    /// `ReactorBase<String>` with an empty string initial state.
    #[test]
    fn empty_string_initial_state() {
        let mut reactor = ReactorBase::new(String::new());
        assert!(reactor.state().is_empty());
        reactor.emit(String::from("non-empty"));
        assert_eq!(reactor.state(), "non-empty");
    }

    /// Emitting the same value twice only "sticks" the first time.
    #[test]
    fn emit_same_value_twice_no_double_update() {
        let mut reactor = ReactorBase::new(0_i32);
        reactor.emit(1); // applied
        reactor.emit(1); // no-op
        assert_eq!(*reactor.state(), 1);
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

        let mut reactor = ReactorBase::new(initial);
        reactor.emit(next.clone());
        assert_eq!(reactor.state(), &next);
    }
}
