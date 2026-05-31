# Testing — v0.2

[← Index](./index.md)

---

## Testing cubits — unchanged from v0.2

Reactors are still plain Rust structs. Nothing changes for basic reactor tests.

```rust
#[test]
fn increment_increases_count() {
    let mut reactor = CounterReactor::new(CounterState { count: 0 });
    reactor.increment();
    assert_eq!(reactor.state().count, 1);
}
```

---

## Testing `on_change` — now receives old and new

The `on_change` signature changed in v0.2. Update your test closures:

```rust
use std::sync::{Arc, Mutex};

#[test]
fn on_change_receives_old_and_new() {
    let mut reactor = CounterReactor::new(CounterState { count: 0 });
    let log: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));
    let log_clone = log.clone();

    reactor.on_change(move |old, new| {
        log_clone.lock().unwrap().push((old.count, new.count));
    });

    reactor.increment();
    reactor.increment();

    assert_eq!(*log.lock().unwrap(), vec![(0, 1), (1, 2)]);
}

#[test]
fn on_change_does_not_fire_on_noop() {
    let mut reactor = CounterReactor::new(CounterState { count: 0 });
    let fired = Arc::new(Mutex::new(false));
    let f = fired.clone();

    reactor.on_change(move |_, _| *f.lock().unwrap() = true);
    reactor.emit(CounterState { count: 0 }); // no-op

    assert!(!*fired.lock().unwrap());
}
```

---

## Testing `GlocStream`

```rust
use gloc_core::stream::GlocStream;

#[test]
fn stream_notifies_listeners() {
    let stream = GlocStream::new(0_i32);
    let log: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));
    let log_clone = log.clone();

    stream.listen(move |old, new| {
        log_clone.lock().unwrap().push((*old, *new));
    });

    stream.emit_transition(&0, &1);
    stream.emit_transition(&1, &2);

    assert_eq!(*log.lock().unwrap(), vec![(0, 1), (1, 2)]);
}

#[test]
fn subscription_sees_latest_state() {
    let stream = GlocStream::new(0_i32);
    let sub = stream.subscribe();

    stream.emit_transition(&0, &99);

    assert_eq!(sub.state(), 99);
}
```

---

## Testing `GlocProvider` (adapter crate) and `GlocConsumer`

```rust
#[test]
fn consumer_update_propagates_to_all_consumers() {
    let provider = GlocProvider::new(CounterReactor::new(CounterState { count: 0 }));
    let c1 = provider.consumer();
    let c2 = provider.consumer();

    c1.update(|c| c.increment());

    assert_eq!(c2.state().count, 1);
}

#[test]
fn consumer_listen_fires_on_transition() {
    let provider = GlocProvider::new(CounterReactor::new(CounterState { count: 0 }));
    let consumer = provider.consumer();
    let log: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(vec![]));
    let log_clone = log.clone();

    consumer.listen(move |_, new| log_clone.lock().unwrap().push(new.count));
    consumer.update(|c| c.increment());
    consumer.update(|c| c.increment());

    assert_eq!(*log.lock().unwrap(), vec![1, 2]);
}
```

---

## Testing `GlocListener`

```rust
use gloc_core::listener::GlocListener;

struct RecordingListener {
    log: Arc<Mutex<Vec<(i32, i32)>>>,
}

impl GlocListener<CounterReactor> for RecordingListener {
    fn on_transition(&self, old: &CounterState, new: &CounterState) {
        self.log.lock().unwrap().push((old.count, new.count));
    }
}

#[test]
fn attach_listener_records_transitions() {
    let provider = GlocProvider::new(CounterReactor::new(CounterState { count: 0 }));
    let consumer = provider.consumer();
    let log: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));

    consumer.attach_listener(RecordingListener { log: log.clone() });
    consumer.update(|c| c.increment());
    consumer.update(|c| c.increment());

    assert_eq!(*log.lock().unwrap(), vec![(0, 1), (1, 2)]);
}
```
