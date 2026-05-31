# GLOC v0.2 Documentation

> **Reactive Core** — `GlocStream`, `GlocConsumer`, `GlocListener`, `GlocObserver` and `#[reactor_state]`.

[← Back to main docs](../index.md)

---

## Contents

- [Overview](./overview.md) — what v0.2 delivers
- [Getting Started](./getting-started.md) — using the reactive layer
- [reactor_state macro](./cubit-state.md) — `#[reactor_state]` and `derive(...)`
- [GlocStream](./stream.md) — reactive state stream
- [GlocConsumer](./consumer.md) — reading and mutating a reactor
- [GlocListener](./listener.md) — trait-based transition observers
- [GlocObserver](./observers.md) — global transition interceptor
- [Testing](./testing.md) — testing the reactive layer
- [Migration from v0.1](./migration.md) — upgrade guide

---

## What's in v0.2

| Added | Description |
|---|---|
| `#[reactor_state]` | Auto-injects `Clone + PartialEq + Debug`; accepts extra derives via `derive(...)` |
| `GlocStream<S>` | Shared reactive state container — pure `std`, zero deps |
| `GlocSubscription<S>` | Read-only handle to a `GlocStream` |
| `GlocConsumer<R>` | Shared mutable handle — `GlocConsumer::new(Arc<Mutex<R>>, GlocStream<R::State>)` |
| `GlocListener<R>` | Trait for typed `old → new` transition observers |
| `GlocObserver` | Global observer — `set_observer`, `clear_observer`, `observer()` |
| `on_change(old, new)` | Receives both old and new state |
| `subscribe()` | Returns a `GlocSubscription` from any reactor |
| `attach_listener()` | Attaches a `GlocListener` impl to any reactor |

> **`GlocProvider` is not in core.** It is a framework-adapter concern that will live
> in `gloc-dioxus` and other adapter crates (planned v0.4). Use `GlocConsumer::new()`
> directly when you need shared mutable access.

---

## Quick example

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

let reactor  = Arc::new(Mutex::new(CounterReactor::new(CounterState { count: 0 })));
let stream   = GlocStream::new(CounterState { count: 0 });
let consumer = GlocConsumer::new(reactor, stream);

consumer.listen(|old, new| println!("{} → {}", old.count, new.count));
consumer.update(|r| r.increment()); // prints: 0 → 1
```

---

[← v0.1 docs](../v0.1/index.md) · [CHANGELOG](../CHANGELOG.md)
