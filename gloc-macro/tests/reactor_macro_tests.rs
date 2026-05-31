//! Integration tests for the `#[reactor]` attribute macro.
//!
//! These tests exercise the generated code at the Rust type and runtime level.
//! They are organised into modules that mirror the two modes and the shared
//! generated capabilities:
//!
//! - `mode_a`       — `#[reactor(state = SomeType)]` bring-your-own state
//! - `mode_b`       — `#[state]` field annotation, generated state struct
//! - `change_det`   — change-detection behaviour (shared across modes)
//! - `observers`    — `on_change` callback behaviour
//! - `suppressions` — `no_new` and `no_observers` flags
//! - `injection`    — dependency inversion via trait objects
//! - `dispatch`     — `events = E` argument and generated `dispatch()` method

use gloc::Reactor;
use gloc_macro::reactor;

// ---------------------------------------------------------------------------
// Module: mode_a — bring-your-own state type
// ---------------------------------------------------------------------------

mod mode_a {
    use super::*;

    /// State defined by the developer — entirely separate from the reactor.
    #[derive(Clone, PartialEq, Debug)]
    pub struct CounterState {
        pub count: i32,
    }

    /// Minimal Mode A reactor — empty body, macro provides everything.
    #[reactor(state = CounterState)]
    pub struct CounterReactor {}

    impl CounterReactor {
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
        let r = CounterReactor::new(CounterState { count: 7 });
        assert_eq!(r.state().count, 7);
    }

    /// `state()` returns the current state via the Reactor trait.
    #[test]
    fn state_returns_current_value() {
        let r = CounterReactor::new(CounterState { count: 0 });
        assert_eq!(r.state().count, 0);
    }

    /// `increment` increases the count through the generated `emit()`.
    #[test]
    fn increment_increases_count() {
        let mut r = CounterReactor::new(CounterState { count: 0 });
        r.increment();
        assert_eq!(r.state().count, 1);
    }

    /// Multiple increments accumulate correctly.
    #[test]
    fn multiple_increments_accumulate() {
        let mut r = CounterReactor::new(CounterState { count: 0 });
        for _ in 0..10 {
            r.increment();
        }
        assert_eq!(r.state().count, 10);
    }

    /// `decrement` reduces the count.
    #[test]
    fn decrement_reduces_count() {
        let mut r = CounterReactor::new(CounterState { count: 5 });
        r.decrement();
        assert_eq!(r.state().count, 4);
    }

    /// `reset` returns count to zero.
    #[test]
    fn reset_returns_to_zero() {
        let mut r = CounterReactor::new(CounterState { count: 100 });
        r.reset();
        assert_eq!(r.state().count, 0);
    }

    /// The generated struct satisfies `Reactor` and can be used as a trait object.
    #[test]
    fn satisfies_reactor_trait() {
        let r = CounterReactor::new(CounterState { count: 0 });
        fn accept(_: &dyn Reactor<State = CounterState>) {}
        accept(&r);
    }

    /// Mode A reactor can hold additional (non-state) fields alongside generated ones.
    #[reactor(state = CounterState)]
    pub struct CounterReactorWithExtra {
        pub step: i32,
    }

    impl CounterReactorWithExtra {
        pub fn step_up(&mut self) {
            let next = self.state().count + self.step;
            self.emit(CounterState { count: next });
        }
    }

    #[test]
    fn extra_fields_are_retained_and_accessible() {
        // `step` is an extra reactor field — generated new() takes it first.
        let mut r = CounterReactorWithExtra::new(5, CounterState { count: 0 });
        r.step_up();
        assert_eq!(r.state().count, 5);
    }
}

// ---------------------------------------------------------------------------
// Module: mode_b — generated state struct
// ---------------------------------------------------------------------------

mod mode_b {
    use super::*;

    /// Mode B: macro generates `ToggleReactorState { active: bool }`.
    #[reactor]
    pub struct ToggleReactor {
        #[state]
        pub active: bool,
    }

    impl ToggleReactor {
        pub fn toggle(&mut self) {
            let next = !self.state().active;
            self.emit(ToggleReactorState { active: next });
        }
    }

    /// Generated `new()` correctly initialises the generated state type.
    #[test]
    fn new_initialises_generated_state() {
        let r = ToggleReactor::new(ToggleReactorState { active: false });
        assert!(!r.state().active);
    }

