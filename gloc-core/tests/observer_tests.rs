//! Integration tests for [`GlocObserver`] — the global transition interceptor.
//!
//! `on_transition` is fired by `GlocProvider::update()` — tested directly here.
//! `on_create` and `on_close` are framework-adapter lifecycle hooks fired by
//! `GlocProvider`. `on_close` can also be triggered via `GlocProvider::release()`.
//!
//! Every test runs serially to avoid global-observer state leakage.

use gloc_core::{
    clear_observer, observer, set_observer, GlocObserver, GlocProvider, GlocStream, Reactor,
};
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Shared fixture
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Debug)]
struct S(i32);

struct R {
    state: S,
}

impl R {
    fn new(v: i32) -> Self {
        Self { state: S(v) }
    }

    fn inc(&mut self) {
        self.emit(S(self.state.0 + 1));
    }
}

impl Reactor for R {
    type State = S;

    fn state(&self) -> &S {
        &self.state
    }

    fn emit(&mut self, next: S) {
        if next != self.state {
            self.state = next;
        }
    }
}

/// Builds a `GlocProvider<R>` directly — no Provider needed.
fn make_consumer(initial: i32) -> GlocProvider<R> {
    let reactor = Arc::new(Mutex::new(R::new(initial)));
    let stream = GlocStream::new(S(initial));
    GlocProvider::new(reactor, stream)
}

// ---------------------------------------------------------------------------
// Recording observer — collects events for assertions
// ---------------------------------------------------------------------------

#[derive(Default)]
struct RecordingInner {
    transitions: Mutex<Vec<(String, String, String)>>,
}

#[derive(Clone)]
struct RecordingObserver(Arc<RecordingInner>);

impl RecordingObserver {
    fn new() -> Self {
        Self(Arc::new(RecordingInner::default()))
    }
}

impl GlocObserver for RecordingObserver {
    fn on_transition(&self, reactor: &str, old: &str, new: &str) {
        self.0.transitions.lock().unwrap().push((
            reactor.to_string(),
            old.to_string(),
            new.to_string(),
        ));
    }
}

// ---------------------------------------------------------------------------
// Helper — fresh recording observer for a test scope, clears after
// ---------------------------------------------------------------------------

fn with_observer<F>(f: F)
where
    F: FnOnce(&RecordingObserver),
{
    clear_observer();
    let rec = RecordingObserver::new();
    let handle = rec.clone();
    set_observer(rec);
    f(&handle);
    clear_observer();
}

// ---------------------------------------------------------------------------
// Tests — registry (set / clear / get)
// ---------------------------------------------------------------------------

#[serial_test::serial]
#[test]
fn no_observer_returns_none() {
    clear_observer();
    assert!(observer().is_none());
}

#[serial_test::serial]
#[test]
fn set_observer_makes_observer_available() {
    clear_observer();
    assert!(observer().is_none());
    set_observer(RecordingObserver::new());
    assert!(observer().is_some());
    clear_observer();
}

#[serial_test::serial]
#[test]
fn set_observer_replaces_previous_observer() {
    clear_observer();
    let rec1 = RecordingObserver::new();
    let rec2 = RecordingObserver::new();

    set_observer(rec1.clone());
    set_observer(rec2.clone()); // replaces rec1

    let consumer = make_consumer(0);
    consumer.update(|r| r.inc());

    // rec2 received the event; rec1 did not
    assert_eq!(rec2.0.transitions.lock().unwrap().len(), 1);
    assert_eq!(rec1.0.transitions.lock().unwrap().len(), 0);

    clear_observer();
}

// ---------------------------------------------------------------------------
// Tests — no observer set
// ---------------------------------------------------------------------------

#[serial_test::serial]
#[test]
fn no_observer_no_panic_on_consumer_update() {
    clear_observer();
    let consumer = make_consumer(0);
    consumer.update(|r| r.inc());
    assert_eq!(consumer.state().0, 1);
}

// ---------------------------------------------------------------------------
// Tests — on_transition
// ---------------------------------------------------------------------------

#[serial_test::serial]
#[test]
fn on_transition_fires_on_consumer_update() {
    with_observer(|rec| {
        let consumer = make_consumer(0);
        consumer.update(|r| r.inc());

        let transitions = rec.0.transitions.lock().unwrap();
        assert_eq!(transitions.len(), 1);
        let (name, old, new) = &transitions[0];
        assert!(name.contains('R'), "got: {name}");
        assert!(old.contains('0'), "old should contain 0, got: {old}");
        assert!(new.contains('1'), "new should contain 1, got: {new}");
    });
}

#[serial_test::serial]
#[test]
fn on_transition_fires_for_each_real_transition() {
    with_observer(|rec| {
        let consumer = make_consumer(0);
        consumer.update(|r| r.inc()); // 0 → 1
        consumer.update(|r| r.inc()); // 1 → 2
        consumer.update(|r| r.inc()); // 2 → 3
        assert_eq!(rec.0.transitions.lock().unwrap().len(), 3);
    });
}

#[serial_test::serial]
#[test]
fn on_transition_does_not_fire_on_noop_update() {
    with_observer(|rec| {
        let consumer = make_consumer(5);
        consumer.update(|r| r.emit(S(5))); // no-op
        assert_eq!(rec.0.transitions.lock().unwrap().len(), 0);
    });
}

#[serial_test::serial]
#[test]
fn on_transition_receives_correct_old_and_new_strings() {
    with_observer(|rec| {
        let consumer = make_consumer(10);
        consumer.update(|r| r.inc());

        let transitions = rec.0.transitions.lock().unwrap();
        let (_, old, new) = &transitions[0];
        assert_eq!(old, "S(10)");
        assert_eq!(new, "S(11)");
    });
}

#[serial_test::serial]
#[test]
fn on_transition_receives_correct_reactor_name() {
    with_observer(|rec| {
        let consumer = make_consumer(0);
        consumer.update(|r| r.inc());

        let transitions = rec.0.transitions.lock().unwrap();
        let (name, _, _) = &transitions[0];
        // type_name returns something like "observer_tests::R"
        assert!(
            name.ends_with('R'),
            "expected name ending in R, got: {name}"
        );
    });
}

#[serial_test::serial]
#[test]
fn on_transition_fires_from_multiple_consumers() {
    with_observer(|rec| {
        let c1 = make_consumer(0);
        let c2 = c1.clone();

        c1.update(|r| r.inc()); // 0 → 1
        c2.update(|r| r.inc()); // 1 → 2

        assert_eq!(rec.0.transitions.lock().unwrap().len(), 2);
    });
}
