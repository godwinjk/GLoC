//! [`GlocListener`] — react to state transitions on any reactor.
//!
//! A `GlocListener` receives **both sides** of every real state transition
//! (`old → new`) and executes side effects in response.
//!
//! # Design
//!
//! Unlike [`stream.listen()`](crate::stream::GlocStream::listen) (which takes
//! a closure), `GlocListener` is a **trait** — meaning any struct can implement
//! it. This enables:
//!
//! - Dependency injection — pass `&dyn GlocListener` through your app
//! - Multiple concerns in one type — a single struct can be both a listener
//!   and something else (e.g. an analytics service)
//! - Testability — mock listeners in unit tests
//!
//! # Example
//!
//! ```rust
//! use gloc_core::{Reactor, State};
//! use gloc_core::stream::GlocStream;
//! use gloc_core::listener::GlocListener;
//!
//! #[derive(Clone, PartialEq, Debug)]
//! struct CounterState { pub count: i32 }
//!
//! struct CounterReactor { state: CounterState, stream: GlocStream<CounterState> }
//!
//! impl CounterReactor {
//!     fn new(n: i32) -> Self {
//!         let state = CounterState { count: n };
//!         Self { stream: GlocStream::new(state.clone()), state }
//!     }
//! }
//!
//! impl Reactor for CounterReactor {
//!     type State = CounterState;
//!     fn state(&self) -> &CounterState { &self.state }
//!     fn emit(&mut self, next: CounterState) {
//!         if next != self.state {
//!             let old = self.state.clone();
//!             self.state = next.clone();
//!             self.stream.emit_transition(&old, &next);
//!         }
//!     }
//!     fn stream(&self) -> GlocStream<CounterState> { self.stream.clone() }
//! }
//!
//! // A listener that prints every transition
//! struct TransitionLogger;
//!
//! impl GlocListener<CounterReactor> for TransitionLogger {
//!     fn on_transition(&self, old: &CounterState, new: &CounterState) {
//!         println!("count: {} → {}", old.count, new.count);
//!     }
//! }
//! ```

use crate::reactor::Reactor;

/// A trait for types that react to state transitions on a reactor.
///
/// Implement `GlocListener<R>` on any type to receive `(&old_state, &new_state)`
/// on every real state transition — i.e. every time `emit()` produces a value
/// that differs from the current state.
///
/// # Type parameter
///
/// - `R` — any type that implements [`Reactor`].
///
/// # `GlocListener` vs `stream.listen()`
///
/// | | `GlocListener` | `stream.listen()` closure |
/// |---|---|---|
/// | Syntax | `impl GlocListener<R> for MyType` | `reactor.stream().listen(\|old, new\| ...)` |
/// | Receives | `(&old, &new)` | `(&old, &new)` |
/// | Testable | Yes — inject `&dyn GlocListener<R>` | Harder |
/// | Composable | Yes — any struct can implement it | No — one-off closure |
/// | Best for | Services, analytics, navigation | Simple one-off side effects |
pub trait GlocListener<R: Reactor> {
    /// Called synchronously on every state transition where `new != old`.
    ///
    /// # Parameters
    ///
    /// - `old` — the state immediately before this transition
    /// - `new` — the state immediately after this transition
    ///
    /// # Contract
    ///
    /// - Must not block — called synchronously inside `emit_transition`.
    /// - May call `stream.listen()` or `stream.state()` safely — the stream
    ///   lock is released before listeners are called.
    /// - Must not call `emit()` on the **same** reactor from within this method —
    ///   the reactor's `Arc<Mutex>` is still locked by the caller.
    fn on_transition(&self, old: &R::State, new: &R::State);
}
