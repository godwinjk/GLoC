# State — v0.1

A `State` is an immutable snapshot of your domain's data at a single point in time.
It has no behaviour — no methods that mutate it, no side effects. It is just data.

---

## The `State` trait

```rust
pub trait State: Clone + PartialEq + std::fmt::Debug {}
```

GLOC's `State` trait is a **marker trait** with a blanket implementation.
Any type that satisfies `Clone + PartialEq + Debug` is automatically a `State`
— you never need to write `impl State for MyType {}`.

---

## Designing state types

### Struct state — the most common pattern

Use a struct when your state has multiple related fields.

```rust
#[derive(Clone, PartialEq, Debug)]
struct UserState {
    pub name: String,
    pub email: String,
    pub is_verified: bool,
}
```

### Enum state — for loading flows

Use an enum when your state represents distinct phases:

```rust
#[derive(Clone, PartialEq, Debug)]
enum FetchState<T> {
    Idle,
    Loading,
    Success(T),
    Failure(String),
}
```

This pattern maps cleanly to a UI:

```rust
match cubit.state() {
    FetchState::Idle       => render_empty(),
    FetchState::Loading    => render_spinner(),
    FetchState::Success(d) => render_data(d),
    FetchState::Failure(e) => render_error(e),
}
```

### Primitive state — simple cases

Any primitive works as state:

```rust
// bool — toggle
let mut cubit: CubitBase<bool> = CubitBase::new(false);
cubit.emit(true);

// String — status
let mut cubit: CubitBase<String> = CubitBase::new("idle".into());
cubit.emit("loading".into());

// Option<T> — nullable value
let mut cubit: CubitBase<Option<User>> = CubitBase::new(None);
cubit.emit(Some(user));
```

---

## State rules

### Rule 1 — State is always replaced, never mutated

You never modify a state in place. Every transition produces a **new value**:

```rust
// ❌ Wrong — mutating state directly
self.state.count += 1;

// ✅ Correct — produce a new state and emit it
let next = CounterState { count: self.state().count + 1 };
self.emit(next);
```

### Rule 2 — State must be cheap to clone

`emit()` clones state internally for change-detection. Keep state flat and
avoid large collections when possible. If you must include a large `Vec`,
consider wrapping it in `Arc` to make cloning cheap:

```rust
use std::sync::Arc;

#[derive(Clone, PartialEq, Debug)]
struct CatalogueState {
    pub items: Arc<Vec<Item>>,  // clone is O(1)
    pub selected: Option<usize>,
}
```

### Rule 3 — State must implement `PartialEq` honestly

Change-detection relies on `PartialEq`. If two states compare as equal but
are logically different, `emit()` will silently discard the transition.
Always derive `PartialEq` — only write a manual impl if you have a specific reason.

### Rule 4 — State carries no behaviour

State structs should have no `impl` methods beyond constructors and
format helpers. All behaviour lives on the cubit.

```rust
// ✅ Acceptable — a factory / display helper
impl CounterState {
    pub fn zero() -> Self { Self { count: 0 } }
}

// ❌ Wrong — behaviour belongs on the cubit
impl CounterState {
    pub fn increment(&mut self) { self.count += 1; }
}
```

---

## Real-world state examples

### Authentication

```rust
#[derive(Clone, PartialEq, Debug)]
enum AuthState {
    Unauthenticated,
    Authenticating,
    Authenticated { user_id: String, token: String },
    Error(String),
}
```

### Shopping cart

```rust
#[derive(Clone, PartialEq, Debug)]
struct CartItem {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub quantity: u32,
}

#[derive(Clone, PartialEq, Debug)]
struct CartState {
    pub items: Vec<CartItem>,
    pub discount_code: Option<String>,
}

impl CartState {
    pub fn total(&self) -> f64 {
        self.items.iter().map(|i| i.price * i.quantity as f64).sum()
    }
}
```

### Form validation

```rust
#[derive(Clone, PartialEq, Debug)]
struct LoginFormState {
    pub email: String,
    pub password: String,
    pub email_error: Option<String>,
    pub password_error: Option<String>,
    pub is_submitting: bool,
}
```

---

## Next steps

- [Cubit](./cubit.md) — implementing the cubit that owns this state
- [Testing](./testing.md) — testing state transitions in isolation
