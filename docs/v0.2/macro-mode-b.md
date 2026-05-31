# Mode B — Generated State

Mode B is activated by annotating fields inside the reactor struct with `#[state]`
and omitting the `state = SomeType` argument:

```rust
#[reactor]
pub struct MyReactor {
    #[state] pub field: Type,
}
```

The macro collects all `#[state]` fields, removes them from the reactor struct,
and generates a new `{CubitName}State` struct containing those fields.

Use Mode B when:
- You want maximum brevity — no separate state file
- Your state is simple and tightly coupled to this reactor only
- You are prototyping and want to move fast

---

## Basic usage

```rust
use gloc::{reactor, Reactor};

#[reactor]
pub struct ToggleReactor {
    #[state] pub active: bool,
}

// Macro generates:
//   #[derive(Clone, PartialEq, Debug)]
//   pub struct ToggleReactorState { pub active: bool }

impl ToggleReactor {
    pub fn toggle(&mut self) {
        let next = !self.state().active;
        self.emit(ToggleReactorState { active: next });
    }
}

let mut toggle = ToggleReactor::new(ToggleReactorState { active: false });
toggle.toggle();
assert!(toggle.state().active);
toggle.toggle();
assert!(!toggle.state().active);
```

---

## Multiple `#[state]` fields

```rust
#[reactor]
pub struct UserReactor {
    #[state] pub name: String,
    #[state] pub age: u32,
    #[state] pub is_admin: bool,
}

// Macro generates:
//   #[derive(Clone, PartialEq, Debug)]
//   pub struct UserReactorState {
//       pub name: String,
//       pub age: u32,
//       pub is_admin: bool,
//   }

impl UserReactor {
    pub fn promote(&mut self) {
        let mut next = self.state().clone();
        next.is_admin = true;
        self.emit(next);
    }
}

let mut user = UserReactor::new(UserReactorState {
    name: "Alice".into(),
    age: 30,
    is_admin: false,
});

user.promote();
assert!(user.state().is_admin);
```

---

## Non-state fields — private reactor data

Fields **without** `#[state]` stay on the reactor struct as private
implementation details. They do not appear in the generated state struct.
They become **additional parameters in `new()`**, before `initial`.

```rust
#[reactor]
pub struct SteppedCubit {
    #[state] pub count: i32,    // goes into SteppedReactorState
    pub step: i32,              // stays on SteppedCubit — not in state
}

// Generated: pub fn new(step: i32, initial: SteppedReactorState) -> Self

impl SteppedCubit {
    pub fn advance(&mut self) {
        let next = self.state().count + self.step;
        self.emit(SteppedReactorState { count: next });
    }
}

let mut reactor = SteppedCubit::new(5, SteppedReactorState { count: 0 });
r.advance(); // count = 5
r.advance(); // count = 10
```

---

## Generated state struct name

The generated state struct name is always:
```
{CubitName}State
```

| Reactor | Generated state |
|---|---|
| `CounterReactor` | `CounterReactorState` |
| `ToggleReactor` | `ToggleReactorState` |
| `UserReactor` | `UserReactorState` |

---

## Generated state visibility

The generated state struct inherits the **reactor's visibility**:

```rust
// pub reactor → pub state struct
pub struct CounterReactor { ... }
// generates: pub struct CounterReactorState { ... }

// private reactor → private state struct
struct InternalCubit { ... }
// generates: struct InternalReactorState { ... }
```

All `#[state]` fields in the generated struct are always `pub` — callers
need to read them.

---

## Mode B vs Mode A — when to use which

| Situation | Use |
|---|---|
| State is simple, only used by this reactor | **Mode B** |
| State is shared between multiple cubits | **Mode A** |
| State needs custom methods | **Mode A** |
| State needs a custom `PartialEq` impl | **Mode A** |
| You want maximum brevity | **Mode B** |
| You want maximum control | **Mode A** |

---

## Compile-time errors

### No `#[state]` fields

```rust
// ❌ Error — no state type found
#[reactor]
struct BadCubit {
    regular_field: i32,
}
```

```
error: #[reactor] requires at least one field annotated with `#[state]`.
       Example: `#[state] count: i32`.
       Alternatively, supply an existing state type: `#[reactor(state = MyState)]`.
```

### Tuple struct

```rust
// ❌ Error — named fields required
#[reactor(state = MyState)]
struct TupleCubit(i32);
```

```
error: #[reactor] does not support tuple structs. Use named fields: `struct Foo { ... }`.
```

---

## Next steps

- [Observers](./observers.md) — subscribing to state changes
- [Testing](./testing.md) — testing Mode B cubits
- [Migration from v0.1](./migration.md) — converting manual cubits
