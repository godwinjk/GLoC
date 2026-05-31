# Migration — v0.2 → v0.2

[← Index](./index.md)

---

## Is migration required?

Only if you use `on_change`. Everything else is additive — v0.2 code
compiles unchanged in v0.2.

---

## Breaking change — `on_change` signature

`on_change` now receives **both** old and new state. Update every call site:

```rust
// v0.2
reactor.on_change(|new_state| {
    println!("{}", new_state.count);
});

// v0.2
reactor.on_change(|old_state, new_state| {
    println!("{} → {}", old_state.count, new_state.count);
});

// v0.2 — if you only care about new state, prefix old with _
reactor.on_change(|_old, new_state| {
    println!("{}", new_state.count);
});
```

**Why:** The reactive layer needs both sides of the transition to notify
`GlocStream` listeners correctly. Passing only new state is no longer
sufficient for the `GlocListener` pattern.

---

## Custom constructors using `__gloc_stream`

If you used `#[reactor(no_new)]` and wrote a custom constructor that
initialised `__gloc_stream` directly, update it to use `__gloc_stream`:

```rust
// v0.2 — initialise listeners list
impl MyReactor {
    pub fn custom() -> Self {
        Self {
            __gloc_state: MyState::default(),
            __gloc_stream: Vec::new(),
        }
    }
}

// v0.2 — initialise stream
impl MyReactor {
    pub fn custom() -> Self {
        Self {
            __gloc_stream: ::gloc::GlocStream::new(MyState::default()),
            __gloc_state: MyState::default(),
        }
    }
}
```

---

## New capabilities — nothing to migrate, just opt in

| Feature | How to use |
|---|---|
| `#[reactor_state]` | Replace `#[derive(Clone, PartialEq, Debug)]` on state structs |
| `GlocProvider` (adapter crate) | `GlocProvider::new(MyReactor::new(...))` |
| `GlocConsumer` | `provider.consumer()` |
| `GlocListener` | `impl GlocListener<R> for MyType` |
| `subscribe()` | `let sub = reactor.subscribe()` |
| `attach_listener()` | `reactor.attach_listener(MyListener)` |

---

## Migration checklist

- [ ] Update `on_change(\|new\| ...)` → `on_change(\|_old, new\| ...)`
- [ ] Update custom constructors — `__gloc_stream` → `__gloc_stream`
- [ ] Optionally replace `#[derive(Clone, PartialEq, Debug)]` with `#[reactor_state]`
- [ ] Run `cargo test` — all existing tests should pass
