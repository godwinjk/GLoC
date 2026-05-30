//! # gloc — Global Logic Component
//!
//! A universal business logic architecture for Rust, inspired by the
//! [Bloc/Cubit pattern](https://bloclibrary.dev) from Flutter.
//!
//! GLOC provides a clean separation between *business logic* and *presentation*
//! (or *infrastructure*) that works in **any** Rust application:
//! web frontends, desktop GUIs, backend servers, CLIs, and embedded targets.
//!
//! ## Core concepts
//!
//! | Concept | Description |
//! |---------|-------------|
//! | [`State`] | A snapshot of some domain's data at a point in time. |
//! | [`Cubit`] | Owns state; exposes methods that call `emit()` to transition it. |
//! | [`CubitBase`] | A ready-made `Cubit` implementation for simple use-cases. |
//!
//! ## Quick start — Cubit
//!
//! ```rust
//! use gloc::{Cubit, State};
//!
//! // 1. Define your state.
//! #[derive(Clone, PartialEq, Debug)]
//! struct CounterState {
//!     count: i32,
//! }
//!
//! // 2. Define your cubit.
//! struct CounterCubit {
//!     state: CounterState,
//! }
//!
//! impl CounterCubit {
//!     pub fn new() -> Self {
//!         Self { state: CounterState { count: 0 } }
//!     }
//!
//!     pub fn increment(&mut self) {
//!         let next = self.state().count + 1;
//!         self.emit(CounterState { count: next });
//!     }
//!
//!     pub fn decrement(&mut self) {
//!         let next = self.state().count - 1;
//!         self.emit(CounterState { count: next });
//!     }
//!
//!     pub fn reset(&mut self) {
//!         self.emit(CounterState { count: 0 });
//!     }
//! }
//!
//! // 3. Implement the trait.
//! impl Cubit for CounterCubit {
//!     type State = CounterState;
//!
//!     fn state(&self) -> &CounterState { &self.state }
//!
//!     fn emit(&mut self, next: CounterState) {
//!         if &next != self.state() {
//!             self.state = next;
//!         }
//!     }
//! }
//!
//! // 4. Use it.
//! let mut counter = CounterCubit::new();
//! counter.increment();
//! counter.increment();
//! assert_eq!(counter.state().count, 2);
//! counter.decrement();
//! assert_eq!(counter.state().count, 1);
//! counter.reset();
//! assert_eq!(counter.state().count, 0);
//! ```
//!
//! ## Roadmap
//!
//! | Phase | Version | Status |
//! |-------|---------|--------|
//! | Cubit core | v0.1 | ✅ current |
//! | `#[cubit]` macro | v0.2 | planned |
//! | Bloc core | v0.3 | planned |
//! | `#[bloc]` macro + adapters | v0.4 | planned |
//! | Stable release | v1.0 | planned |

pub mod cubit;
pub mod state;

pub use cubit::{Cubit, CubitBase};
pub use state::State;
