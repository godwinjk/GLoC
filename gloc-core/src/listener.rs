//! [`GlocListener`] — react to state transitions on any reactor.
//!
//! A `GlocListener` receives **both sides** of every real state transition
//! (`old → new`) and executes side effects in response.
//!
//! # Design
//!
//! Unlike [`on_change`](crate::stream::GlocStream::listen) (which receives a
//! closure), `GlocListener` is a **trait** — meaning any struct can implement
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
//! use gloc_core::listener::GlocListener;
//!
//! #[derive(Clone, PartialEq, Debug)]
//! struct CounterState { pub count: i32 }
//!
//! struct CounterReactor { state: CounterState }
//! impl Reactor for CounterReactor {
//!     type State = CounterState;
//!     fn state(&self) -> &CounterState { &self.state }
//!     fn emit(&mut self, next: CounterState) {
//!         if next != self.state { self.state = next; }
//!     }
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
/// # When to use `GlocListener` vs `on_change`
///
/// | | `GlocListener` | `on_change` closure |
/// |---|---|---|
/// | Syntax | `impl GlocListener<R> for MyType` | `reactor.on_change(\|old, new\| ...)` |
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
    /// - This method must not block — it is called synchronously inside `emit()`
    /// - Do not call `emit()` from inside this method — it will deadlock
    ///   on the stream's internal mutex
    fn on_transition(&self, old: &R::State, new: &R::State);
}
