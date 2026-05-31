# `#[reactor_state]` — v0.2

[← Index](./index.md)

---

## What it does

`#[reactor_state]` automatically injects the three derives that every GLoC
state type requires: `Clone`, `PartialEq`, and `Debug`.

```rust
// Without — user writes this every time
#[derive(Clone, PartialEq, Debug)]
struct CounterState { pub count: i32 }

// With — one attribute, same result
#[reactor_state]
struct CounterState { pub count: i32 }
```

---

## Adding extra derives

Pass extra derives through `derive(...)`. They are appended after the
required three — you never need to repeat `Clone`, `PartialEq`, or `Debug`.

```rust
// Hash and Eq — useful for using state in HashSets or as HashMap keys
#[reactor_state(derive(Eq, Hash))]
struct TagState { pub tag: u32 }

// Multiple extras
#[reactor_state(derive(Eq, Hash, serde::Serialize, serde::Deserialize))]
struct PersistableState { pub value: String }
```

---

## Works on structs and enums

```rust
// Struct
#[reactor_state]
struct AuthState {
    pub is_authenticated: bool,
    pub username: Option<String>,
}

// Enum
#[reactor_state]
enum LoadingState {
    Idle,
    Loading,
    Success(String),
    Error(String),
}
```

---

## Preserves other attributes

Any other attributes on the item are kept as-is:

```rust
#[reactor_state]
#[serde(rename_all = "camelCase")]
struct ApiState {
    pub user_id: String,
    pub access_token: String,
}
```

---

## Used alongside `#[reactor]`

```rust
use gloc::{reactor, reactor_state};

#[reactor_state]
pub struct ScoreState { pub score: u32 }

#[reactor(state = ScoreState)]
pub struct ScoreReactor {}

impl ScoreReactor {
    pub fn add(&mut self, pts: u32) {
        self.emit(ScoreState { score: self.state().score + pts });
    }
}
```

---

## Why it exists

Mode A (`#[reactor(state = T)]`) requires the user to bring their own state
type. The only obligation is that the type satisfies `Clone + PartialEq + Debug`.
`#[reactor_state]` removes that obligation — the user declares their fields
and the macro handles the rest.

Mode B (`#[state]` fields) never needs this because GLoC generates the
state struct with the correct derives automatically.

---

## Attribute reference

```
#[reactor_state]
#[reactor_state(derive(ExtraTrait, another::Trait))]
```

| Argument | Effect |
|---|---|
| *(none)* | Inject `Clone + PartialEq + Debug` only |
| `derive(A, B, ...)` | Inject required three + append A, B, ... |