    /// The generated State struct is public and constructible by callers.
    #[test]
    fn generated_state_struct_is_constructible() {
        let s = ToggleReactorState { active: true };
        assert!(s.active);
    }

    /// `toggle` flips the boolean state.
    #[test]
    fn toggle_flips_state() {
        let mut r = ToggleReactor::new(ToggleReactorState { active: false });
        r.toggle();
        assert!(r.state().active);
        r.toggle();
        assert!(!r.state().active);
    }

    /// Generated State struct derives `Clone`.
    #[test]
    fn generated_state_is_cloneable() {
        let s = ToggleReactorState { active: true };
        let cloned = s.clone();
        assert_eq!(s, cloned);
    }

    /// Generated State struct derives `Debug`.
    #[test]
    fn generated_state_is_debuggable() {
        let s = ToggleReactorState { active: false };
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("active"));
    }

    /// Mode B with multiple `#[state]` fields.
    #[reactor]
    pub struct UserReactor {
        #[state]
        pub name: String,
        #[state]
        pub age: u32,
    }

    #[test]
    fn multiple_state_fields_generate_correctly() {
        let initial = UserReactorState {
            name: "Alice".into(),
            age: 30,
        };
        let r = UserReactor::new(initial);
        assert_eq!(r.state().name, "Alice");
        assert_eq!(r.state().age, 30);
    }

    /// Mode B: non-`#[state]` fields stay on the reactor struct (not in state).
    #[reactor]
    pub struct SteppedReactor {
        #[state]
        pub count: i32,
        pub step: i32, // non-state — remains on reactor, not in generated state
    }

    impl SteppedReactor {
        pub fn advance(&mut self) {
            let next = self.state().count + self.step;
            self.emit(SteppedReactorState { count: next });
        }
    }

    #[test]
    fn non_state_fields_remain_on_reactor() {
        // `step` is a non-state reactor field — generated new() takes it first.
        let mut r = SteppedReactor::new(3, SteppedReactorState { count: 0 });
        r.advance();
        assert_eq!(r.state().count, 3);
    }

    #[test]
    fn non_state_field_not_in_generated_state() {
        // SteppedReactorState should only have `count` — if it had `step` this
        // would not compile (no such field).
        let s = SteppedReactorState { count: 42 };
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

    #[reactor(state = S)]
    pub struct Reactor1 {}

    /// Emitting the same value as the current state is a no-op.
    #[test]
    fn emit_same_value_is_noop() {
        let mut r = Reactor1::new(S(5));
        r.emit(S(5));
        assert_eq!(r.state(), &S(5));
        // Ensure it can still transition after a no-op.
        r.emit(S(6));
        assert_eq!(r.state(), &S(6));
    }

    /// Emitting a different value updates the state.
    #[test]
    fn emit_different_value_updates_state() {
        let mut r = Reactor1::new(S(0));
        r.emit(S(99));
        assert_eq!(r.state(), &S(99));
    }

    /// Rapid alternation between two values converges on the last one.
    #[test]
    fn rapid_alternation_converges() {
        let mut r = Reactor1::new(S(0));
        for i in 0..1000_i32 {
            r.emit(S(i % 2));
        }
        // 999 % 2 == 1
        assert_eq!(r.state(), &S(1));
    }
}

// ---------------------------------------------------------------------------
// Module: observers — on_change callback behaviour
// ---------------------------------------------------------------------------

