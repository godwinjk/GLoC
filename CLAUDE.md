# GLoC — Claude Code Context

## What this project is

GLoC (Global Logic Component) is a universal Rust state management library.
Inspired by Flutter Bloc but with its own identity and a **nuclear fission theme**
(`Reactor`, `Neutron`, `fire()`). The core abstraction is `Reactor` — one type
that handles both direct method calls and event-driven dispatch.

---

## Workspace layout

```
GLoC/
├── gloc-core/           Core traits and reactive primitives (no UI deps)
├── gloc-macro/          Proc macros: #[reactor], #[reactor_state]
├── gloc/                Umbrella re-export crate
├── gloc-axum/           Axum HTTP adapter — AxumReactor<R>
├── gloc-bevy/           Bevy ECS adapter  — GlocPlugin<R>, GlocResource<R>
├── examples/
│   ├── dioxus/          Desktop UI showcase — 5-section feature demo
│   ├── cli/             Terminal REPL — task manager with neutron firing
│   ├── axum/            Shop API — CartReactor + InventoryReactor, 9 routes
│   └── bevy/            Space game (headless) — PlayerReactor + WaveReactor
└── docs/
    ├── v0.1/            Released docs
    └── v0.2/            In-development docs (current)
```

---

## Core public API (gloc-core)

| Type | File | Role |
|------|------|------|
| `Reactor` trait | `reactor.rs` | `state()` + `emit()` + `on_close()` with change-detection |
| `ReactorBase<S>` | `reactor.rs` | Ready-made reactor for simple cases |
| `State` trait | `state.rs` | Blanket impl: `Clone + PartialEq + Debug` |
| `Neutron` trait | `event.rs` | Blanket impl: `Debug + Send + 'static`; `Event` is a type alias |
| `GlocStream<S>` | `stream.rs` | Internal reactive engine behind every reactor |
| `GlocSubscription<S>` | `stream.rs` | Read-only stream handle (`subscribe()`) |
| `GlocProvider<R>` | `provider.rs` | Shared mutable handle — `Arc<Mutex<R>>` + `GlocStream` |
| `GlocListener<R>` | `listener.rs` | Typed `old → new` observer trait (`attach_listener()`) |
| `GlocObserver` | `observer.rs` | Global lifecycle hook — create / transition / close |

---

## Macro API (gloc-macro)

| Macro | What it generates |
|-------|------------------|
| `#[reactor(state = T)]` | Mode A — `impl Reactor`, `impl Deref<Target=T>`, `new()`, `subscribe()`, `attach_listener()` |
| `#[reactor]` + `#[state]` fields | Mode B — same + generates `{StructName}State` struct |
| `#[reactor(neutrons = N)]` | Adds `pub fn fire(&mut self, neutron: N)` → calls `self.on_event(neutron)` |
| `#[reactor(no_new)]` | Suppresses `new()` generation |
| `#[reactor_state]` | Injects `#[derive(Clone, PartialEq, Debug)]`; accepts `derive(Extra, ...)` |

### fire() / on_event pattern

```rust
#[reactor(state = CounterState, neutrons = CounterEvent)]
pub struct CounterReactor {}

impl CounterReactor {
    fn on_event(&mut self, neutron: CounterEvent) {
        match neutron {
            CounterEvent::Increment => self.emit(CounterState { count: self.count + 1 }),
            CounterEvent::Reset     => self.emit(CounterState { count: 0 }),
        }
    }
}

// Generated: pub fn fire(&mut self, neutron: CounterEvent) { self.on_event(neutron); }

// Usage
reactor.increment();                    // direct method
reactor.fire(CounterEvent::Increment);  // neutron dispatch
```

### Deref to state fields

`#[reactor]` generates `impl Deref<Target = State>` — access state fields directly:

```rust
reactor.count          // instead of reactor.state().count
reactor.read().count   // in Dioxus: Signal<R>.read() → R → State
```

---

## Architectural decisions (do not change without discussion)

1. **One abstraction** — `Reactor` covers direct methods AND neutron dispatch. No `Bloc` type.
2. **`GlocProvider` is the sharing primitive** — wraps `Arc<Mutex<R>>` + `GlocStream`. Use it for Axum, Bevy, CLI threads. `release()` calls `on_close()`.
3. **`on_change` is NOT generated** — observation is via `GlocObserver` (global) or `GlocSubscription` (per-reactor). No memory leak risk.
4. **`on_close` is provider-driven** — only fires when `GlocProvider::release()` is called. NOT a `Drop` impl. Direct/CLI usage does not fire it.
5. **Neutron trait bounds** — `Debug + Send + 'static`. Neutrons are consumed, not cloned or compared. `Event` is a type alias for backward compat.
6. **State trait bounds** — `Clone + PartialEq + Debug`. States are change-detected.
7. **on_event is user-written** — macro only generates `fire()`. User writes the handler body.
8. **Observer tests run serially** — `#[serial_test::serial]` on all observer tests (global `OBSERVER` static).
9. **Adapters are framework concerns** — `GlocProvider` is in core. `GlocBuilder`, `use_gloc()` belong in `gloc-dioxus`. `AxumReactor` in `gloc-axum`. `GlocPlugin` in `gloc-bevy`.

---

## Running and testing

