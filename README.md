<div align="center">

# GLoC
![GLoC](asset/gloc_logo.svg)
_The **G** is intentional. GLoC started as a hobby project called **G**odwin's **L**ogic **C**omponent,
born from a mission to bring Flutter's legendary **BLoC** architecture into Rust.
But as it grows to serve the wider open-source community, that **G** now stands for **Global**.
One pattern. Universal. Everywhere Rust runs._

A universal business logic architecture for Rust.

[![CI — Main](https://github.com/godwinjk/gloc/actions/workflows/main.yml/badge.svg)](https://github.com/godwinjk/gloc/actions/workflows/main.yml)
[![CI — PR](https://github.com/godwinjk/gloc/actions/workflows/pr.yml/badge.svg)](https://github.com/godwinjk/gloc/actions/workflows/pr.yml)
[![CI — Publish](https://github.com/godwinjk/gloc/actions/workflows/publish.yml/badge.svg)](https://github.com/godwinjk/gloc/actions/workflows/publish.yml)
[![Crates.io](https://img.shields.io/crates/v/gloc.svg)](https://crates.io/crates/gloc)
[![Docs.rs](https://docs.rs/gloc/badge.svg)](https://docs.rs/gloc)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](#license)
[![VS Code](https://img.shields.io/visual-studio-marketplace/v/godwinjoseph.gloc-vscode.svg?label=VS%20Code)](https://marketplace.visualstudio.com/items?itemName=godwinjoseph.gloc-vscode)
[![Open VSX](https://img.shields.io/open-vsx/v/GodwinJoseph/gloc-vscode.svg?label=Open%20VSX)](https://open-vsx.org/extension/GodwinJoseph/gloc-vscode)
[![JetBrains](https://img.shields.io/jetbrains/plugin/v/32105.svg?label=JetBrains)](https://plugins.jetbrains.com/plugin/32105-gloc-reactor-generator)

</div>

---

## What is GLoC?

GLoC is inspired by Flutter's [Bloc](https://bloclibrary.dev) architecture — but it's its own thing.
It separates **business logic** from **presentation** in any Rust application and works
anywhere Rust runs: web frontends, desktop GUIs, backend servers, CLIs, and embedded targets.

The core abstraction is **`Reactor`** — a single unit that owns one slice of domain state,
exposes domain methods that transition it, and carries a **built-in reactive stream** that
broadcasts every real transition to all subscribers automatically.

```
┌─────────────────────────────────────────────────────────────┐
│  Without GLoC           │  With GLoC                        │
│─────────────────────────│───────────────────────────────────│
│  Logic tangled in UI    │  Reactor owns logic               │
│  State scattered        │  Single source of truth           │
│  Hard to test           │  Fully injectable & mockable      │
│  Framework-locked       │  Web · Desktop · CLI · Embedded   │
└─────────────────────────────────────────────────────────────┘
```

**One pattern. Everywhere Rust runs.**

---

## Table of Contents

- [Concepts](#concepts)
- [Installation](#installation)
- [Ecosystem](#ecosystem)
- [Quick Start](#quick-start)
  - [Reactor — direct methods](#reactor--direct-methods)
  - [Reactor — event-driven dispatch](#reactor--event-driven-dispatch)
  - [Shared reactor across owners](#shared-reactor-across-owners)
- [Define State](#define-state)
- [Define a Reactor](#define-a-reactor)
- [Reactive Stream](#reactive-stream)
- [Observers](#observers)
- [Dioxus Integration](#dioxus-integration)
- [Feature Flags](#feature-flags)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [License](#license)

---

## Concepts

### ⚛️ Reactor

A **Reactor** owns one slice of domain state, exposes methods to mutate it, and
carries a **built-in `GlocStream`** — a fan-out reactive stream that broadcasts
every real transition to all subscribers automatically.

```rust
#[reactor(state = CounterState)]
pub struct CounterReactor {}

impl CounterReactor {
    pub fn increment(&mut self) {
        self.emit(CounterState { count: self.count + 1 });
        // emit() → stream fires → all subscribers notified
    }
}
```

Unlike Flutter Bloc which has separate `Cubit` and `Bloc` types, GLoC has one:
a `Reactor` supports both direct method calls and event dispatch via `fire()`.

---

### ☢️ Neutron (Event)

A **Neutron** is an immutable event fired *at* a reactor. The name follows GLoC's
nuclear fission theme: a neutron strikes the reactor and causes a reaction.

Any type satisfying `Debug + Send + 'static` is automatically a `Neutron`:

```rust
#[derive(Debug)]
pub enum CounterEvent { Increment, Decrement, AddBy(i32), Reset }

impl CounterReactor {
    fn on_event(&mut self, event: CounterEvent) {
        match event {
            CounterEvent::Increment  => self.emit(CounterState { count: self.count + 1 }),
            CounterEvent::Decrement  => self.emit(CounterState { count: self.count - 1 }),
            CounterEvent::AddBy(n)   => self.emit(CounterState { count: self.count + n }),
            CounterEvent::Reset      => self.emit(CounterState { count: 0 }),
        }
    }
}

reactor.fire(CounterEvent::Increment);
```

---

### 🔋 State

Any `Clone + PartialEq + Debug` type is automatically a `State`. Use `#[reactor_state]`
to skip writing the derives:

```rust
#[reactor_state]
pub struct CounterState { pub count: i32 }
```

GLoC performs **change detection**: emitting a value equal to the current state is a
no-op — no stream notification, no re-render.

---

### Other primitives

| Concept | Description |
|---------|-------------|
| **`GlocStream`** | Built-in fan-out stream on every reactor. Notifies all subscribers on every real transition. |
| **`ListenerHandle`** | RAII cancel token returned by every `listen()` call. Drop to cancel automatically. |
| **`GlocProvider`** | `Arc<Mutex<R>>` wrapper for shared multi-owner reactor access across threads. |
| **`GlocListener`** | Typed trait for `old → new` observation on a specific reactor. |
| **`GlocObserver`** | Global hook — sees every reactor in the app. Supports both Debug strings and typed `&dyn Any`. |

---

## Installation

> **Note:** GLoC has not yet been published to crates.io. Add it as a git dependency for now:

```toml
[dependencies]
gloc = { git = "https://github.com/godwinjk/gloc" }
```

For Dioxus desktop:

```toml
[dependencies]
gloc        = { git = "https://github.com/godwinjk/gloc" }
gloc-dioxus = { git = "https://github.com/godwinjk/gloc" }
dioxus      = { version = "0.7", features = ["desktop"] }
```

With tracing:

```toml
[dependencies]
gloc    = { git = "https://github.com/godwinjk/gloc", features = ["tracing"] }
tracing = "0.1"
```

---

## Ecosystem

Official IDE plugins for GLoC — generate reactors, states, and events without boilerplate.

| Plugin | Install |
|--------|---------|
| **GLoC for VS Code** — snippets and reactor scaffolding for VS Code and VS Code-compatible editors | [![VS Code](https://img.shields.io/visual-studio-marketplace/v/godwinjoseph.gloc-vscode.svg?label=Marketplace)](https://marketplace.visualstudio.com/items?itemName=godwinjoseph.gloc-vscode) [![Open VSX](https://img.shields.io/open-vsx/v/GodwinJoseph/gloc-vscode.svg?label=Open%20VSX)](https://open-vsx.org/extension/GodwinJoseph/gloc-vscode) |
| **GLoC Reactor Generator for IntelliJ** — reactor and state generation for IntelliJ IDEA, CLion, and RustRover | [![JetBrains](https://img.shields.io/jetbrains/plugin/v/32105.svg?label=JetBrains%20Marketplace)](https://plugins.jetbrains.com/plugin/32105-gloc-reactor-generator) |

---

## Quick Start

### Reactor — direct methods

```rust
use gloc::{reactor, reactor_state, Reactor};

#[reactor_state]
pub struct CounterState { pub count: i32 }

#[reactor(state = CounterState)]
pub struct CounterReactor {}

impl CounterReactor {
    pub fn increment(&mut self) {
        self.emit(CounterState { count: self.state().count + 1 });
    }
}

fn main() {
    let reactor = CounterReactor::new(CounterState { count: 0 });

    // Subscribe to the reactor's built-in stream
    // Returns a ListenerHandle — keep it alive to keep listening
    let _h = reactor.stream().listen(|old, new| {
        println!("{} → {}", old.count, new.count);
    });

    reactor.increment(); // prints: 0 → 1
    reactor.increment(); // prints: 1 → 2

    assert_eq!(reactor.state().count, 2);
} // _h dropped → listener cancelled
```

---

### Reactor — event-driven dispatch

```rust
use gloc::{reactor, reactor_state, Reactor};

#[reactor_state]
pub struct CounterState { pub count: i32 }

#[derive(Debug)]
pub enum CounterEvent { Increment, Decrement, AddBy(i32), Reset }

#[reactor(state = CounterState, neutrons = CounterEvent)]
pub struct CounterReactor {}

impl CounterReactor {
    fn on_event(&mut self, event: CounterEvent) {
        match event {
            CounterEvent::Increment  => self.emit(CounterState { count: self.count + 1 }),
            CounterEvent::Decrement  => self.emit(CounterState { count: self.count - 1 }),
            CounterEvent::AddBy(n)   => self.emit(CounterState { count: self.count + n }),
            CounterEvent::Reset      => self.emit(CounterState { count: 0 }),
        }
    }
}

fn main() {
    let reactor = CounterReactor::new(CounterState { count: 0 });

    let _h = reactor.stream().listen(|_, new| println!("count: {}", new.count));

    reactor.fire(CounterEvent::Increment); // count: 1
    reactor.fire(CounterEvent::AddBy(4)); // count: 5
    reactor.fire(CounterEvent::Reset);    // count: 0
}
```

---

### Shared reactor across owners

When multiple threads or components need to share one reactor, wrap it in `GlocProvider`:

```rust
use std::sync::{Arc, Mutex};
use gloc::{reactor, reactor_state, Reactor, GlocProvider};

// ... reactor definition ...

fn main() {
    let provider = GlocProvider::new(Arc::new(Mutex::new(
        CounterReactor::new(CounterState { count: 0 })
    )));

    let p1 = provider.clone(); // cheap Arc clone — same reactor
    let p2 = provider.clone();

    // Listen through the shared stream — keep handle alive
    let _h = p1.listen(|old, new| println!("{} → {}", old.count, new.count));

    p2.update(|r| r.increment()); // p1's listener fires: 0 → 1
    p2.update(|r| r.increment()); // p1's listener fires: 1 → 2

    assert_eq!(p1.state().count, 2);
}
```

---

## Define State

```rust
use gloc::reactor_state;

// Struct state
#[reactor_state]
pub struct CounterState { pub count: i32 }

// Enum state — great for loading flows
#[reactor_state]
pub enum FetchState { Idle, Loading, Success(String), Error(String) }

// With extra derives
#[reactor_state(derive(Hash, Eq))]
pub struct TagState { pub tag: u32 }
```

---

## Define a Reactor

### Mode A — bring your own state

```rust
use gloc::{reactor, reactor_state, Reactor};

#[reactor_state]
pub struct CounterState { pub count: i32 }

#[reactor(state = CounterState)]
pub struct CounterReactor {}

impl CounterReactor {
    pub fn increment(&mut self) { self.emit(CounterState { count: self.count + 1 }); }
    pub fn decrement(&mut self) { self.emit(CounterState { count: self.count - 1 }); }
    pub fn reset(&mut self)     { self.emit(CounterState { count: 0 }); }
}
```

### Mode B — let GLoC generate the state struct

```rust
use gloc::{reactor, Reactor};

#[reactor]
pub struct ToggleReactor {
    #[state] pub active: bool,
}

// Macro generates: pub struct ToggleReactorState { pub active: bool }

impl ToggleReactor {
    pub fn toggle(&mut self) {
        self.emit(ToggleReactorState { active: !self.active });
    }
}
```

### What the macro generates

| Generated | Description |
|---|---|
| `impl Reactor` | `state()`, `emit()` with change-detection, `stream()` |
| `new(initial)` | Constructor + fires `GlocObserver::on_create` |
| `fire(neutron)` | Event dispatch — only when `neutrons = N` is set |
| `impl Deref<Target = State>` | Access state fields directly: `reactor.count` |

### Attribute options

| Argument | Effect |
|---|---|
| `state = SomeType` | Mode A — use an existing type as state |
| `neutrons = SomeType` | Opt-in event dispatch — generates `fire()`; you write `on_event()` |
| `no_new` | Skip `new()` generation |

---

## Reactive Stream

Every reactor carries a **built-in `GlocStream`**. Subscribe to it for fan-out
reactive notifications:

```rust
let reactor = CounterReactor::new(CounterState { count: 0 });

// Multiple subscribers — all fire on every emit()
let _h1 = reactor.stream().listen(|_, new| println!("UI: {}", new.count));
let _h2 = reactor.stream().listen(|old, new| log::info!("{old:?} → {new:?}"));

reactor.increment(); // both listeners fire
```

**`ListenerHandle`** — `listen()` returns a handle. Drop it to cancel:

```rust
{
    let _h = reactor.stream().listen(|_, new| println!("{}", new.count));
    reactor.increment(); // fires
} // _h dropped → listener cancelled
reactor.increment(); // silent
```

Call `handle.forget()` to keep the listener permanently:

```rust
reactor.stream().listen(|_, new| println!("{}", new.count)).forget();
```

**Close signal** — get notified when a reactor shuts down:

```rust
let _h = reactor.stream().on_close(|| println!("reactor closed — cleaning up"));

let provider = GlocProvider::new(Arc::new(Mutex::new(reactor)));
provider.release(); // → on_close() fires, stream.close() fires callbacks
```

**Reactor-to-reactor** — one reactor subscribes to another:

```rust
// OrderReactor watches CartReactor
let _h = cart.stream().listen(move |_, new| {
    if new.status == CartStatus::CheckedOut {
        order.emit(OrderState::placed());
    }
});

// Clean up when cart is gone
let _close = cart.stream().on_close(|| println!("cart gone"));
```

---

## Observers

### Typed listener — `GlocListener`

```rust
use gloc::GlocListener;

struct Logger;

impl GlocListener<CounterReactor> for Logger {
    fn on_transition(&self, old: &CounterState, new: &CounterState) {
        println!("{} → {}", old.count, new.count);
    }
}

let provider = GlocProvider::new(Arc::new(Mutex::new(counter)));
let _h = provider.attach_listener(Logger);
provider.update(|r| r.increment()); // prints: 0 → 1
```

### Global observer — `GlocObserver`

Observe every reactor in the app from one place. Two methods for transitions:

```rust
use gloc::{GlocObserver, set_observer};

struct AppLogger;

impl GlocObserver for AppLogger {
    // Debug-formatted strings — simple logging
    fn on_transition(&self, reactor: &str, old: &str, new: &str) {
        println!("[{reactor}] {old} → {new}");
    }

    // Typed state — structured analytics, downcast to real types
    fn on_change(&self, reactor: &str, _old: &dyn std::any::Any, new: &dyn std::any::Any) {
        if let Some(s) = new.downcast_ref::<CounterState>() {
            println!("counter is now {}", s.count);
        }
    }

    fn on_create(&self, reactor: &str) { println!("[{reactor}] created"); }
    fn on_close(&self, reactor: &str)  { println!("[{reactor}] closed"); }
}

fn main() {
    set_observer(AppLogger); // once, before any reactor is created
    // ...
}
```

---

## Dioxus Integration

`gloc-dioxus` connects reactors to Dioxus with zero prop drilling.
The stream→signal bridge is automatic — any `emit()` call updates the Dioxus
signal and schedules a re-render without any manual wiring.

```rust
use dioxus::prelude::*;
use gloc::{reactor, reactor_state, Reactor};
use gloc_dioxus::{gloc_builder, use_gloc, use_gloc_provide};

#[reactor_state]
pub struct CounterState { pub count: i32 }

#[reactor(state = CounterState)]
pub struct CounterReactor {}

impl CounterReactor {
    pub fn increment(&mut self) { self.emit(CounterState { count: self.count + 1 }); }
    pub fn decrement(&mut self) { self.emit(CounterState { count: self.count - 1 }); }
}

#[component]
fn App() -> Element {
    // Provide once at the root — accessible anywhere in the tree
    use_gloc_provide(|| CounterReactor::new(CounterState { count: 0 }));
    rsx! { Counter {} }
}

#[component]
fn Counter() -> Element {
    let counter = use_gloc::<CounterReactor>(); // no prop drilling

    // gloc_builder! re-runs closure on every emit() — no manual signal.set()
    gloc_builder!(CounterReactor, |state| rsx! {
        div {
            p { "Count: {state.count}" }
            button { onclick: move |_| counter.update(|r| r.decrement()), "−" }
            button { onclick: move |_| counter.update(|r| r.increment()), "+" }
        }
    })
}

fn main() { dioxus::launch(App); }
```

Full showcase with 5 pages:

```sh
cargo run -p gloc-example-dioxus
```

| Page | Feature |
|------|---------|
| /counter  | `gloc_builder!` — rebuilds on every emit |
| /neutrons | `gloc_builder!(when:)` — rebuild guard + neutron dispatch |
| /theme    | `gloc_consumer!(build_when:, listen_when:)` — both guards |
| /cart     | `gloc_listener!(when:)` — side effect gated on status transition |
| sidebar   | Mode B `#[reactor]` — shared across all pages |

---

## Feature Flags

| Crate | Feature | Effect |
|---|---|---|
| `gloc` | `tracing` | `tracing::debug!` inside `emit()` — logs every state transition. Zero cost when disabled. |

---

## Project Structure

```
GLoC/
├── gloc-core/          Reactor, State, GlocStream, GlocProvider, GlocListener, GlocObserver
├── gloc-macro/         #[reactor], #[reactor_state]
├── gloc/               Umbrella crate
├── gloc-test/          ReactorTester + reactor_test! macro
├── gloc-dioxus/        Dioxus adapter — use_gloc_provide, use_gloc, gloc_builder!
├── gloc-axum/          Axum adapter — AxumReactor, new_axum_state
├── gloc-bevy/          Bevy adapter — GlocPlugin, GlocResource
└── examples/
    ├── dioxus/         Desktop UI — 5-page feature showcase
    ├── axum/           HTTP API — CartReactor + InventoryReactor
    ├── bevy/           Headless game — PlayerReactor + WaveReactor
    └── cli/            Terminal REPL — task manager
```

---

## Contributing

GLoC welcomes contributions of every kind.

> **The only hard rule:** every change must go through a Pull Request and pass CI.

```sh
# Clone and branch
git clone https://github.com/<your-username>/gloc.git
cd gloc
git checkout -b feat/your-feature

# Full local check suite
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p gloc-macro --test ui_tests
```

| CI Job | Local command |
|--------|--------------|
| build | `cargo build --workspace` |
| test | `cargo test --workspace` |
| fmt | `cargo fmt --all -- --check` |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` |

---

## License

Licensed under the [MIT License](LICENSE-MIT).

---

<div align="center">

Built with Rust 🦀 — designed for everyone.

[github.com/godwinjk/gloc](https://github.com/godwinjk/gloc) · [crates.io/crates/gloc](https://crates.io/crates/gloc) · [docs.rs/gloc](https://docs.rs/gloc)

</div>
