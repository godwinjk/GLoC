//! Integration tests for the `#[cubit]` attribute macro.
//!
//! These tests exercise the generated code at the Rust type and runtime level.
//! They are organised into modules that mirror the two modes and the shared
//! generated capabilities:
//!
//! - `mode_a`       — `#[cubit(state = SomeType)]` bring-your-own state
//! - `mode_b`       — `#[state]` field annotation, generated state struct
//! - `change_det`   — change-detection behaviour (shared across modes)
//! - `observers`    — `on_change` callback behaviour
//! - `suppressions` — `no_new` and `no_observers` flags
//! - `injection`    — dependency inversion via trait objects

use gloc::Cubit;
use gloc_macro::cubit;

// ---------------------------------------------------------------------------
// Module: mode_a — bring-your-own state type
// ---------------------------------------------------------------------------

mod mode_a {
    use super::*;

    /// State defined by the developer — entirely separate from the cubit.
    #[derive(Clone, PartialEq, Debug)]
    pub struct CounterState {
        pub count: i32,
    }

    /// Minimal Mode A cubit — empty body, macro provides everything.
    #[cubit(state = CounterState)]
    pub struct CounterCubit {}

    impl CounterCubit {
        pub fn increment(&mut self) {
            let next = self.state().count + 1;
            self.emit(CounterState { count: next });
        }

        pub fn decrement(&mut self) {
            let next = self.state().count - 1;
            self.emit(CounterState { count: next });
        }

        pub fn reset(&mut self) {
            self.emit(CounterState { count: 0 });
        }
    }

    /// Generated `new()` sets the correct initial state.
    #[test]
    fn new_sets_initial_state() {
        let c = CounterCubit::new(CounterState { count: 7 });
        assert_eq!(c.state().count, 7);
    }

    /// `state()` returns the current state via the Cubit trait.
    #[test]
    fn state_returns_current_value() {
        let c = CounterCubit::new(CounterState { count: 0 });
        assert_eq!(c.state().count, 0);
    }

    /// `increment` increases the count through the generated `emit()`.
    #[test]
    fn increment_increases_count() {
        let mut c = CounterCubit::new(CounterState { count: 0 });
        c.increment();
        assert_eq!(c.state().count, 1);
    }

    /// Multiple increments accumulate correctly.
    #[test]
    fn multiple_increments_accumulate() {
        let mut c = CounterCubit::new(CounterState { count: 0 });
        for _ in 0..10 {
            c.increment();
        }
        assert_eq!(c.state().count, 10);
    }

    /// `decrement` reduces the count.
    #[test]
    fn decrement_reduces_count() {
        let mut c = CounterCubit::new(CounterState { count: 5 });
        c.decrement();
        assert_eq!(c.state().count, 4);
    }

    /// `reset` returns count to zero.
    #[test]
    fn reset_returns_to_zero() {
        let mut c = CounterCubit::new(CounterState { count: 100 });
        c.reset();
        assert_eq!(c.state().count, 0);
    }

    /// The generated struct satisfies `Cubit` and can be used as a trait object.
    #[test]
    fn satisfies_cubit_trait() {
        let c = CounterCubit::new(CounterState { count: 0 });
        fn accept(_: &dyn Cubit<State = CounterState>) {}
        accept(&c);
    }

    /// Mode A cubit can hold additional (non-state) fields alongside generated ones.
    #[cubit(state = CounterState)]
    pub struct CounterCubitWithExtra {
        pub step: i32,
    }

    impl CounterCubitWithExtra {
        pub fn step_up(&mut self) {
            let next = self.state().count + self.step;
            self.emit(CounterState { count: next });
        }
    }

    #[test]
    fn extra_fields_are_retained_and_accessible() {
        // `step` is an extra cubit field — generated new() takes it first.
        let mut c = CounterCubitWithExtra::new(5, CounterState { count: 0 });
        c.step_up();
        assert_eq!(c.state().count, 5);
    }
}

// ---------------------------------------------------------------------------
// Module: mode_b — generated state struct
// ---------------------------------------------------------------------------

mod mode_b {
    use super::*;

    /// Mode B: macro generates `ToggleCubitState { active: bool }`.
    #[cubit]
    pub struct ToggleCubit {
        #[state]
        pub active: bool,
    }

    impl ToggleCubit {
        pub fn toggle(&mut self) {
            let next = !self.state().active;
            self.emit(ToggleCubitState { active: next });
        }
    }

    /// Generated `new()` correctly initialises the generated state type.
    #[test]
    fn new_initialises_generated_state() {
        let c = ToggleCubit::new(ToggleCubitState { active: false });
        assert!(!c.state().active);
    }

