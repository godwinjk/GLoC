# Mode A — Bring Your Own State

Mode A is activated by supplying `state = SomeType` inside the attribute:

```rust
#[reactor(state = SomeType)]
pub struct MyReactor {}
```

Use Mode A when:
- You already have a state type defined elsewhere
- Your state type has custom methods or complex logic
- You want full control over state naming and structure
- Your state is shared between multiple cubits

---

## Basic usage

```rust
use gloc::{reactor, Reactor};

#[derive(Clone, PartialEq, Debug)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub username: Option<String>,
}

#[reactor(state = AuthState)]
pub struct AuthCubit {}

impl AuthCubit {
    pub fn login(&mut self, username: String) {
        self.emit(AuthState {
            is_authenticated: true,
            username: Some(username),
        });
    }

    pub fn logout(&mut self) {
        self.emit(AuthState {
            is_authenticated: false,
            username: None,
        });
    }
}

// Usage
let mut auth = AuthCubit::new(AuthState {
    is_authenticated: false,
    username: None,
});

auth.login("alice".into());
assert!(auth.state().is_authenticated);
assert_eq!(auth.state().username, Some("alice".into()));

auth.logout();
assert!(!auth.state().is_authenticated);
```

---

## Extra fields on the reactor

Mode A cubits can hold fields that are not part of the state.
These become **additional parameters in the generated `new()`**,
appearing before `initial`, in declaration order.

```rust
#[derive(Clone, PartialEq, Debug)]
pub struct CounterState { pub count: i32 }

#[reactor(state = CounterState)]
pub struct CounterReactor {
    pub step: i32,      // extra field — not state
    pub max: i32,       // extra field — not state
}

impl CounterReactor {
    pub fn advance(&mut self) {
        let next = (self.state().count + self.step).min(self.max);
        self.emit(CounterState { count: next });
    }
}

// Generated: pub fn new(step: i32, max: i32, initial: CounterState) -> Self
let mut reactor = CounterReactor::new(5, 100, CounterState { count: 0 });
reactor.advance(); // count = 5
reactor.advance(); // count = 10
```

---

## State with custom methods

Mode A lets you add helper methods to the state type without any restrictions:

```rust
#[derive(Clone, PartialEq, Debug)]
pub struct CartState {
    pub items: Vec<String>,
    pub discount: f64,
}

impl CartState {
    pub fn empty() -> Self {
        Self { items: vec![], discount: 0.0 }
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

#[reactor(state = CartState)]
pub struct CartReactor {}

impl CartReactor {
    pub fn add_item(&mut self, item: String) {
        let mut next = self.state().clone();
        next.items.push(item);
        self.emit(next);
    }

    pub fn apply_discount(&mut self, pct: f64) {
        let mut next = self.state().clone();
        next.discount = pct;
        self.emit(next);
    }

    pub fn clear(&mut self) {
        self.emit(CartState::empty());
    }
}

let mut cart = CartReactor::new(CartState::empty());
cart.add_item("Book".into());
cart.add_item("Pen".into());
assert_eq!(cart.state().item_count(), 2);
```

---

## Shared state across cubits

Because Mode A references an externally defined type, the same state type
can be used by multiple cubits:

```rust
#[derive(Clone, PartialEq, Debug)]
pub struct UserPreferences {
    pub theme: String,
    pub language: String,
}

// Both cubits share the same state type
#[reactor(state = UserPreferences)]
pub struct SettingsCubit {}

#[reactor(state = UserPreferences)]
pub struct ProfileCubit {}
```

---

## Suppression options

### `no_new` — write your own constructor

```rust
#[reactor(state = AuthState, no_new)]
pub struct AuthCubit {}

impl AuthCubit {
    /// Custom constructor that reads from config.
    pub fn from_session(token: Option<String>) -> Self {
        let state = match token {
            Some(_) => AuthState { is_authenticated: true, username: None },
            None    => AuthState { is_authenticated: false, username: None },
        };
        Self {
            __gloc_state: state,
            __gloc_stream: Vec::new(),
        }
    }
}
```

### `no_observers` — no `on_change` needed

```rust
#[reactor(state = JobState, no_observers)]
pub struct JobCubit {}

// Generated new() only initialises __gloc_state (no __gloc_stream field)
let mut job = JobCubit::new(JobState { status: "pending".into() });
```

---

## Reference — all Mode A attribute arguments

```rust
#[reactor(
    state = SomeType,     // required in Mode A — the state type to use
    no_new,               // optional — suppress new() generation
    no_observers,         // optional — suppress on_change() generation
)]
```

---

## Next steps

- [Mode B](./macro-mode-b.md) — let GLOC generate the state struct
- [Observers](./observers.md) — using `on_change()`
- [Testing](./testing.md) — testing Mode A cubits
