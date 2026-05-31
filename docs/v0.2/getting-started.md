# Getting Started — v0.2

[← Index](./index.md)

---

## Installation

```toml
[dependencies]
gloc = "0.2"
```

---

## Minimal example — reactor with on_change

```rust
use gloc::{reactor, reactor_state, Reactor};

#[reactor_state]
pub struct CounterState { pub count: i32 }

#[reactor(state = CounterState)]
pub struct CounterReactor {}

impl CounterReactor {
    pub fn increment(&mut self) {
        self.emit(CounterState { count: self.state().count + 1 });
    }
}

fn main() {
    let mut r = CounterReactor::new(CounterState { count: 0 });

    r.on_change(|old, new| {
        println!("{} → {}", old.count, new.count);
    });

    r.increment(); // prints: 0 → 1
    r.increment(); // prints: 1 → 2
}
```

---

## Sharing a reactor — GlocConsumer

When you need to share a reactor across components or threads, use
`GlocConsumer`. Construct it directly with an `Arc<Mutex<R>>` and a `GlocStream`:

```rust
use gloc::{reactor, reactor_state, Reactor, GlocConsumer, GlocStream};
use std::sync::{Arc, Mutex};

#[reactor_state]
pub struct CounterState { pub count: i32 }

#[reactor(state = CounterState)]
pub struct CounterReactor {}

impl CounterReactor {
    pub fn increment(&mut self) {
        self.emit(CounterState { count: self.state().count + 1 });
    }
}

fn main() {
    let initial  = CounterState { count: 0 };
    let reactor  = Arc::new(Mutex::new(CounterReactor::new(initial.clone())));
    let stream   = GlocStream::new(initial);
    let consumer = GlocConsumer::new(reactor, stream);

    // Multiple consumers share the same reactor
    let c1 = consumer.clone();
    let c2 = consumer.clone();

    c1.listen(|old, new| println!("{} → {}", old.count, new.count));
    c2.update(|r| r.increment()); // c1's listener prints: 0 → 1

    println!("final: {}", c1.state().count); // 1
}
```

---

## Using `subscribe()` for a read-only handle

```rust
let mut r = CounterReactor::new(CounterState { count: 0 });
let sub = r.subscribe();

r.increment();

println!("{}", sub.state().count); // 1
```

---

## Using `attach_listener()` with a typed struct

```rust
use gloc::GlocListener;

struct Logger;

impl GlocListener<CounterReactor> for Logger {
    fn on_transition(&self, old: &CounterState, new: &CounterState) {
        println!("{} → {}", old.count, new.count);
    }
}

let mut r = CounterReactor::new(CounterState { count: 0 });
r.attach_listener(Logger);
r.increment(); // prints: 0 → 1
```

---

## Global observer

To observe all reactors from one place, register a `GlocObserver` at startup:

```rust
use gloc::{GlocObserver, set_observer};

struct Logger;

impl GlocObserver for Logger {
    fn on_transition(&self, reactor: &str, old: &str, new: &str) {
        println!("[{reactor}] {old} → {new}");
    }
}

set_observer(Logger); // call once before any consumer updates
```

---

## Next steps

- [`#[reactor_state]` in depth](./cubit-state.md)
- [GlocConsumer](./consumer.md) — all consumer methods
- [GlocListener](./listener.md) — typed observers
- [GlocObserver](./observers.md) — global observer
- [Testing](./testing.md)
