# GlocStream — v0.2

[← Index](./index.md)

---

## What it is

`GlocStream<S>` is the reactive backbone of every reactor. It is a shared,
observable state container that notifies all registered listeners
synchronously whenever the state transitions to a new value.

It is built entirely on `std` — `Arc<Mutex<_>>` provides thread-safe
shared ownership with no external dependencies.

---

## How it works

```
GlocStream<S>
  └── Arc<SharedState<S>>
        ├── current:   Mutex<S>              — the latest state
        └── listeners: Mutex<Vec<Listener>>  — registered callbacks
```

When `emit_transition(old, new)` is called:
1. Acquires the mutex and updates `current` to `new`
2. Releases the mutex
3. Calls every listener synchronously with `(&old, &new)`

The mutex is released before calling listeners — listeners can safely
call `stream.state()` without deadlocking.

---

## Cloning is cheap

Cloning a `GlocStream` increments the `Arc` reference count — it does
not copy state or listeners. All clones share the same underlying data.

```rust
let stream = GlocStream::new(0_i32);
let clone  = stream.clone();

stream.emit_transition(&0, &42);

assert_eq!(clone.state(), 42); // clone sees the update
```

---

## API reference

### `GlocStream::new(initial: S) -> Self`

Creates a new stream with the given initial state.

### `stream.state() -> S`

Returns a clone of the current state. Acquires and releases the mutex.

### `stream.emit_transition(old: &S, next: &S)`

Updates `current` to `next` and fires all listeners with `(old, next)`.
Called by the reactor's generated `emit()` — not typically called directly.

### `stream.listen(f: impl Fn(&S, &S) + Send + 'static)`

Registers a listener closure. Receives `(&old, &new)` on every transition.

### `stream.subscribe() -> GlocSubscription<S>`

Returns a lightweight read-only handle to this stream.

---

## `GlocSubscription<S>`

A `GlocSubscription` is a cloneable read-only view of a `GlocStream`.
Use it to pass state observation into framework adapters or background
tasks without giving them the ability to emit transitions.

```rust
let stream = GlocStream::new(0_i32);
let sub1   = stream.subscribe();
let sub2   = stream.subscribe(); // independent handle

stream.emit_transition(&0, &5);

assert_eq!(sub1.state(), 5);
assert_eq!(sub2.state(), 5);
```

---

## Accessing the stream from a reactor

Every `#[reactor]`-generated struct exposes `subscribe()`:

```rust
let mut reactor = CounterReactor::new(CounterState { count: 0 });
let sub = reactor.subscribe();

reactor.increment();

println!("{}", sub.state().count); // 1
```
