//! Integration tests for the reactive layer:
//! `GlocStream`, `GlocSubscription`, `GlocProvider`, `GlocListener`.
//!
//! `GlocProvider` is constructed directly with an `Arc<Mutex<R>>` and a
//! `GlocStream` — providing shared reactor access and lifecycle management.

use gloc_core::{GlocListener, GlocProvider, GlocStream, Reactor};
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Shared fixture
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Debug)]
struct CounterState {
    pub count: i32,
}

impl CounterState {
    fn new(count: i32) -> Self {
        Self { count }
    }
}

struct CounterReactor {
    state: CounterState,
    stream: GlocStream<CounterState>,
}

impl CounterReactor {
    fn new(initial: i32) -> Self {
        let state = CounterState::new(initial);
        Self {
            stream: GlocStream::new(state.clone()),
            state,
        }
    }

    fn increment(&mut self) {
        let next = self.state().count + 1;
        self.emit(CounterState::new(next));
    }

    fn reset(&mut self) {
        self.emit(CounterState::new(0));
    }
}

impl Reactor for CounterReactor {
    type State = CounterState;

    fn state(&self) -> &CounterState {
        &self.state
    }

    fn emit(&mut self, next: CounterState) {
        if next != self.state {
            let old = self.state.clone();
            self.state = next.clone();
            self.stream.emit_transition(&old, &next);
        }
    }

    fn stream(&self) -> GlocStream<CounterState> {
        self.stream.clone()
    }
}

fn make_consumer(initial: i32) -> GlocProvider<CounterReactor> {
    let reactor = Arc::new(Mutex::new(CounterReactor::new(initial)));
    GlocProvider::new(reactor)
}

// ---------------------------------------------------------------------------
// GlocStream tests
// ---------------------------------------------------------------------------

mod stream_tests {
    use super::*;

    #[test]
    fn new_stores_initial_state() {
        let stream = GlocStream::new(CounterState::new(5));
        assert_eq!(stream.state().count, 5);
    }

    #[test]
    fn emit_transition_updates_state() {
        let stream = GlocStream::new(CounterState::new(0));
        stream.emit_transition(&CounterState::new(0), &CounterState::new(1));
        assert_eq!(stream.state().count, 1);
    }

    #[test]
    fn listen_fires_on_transition() {
        let stream = GlocStream::new(CounterState::new(0));
        let log: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));
        let log_clone = log.clone();

        let _h = stream.listen(move |old, new| {
            log_clone.lock().unwrap().push((old.count, new.count));
        });

        stream.emit_transition(&CounterState::new(0), &CounterState::new(1));
        stream.emit_transition(&CounterState::new(1), &CounterState::new(2));

        assert_eq!(*log.lock().unwrap(), vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn multiple_listeners_fire_in_order() {
        let stream = GlocStream::new(0_i32);
        let log: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(vec![]));

        let l1 = log.clone();
        let _h1 = stream.listen(move |_, new| l1.lock().unwrap().push(*new * 10));

        let l2 = log.clone();
        let _h2 = stream.listen(move |_, new| l2.lock().unwrap().push(*new * 100));

        stream.emit_transition(&0, &1);

        assert_eq!(*log.lock().unwrap(), vec![10, 100]);
    }

    #[test]
    fn clone_shares_state_and_listeners() {
        let stream = GlocStream::new(0_i32);
        let clone = stream.clone();

        stream.emit_transition(&0, &42);

        assert_eq!(clone.state(), 42);
    }

    #[test]
    fn subscribe_returns_subscription_with_current_state() {
        let stream = GlocStream::new(CounterState::new(7));
        let sub = stream.subscribe();
        assert_eq!(sub.state().count, 7);
    }

    #[test]
    fn subscription_sees_future_transitions() {
        let stream = GlocStream::new(0_i32);
        let sub = stream.subscribe();

        stream.emit_transition(&0, &99);

        assert_eq!(sub.state(), 99);
    }

    #[test]
    fn multiple_subscriptions_are_independent() {
        let stream = GlocStream::new(0_i32);
        let sub1 = stream.subscribe();
        let sub2 = stream.subscribe();

        stream.emit_transition(&0, &5);

        assert_eq!(sub1.state(), 5);
        assert_eq!(sub2.state(), 5);
    }

    #[test]
    fn subscription_can_register_listeners() {
        let stream = GlocStream::new(0_i32);
        let sub = stream.subscribe();
        let fired = Arc::new(Mutex::new(false));
        let f = fired.clone();

        let _h = sub.listen(move |_, _| *f.lock().unwrap() = true);
        stream.emit_transition(&0, &1);

        assert!(*fired.lock().unwrap());
    }
}