mod observers {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, PartialEq, Debug)]
    pub struct Val(i32);

    #[reactor(state = Val)]
    pub struct ObsReactor {}

    /// `on_change` callback fires when state transitions.
    #[test]
    fn on_change_fires_on_transition() {
        let mut r = ObsReactor::new(Val(0));
        let fired = Arc::new(Mutex::new(false));
        let fired_clone = fired.clone();

        r.on_change(move |_old, _new| *fired_clone.lock().unwrap() = true);
        r.emit(Val(1));

        assert!(
            *fired.lock().unwrap(),
            "on_change callback should have fired"
        );
    }

    /// `on_change` does NOT fire when state is unchanged (change-detection).
    #[test]
    fn on_change_does_not_fire_on_noop_emit() {
        let mut r = ObsReactor::new(Val(5));
        let count = Arc::new(Mutex::new(0_u32));
        let count_clone = count.clone();

        r.on_change(move |_old, _new| *count_clone.lock().unwrap() += 1);
        r.emit(Val(5)); // no-op
        r.emit(Val(5)); // no-op

        assert_eq!(
            *count.lock().unwrap(),
            0,
            "callback must not fire for no-op emits"
        );
    }

    /// Multiple `on_change` callbacks all fire in registration order.
    #[test]
    fn multiple_observers_fire_in_order() {
        let mut r = ObsReactor::new(Val(0));
        let log: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));

        let log1 = log.clone();
        r.on_change(move |_old, v| log1.lock().unwrap().push(v.0 * 10));

        let log2 = log.clone();
        r.on_change(move |_old, v| log2.lock().unwrap().push(v.0 * 100));

        r.emit(Val(1));
        r.emit(Val(2));

        assert_eq!(*log.lock().unwrap(), vec![10, 100, 20, 200]);
    }

    /// `on_change` receives both old and new state values.
    #[test]
    fn on_change_receives_old_and_new_state() {
        let mut r = ObsReactor::new(Val(0));
        let transitions: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(Vec::new()));
        let trans_clone = transitions.clone();

        r.on_change(move |old, new| trans_clone.lock().unwrap().push((old.0, new.0)));
        r.emit(Val(42));

        assert_eq!(*transitions.lock().unwrap(), vec![(0, 42)]);
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
    #[reactor(state = Config, no_new)]
    pub struct ConfigReactor {}

    impl ConfigReactor {
        /// Custom constructor with extra validation logic.
        pub fn with_default() -> Self {
            Self {
                __gloc_state: Config {
                    value: "default".into(),
                },
                __gloc_stream: ::gloc::GlocStream::new(Config {
                    value: "default".into(),
                }),
            }
        }
    }

    #[test]
    fn no_new_custom_constructor_works() {
        let r = ConfigReactor::with_default();
        assert_eq!(r.state().value, "default");
    }

    #[test]
    fn no_new_reactor_can_emit() {
        let mut r = ConfigReactor::with_default();
        r.emit(Config {
            value: "updated".into(),
        });
        assert_eq!(r.state().value, "updated");
    }

    /// `no_observers` — macro skips `on_change` and the stream field.
    #[reactor(state = Config, no_observers)]
    pub struct LeanReactor {}

    #[test]
    fn no_observers_reactor_builds_and_transitions() {
        let mut r = LeanReactor::new(Config {
            value: "lean".into(),
        });
        r.emit(Config {
            value: "updated".into(),
        });
        assert_eq!(r.state().value, "updated");
    }

    /// Both suppressions together — developer writes new() and skips observers.
    #[reactor(state = Config, no_new, no_observers)]
    pub struct BareReactor {}

    impl BareReactor {
        pub fn bare() -> Self {
            Self {
                __gloc_state: Config {
                    value: "bare".into(),
                },
            }
        }
    }

    #[test]
    fn bare_reactor_works_with_both_suppressions() {
        let mut r = BareReactor::bare();
        assert_eq!(r.state().value, "bare");
        r.emit(Config {
            value: "modified".into(),
        });
        assert_eq!(r.state().value, "modified");
    }
}

// ---------------------------------------------------------------------------
// Module: injection — Dependency Inversion via trait objects
// ---------------------------------------------------------------------------

mod injection {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    pub struct Score(u32);

    #[reactor(state = Score)]
    pub struct ScoreReactor {}

    /// A function that only knows about `dyn Reactor` — framework-agnostic.
    fn add_points(reactor: &mut dyn Reactor<State = Score>, points: u32) {
        let next = reactor.state().0 + points;
        reactor.emit(Score(next));
    }

    #[test]
    fn mutable_trait_object_works() {
        let mut r = ScoreReactor::new(Score(0));
        add_points(&mut r, 10);
        add_points(&mut r, 5);
        assert_eq!(r.state().0, 15);
    }

    fn read_score(reactor: &dyn Reactor<State = Score>) -> u32 {
        reactor.state().0
    }

    #[test]
    fn shared_trait_object_works() {
        let r = ScoreReactor::new(Score(99));
        assert_eq!(read_score(&r), 99);
    }

    /// Mode B reactor also satisfies dyn Reactor.
    #[reactor]
    pub struct FlagReactor {
        #[state]
        pub on: bool,
    }

    #[test]
    fn mode_b_reactor_as_trait_object() {
        let r = FlagReactor::new(FlagReactorState { on: true });
        fn accept(_: &dyn Reactor<State = FlagReactorState>) {}
        accept(&r);
    }
}

// ---------------------------------------------------------------------------
// Module: dispatch — events = E argument and generated dispatch() method
// ---------------------------------------------------------------------------

