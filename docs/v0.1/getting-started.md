# Getting Started — v0.1

This guide takes you from zero to a working cubit in about 5 minutes.

---

## Prerequisites

- Rust stable toolchain (`rustup update stable`)
- A Cargo project (`cargo new my-app` or an existing workspace)

---

## Step 1 — Add the dependency

```toml
# Cargo.toml
[dependencies]
gloc = "0.1"
```

---

## Step 2 — Define your State

A state is a plain data struct that represents a snapshot of your domain
at a single point in time. It must derive three traits:

```rust
#[derive(Clone, PartialEq, Debug)]
struct CounterState {
    pub count: i32,
}
```

| Derive | Why it is required |
|---|---|
| `Clone` | Every state transition produces a new owned value |
| `PartialEq` | Change-detection — `emit()` compares old and new |
| `Debug` | Diagnostics, logging, and test output |

That's it. No explicit `impl State` — the blanket implementation handles it.

---

## Step 3 — Implement your Cubit

A cubit is a struct that owns the state and exposes domain methods.
Each method computes the next state and calls `self.emit()`.

```rust
use gloc::Cubit;

struct CounterCubit {
    state: CounterState,
}

impl CounterCubit {
    pub fn new(initial: i32) -> Self {
        Self {
            state: CounterState { count: initial },
        }
    }

    pub fn increment(&mut self) {
        let next = self.state().count + 1;
        self.emit(CounterState { count: next });
    }

    pub fn decrement(&mut self) {
        let next = self.state().count - 1;
        self.emit(CounterState { count: next });
    }

    pub fn reset(&mut self) {
        self.emit(CounterState { count: 0 });
    }
}

impl Cubit for CounterCubit {
    type State = CounterState;

    fn state(&self) -> &CounterState {
        &self.state
    }

    fn emit(&mut self, next: CounterState) {
        if next != self.state {
            self.state = next;
        }
    }
}
```

---

## Step 4 — Use it

```rust
fn main() {
    let mut counter = CounterCubit::new(0);

    println!("initial: {}", counter.state().count); // 0

    counter.increment();
    counter.increment();
    println!("after 2 increments: {}", counter.state().count); // 2

    counter.decrement();
    println!("after decrement: {}", counter.state().count); // 1

    counter.reset();
    println!("after reset: {}", counter.state().count); // 0
}
```

---

## Step 5 — Write a test

Cubits are plain Rust structs — no test harness, no mocks, no setup required.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use gloc::Cubit;

    #[test]
    fn increment_increases_count() {
        let mut cubit = CounterCubit::new(0);
        cubit.increment();
        assert_eq!(cubit.state().count, 1);
    }

    #[test]
    fn reset_returns_to_zero() {
        let mut cubit = CounterCubit::new(100);
        cubit.reset();
        assert_eq!(cubit.state().count, 0);
    }

    #[test]
    fn emit_same_state_is_noop() {
        let mut cubit = CounterCubit::new(0);
        cubit.reset(); // already at 0 — no change
        assert_eq!(cubit.state().count, 0);
    }
}
```

Run with:

```sh
cargo test
```

---

## Full example

```rust
use gloc::Cubit;

#[derive(Clone, PartialEq, Debug)]
struct CounterState {
    pub count: i32,
}

struct CounterCubit {
    state: CounterState,
}

impl CounterCubit {
    pub fn new(initial: i32) -> Self {
        Self { state: CounterState { count: initial } }
    }

    pub fn increment(&mut self) {
        let next = self.state().count + 1;
        self.emit(CounterState { count: next });
    }

    pub fn decrement(&mut self) {
        let next = self.state().count - 1;
        self.emit(CounterState { count: next });
    }

    pub fn reset(&mut self) {
        self.emit(CounterState { count: 0 });
    }
}

impl Cubit for CounterCubit {
    type State = CounterState;

    fn state(&self) -> &CounterState { &self.state }

    fn emit(&mut self, next: CounterState) {
        if next != self.state { self.state = next; }
    }
}

fn main() {
    let mut counter = CounterCubit::new(0);
    counter.increment();
    counter.increment();
    assert_eq!(counter.state().count, 2);
    println!("count: {}", counter.state().count);
}
```

---

## Next steps

- [State](./state.md) — designing state types for real domains
- [Cubit](./cubit.md) — advanced cubit patterns
- [Testing](./testing.md) — dependency injection and mocking
- [v0.2 Getting Started](../v0.2/getting-started.md) — eliminate the boilerplate with `#[cubit]`
