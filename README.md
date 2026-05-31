<div align="center">

# GLOC
![Bloc](asset/gloc_logo.svg)
_The **G** is intentional. GLoc started as a hobby project called **G**odwin's **B**usiness **L**ogic **C**omponent,
born from a mission to bring Flutter’s legendary **BLoC** architecture into Rust. 
But as it grows to serve the wider open-source community, that **G** now stands for **Global**. 
One pattern. Universal. Everywhere Rust runs._

A universal business logic architecture for Rust,  
a faithful recreation of the [Bloc/Cubit pattern](https://bloclibrary.dev) from Flutter.

[![CI — PR](https://github.com/godwinjk/gloc/actions/workflows/pr.yml/badge.svg)](https://github.com/godwinjk/gloc/actions/workflows/pr.yml)
[![CI — Main](https://github.com/godwinjk/gloc/actions/workflows/main.yml/badge.svg)](https://github.com/godwinjk/gloc/actions/workflows/main.yml)
[![Crates.io](https://img.shields.io/crates/v/gloc.svg)](https://crates.io/crates/gloc)
[![Docs.rs](https://docs.rs/gloc/badge.svg)](https://docs.rs/gloc)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](#license)

</div>

---

## What is GLOC?


GLOC - a Rust port of the [Bloc](https://bloclibrary.dev) architecture that powers state management in Flutter.
Flutter's Bloc is one of the most battle-tested and beloved patterns in mobile
development. GLOC's goal was simple: _recreate that same clean separation of
business logic and UI, but make it work anywhere Rust runs_, not just in one framework.

GLOC separates **business logic** from **presentation** in any Rust application.  
Write your domain logic once, run it everywhere Rust runs.

```
┌─────────────────────────────────────────────────────────────┐
│  Without GLoC           │  With GLoC                        │
│─────────────────────────│───────────────────────────────────│
│  Logic tangled in UI    │  Cubit owns logic                 │
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
- [Quick Start](#quick-start)
- [Define State](#define-state)
- [Define a Cubit](#define-a-cubit)
- [Observers](#observers)
- [Reactive Layer](#reactive-layer)
- [Dioxus Example](#dioxus-example)
- [Feature Flags](#feature-flags)
- [Comparison with Flutter Bloc](#comparison-with-flutter-bloc)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Concepts

| Concept | Description |
|---------|-------------|
| **State** | An immutable snapshot of your domain's data. Any `Clone + PartialEq + Debug` type is automatically a `State`. |
| **Cubit** | Owns one slice of state. Exposes domain methods that call `emit()` to transition to the next state. |
| **emit()** | State-transition primitive. Built-in change-detection — emitting the same value is a no-op. |
| **GlocStream** | Reactive state container inside every cubit — notifies listeners on every transition. |
| **GlocProvider** | Shares a cubit across a scope via `Arc<Mutex<C>>` — no prop drilling. |
| **GlocConsumer** | Reads and mutates through a provider. Multiple consumers share the same cubit. |
| **GlocListener** | Trait for typed `old → new` transition observers. |

---

## Installation

Add a single dependency — `gloc` includes both the core traits and the `#[cubit]` macro:

```toml
[dependencies]
gloc = "0.2"
```

Then import everything from one place:

```rust
use gloc::{cubit, Cubit, State, CubitBase};
```

**Advanced** — use the individual crates if you only need part of the library:

```toml
[dependencies]
gloc-core  = "0.2"   # traits only — Cubit, State, CubitBase
gloc-macro = "0.2"   # #[cubit] macro only
```

**With tracing** — logs every state transition via the [`tracing`](https://crates.io/crates/tracing) crate:

```toml
[dependencies]
gloc    = { version = "0.2", features = ["tracing"] }
tracing = "0.1"
```

---

## Quick Start

```rust
use gloc::{cubit, cubit_state, Cubit, GlocProvider};

// 1. State — derives are automatic
#[cubit_state]
pub struct CounterState { pub count: i32 }

// 2. Cubit — one line, macro generates impl Cubit, new(), on_change(), subscribe()
#[cubit(state = CounterState)]
pub struct CounterCubit {}

// 3. Domain logic only — no boilerplate
impl CounterCubit {
    pub fn increment(&mut self) {
        self.emit(CounterState { count: self.state().count + 1 });
    }
}

fn main() {
    // 4. Provide and consume
    let provider = GlocProvider::new(CounterCubit::new(CounterState { count: 0 }));
    let consumer = provider.consumer();

    consumer.listen(|old, new| println!("{} → {}", old.count, new.count));
    consumer.update(|c| c.increment()); // prints: 0 → 1
    consumer.update(|c| c.increment()); // prints: 1 → 2

    assert_eq!(consumer.state().count, 2);
}
```

---

## Define State

Any `Clone + PartialEq + Debug` type is automatically a `State` — no explicit impl needed.
Use `#[cubit_state]` to skip writing the derives:

```rust
use gloc::cubit_state;

// Struct state
#[cubit_state]
pub struct CounterState { pub count: i32 }

// Enum state — great for loading flows
#[cubit_state]
pub enum FetchState { Idle, Loading, Success(String), Error(String) }

// With extra derives
#[cubit_state(derive(Hash, Eq))]
pub struct TagState { pub tag: u32 }
```

---

## Define a Cubit

### Mode A — bring your own state

```rust
use gloc::{cubit, cubit_state, Cubit};

#[cubit_state]
pub struct CounterState { pub count: i32 }

#[cubit(state = CounterState)]
pub struct CounterCubit {}

impl CounterCubit {
    pub fn increment(&mut self) {
        self.emit(CounterState { count: self.state().count + 1 });
    }
    pub fn decrement(&mut self) {
        self.emit(CounterState { count: self.state().count - 1 });
    }
    pub fn reset(&mut self) {
        self.emit(CounterState { count: 0 });
    }
}

let mut counter = CounterCubit::new(CounterState { count: 0 });
counter.increment();
counter.increment();
assert_eq!(counter.state().count, 2);
```

### Mode B — let GLOC generate the state struct

Annotate fields with `#[state]` — the macro generates `{CubitName}State` automatically:

```rust
use gloc::{cubit, Cubit};

#[cubit]
pub struct ToggleCubit {
    #[state] pub active: bool,
}

// Macro generates: pub struct ToggleCubitState { pub active: bool }

impl ToggleCubit {
    pub fn toggle(&mut self) {
        self.emit(ToggleCubitState { active: !self.state().active });
    }
}

let mut toggle = ToggleCubit::new(ToggleCubitState { active: false });
toggle.toggle();
assert!(toggle.state().active);
```

### What the macro generates

Every `#[cubit]` struct gets:

| Generated | Description |
|---|---|
| `impl Cubit` | `state()`, `emit()` with change-detection |
| `new(initial)` | Constructor — suppress with `no_new` |
| `on_change(old, new)` | Observer registration — suppress with `no_observers` |
| `subscribe()` | Returns a `GlocSubscription` read-only handle |
| `attach_listener(l)` | Attaches a `GlocListener` impl |

### Attribute options

| Argument | Effect |
|---|---|
| `state = SomeType` | Mode A — use an existing type |
| `no_new` | Skip `new()` generation |
| `no_observers` | Skip `on_change()` and stream field |

---

## Observers

`on_change` receives **both** old and new state on every real transition:

```rust
let mut cubit = CounterCubit::new(CounterState { count: 0 });

cubit.on_change(|old, new| {
    println!("{} → {}", old.count, new.count);
});

cubit.increment(); // prints: 0 → 1
cubit.increment(); // prints: 1 → 2
cubit.emit(CounterState { count: 2 }); // no-op — no print
```

For typed observers implement `GlocListener`:

```rust
use gloc::GlocListener;

struct Logger;

impl GlocListener<CounterCubit> for Logger {
    fn on_transition(&self, old: &CounterState, new: &CounterState) {
        println!("{} → {}", old.count, new.count);
    }
}

cubit.attach_listener(Logger);
```

---

## Reactive Layer

Share a cubit across components or threads without prop drilling:

```rust
use gloc::{cubit, cubit_state, Cubit, GlocProvider};

#[cubit_state]
pub struct CounterState { pub count: i32 }

#[cubit(state = CounterState)]
pub struct CounterCubit {}

impl CounterCubit {
    pub fn increment(&mut self) {
        self.emit(CounterState { count: self.state().count + 1 });
    }
}

// Provider — owns the cubit
let provider = GlocProvider::new(CounterCubit::new(CounterState { count: 0 }));

// Multiple consumers — all share the same cubit
let c1 = provider.consumer();
let c2 = provider.consumer();

// Listen to transitions from any consumer
c1.listen(|old, new| println!("{} → {}", old.count, new.count));

// Mutate through any consumer — all observers are notified
c2.update(|c| c.increment()); // c1's listener prints: 0 → 1

assert_eq!(c1.state().count, 1);
assert_eq!(c2.state().count, 1);
```

---

## Dioxus Example

GLOC cubits integrate cleanly with any Rust UI framework. Here is the full counter example using [Dioxus](https://dioxuslabs.com) 0.7 desktop.

The cubit is stored in a Dioxus `Signal` — reads register the component as a subscriber, writes trigger re-renders.

```rust
// src/cubits/counter.rs — zero Dioxus imports, pure domain logic
use gloc::Cubit;
use gloc::cubit;

#[derive(Clone, PartialEq, Debug)]
pub struct CounterState {
    pub count: i32,
    pub label: String,
}

impl CounterState {
    pub fn new(count: i32) -> Self {
        let label = match count {
            i32::MIN..=-1 => "Negative",
            0             => "Zero",
            1..=9         => "Low",
            10..=99       => "Medium",
            _             => "High",
        }.into();
        Self { count, label }
    }
}

#[cubit(state = CounterState)]
pub struct CounterCubit {}

impl CounterCubit {
    pub fn increment(&mut self) {
        self.emit(CounterState::new(self.state().count + 1));
    }
    pub fn decrement(&mut self) {
        self.emit(CounterState::new(self.state().count - 1));
    }
    pub fn reset(&mut self) {
        self.emit(CounterState::new(0));
    }
}
```

```rust
// src/main.rs — Dioxus wiring
#![allow(non_snake_case)]
mod cubits;

use cubits::{CounterCubit, CounterState};
use dioxus::prelude::*;
use gloc::Cubit;

fn main() { dioxus::launch(App); }

#[component]
fn App() -> Element {
    // CounterCubit::new() is generated by #[cubit]
    let cubit = use_signal(|| CounterCubit::new(CounterState::new(0)));
    rsx! { CounterView { cubit } }
}

#[component]
fn CounterView(cubit: Signal<CounterCubit>) -> Element {
    let state = cubit.read().state().clone();
    rsx! {
        div {
            p { "{state.label}: {state.count}" }
            button { onclick: move |_| cubit.write().decrement(), "−" }
            button { onclick: move |_| cubit.write().reset(),     "Reset" }
            button { onclick: move |_| cubit.write().increment(), "+" }
        }
    }
}
```

Run it:

```sh
cargo run -p counter-dioxus-v02
```

Full example source: [`examples/v0.2/counter-dioxus/`](examples/v0.2/counter-dioxus/)

---

## Feature Flags

| Crate | Feature | Effect |
|---|---|---|
| `gloc` | `tracing` | Enables `tracing::debug!` inside `emit()` — logs every state transition. Zero cost when disabled. |
| `gloc-macro` | `tracing` | Same — gates the tracing call in macro-generated `emit()`. |

Enable tracing:

```toml
[dependencies]
gloc       = { version = "0.2", features = ["tracing"] }
gloc-macro = { version = "0.2", features = ["tracing"] }
tracing    = "0.1"
tracing-subscriber = "0.3"
```

Every `emit()` call that transitions state will log:

```
DEBUG CounterCubit{old=CounterState { count: 0 }, new=CounterState { count: 1 }}
```

---

## Comparison with Flutter Bloc

GLOC is a deliberate port of Flutter's Bloc/Cubit pattern into idiomatic Rust.

| Concept | Flutter Bloc | GLOC |
|---|---|---|
| State container | `Cubit<State>` | `Cubit` trait + `#[cubit]` |
| State type | `class CounterState` | Any `Clone + PartialEq + Debug` |
| State transition | `emit(nextState)` | `self.emit(next_state)` |
| Change detection | built-in | built-in (PartialEq guard) |
| Boilerplate removal | `@cubit` annotation | `#[cubit]` proc macro |
| Code generation | `build_runner` (runtime) | proc macro (compile-time, zero overhead) |
| State provider | `BlocProvider` widget | `Signal<MyCubit>` (framework-specific) |
| State listener | `BlocListener` | `on_change(callback)` |
| Observer | `BlocObserver` | `on_change` + tracing feature |
| Scope | Flutter only | **Any Rust application** |

---

## Roadmap

| Phase | Version | Status | Description |
|---|---|---|---|
| 1 | v0.1 | ✅ Released | `Cubit` trait, `CubitBase`, `State` blanket impl |
| 2 | v0.2 | ✅ Released | `#[cubit]` proc macro — Mode A, Mode B, `on_change`, tracing |
| 3 | v0.2 | ✅ Latest | `#[cubit_state]`, `GlocStream`, `GlocProvider`, `GlocConsumer`, `GlocListener` |
| 4 | v0.3 | 🔲 Planned | Framework adapters — `gloc-dioxus`, `gloc-axum`, `gloc-bevy` |
| 5 | v1.0 | 🔲 Planned | Stable API, dedicated docs site, DevTools |

---

## Project Structure

```
GLoC/
├── gloc-core/                  Core crate — published as `gloc-core`
│   └── src/
│       ├── lib.rs
│       ├── state.rs            State trait (blanket impl)
│       └── cubit.rs            Cubit trait + CubitBase
│   └── tests/
│       └── cubit_tests.rs      39 integration tests
│
├── gloc-macro/                 Proc macro crate — published as `gloc-macro`
│   └── src/
│       ├── lib.rs              #[cubit] entry point
│       ├── args.rs             Attribute argument parsing (darling)
│       ├── codegen.rs          Shared code generation helpers
│       ├── mode_a.rs           Mode A — bring-your-own state
│       ├── mode_b.rs           Mode B — generated state struct
│       └── errors.rs           Compile-time diagnostic helpers
│   └── tests/
│       ├── cubit_macro_tests.rs   30 integration tests
│       ├── ui_tests.rs            trybuild runner
│       └── ui/pass|fail/          9 compile-pass/fail scenarios
│
├── gloc/                       Umbrella crate — published as `gloc`
│
├── examples/
│   ├── v0.1/counter-dioxus/    Dioxus 0.7 desktop — manual Cubit
│   └── v0.2/counter-dioxus/    Dioxus 0.7 desktop — #[cubit] macro
│
└── .github/
    ├── CODEOWNERS
    └── workflows/
        ├── pr.yml              PR gate (build, test, fmt, clippy)
        └── main.yml            Post-merge verification
```

---

## Contributing

GLOC welcomes contributions of **every kind** — from first-time open-source
contributors to seasoned Rust experts. No contribution is too small. Whether
you are fixing a typo, improving a doc comment, adding a test case, proposing
a new feature, or porting a framework adapter, you are welcome here.

> **The only hard rule:** every change must go through a Pull Request and
> pass the full CI pipeline before it can be merged. This is not bureaucracy —
> it is how we protect every contributor's work, including yours.

---

### Ways to Contribute

| Type | Examples |
|---|---|
| **Bug reports** | Something panics unexpectedly, wrong behaviour, misleading error message |
| **Documentation** | Improve doc comments, fix typos, add usage examples, translate |
| **Tests** | Add missing test cases, improve coverage, add trybuild fail scenarios |
| **Bug fixes** | Fix a reported issue, improve edge-case handling |
| **New features** | New macro arguments, new generated methods, new `CubitBase` helpers |
| **Framework adapters** | Dioxus, Axum, Bevy, Tauri, Leptos, or any other Rust framework |
| **Performance** | Reduce allocations, improve compile times, benchmark regressions |
| **Tooling** | CI improvements, release automation, dev experience |

---

### Getting Started

**1. Fork and clone**

```sh
git clone https://github.com/<your-username>/gloc.git
cd gloc
```

**2. Create a focused branch**

Branch names should describe the change clearly:

```sh
git checkout -b fix/emit-change-detection-edge-case
git checkout -b feat/cubit-history-observer
git checkout -b docs/improve-mode-b-examples
git checkout -b test/add-trybuild-unit-struct-fail
```

**3. Make your changes**

Run the full local check suite before every push — the same checks CI runs:

```sh
# Format (required — CI will reject unformatted code)
cargo fmt --all

# Lint (required — warnings are treated as errors in CI)
cargo clippy --workspace --all-targets -- -D warnings

# Tests (required — all must pass)
cargo test --workspace

# Trybuild UI tests (required if you touched gloc-macro)
cargo test -p gloc-macro --test ui_tests
```

**4. Open a Pull Request**

- Target the `main` branch
- Fill in the PR description: what changed, why, and how to test it
- The CI pipeline runs automatically — **all four jobs must be green** before the PR can be merged
- `@godwinjk` will review every PR (required by CODEOWNERS)

---

### CI Pipeline — What Must Pass

Every PR must pass all four jobs. You can replicate the exact CI checks locally using the commands below.

| Job | What it checks | Local command |
|---|---|---|
| **build** | `cargo build` in debug and release | `cargo build --workspace` |
| **test** | Unit, integration, doc-tests, trybuild | `cargo test --workspace` |
| **fmt** | Code formatted with `rustfmt` | `cargo fmt --all -- --check` |
| **clippy** | No clippy warnings (treated as errors) | `cargo clippy --workspace --all-targets -- -D warnings` |

### Cleaning the Project

Build artifacts accumulate in `target/` and can grow to several gigabytes.
Clean them before a fresh CI-equivalent run or when diagnosing stale-cache issues.

| Command | What it removes | When to use |
|---|---|---|
| `cargo clean` | Entire `target/` directory (all profiles, all crates) | Full clean before a release or when something feels wrong |
| `cargo clean -p gloc` | Artifacts for the `gloc` crate only | Faster rebuild when only the core crate changed |
| `cargo clean -p gloc-macro` | Artifacts for `gloc-macro` only | After changing the proc macro |
| `rm -rf target/release` | Release profile only, keeps debug | Free space without losing incremental debug builds |
| `rm -rf target/tests` | trybuild test cache only | When UI test snapshots behave unexpectedly |

**Full deep clean** (re-downloads all crates — use sparingly):

```sh
cargo clean
rm -rf ~/.cargo/registry/cache
rm -rf ~/.cargo/registry/src
```

**Recommended before pushing a PR** — run a clean build to make sure nothing relies on stale artifacts:

```sh
cargo clean && cargo test --workspace
```

If CI fails on your PR, check the failing job's log in the Actions tab. Fix
the issue and push a new commit — CI re-runs automatically. Do not force-push
over a failing CI run while a review is in progress.

---

### Code Quality Standards

These standards apply to all contributed code and are enforced in code review:

**Documentation**
- Every `pub` item (struct, trait, fn, type) must have a `///` doc comment
- Doc comments must explain: what it does, parameters, return value, panics (if any), and include at least one `# Example` for non-trivial items
- Do not describe *what* the code does — describe *why* it exists and what a caller needs to know

**Testing**
- Every new feature must ship with tests that cover: the happy path, at least one edge case, and at least one boundary condition
- Tests that verify `Cubit` trait implementations must include a trait-object (`dyn Cubit<State = …>`) test to confirm Dependency Inversion compatibility
- New error paths in `gloc-macro` must have a corresponding `trybuild` fail case with a `.stderr` snapshot

**Design**
- Follow SOLID principles — especially **Single Responsibility** (one cubit, one concern) and **Dependency Inversion** (depend on traits, not concrete types)
- Prefer extending existing abstractions over adding new ones
- Do not introduce breaking changes to the public API without a major version discussion in an issue first
- Generated code (proc macro output) must compile without warnings on the consumer's side

**Style**
- `rustfmt` is the style guide — no manual formatting discussions
- Clippy is the linter — fix all warnings, do not `#[allow(...)]` without a comment explaining why
- No `unwrap()` or `expect()` in library code — return a `Result` or emit a compile-time error
- Comments in source explain *why*, not *what*

---

### Reporting Bugs

Open a [GitHub Issue](https://github.com/godwinjk/gloc/issues) and include:

- GLOC version (`cargo tree | grep gloc`)
- Rust version (`rustc --version`)
- A minimal reproducible example
- The behaviour you expected vs. what actually happened

---

### Suggesting Features

Open a GitHub Issue with the `enhancement` label before writing code.
Describe the use case, not just the implementation. This gives maintainers
a chance to confirm the direction before you invest time building it.

---

## Code of Practice

GLOC is built on the belief that great software comes from a community where
every contributor feels safe, respected, and valued. The following principles
govern how we work together.

### Our Pledge

We pledge to make participation in GLOC a harassment-free experience for
everyone, regardless of age, body size, disability, ethnicity, gender identity
and expression, level of experience, nationality, personal appearance, race,
religion, or sexual identity and orientation.

### Expected Behaviour

- **Be respectful.** Treat every contributor with the same respect you would
  want in return. Disagree with ideas, never with people.
- **Be constructive.** Code review feedback should explain *why* a change is
  needed and suggest *how* to improve it. "This is wrong" is not feedback;
  "this will panic when the Vec is empty — consider adding a guard here" is.
- **Be patient.** Contributors work at different paces and in different time
  zones. Maintainers review PRs as promptly as possible, but response time
  is not guaranteed. Do not chase or demand.
- **Be inclusive.** Write code, comments, and documentation that a developer
  new to Rust, new to state management, or new to open source can understand.
  Avoid jargon where plain language works just as well.
- **Give credit.** Acknowledge others' work. If a PR builds on someone else's
  idea or prior work, say so in the description.
- **Ask questions.** There are no stupid questions. If something in the codebase,
  a doc comment, or a review comment is unclear, ask. Clarity is a contribution.

### Unacceptable Behaviour

The following will not be tolerated in any GLOC space (issues, PRs, discussions,
or any affiliated communication channel):

- Harassment, insults, or personal attacks of any kind
- Discriminatory jokes or language
- Posting others' private information without explicit permission
- Deliberately dismissing or belittling contributions based on experience level
- Sustained disruptive behaviour after being asked to stop

### Enforcement

Violations may be reported by contacting `@godwinjk` directly via GitHub.
All reports will be reviewed promptly and handled with confidentiality.
Maintainers reserve the right to remove, edit, or reject contributions
that do not align with this Code of Practice, and to ban contributors
who engage in unacceptable behaviour.

### Attribution

This Code of Practice is adapted from the
[Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).

---

## Support GLOC

GLOC is free, open-source, and built entirely in personal time driven by a
genuine belief that Rust deserves the same elegant state management patterns
that Flutter developers enjoy every day. If GLOC saves you time, simplifies
your architecture, or just sparks joy — any form of support means the world
and keeps the project moving forward.

---

### ⭐ Star the Repository

The simplest thing you can do. A GitHub star signals to other Rust developers
that this project is worth their attention, helps GLOC surface in search
results, and genuinely motivates continued development.

**[→ Star GLOC on GitHub](https://github.com/godwinjk/gloc)**

---

### 📣 Spread the Word

Every share reaches developers who might never have found GLOC otherwise.

- **Write about it** — blog post, dev.to article, or a Twitter/X thread
- **Talk about it** — mention it at your local Rust meetup or in a conference talk
- **Recommend it** — if GLOC helps your team, tell other teams
- **Share on Reddit** — post to [r/rust](https://www.reddit.com/r/rust/) or [r/flutterdev](https://www.reddit.com/r/flutterdev/) — the Flutter community discovering Rust is a beautiful thing
- **Add it to your project's README** — if you build something with GLOC, a link back helps everyone

---

### ☕ Buy me a Coffee

If GLOC has saved you hours of architecture work, consider buying a coffee.
Every contribution — no matter the size — directly funds time spent on new
features, documentation, framework adapters, and keeping the project alive.

<div align="center">

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/S6S0176OVQ)

</div>

---

### 💙 Donate via PayPal

Prefer PayPal? Donations of any amount are deeply appreciated and go directly
toward GLOC development time.

<div align="center">

[![Donate with PayPal](https://www.paypalobjects.com/en_US/i/btn/btn_donateCC_LG.gif)](https://paypal.me/godwinj)

</div>

---

### 🤝 Sponsor the Project

Interested in a longer-term sponsorship — for your company, team, or
open-source fund? Reach out via GitHub to discuss sponsorship tiers,
acknowledgement in the README, and priority feature requests.

**[→ Open a sponsorship discussion](https://github.com/godwinjk/gloc/discussions)**

---

> Thank you. Truly. Every star, every share, every coffee, every line of
> contributed code makes GLOC better for every Rust developer who uses it.
> — Godwin

---

## License

Licensed under the [MIT License](LICENSE-MIT).

---

<div align="center">

Built with Rust 🦀 — designed for everyone.

[crates.io/crates/gloc](https://crates.io/crates/gloc) · [docs.rs/gloc](https://docs.rs/gloc)

</div>
