# GLOC v0.2 — Overview

> **Reactive Core** — a proper reactive system built on pure `std`.

[← Index](./index.md)

---

## What v0.2 delivers

### 1. `#[reactor_state]` macro

Eliminates the one remaining piece of boilerplate — the
`#[derive(Clone, PartialEq, Debug)]` line on every state struct.

```rust
// Before — user writes this every time
#[derive(Clone, PartialEq, Debug)]
struct CounterState { pub count: i32 }

// After — one attribute, derives are automatic
#[reactor_state]
struct CounterState { pub count: i32 }

// With extras
#[reactor_state(derive(Hash, Eq))]
struct CounterState { pub count: i32 }
```

### 2. Reactive layer — `GlocStream`, `GlocConsumer`, `GlocListener`

A proper reactive system replaces the previous callback list.
Every reactor now has a built-in `GlocStream` — a shared, observable
state container backed by `Arc<Mutex<_>>`.

```
Before v0.2                    v0.2
──────────────                 ──────────────────────────────
Vec<Box<dyn Fn(&S)>>    →      GlocStream<S>
on_change(|new| …)      →      on_change(|old, new| …)
(no consumer)           →      GlocConsumer<R>
(no listener trait)     →      GlocListener<R>
(no subscribe)          →      subscribe() → GlocSubscription<S>
```

### 3. `GlocObserver` — global transition interceptor

Register one observer at startup to receive every state transition
across all reactors in the application.

```rust
use gloc::GlocObserver;

struct AppLogger;

impl GlocObserver for AppLogger {
    fn on_transition(&self, reactor: &str, old: &str, new: &str) {
        println!("[{reactor}] {old} → {new}");
    }
}

gloc::set_observer(AppLogger);
```

---

## Architectural note — no GlocProvider in core

`GlocProvider` (adapter crate) (context-tree injection) is a framework concern, not a
core concern. `gloc-core` has no UI tree, no component lifecycle, and no
concept of "scope". Framework adapters (`gloc-dioxus`, etc.) will provide
their own `GlocProvider` (adapter crate) built on top of `GlocConsumer::new()`.

Use `GlocConsumer::new(Arc<Mutex<R>>, GlocStream<R::State>)` directly
when you need shared mutable access to a reactor.

---

## Zero new dependencies

The entire reactive layer is built on `std` only:

| Component | Backed by |
|---|---|
| `GlocStream<S>` | `Arc<SharedState<S>>` — inner uses `Mutex` |
| `GlocConsumer<R>` | `Arc<Mutex<R>>` + `GlocStream<R::State>` |
| `GlocListener<R>` | Plain trait — no storage needed |
| `GlocObserver` | `OnceLock<RwLock<Option<Arc<dyn GlocObserver>>>>` |

---

## What comes next

- [Getting Started](./getting-started.md) — using the reactive layer
- [reactor_state macro](./cubit-state.md) — `#[reactor_state]` in depth
- [GlocStream](./stream.md) — reactive state stream
- [GlocConsumer](./consumer.md) — reading and mutating
- [GlocListener](./listener.md) — typed transition observers
- [GlocObserver](./observers.md) — global observer
- [Testing](./testing.md) — testing the reactive layer
- [Migration from v0.1](./migration.md) — upgrade guide