// ---------------------------------------------------------------------------
// GlocProvider tests
// ---------------------------------------------------------------------------

mod consumer_tests {
    use super::*;

    #[test]
    fn update_calls_reactor_method() {
        let consumer = make_consumer(0);
        consumer.update(|r| r.increment());
        assert_eq!(consumer.state().count, 1);
    }

    #[test]
    fn multiple_updates_accumulate() {
        let consumer = make_consumer(0);
        for _ in 0..5 {
            consumer.update(|r| r.increment());
        }
        assert_eq!(consumer.state().count, 5);
    }

    #[test]
    fn two_consumers_share_same_state() {
        let c1 = make_consumer(0);
        let c2 = c1.clone();

        c1.update(|r| r.increment());

        assert_eq!(c2.state().count, 1);
    }

    #[test]
    fn listen_fires_on_update() {
        let consumer = make_consumer(0);
        let log: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));
        let log_clone = log.clone();

        let _h = consumer.listen(move |old, new| {
            log_clone.lock().unwrap().push((old.count, new.count));
        });

        consumer.update(|r| r.increment());
        consumer.update(|r| r.increment());

        assert_eq!(*log.lock().unwrap(), vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn listen_does_not_fire_on_noop_update() {
        let consumer = make_consumer(0);
        let count = Arc::new(Mutex::new(0_u32));
        let c = count.clone();

        let _h = consumer.listen(move |_, _| *c.lock().unwrap() += 1);
        consumer.update(|r| r.reset()); // already at 0 — no-op

        assert_eq!(*count.lock().unwrap(), 0);
    }

    #[test]
    fn stream_from_consumer_reflects_updates() {
        let consumer = make_consumer(0);
        let stream = consumer.stream();
        let sub = stream.subscribe();

        consumer.update(|r| r.increment());

        assert_eq!(sub.state().count, 1);
    }
}

// ---------------------------------------------------------------------------
// GlocListener tests
// ---------------------------------------------------------------------------

mod listener_tests {
    use super::*;

    struct TransitionLogger {
        log: Arc<Mutex<Vec<(i32, i32)>>>,
    }

    impl GlocListener<CounterReactor> for TransitionLogger {
        fn on_transition(&self, old: &CounterState, new: &CounterState) {
            self.log.lock().unwrap().push((old.count, new.count));
        }
    }

    #[test]
    fn attach_listener_fires_on_transition() {
        let consumer = make_consumer(0);
        let log: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));

        let _h = consumer.attach_listener(TransitionLogger { log: log.clone() });
        consumer.update(|r| r.increment());
        consumer.update(|r| r.increment());

        assert_eq!(*log.lock().unwrap(), vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn attach_listener_does_not_fire_on_noop() {
        let consumer = make_consumer(0);
        let log: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));

        let _h = consumer.attach_listener(TransitionLogger { log: log.clone() });
        consumer.update(|r| r.reset()); // already 0 — no-op

        assert!(log.lock().unwrap().is_empty());
    }

    #[test]
    fn multiple_listeners_all_receive_transition() {
        let consumer = make_consumer(0);

        let log1: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));
        let log2: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));

        let _h1 = consumer.attach_listener(TransitionLogger { log: log1.clone() });
        let _h2 = consumer.attach_listener(TransitionLogger { log: log2.clone() });

        consumer.update(|r| r.increment());

        assert_eq!(*log1.lock().unwrap(), vec![(0, 1)]);
        assert_eq!(*log2.lock().unwrap(), vec![(0, 1)]);
    }
}
