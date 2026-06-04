# GLoC Reactor Generator

A [RustRover](https://www.jetbrains.com/rust/) plugin that scaffolds
[GLoC](https://github.com/godwinjk/gloc) reactor files directly from the Project
view, so you never have to write the boilerplate by hand.

## Features

- **New → GLoC Reactor** context-menu action on any directory in the Project view
- Live preview of the generated Rust source before the file is created
- Two reactor modes:
  - **Simple** — state struct + `#[reactor(state = …)]` with `self.emit(…)` stubs
  - **With Neutrons** — adds a `#[derive(Debug)]` neutron enum and a `fire()` /
    `on_event()` dispatch handler
- PascalCase name validation with inline error messages
- File opens automatically in the editor after creation

## Usage

1. Right-click a directory in the **Project** tool window.
2. Choose **New → GLoC Reactor**.
3. Enter a PascalCase reactor name (e.g. `Counter`, `CartItem`).
4. Select **Without Neutrons** or **With Neutrons**.
5. Review the preview, then click **OK**.

The plugin creates `<Name>.rs` in the chosen directory with the correct GLoC
macros wired up and ready to fill in.

## Generated code examples

### Simple reactor

```rust
use gloc::prelude::*;

#[reactor_state]
pub struct CounterState {
    // TODO: add state fields
}

impl Default for CounterState {
    fn default() -> Self { Self {} }
}

#[reactor(state = CounterState)]
pub struct CounterReactor {}

impl CounterReactor {
    // TODO: add methods that call self.emit(...)
}
```

### Reactor with neutrons

```rust
use gloc::prelude::*;

#[reactor_state]
pub struct CounterState {
    // TODO: add state fields
}

impl Default for CounterState {
    fn default() -> Self { Self {} }
}

#[derive(Debug)]
pub enum CounterNeutron {
    // TODO: add neutron variants
}

#[reactor(state = CounterState, neutrons = CounterNeutron)]
pub struct CounterReactor {}

impl CounterReactor {
    fn on_event(&mut self, neutron: CounterNeutron) {
        match neutron {
            // TODO: handle neutron variants
        }
    }
}
```

## Requirements

- RustRover 2024.3 or later
- [GLoC](https://crates.io/crates/gloc) added to your `Cargo.toml`
