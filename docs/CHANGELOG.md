# Changelog

All notable changes to GLoC are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
GLoC adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned — v0.3
- Event dispatch — `reactor.dispatch(Event)` opt-in pattern
- Optional `#[reactor(events = MyEvent)]` macro argument

### Planned — v0.4
- `gloc-dioxus` adapter — `GlocProvider`, context hooks
- `gloc-axum` adapter
- `gloc-bevy` adapter

---

## [0.2.0] — 2026

### Renamed (breaking)
- `Cubit` trait → `Reactor` trait
- `CubitBase<S>` → `ReactorBase<S>`
- `#[cubit]` macro → `#[reactor]` macro
- `#[cubit_state]` macro → `#[reactor_state]` macro
- Mode B generated state: `{Name}CubitState` → `{Name}ReactorState`
- `GlocListener<C: Cubit>` → `GlocListener<R: Reactor>`
- `GlocConsumer<C: Cubit>` → `GlocConsumer<R: Reactor>`
- `GlocObserver::on_transition` param `cubit_name` → `reactor_name`

### Removed
- `GlocProvider` from `gloc-core` public API — it is a framework-adapter concern
  and will live in `gloc-dioxus` (planned v0.4). Use `GlocConsumer::new()` directly.

### Changed
- `GlocConsumer::new()` promoted to `pub` — primary construction point for shared reactors

### Added
- `#[reactor_state]` attribute macro — auto-injects `Clone + PartialEq + Debug`; accepts extra derives via `derive(...)`
- `GlocStream<S>` — shared reactive state container, pure `std`, zero new dependencies
- `GlocSubscription<S>` — lightweight read-only handle to a `GlocStream`
- `GlocConsumer<R>` — shared mutable handle; `update()`, `listen()`, `attach_listener()`, `stream()`
- `GlocListener<R>` — trait for typed `old → new` transition observers
- `GlocObserver` — global observer: `set_observer`, `clear_observer`, `observer()`
- `subscribe()` — new method on every `#[reactor]` struct, returns `GlocSubscription`
- `attach_listener()` — new method on every `#[reactor]` struct, attaches a `GlocListener`
- `serial_test` dev-dependency in `gloc-core` — observer tests run serially (global state safety)

### Breaking change — `on_change` signature
`on_change` now receives **both** `(&old, &new)` instead of just `(&new)`.

---

## [0.1.0] — 2024

### Added
- `State` trait — blanket impl for any `Clone + PartialEq + Debug` type
- `Reactor` trait — core interface with `state()` and `emit()`
- `ReactorBase<S>` — ready-made reactor implementation with change-detection
- Cargo workspace setup — `gloc-core`, `gloc-macro`, `gloc`
- Dioxus 0.7 desktop example
- GitHub Actions CI — build, test, fmt, clippy, publish workflows
- MIT license