```sh
# Build everything
cargo build --workspace

# Full test suite (excludes Dioxus desktop app)
cargo test --workspace --exclude gloc-example-dioxus

# UI tests (trybuild compile-pass + compile-fail)
cargo test -p gloc-macro --test ui_tests

# Bless stale trybuild snapshots (after macro changes)
TRYBUILD=overwrite cargo test -p gloc-macro --test ui_tests

# Run examples
cargo run -p gloc-example-dioxus    # desktop UI
cargo run -p gloc-example-cli       # interactive REPL
cargo run -p gloc-example-axum      # HTTP API (localhost:3000)
cargo run -p gloc-example-bevy      # headless space game

# Clean rebuild when proc macro changes aren't picked up
cargo clean -p gloc-macro && cargo build --workspace
```

---

## Common issues

| Symptom | Cause | Fix |
|---------|-------|-----|
| `Unknown field: neutrons` | Stale proc macro binary | `cargo clean -p gloc-macro && cargo build --workspace`, restart rust-analyzer |
| `Method not found` in IDE after macro change | Stale analysis | Same as above |
| Observer tests fail in parallel | Global `OBSERVER` static | All observer tests must have `#[serial_test::serial]` |
| `on_change` not found | Removed from macro | Use `GlocObserver` or `reactor.subscribe().listen(...)` |

---

## Code standards (always follow)

- Every `pub` item must have a `///` doc comment explaining **why** it exists.
- Tests: happy path + at least one edge case + one boundary condition.
- Reactors implementing `Reactor` manually must include a `dyn Reactor<State = S>` trait-object test.
- New `#[reactor]` error paths need a `ui/fail/` trybuild scenario with `.stderr` snapshot.
- No `unwrap()` or `expect()` in library code.
- Proc macro generated code must use fully-qualified `::gloc::` paths for hygiene.
- `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings` must pass clean.

---

## Session summary — what was built

### Architecture refactor

| Change | Detail |
|--------|--------|
| `GlocConsumer` → `GlocProvider` | Renamed everywhere — file, struct, exports, adapters, examples, docs |
| `on_change` removed from macro | Was `Vec<Box<dyn Fn>>` with no unsubscribe — memory leak risk. Replaced by `GlocObserver` + `GlocSubscription` |
| `on_close()` added to `Reactor` trait | Default no-op. Called by `GlocProvider::release()`. User overrides for cleanup |
| `GlocProvider::release()` | Calls `reactor.on_close()` + `GlocObserver::on_close()` then drops |
| `GlocObserver` fully wired | `on_create` fires in `new()`, `on_transition` fires in `emit()`, `on_close` fires in `release()` |

### Nuclear theme rename

| Old | New | Note |
|-----|-----|------|
| `Event` trait | `Neutron` trait | `Event` kept as type alias |
| `events = E` | `neutrons = E` | macro attribute arg |
| `dispatch()` | `fire()` | macro generated method |
| `on_event()` | `on_event()` | unchanged — user written |

### Deref to state fields

`#[reactor]` macro now generates `impl Deref<Target = State>` so:
```rust
reactor.count           // direct field access
signal.read().count     // in Dioxus, no .state() needed
```

### Framework adapters built

| Crate | What it provides |
|-------|-----------------|
| `gloc-axum` | `AxumReactor<R>` wraps `GlocProvider` for Axum `State<T>` |
| `gloc-bevy` | `GlocPlugin<R>` inserts reactor as Bevy `Resource`; `GlocResource<R>` newtype |
| `examples/axum` | Shop API — CartReactor (neutrons) + InventoryReactor (direct) |
| `examples/bevy` | Space game — PlayerReactor (neutrons) + WaveReactor (direct) |
| `examples/cli` | Task manager REPL — neutron dispatch, filters, IDs |

### Documentation

All `docs/v0.2/` pages updated. New pages: `dioxus.md`, `provider.md` (replaces `consumer.md`), `axum.md`, `bevy.md`, `cli.md`.

---

## What's next — gloc-dioxus

This is the immediate next task. The goal is a proper `gloc-dioxus` crate that makes GLoC fully reactive in Dioxus without the user needing `Signal<R>` directly.

### Planned API

```rust
// App root — provide reactors into Dioxus context tree
fn App() -> Element {
    GlocBuilder::new()
        .provide(|| CartReactor::new(CartState::empty()))
        .provide(|| ThemeReactor::new(Theme::Light))
        .build(|| rsx! { Router::<Route> {} })
}

// Any child component — consume from context, fully reactive
fn CartPage() -> Element {
    let cart = use_gloc::<CartReactor>();
    let total = cart.total;              // Deref to state field, reactive
    rsx! {
        p { "Total: ${total:.2}" }
        button { onclick: move |_| cart.write().add_item("Book", 12.99), "+ Book" }
    }
}
```

### Key design points

- `GlocBuilder` — wires `GlocProvider` into Dioxus `provide_context()`
- `use_gloc::<R>()` — `use_context::<Signal<R>>()` under the hood, returns a reactive handle
- Bridge: `GlocStream` transitions update the Dioxus `Signal` → re-render
- `GlocProvider::release()` called on component unmount
- No `Signal<R>` exposed to the user — they only see the reactor handle

### How it differs from current Dioxus usage

| Today (`Signal<R>` directly) | With `gloc-dioxus` |
|---|---|
| `counter.read().count` | `counter.count` |
| `counter.write().increment()` | `counter.write().increment()` |
| Pass signal as prop | `use_gloc::<R>()` from anywhere in tree |
| `on_change` removed | `GlocObserver` for side effects |