    /// The generated State struct is public and constructible by callers.
    #[test]
    fn generated_state_struct_is_constructible() {
        let s = ToggleCubitState { active: true };
        assert!(s.active);
    }

    /// `toggle` flips the boolean state.
    #[test]
    fn toggle_flips_state() {
        let mut c = ToggleCubit::new(ToggleCubitState { active: false });
        c.toggle();
        assert!(c.state().active);
        c.toggle();
        assert!(!c.state().active);
    }

    /// Generated State struct derives `Clone`.
    #[test]
    fn generated_state_is_cloneable() {
        let s = ToggleCubitState { active: true };
        let cloned = s.clone();
        assert_eq!(s, cloned);
    }

    /// Generated State struct derives `Debug`.
    #[test]
    fn generated_state_is_debuggable() {
        let s = ToggleCubitState { active: false };
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("active"));
    }

    /// Mode B with multiple `#[state]` fields.
    #[cubit]
    pub struct UserCubit {
        #[state]
        pub name: String,
        #[state]
        pub age: u32,
    }

    #[test]
    fn multiple_state_fields_generate_correctly() {
        let initial = UserCubitState {
            name: "Alice".into(),
            age: 30,
        };
        let c = UserCubit::new(initial);
        assert_eq!(c.state().name, "Alice");
        assert_eq!(c.state().age, 30);
    }

    /// Mode B: non-`#[state]` fields stay on the cubit struct (not in state).
    #[cubit]
    pub struct SteppedCubit {
        #[state]
        pub count: i32,
        pub step: i32, // non-state — remains on cubit, not in generated state
    }

    impl SteppedCubit {
        pub fn advance(&mut self) {
            let next = self.state().count + self.step;
            self.emit(SteppedCubitState { count: next });
        }
    }

    #[test]
    fn non_state_fields_remain_on_cubit() {
        // `step` is a non-state cubit field — generated new() takes it first.
        let mut c = SteppedCubit::new(3, SteppedCubitState { count: 0 });
        c.advance();
        assert_eq!(c.state().count, 3);
    }

    #[test]
    fn non_state_field_not_in_generated_state() {
        // SteppedCubitState should only have `count` — if it had `step` this
        // would not compile (no such field).
        let s = SteppedCubitState { count: 42 };
        assert_eq!(s.count, 42);
    }
}

// ---------------------------------------------------------------------------
// Module: change_det — change-detection (shared across both modes)
// ---------------------------------------------------------------------------

mod change_det {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    pub struct S(i32);

    #[cubit(state = S)]
    pub struct Cubit1 {}

    /// Emitting the same value as the current state is a no-op.
    #[test]
    fn emit_same_value_is_noop() {
        let mut c = Cubit1::new(S(5));
        c.emit(S(5));
        assert_eq!(c.state(), &S(5));
        // Ensure it can still transition after a no-op.
        c.emit(S(6));
        assert_eq!(c.state(), &S(6));
    }

    /// Emitting a different value updates the state.
    #[test]
    fn emit_different_value_updates_state() {
        let mut c = Cubit1::new(S(0));
        c.emit(S(99));
        assert_eq!(c.state(), &S(99));
    }

    /// Rapid alternation between two values converges on the last one.
    #[test]
    fn rapid_alternation_converges() {
        let mut c = Cubit1::new(S(0));
        for i in 0..1000_i32 {
            c.emit(S(i % 2));
        }
        // 999 % 2 == 1
        assert_eq!(c.state(), &S(1));
    }
}

// ---------------------------------------------------------------------------
// Module: observers — on_change callback behaviour
// ---------------------------------------------------------------------------

mod observers {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[derive(Clone, PartialEq, Debug)]
    pub struct Val(i32);

    #[cubit(state = Val)]
    pub struct ObsCubit {}

    /// `on_change` callback fires when state transitions.
    #[test]
    fn on_change_fires_on_transition() {
        let mut c = ObsCubit::new(Val(0));
        let fired = Rc::new(Cell::new(false));
        let fired_clone = fired.clone();

        c.on_change(move |_| fired_clone.set(true));
        c.emit(Val(1));

        assert!(fired.get(), "on_change callback should have fired");
    }

    /// `on_change` does NOT fire when state is unchanged (change-detection).
    #[test]
    fn on_change_does_not_fire_on_noop_emit() {
        let mut c = ObsCubit::new(Val(5));
        let count = Rc::new(Cell::new(0_u32));
        let count_clone = count.clone();

        c.on_change(move |_| count_clone.set(count_clone.get() + 1));
        c.emit(Val(5)); // no-op
        c.emit(Val(5)); // no-op

        assert_eq!(count.get(), 0, "callback must not fire for no-op emits");
    }

