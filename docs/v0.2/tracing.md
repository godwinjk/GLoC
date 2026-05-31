# Tracing — v0.2

GLOC has built-in support for the [`tracing`](https://crates.io/crates/tracing)
crate. When enabled, every `emit()` call that actually transitions state
automatically logs the old and new values — with zero extra code in your reactors.

---

## What is tracing?

`tracing` is the standard structured logging and diagnostics framework for Rust.
It is async-aware, zero-cost when disabled, and integrates with tools like
`tokio-console`, Jaeger, and DataDog.

GLOC uses it to emit a `DEBUG` event on every real state transition.

---

## Enabling the feature

```toml
[dependencies]
gloc    = { version = "0.1", features = ["tracing"] }
tracing = "0.1"

# A subscriber to actually output the logs
tracing-subscriber = "0.3"
```

---

## Setting up a subscriber

`tracing` emits events but does not print them. You need a **subscriber**
to decide what to do with them. The simplest setup:

```rust
fn main() {
    // Initialise once at the start of your application
    tracing_subscriber::fmt::init();

    let mut counter = CounterReactor::new(CounterState { count: 0 });
    counter.increment();
    counter.increment();
}
```

Output:

```
DEBUG gloc: reactor="CounterReactor" old=CounterState { count: 0 } new=CounterState { count: 1 }
DEBUG gloc: reactor="CounterReactor" old=CounterState { count: 1 } new=CounterState { count: 2 }
```

---

## How it works

When `tracing` feature is enabled, the macro bakes a `tracing::debug!` call
into the generated `emit()` method — decided **at macro-expansion time**,
not at runtime:

```rust
// What the macro generates when `tracing` feature is ON
fn emit(&mut self, next: CounterState) {
    if next != self.__gloc_state {
        tracing::debug!(
            reactor = "CounterReactor",
            old   = ?self.__gloc_state,
            new   = ?next,
        );
        self.__gloc_state = next;
        // ... listeners
    }
}

// What the macro generates when `tracing` feature is OFF
fn emit(&mut self, next: CounterState) {
    if next != self.__gloc_state {
        self.__gloc_state = next;
        // ... listeners
    }
}
```

There is no runtime flag, no branch, no overhead when disabled.

---

## Zero cost when disabled

When the `tracing` feature is not enabled, no tracing code exists in the
compiled binary at all. The decision is made at compile time by the macro —
not at runtime by an `if` statement.

```
With tracing feature:    emit() contains tracing::debug!(...)
Without tracing feature: emit() contains no tracing code whatsoever
```

---

## Filtering log output

`tracing-subscriber` supports environment-based filtering:

```sh
# Show only GLOC debug logs
RUST_LOG=debug cargo run

# Show only errors from everything except GLOC
RUST_LOG=error,gloc=debug cargo run
```

Or in code:

```rust
use tracing_subscriber::EnvFilter;

tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

---

## Tracing vs observers

Both `tracing` and `on_change()` fire on state transitions. They serve different purposes:

| | `tracing` feature | `on_change()` observer |
|---|---|---|
| **Purpose** | Diagnostics / logging | Application logic |
| **Setup** | Feature flag in Cargo.toml | `reactor.on_change(...)` at runtime |
| **Output** | Structured log events | Your closure |
| **Cost when off** | Zero — code not generated | N/A — always present if registered |
| **Who reads it** | Developers / monitoring tools | Your application code |

Use **tracing** for developer observability.
Use **observers** for application-level reactions.

---

## Next steps

- [Observers](./observers.md) — application-level state reactions
- [Testing](./testing.md) — testing cubits with tracing enabled
