# GlocListener — v0.2

[← Index](./index.md)

---

## What it is

`GlocListener<R>` is a trait for types that react to state transitions.
Implement it on any struct to receive `(&old_state, &new_state)` on
every real state change.

```rust
use gloc::GlocListener;

struct Logger;

impl GlocListener<CounterReactor> for Logger {
    fn on_transition(&self, old: &CounterState, new: &CounterState) {
        println!("{} → {}", old.count, new.count);
    }
}
```

---

## Why a trait instead of a closure

| | Closure (`on_change`) | `GlocListener` |
|---|---|---|
| Syntax | `reactor.on_change(\|old, new\| ...)` | `impl GlocListener<R> for MyType` |
| Testable | Harder | Yes — inject `&dyn GlocListener<R>` |
| Reusable | One-off | Any struct can implement it |
| Composable | No | Yes — the same struct can implement multiple traits |
| Best for | Simple one-off side effects | Services, analytics, navigation |

---

## Attaching to a reactor

```rust
let mut reactor = CounterReactor::new(CounterState { count: 0 });
reactor.attach_listener(Logger);
reactor.increment(); // prints: 0 → 1
```

## Attaching to a consumer

```rust
let consumer = provider.consumer();
consumer.attach_listener(Logger);
consumer.update(|c| c.increment()); // prints: 0 → 1
```

---

## Real-world examples

### Analytics listener

```rust
struct Analytics { endpoint: String }

impl GlocListener<CartReactor> for Analytics {
    fn on_transition(&self, _old: &CartState, new: &CartState) {
        if matches!(new.status, CartStatus::CheckedOut) {
            self.track("checkout_completed", new.total);
        }
    }
}
```

### Navigation listener

```rust
struct Navigator;

impl GlocListener<AuthCubit> for Navigator {
    fn on_transition(&self, _old: &AuthState, new: &AuthState) {
        if new.is_authenticated {
            navigate_to("/dashboard");
        } else {
            navigate_to("/login");
        }
    }
}
```

### Persistence listener

```rust
struct AutoSave { path: PathBuf }

impl GlocListener<SettingsCubit> for AutoSave {
    fn on_transition(&self, _old: &SettingsState, new: &SettingsState) {
        if let Ok(json) = serde_json::to_string(new) {
            let _ = std::fs::write(&self.path, json);
        }
    }
}
```

---

## Contract

- Must not block — called synchronously inside `emit()`
- Must not call `emit()` recursively — will deadlock on the stream mutex
- `Send + 'static` required — the listener is stored in the stream
