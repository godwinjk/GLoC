//! # gloc-core — Global Logic Component
//!
//! A universal business logic architecture for Rust.
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
//! | [`Reactor`] | Owns state; exposes methods that call `emit()` to transition it. |
//! | [`ReactorBase`] | A ready-made `Reactor` implementation for simple use-cases. |
//! | [`GlocStream`] | A shared, observable state stream. |
//! | [`GlocConsumer`] | A shared handle for reading and mutating a reactor. |
//! | [`GlocListener`] | A trait for reacting to state transitions. |
//! | [`GlocObserver`] | A global observer for all reactor transitions. |
//!
//! ## Quick start — Reactor
//!
//! ```rust
//! use gloc_core::{Reactor, State};
//!
//! // 1. Define your state.
//! #[derive(Clone, PartialEq, Debug)]
//! struct CounterState {
//!     count: i32,
//! }
//!
//! // 2. Define your reactor.
//! struct CounterReactor {
//!     state: CounterState,
//! }
//!
//! impl CounterReactor {
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
//! impl Reactor for CounterReactor {
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
//! let mut counter = CounterReactor::new();
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
//! | Reactor core | v0.1 | ✅ current |
//! | `#[reactor]` macro | v0.2 | planned |
//! | Event dispatch | v0.3 | planned |
//! | Adapters (Dioxus, Axum, Bevy) | v0.4 | planned |
//! | Stable release | v1.0 | planned |

pub mod consumer;
pub mod event;
pub mod listener;
pub mod observer;
pub mod reactor;
pub mod state;
pub mod stream;

pub use consumer::GlocConsumer;
pub use event::Event;
pub use listener::GlocListener;
pub use observer::{clear_observer, observer, set_observer, GlocObserver};
pub use reactor::{Reactor, ReactorBase};
pub use state::State;
pub use stream::{GlocStream, GlocSubscription};
