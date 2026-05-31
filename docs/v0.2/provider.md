# GlocProvider — moved to adapter crates

[← Index](./index.md)

---

`GlocProvider` (adapter crate) is a **framework-adapter concern** and is not part of `gloc-core`.

It will be implemented in adapter crates where a component tree or context
system is available:

- `gloc-dioxus` — planned v0.4
- `gloc-axum` — planned v0.4
- `gloc-bevy` — planned v0.4

---

## What to use instead

Use `GlocConsumer::new()` directly. It takes an `Arc<Mutex<R>>` and a
`GlocStream<R::State>` — this is exactly what `GlocProvider` (adapter crate) would wrap.

```rust
use gloc::{GlocConsumer, GlocStream};
use std::sync::{Arc, Mutex};

let reactor  = Arc::new(Mutex::new(MyReactor::new(initial.clone())));
let stream   = GlocStream::new(initial);
let consumer = GlocConsumer::new(reactor, stream);

// Clone to create additional consumers sharing the same reactor
let c2 = consumer.clone();
```

See [GlocConsumer](./consumer.md) for the full API.
