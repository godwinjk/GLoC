<div align="center">

# GLoC
![GLoC](asset/gloc_logo.svg)
_The **G** is intentional. GLoC started as a hobby project called **G**odwin's **B**usiness **L**ogic **C**omponent,
born from a mission to bring Flutter's legendary **BLoC** architecture into Rust.
But as it grows to serve the wider open-source community, that **G** now stands for **Global**.
One pattern. Universal. Everywhere Rust runs._

A universal business logic architecture for Rust.

[![CI — PR](https://github.com/godwinjk/gloc/actions/workflows/pr.yml/badge.svg)](https://github.com/godwinjk/gloc/actions/workflows/pr.yml)
[![CI — Main](https://github.com/godwinjk/gloc/actions/workflows/main.yml/badge.svg)](https://github.com/godwinjk/gloc/actions/workflows/main.yml)
[![Crates.io](https://img.shields.io/crates/v/gloc.svg)](https://crates.io/crates/gloc)
[![Docs.rs](https://docs.rs/gloc/badge.svg)](https://docs.rs/gloc)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](#license)

</div>

---

## What is GLoC?

GLoC separates **business logic** from **presentation** in any Rust application — inspired by Flutter's BLoC architecture but with its own identity.

The core abstraction is a **`Reactor`**: one type that owns a slice of state, exposes domain methods, and supports both direct calls and event dispatch. Works anywhere Rust runs — desktop, web, server, CLI, embedded.

---

## Concepts

### ⚛️ Reactor

The central unit of business logic. Owns the current **State**, exposes methods to mutate it, and notifies subscribers on every real transition.

```rust
#[reactor(state = CounterState)]
pub struct CounterReactor {}

impl CounterReactor {
    pub fn increment(&mut self) {
        self.emit(CounterState { count: self.count + 1 });
    }
}
```

### ☢️ Neutron (Event)

An immutable event fired *at* a reactor via `fire()`. Any type that satisfies `Debug + Send + 'static` is a Neutron — no trait to implement, no import needed.

```rust
#[derive(Debug)]
pub enum CounterNeutron { Increment, Decrement, Reset }

#[reactor(state = CounterState, neutrons = CounterNeutron)]
pub struct CounterReactor {}

impl CounterReactor {
    fn on_event(&mut self, neutron: CounterNeutron) {
        match neutron {
            CounterNeutron::Increment => self.emit(CounterState { count: self.count + 1 }),
            CounterNeutron::Decrement => self.emit(CounterState { count: self.count - 1 }),
            CounterNeutron::Reset     => self.emit(CounterState { count: 0 }),
        }
    }
}

reactor.fire(CounterNeutron::Increment);
```

### 🔋 State

Pure data — a snapshot of what the reactor knows. Any `Clone + PartialEq + Debug` type is a State. `#[reactor_state]` injects the derives automatically.

```rust
#[reactor_state]
pub struct CounterState { pub count: i32 }
```

GLoC performs **change detection** — `emit()` with an equal value is a no-op.

### Other primitives

| | |
|---|---|
| `GlocProvider` | Shared `Arc<Mutex<R>>` handle — use when multiple components or threads own the same reactor |
| `GlocListener` | Typed `old → new` observer trait |
| `GlocObserver` | Global hook — receives every transition across all reactors |

---

## Installation

```toml
[dependencies]
gloc = "0.2"
```

With tracing:

```toml
[dependencies]
gloc    = { version = "0.2", features = ["tracing"] }
tracing = "0.1"
```

---

## Quick Start

```rust
use gloc::{reactor, reactor_state, Reactor};

#[reactor_state]
pub struct CounterState { pub count: i32 }

#[reactor(state = CounterState)]
pub struct CounterReactor {}

impl CounterReactor {
    pub fn increment(&mut self) {
        self.emit(CounterState { count: self.count + 1 });
    }
}

fn main() {
    let mut counter = CounterReactor::new(CounterState { count: 0 });

    counter.subscribe().listen(|old, new| {
        println!("{} → {}", old.count, new.count);
    });

    counter.increment(); // 0 → 1
    counter.increment(); // 1 → 2
}
```

**Mode B** — let GLoC generate the state struct from annotated fields:

```rust
#[reactor]
pub struct ToggleReactor {
    #[state] pub active: bool,
}
// Generates: pub struct ToggleReactorState { pub active: bool }
```

---

## Roadmap

| Version | Status | Description |
|---|---|---|
| v0.1 | ✅ Released | `Reactor` trait, `ReactorBase`, `State` blanket impl |
| v0.2 | ✅ Latest | `#[reactor]` macro, neutron dispatch, reactive layer, framework adapters |
| v0.3 | 🔲 Planned | `gloc-dioxus` — first-class Dioxus adapter with `use_gloc()` hook |
| v1.0 | 🔲 Planned | Stable API, dedicated docs site, DevTools |

---

## Contributing

Every contribution is welcome — bug reports, docs, tests, new features, or framework adapters.

```sh
git clone https://github.com/<your-username>/gloc.git
cd gloc && git checkout -b feat/your-feature

# Before pushing
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

> Every change must go through a Pull Request and pass CI before merging.

---

## License

Licensed under the [MIT License](LICENSE-MIT).

---

<div align="center">

Built with Rust 🦀 — designed for everyone.

[crates.io/crates/gloc](https://crates.io/crates/gloc) · [docs.rs/gloc](https://docs.rs/gloc) · [Wiki](https://github.com/godwinjk/GLoC/wiki)

</div>
