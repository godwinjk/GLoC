# GLoC Reactor Generator

Generate [GLoC](https://github.com/godwinjk/GLoC) reactor files directly from the VS Code Explorer — no boilerplate, no copy-paste.

---

## Features

- **Right-click any folder** in the Explorer → **New Reactor**
- Choose between a **simple reactor** or one **with neutrons** (event-driven dispatch)
- Live **code preview** updates as you type the reactor name
- Validates that the name is PascalCase and the file doesn't already exist
- Creates a ready-to-use `.rs` file in the selected folder and opens it immediately

---

## Usage

1. Right-click a folder in the Explorer sidebar
2. Select **New Reactor**
3. Enter a PascalCase name (e.g. `Counter`, `ShoppingCart`)
4. Pick the reactor type:
   - **Without Neutrons** — plain reactor with direct method calls
   - **With Neutrons** — reactor with an event enum and `fire()` dispatch
5. Click **Create**

The file is created as `snake_case.rs` (e.g. `ShoppingCart` → `shopping_cart.rs`) and opened in the editor.

---

## Generated Code

**Without Neutrons**

```rust
use gloc::prelude::*;

#[reactor_state]
pub struct CounterState {
    // TODO: add state fields
}

impl Default for CounterState {
    fn default() -> Self {
        Self {}
    }
}

#[reactor(state = CounterState)]
pub struct CounterReactor {}

impl CounterReactor {
    // TODO: add methods that call self.emit(...)
}
```

**With Neutrons**

```rust
use gloc::prelude::*;

#[reactor_state]
pub struct CounterState {
    // TODO: add state fields
}

impl Default for CounterState {
    fn default() -> Self {
        Self {}
    }
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

---

## Requirements

Add GLoC to your `Cargo.toml`:

```toml
[dependencies]
gloc = "0.1"
```

---

## License

MIT © [Godwin Joseph](https://github.com/godwinjk)
