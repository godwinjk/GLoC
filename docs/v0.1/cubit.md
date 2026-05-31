# Cubit — v0.1

A `Cubit` owns one slice of domain state and exposes methods that transition it.
It is the heart of GLOC — where all business logic lives.

---

## The `Cubit` trait

```rust
pub trait Cubit {
    type State: State;

    fn state(&self) -> &Self::State;
    fn emit(&mut self, next: Self::State);
}
```

Two methods. That is the entire interface.

| Method | What it does |
|---|---|
| `state()` | Returns a shared reference to the current state |
| `emit(next)` | Transitions to `next` — ignored if `next == current` |

---

## Anatomy of a cubit

```rust
use gloc::Cubit;

// 1. State — the data
#[derive(Clone, PartialEq, Debug)]
struct CounterState {
    pub count: i32,
}

// 2. Cubit — the logic
struct CounterCubit {
    state: CounterState,    // owns the state
}

// 3. Domain methods — the API
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

// 4. Trait impl — wires it all together
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

## Change detection

The guard in `emit()` is critical:

```rust
fn emit(&mut self, next: CounterState) {
    if next != self.state {   // only update if actually different
        self.state = next;
    }
}
```

Without it, calling `reset()` on an already-zero counter would still
trigger downstream reactions. With it, nothing happens — the state is
identical and the transition is silently discarded.

```rust
let mut cubit = CounterCubit::new(0);
cubit.reset(); // state is already 0 — no-op
cubit.reset(); // still 0 — still no-op
assert_eq!(cubit.state().count, 0);
```

---

## `CubitBase<S>` — the ready-made implementation

If your cubit needs no custom methods, `CubitBase<S>` gives you a
working cubit for any `State` type with zero boilerplate:

```rust
use gloc::{Cubit, CubitBase};

// Works with any State type
let mut cubit = CubitBase::new(String::from("idle"));
cubit.emit(String::from("loading"));
cubit.emit(String::from("success"));

assert_eq!(cubit.state(), "success");

// Change detection built in
cubit.emit(String::from("success")); // no-op
assert_eq!(cubit.state(), "success");
```

---

## Extra fields on the cubit

A cubit can hold fields that are not part of its state — implementation
details the caller does not need to know about:

```rust
struct SteppedCubit {
    state: CounterState,
    step: i32,          // private implementation detail
}

impl SteppedCubit {
    pub fn new(step: i32) -> Self {
        Self { state: CounterState { count: 0 }, step }
    }

    pub fn advance(&mut self) {
        let next = self.state().count + self.step;
        self.emit(CounterState { count: next });
    }
}
```

---

## Composing cubits

Cubits are independent units — they do not call each other directly.
If one cubit needs to react to another, wire them at the application layer:

```rust
let mut auth = AuthCubit::new();
let mut profile = ProfileCubit::new();

auth.login("user@example.com", "password");

// Application layer wires the two cubits
if let AuthState::Authenticated { user_id, .. } = auth.state() {
    profile.load(user_id.clone());
}
```

---

## Naming conventions

| Item | Convention | Example |
|---|---|---|
| State struct | `{Domain}State` | `CounterState`, `AuthState` |
| Cubit struct | `{Domain}Cubit` | `CounterCubit`, `AuthCubit` |
| State fields | `snake_case`, public | `pub count: i32` |
| Cubit methods | `snake_case`, verb | `fn increment()`, `fn load_user()` |

---

## Next steps

- [Testing](./testing.md) — test cubits without touching the UI
- [v0.2 Overview](../v0.2/overview.md) — let the `#[cubit]` macro write the boilerplate for you