mod dispatch {
    use super::*;

    // --- Mode A with events ---

    #[derive(Clone, PartialEq, Debug)]
    pub struct S(i32);

    #[derive(Debug)]
    pub enum Ev {
        Inc,
        Dec,
        Set(i32),
    }

    #[reactor(state = S, events = Ev)]
    pub struct R {}

    impl R {
        /// Direct method — coexists with dispatch.
        pub fn increment(&mut self) {
            self.emit(S(self.state().0 + 1));
        }

        /// Event handler — called by dispatch().
        fn on_event(&mut self, event: Ev) {
            match event {
                Ev::Inc => self.emit(S(self.state().0 + 1)),
                Ev::Dec => self.emit(S(self.state().0 - 1)),
                Ev::Set(n) => self.emit(S(n)),
            }
        }
    }

    #[test]
    fn dispatch_increment() {
        let mut r = R::new(S(0));
        r.dispatch(Ev::Inc);
        assert_eq!(r.state(), &S(1));
    }

    #[test]
    fn dispatch_decrement() {
        let mut r = R::new(S(10));
        r.dispatch(Ev::Dec);
        assert_eq!(r.state(), &S(9));
    }

    #[test]
    fn dispatch_with_payload() {
        let mut r = R::new(S(0));
        r.dispatch(Ev::Set(42));
        assert_eq!(r.state(), &S(42));
    }

    /// change-detection still fires through dispatch — redundant emit is a no-op.
    #[test]
    fn dispatch_no_op_same_state() {
        let mut r = R::new(S(5));
        let fired = std::sync::Arc::new(std::sync::Mutex::new(false));
        let f = fired.clone();
        r.on_change(move |_, _| *f.lock().unwrap() = true);

        r.dispatch(Ev::Set(5)); // same value — no-op

        assert!(
            !*fired.lock().unwrap(),
            "on_change must not fire for a no-op dispatch"
        );
    }

    /// Direct method and dispatch work on the same reactor without conflict.
    #[test]
    fn direct_and_dispatch_coexist() {
        let mut r = R::new(S(0));
        r.increment(); // direct → 1
        r.dispatch(Ev::Inc); // dispatch → 2
        r.dispatch(Ev::Set(10)); // dispatch with payload → 10
        r.increment(); // direct → 11
        assert_eq!(r.state(), &S(11));
    }

    /// dispatch() result propagates through on_change listeners.
    #[test]
    fn dispatch_fires_on_change() {
        use std::sync::{Arc, Mutex};
        let mut r = R::new(S(0));
        let log: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));
        let log_c = log.clone();
        r.on_change(move |old, new| log_c.lock().unwrap().push((old.0, new.0)));

        r.dispatch(Ev::Inc); // 0 → 1
        r.dispatch(Ev::Set(5)); // 1 → 5
        r.dispatch(Ev::Inc); // 5 → 6

        assert_eq!(*log.lock().unwrap(), vec![(0, 1), (1, 5), (5, 6)]);
    }

    /// Reactor with events still satisfies the Reactor trait object.
    #[test]
    fn dispatch_reactor_satisfies_trait() {
        let r = R::new(S(0));
        fn accept(_: &dyn Reactor<State = S>) {}
        accept(&r);
    }

    // --- Mode B with events ---

    #[derive(Debug)]
    pub enum ToggleEv {
        Toggle,
        SetOn,
        SetOff,
    }

    #[reactor(events = ToggleEv)]
    pub struct ToggleReactor {
        #[state]
        pub active: bool,
    }

    impl ToggleReactor {
        fn on_event(&mut self, event: ToggleEv) {
            match event {
                ToggleEv::Toggle => self.emit(ToggleReactorState {
                    active: !self.state().active,
                }),
                ToggleEv::SetOn => self.emit(ToggleReactorState { active: true }),
                ToggleEv::SetOff => self.emit(ToggleReactorState { active: false }),
            }
        }
    }

    #[test]
    fn mode_b_dispatch_works() {
        let mut r = ToggleReactor::new(ToggleReactorState { active: false });
        r.dispatch(ToggleEv::Toggle);
        assert!(r.state().active);
        r.dispatch(ToggleEv::Toggle);
        assert!(!r.state().active);
        r.dispatch(ToggleEv::SetOn);
        assert!(r.state().active);
        r.dispatch(ToggleEv::SetOff);
        assert!(!r.state().active);
    }
}
