# GlocConsumer — v0.2

[← Index](./index.md)

---

## What it is

`GlocConsumer<R>` is a shared handle to a reactor provided by
`GlocProvider` (adapter crate). It gives read and write access to the reactor without
owning it. Multiple consumers can coexist — mutations from any one are
immediately visible to all others.

```rust
let provider = GlocProvider::new(CounterReactor::new(CounterState { count: 0 }));
let consumer = provider.consumer();
```

---

## API reference

### `consumer.state() -> C::State`

Returns a clone of the current state.

```rust
println!("{}", consumer.state().count);
```

### `consumer.update(f: impl FnOnce(&mut C))`

Calls a closure that mutates the reactor. If the state changes, the
transition is propagated to the stream and all listeners are notified.

```rust
consumer.update(|c| c.increment());
consumer.update(|c| c.reset());
```

**How it works:**
1. Captures state before the closure runs
2. Runs the closure with `&mut C`
3. Captures state after the closure
4. If `old != new` → calls `stream.emit_transition(old, new)`

### `consumer.listen(f: impl Fn(&C::State, &C::State) + Send + 'static)`

Registers a closure listener. Receives `(&old, &new)` on every real
transition. No-op emits (same state) never fire listeners.

```rust
consumer.listen(|old, new| {
    println!("{} → {}", old.count, new.count);
});
```

### `consumer.attach_listener(listener: impl GlocListener<R> + Send + 'static)`

Attaches a typed `GlocListener` implementation. The trait-based version
of `listen` — useful when you have a service struct that should observe
transitions.

```rust
consumer.attach_listener(AnalyticsService::new());
```

### `consumer.stream() -> GlocStream<C::State>`

Returns the underlying stream. Pass it to framework adapters that need
raw reactive access.

---

## Multiple consumers

```rust
let provider = GlocProvider::new(CounterReactor::new(CounterState { count: 0 }));
let c1 = provider.consumer();
let c2 = provider.consumer();

c1.update(|c| c.increment());

assert_eq!(c2.state().count, 1); // c2 sees c1's mutation
```

---

## Cloning a consumer

Cloning is free — only the `Arc` reference count is incremented.
The clone shares the same reactor and stream.

```rust
let c1 = provider.consumer();
let c2 = c1.clone(); // shares everything
```

---

## Consumer vs direct reactor access

| | Direct reactor | `GlocConsumer` |
|---|---|---|
| Ownership | Exclusive | Shared — multiple consumers |
| Thread-safe | No | Yes (`Arc<Mutex<C>>`) |
| Notifies stream | Yes (built-in) | Yes (via `update()`) |
| Prop drilling | Required | Not needed — clone the consumer |
