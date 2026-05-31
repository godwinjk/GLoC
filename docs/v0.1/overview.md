# GLOC v0.1 — Overview

> **Cubit Core** — the foundation of GLOC. Zero dependencies, pure traits.

---

## What v0.1 delivers

Version 0.1 is the bedrock everything else builds on. It introduces three
things and nothing more:

| Item | What it is |
|---|---|
| `State` | A marker trait — any `Clone + PartialEq + Debug` type is automatically a `State` |
| `Cubit` | A trait that owns one slice of state and transitions it via `emit()` |
| `CubitBase<S>` | A ready-made `Cubit` implementation for simple use-cases |

There are no macros, no code generation, no framework adapters. Just clean,
composable Rust traits that work anywhere.

---

## The problem it solves

In most Rust applications, business logic leaks into the wrong layer:

```
❌ Without GLOC
  UI component reads from database
  HTTP handler mutates shared state directly
  CLI command mixes parsing with domain logic
  Tests require spinning up the full stack
```

GLOC enforces a clean boundary:

```
✅ With GLOC
  Cubit owns the domain logic
  UI / handler / CLI only calls cubit methods
  State is a plain data snapshot — no behaviour
  Tests mock the cubit via trait objects
```

---

## Design principles

### Single Responsibility
Each cubit manages **one** cohesive slice of domain state.
A `CartCubit` handles cart logic. An `AuthCubit` handles authentication.
They never overlap.

### Dependency Inversion
Callers depend on the `Cubit` **trait**, not concrete types.
This means any cubit can be swapped for a mock in tests without changing
the code that uses it.

### Change Detection
`emit()` is a **no-op when the new state equals the current state**.
This prevents unnecessary work downstream — no re-renders, no redundant
callbacks, no wasted compute.

---

## How it fits together

```
┌──────────────────────────────────────────────┐
│                  Your App                    │
│                                              │
│   UI / Handler / CLI                         │
│         │                                    │
│         │  calls methods                     │
│         ▼                                    │
│   ┌─────────────┐                            │
│   │   Cubit     │  owns state                │
│   │             │  calls emit()              │
│   └──────┬──────┘                            │
│          │                                   │
│          │  produces                         │
│          ▼                                   │
│   ┌─────────────┐                            │
│   │    State    │  plain data, no behaviour  │
│   └─────────────┘                            │
└──────────────────────────────────────────────┘
```

---

## Installation

```toml
[dependencies]
gloc = "0.1"
```

---

## What comes next

- [Getting Started](./getting-started.md) — your first cubit in 5 minutes
- [State](./state.md) — how to design good state types
- [Cubit](./cubit.md) — implementing and using cubits
- [Testing](./testing.md) — testing cubits in isolation

Or jump to [v0.2](../v0.2/overview.md) to see how the `#[cubit]` macro
eliminates all the boilerplate v0.1 requires.
