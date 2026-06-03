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
//! | [`GlocProvider`] | Provides shared read/write access and lifecycle management for a reactor. |
//! | [`GlocListener`] | A trait for reacting to state transitions. |
//! | [`GlocObserver`] | A global observer for all reactor transitions. |
//!
//! ## Quick start — Reactor
//!
//! The recommended way is the `#[reactor]` macro (from the `gloc` crate) which
//! generates all boilerplate automatically. For manual implementation:
//!
//! ```rust
//! use gloc_core::{Reactor, State};
//! use gloc_core::stream::GlocStream;
//!
//! // 1. Define your state.
//! #[derive(Clone, PartialEq, Debug)]
//! struct CounterState { count: i32 }
//!
//! // 2. Define your reactor — carry a GlocStream for fan-out reactivity.
//! struct CounterReactor {
//!     state:  CounterState,
//!     stream: GlocStream<CounterState>,
//! }
//!
//! impl CounterReactor {
//!     pub fn new(count: i32) -> Self {
//!         let state = CounterState { count };
//!         Self { stream: GlocStream::new(state.clone()), state }
//!     }
//!     pub fn increment(&mut self) { self.emit(CounterState { count: self.state().count + 1 }); }
//!     pub fn decrement(&mut self) { self.emit(CounterState { count: self.state().count - 1 }); }
//!     pub fn reset(&mut self)     { self.emit(CounterState { count: 0 }); }
//! }
//!
//! // 3. Implement the trait.
//! impl Reactor for CounterReactor {
//!     type State = CounterState;
//!
//!     fn state(&self) -> &CounterState { &self.state }
//!
//!     fn emit(&mut self, next: CounterState) {
//!         if next != self.state {
//!             let old = self.state.clone();
//!             self.state = next.clone();
//!             self.stream.emit_transition(&old, &next); // notifies all subscribers
//!         }
//!     }
//!
//!     fn stream(&self) -> GlocStream<CounterState> { self.stream.clone() }
//! }
//!
//! // 4. Use it.
//! let mut counter = CounterReactor::new(0);
//!
//! // Subscribe before mutating — receives every real transition.
//! counter.stream().listen(|old, new| {
//!     println!("{} → {}", old.count, new.count);
//! });
//!
//! counter.increment();
//! counter.increment();
//! assert_eq!(counter.state().count, 2);
//! counter.decrement();
//! assert_eq!(counter.state().count, 1);
//! counter.reset();
//! assert_eq!(counter.state().count, 0);
//! ```
//!

pub mod event;
pub mod listener;
pub mod observer;
pub mod provider;
pub mod reactor;
pub mod state;
pub mod stream;

pub use event::{Event, Neutron};
pub use listener::GlocListener;
pub use observer::{clear_observer, observer, set_observer, GlocObserver};
pub use provider::GlocProvider;
pub use reactor::{Reactor, ReactorBase};
pub use state::State;
pub use stream::{GlocStream, GlocSubscription, ListenerHandle};
