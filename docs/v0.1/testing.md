# Testing — v0.1

Cubits are designed to be the easiest part of your application to test.
They are plain Rust structs with no framework dependencies, no async runtime,
no database — just logic and state.

---

## Why cubits are easy to test

```
┌──────────────────────────────────────────┐
│  Traditional testing problem             │
│                                          │
│  Test → UI → Business Logic → Database   │
│         ↑                                │
│         Hard to isolate                  │
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│  With GLOC                               │
│                                          │
│  Test → Cubit                            │
│         ↑                                │
│         No UI, no DB, no framework       │
└──────────────────────────────────────────┘
```

---

## Basic tests

Every cubit test follows the same three-step pattern:
**Arrange → Act → Assert**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use gloc::Cubit;

    // Arrange: create the cubit with a known initial state
    // Act:     call a domain method
    // Assert:  check the resulting state

    #[test]
    fn increment_increases_count_by_one() {
        let mut cubit = CounterCubit::new(0);   // arrange
        cubit.increment();                       // act
        assert_eq!(cubit.state().count, 1);     // assert
    }

    #[test]
    fn decrement_can_go_below_zero() {
        let mut cubit = CounterCubit::new(0);
        cubit.decrement();
        assert_eq!(cubit.state().count, -1);
    }

    #[test]
    fn reset_always_returns_to_zero() {
        let mut cubit = CounterCubit::new(999);
        cubit.reset();
        assert_eq!(cubit.state().count, 0);
    }
}
```

---

## Testing change detection

Always verify that redundant emissions do not change state:

```rust
#[test]
fn reset_on_zero_is_noop() {
    let mut cubit = CounterCubit::new(0);
    let before = cubit.state().clone();

    cubit.reset();   // already at 0

    assert_eq!(cubit.state(), &before);
}

#[test]
fn emit_same_value_has_no_effect() {
    let mut cubit = CounterCubit::new(5);

    cubit.emit(CounterState { count: 5 }); // identical — no-op

    assert_eq!(cubit.state().count, 5);
}
```

---

## Testing sequences of operations

Test the final state after a series of operations:

```rust
#[test]
fn mixed_operations_produce_correct_final_state() {
    let mut cubit = CounterCubit::new(0);

    cubit.increment(); // 1
    cubit.increment(); // 2
    cubit.increment(); // 3
    cubit.decrement(); // 2
    cubit.reset();     // 0
    cubit.increment(); // 1

    assert_eq!(cubit.state().count, 1);
}
```

---

## Dependency injection via trait objects

This is the most powerful testing pattern GLOC enables.
Write your application code against `dyn Cubit<State = S>` instead of a
concrete type — then inject a real or mock cubit in tests.

```rust
// Application code — depends only on the trait, not the concrete type
fn display_count(cubit: &dyn Cubit<State = CounterState>) -> String {
    format!("Count: {}", cubit.state().count)
}

fn apply_increments(cubit: &mut dyn Cubit<State = CounterState>, n: u32) {
    for _ in 0..n {
        let next = cubit.state().count + 1;
        cubit.emit(CounterState { count: next });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gloc::Cubit;

    #[test]
    fn display_count_formats_correctly() {
        let cubit = CounterCubit::new(42);
        assert_eq!(display_count(&cubit), "Count: 42");
    }

    #[test]
    fn apply_increments_reaches_correct_total() {
        let mut cubit = CounterCubit::new(0);
        apply_increments(&mut cubit, 10);
        assert_eq!(cubit.state().count, 10);
    }
}
```

---

## Recording cubits — capture every emitted state

A recording cubit stores the full history of state transitions.
Use it to verify that a sequence of operations produced the right intermediate
states, not just the final one.

```rust
struct RecordingCubit {
    state: CounterState,
    pub history: Vec<CounterState>,
}

impl RecordingCubit {
    fn new(initial: i32) -> Self {
        let state = CounterState { count: initial };
        Self { history: vec![state.clone()], state }
    }
}

impl Cubit for RecordingCubit {
    type State = CounterState;

    fn state(&self) -> &CounterState { &self.state }

    fn emit(&mut self, next: CounterState) {
        if next != self.state {
            self.state = next.clone();
            self.history.push(next);
        }
    }
}

#[test]
fn increment_three_times_produces_three_history_entries() {
    let mut cubit = RecordingCubit::new(0);
    apply_increments(&mut cubit, 3);

    // history: [0, 1, 2, 3]
    assert_eq!(cubit.history.len(), 4); // initial + 3 transitions
    assert_eq!(cubit.history[0].count, 0);
    assert_eq!(cubit.history[3].count, 3);
}

#[test]
fn noop_emit_is_not_recorded_in_history() {
    let mut cubit = RecordingCubit::new(0);
    cubit.emit(CounterState { count: 0 }); // no-op
    cubit.emit(CounterState { count: 0 }); // no-op

    // only the initial state is in history — no duplicates
    assert_eq!(cubit.history.len(), 1);
}
```

---

## Testing with `CubitBase`

`CubitBase` satisfies `dyn Cubit` — use it in tests that need a cubit
but don't care about the specific implementation:

```rust
#[test]
fn display_works_with_cubit_base() {
    use gloc::CubitBase;

    let cubit = CubitBase::new(CounterState { count: 7 });
    assert_eq!(display_count(&cubit), "Count: 7");
}
```

---

## What to test — checklist

- [ ] Initial state is correct
- [ ] Each domain method produces the right next state
- [ ] Emitting the same state is a no-op
- [ ] Boundary values (0, negative, max) behave correctly
- [ ] Sequences of operations produce the correct final state
- [ ] Functions that accept `dyn Cubit<State = S>` work with any implementation

---

## Next steps

- [v0.2 Testing](../v0.2/testing.md) — testing cubits generated by `#[cubit]`
- [v0.2 Getting Started](../v0.2/getting-started.md) — reduce boilerplate with the macro