    /// Multiple `on_change` callbacks all fire in registration order.
    #[test]
    fn multiple_observers_fire_in_order() {
        let mut c = ObsCubit::new(Val(0));
        let log = Rc::new(std::cell::RefCell::new(Vec::<i32>::new()));

        let log1 = log.clone();
        c.on_change(move |v| log1.borrow_mut().push(v.0 * 10));

        let log2 = log.clone();
        c.on_change(move |v| log2.borrow_mut().push(v.0 * 100));

        c.emit(Val(1));
        c.emit(Val(2));

        let recorded = log.borrow().clone();
        assert_eq!(recorded, vec![10, 100, 20, 200]);
    }

    /// `on_change` receives the new state value (not the old one).
    #[test]
    fn on_change_receives_new_state() {
        let mut c = ObsCubit::new(Val(0));
        let received = Rc::new(Cell::new(-1_i32));
        let recv_clone = received.clone();

        c.on_change(move |v| recv_clone.set(v.0));
        c.emit(Val(42));

        assert_eq!(received.get(), 42);
    }
}

// ---------------------------------------------------------------------------
// Module: suppressions — no_new and no_observers
// ---------------------------------------------------------------------------

mod suppressions {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    pub struct Config {
        pub value: String,
    }

    /// `no_new` — macro skips generating `new()`; developer writes their own.
    #[cubit(state = Config, no_new)]
    pub struct ConfigCubit {}

    impl ConfigCubit {
        /// Custom constructor with extra validation logic.
        pub fn with_default() -> Self {
            Self {
                __gloc_state: Config {
                    value: "default".into(),
                },
                __gloc_listeners: Vec::new(),
            }
        }
    }

    #[test]
    fn no_new_custom_constructor_works() {
        let c = ConfigCubit::with_default();
        assert_eq!(c.state().value, "default");
    }

    #[test]
    fn no_new_cubit_can_emit() {
        let mut c = ConfigCubit::with_default();
        c.emit(Config {
            value: "updated".into(),
        });
        assert_eq!(c.state().value, "updated");
    }

    /// `no_observers` — macro skips `on_change` and the listeners field.
    #[cubit(state = Config, no_observers)]
    pub struct LeanCubit {}

    #[test]
    fn no_observers_cubit_builds_and_transitions() {
        let mut c = LeanCubit::new(Config {
            value: "lean".into(),
        });
        c.emit(Config {
            value: "updated".into(),
        });
        assert_eq!(c.state().value, "updated");
    }

    /// Both suppressions together — developer writes new() and skips observers.
    #[cubit(state = Config, no_new, no_observers)]
    pub struct BareCubit {}

    impl BareCubit {
        pub fn bare() -> Self {
            Self {
                __gloc_state: Config {
                    value: "bare".into(),
                },
            }
        }
    }

    #[test]
    fn bare_cubit_works_with_both_suppressions() {
        let mut c = BareCubit::bare();
        assert_eq!(c.state().value, "bare");
        c.emit(Config {
            value: "modified".into(),
        });
        assert_eq!(c.state().value, "modified");
    }
}

// ---------------------------------------------------------------------------
// Module: injection — Dependency Inversion via trait objects
// ---------------------------------------------------------------------------

mod injection {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    pub struct Score(u32);

    #[cubit(state = Score)]
    pub struct ScoreCubit {}

    /// A function that only knows about `dyn Cubit` — framework-agnostic.
    fn add_points(cubit: &mut dyn Cubit<State = Score>, points: u32) {
        let next = cubit.state().0 + points;
        cubit.emit(Score(next));
    }

    #[test]
    fn mutable_trait_object_works() {
        let mut c = ScoreCubit::new(Score(0));
        add_points(&mut c, 10);
        add_points(&mut c, 5);
        assert_eq!(c.state().0, 15);
    }

    fn read_score(cubit: &dyn Cubit<State = Score>) -> u32 {
        cubit.state().0
    }

    #[test]
    fn shared_trait_object_works() {
        let c = ScoreCubit::new(Score(99));
        assert_eq!(read_score(&c), 99);
    }

    /// Mode B cubit also satisfies dyn Cubit.
    #[cubit]
    pub struct FlagCubit {
        #[state]
        pub on: bool,
    }

    #[test]
    fn mode_b_cubit_as_trait_object() {
        let c = FlagCubit::new(FlagCubitState { on: true });
        fn accept(_: &dyn Cubit<State = FlagCubitState>) {}
        accept(&c);
    }
}
