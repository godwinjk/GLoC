# GLOC v0.1 Documentation

> **Cubit Core** — the foundation. Zero dependencies, pure traits.

[← Back to main docs](../index.md)

---

## Contents

- [Overview](./overview.md) — what v0.1 delivers and why
- [Getting Started](./getting-started.md) — your first cubit in 5 minutes
- [State](./state.md) — designing state types
- [Cubit](./cubit.md) — implementing and using cubits
- [Testing](./testing.md) — testing cubits in isolation

---

## What's in v0.1

| Added | Description |
|---|---|
| `State` | Marker trait — any `Clone + PartialEq + Debug` type is automatically a `State` |
| `Cubit` | Core trait — `state()` + `emit()` with change-detection |
| `CubitBase<S>` | Ready-made cubit implementation |

---

[→ Upgrade to v0.2](../v0.2/index.md) · [CHANGELOG](../CHANGELOG.md)
