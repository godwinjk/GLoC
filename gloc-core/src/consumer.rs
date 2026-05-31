//! [`GlocConsumer`] — read from and mutate a reactor via shared ownership.
//!
//! A `GlocConsumer` holds a shared reference to a reactor and its stream.
//! Multiple consumers can coexist for the same reactor; mutations from any
//! one are immediately visible to all others.
//!
//! # Example
//!
//! ```rust
//! use gloc_core::{Reactor, State};
//! use gloc_core::stream::GlocStream;
//! use gloc_core::consumer::GlocConsumer;
//! use std::sync::{Arc, Mutex};
//!
//! #[derive(Clone, PartialEq, Debug)]
//! struct CounterState { pub count: i32 }
//!
//! struct CounterReactor { state: CounterState }
//! impl CounterReactor {
//!     fn new() -> Self { Self { state: CounterState { count: 0 } } }
//!     fn increment(&mut self) {
//!         let next = self.state().count + 1;
//!         self.emit(CounterState { count: next });
//!     }
//! }
//! impl Reactor for CounterReactor {
//!     type State = CounterState;
//!     fn state(&self) -> &CounterState { &self.state }
//!     fn emit(&mut self, next: CounterState) {
//!         if next != self.state { self.state = next; }
//!     }
//! }
//! ```

use std::sync::{Arc, Mutex};

use crate::listener::GlocListener;
use crate::reactor::Reactor;
use crate::stream::GlocStream;

/// A shared handle for reading and mutating a reactor.
///
/// `GlocConsumer<R>` gives read and write access to a reactor without owning
/// it. Multiple consumers can exist for the same reactor simultaneously — they
/// all share the same `Arc<Mutex<R>>` and [`GlocStream`].
///
/// # Cloning
///
/// Cloning is free — it only increments `Arc` reference counts.
/// Does not require `R: Clone`.
///
/// # Thread safety
///
/// `GlocConsumer<R>` is `Send + Sync` when `R: Send`.
pub struct GlocConsumer<R: Reactor>
where
    R::State: Send,
{
    /// Shared reactor behind a mutex.
    reactor: Arc<Mutex<R>>,

    /// Shared stream — carries every state transition to listeners.
    stream: GlocStream<R::State>,
}

/// Manual `Clone` — does not require `R: Clone`.
impl<R: Reactor> Clone for GlocConsumer<R>
where
    R::State: Send,
{
    fn clone(&self) -> Self {
        Self {
            reactor: Arc::clone(&self.reactor),
            stream: self.stream.clone(),
        }
    }
}

impl<R: Reactor> GlocConsumer<R>
where
    R::State: Send,
{
    /// Constructs a `GlocConsumer` from a shared reactor and stream.
    ///
    /// Framework adapters (e.g. `gloc-dioxus`) call this when wiring a reactor
    /// into a component tree. Application code can also call it directly when
    /// sharing a reactor across threads without a UI context.
    pub fn new(reactor: Arc<Mutex<R>>, stream: GlocStream<R::State>) -> Self {
        Self { reactor, stream }
    }

    /// Returns a clone of the current state.
    ///
    /// Acquires the reactor mutex, clones the state, and releases the lock.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gloc_core::{Reactor, State};
    /// # use gloc_core::consumer::GlocConsumer;
    /// # use gloc_core::stream::GlocStream;
    /// # use std::sync::{Arc, Mutex};
    /// # #[derive(Clone, PartialEq, Debug)] struct S(i32);
    /// # struct R { s: S } impl R { fn new() -> Self { Self { s: S(5) } } }
    /// # impl Reactor for R { type State = S; fn state(&self) -> &S { &self.s } fn emit(&mut self, s: S) { self.s = s; } }
    /// let reactor = Arc::new(Mutex::new(R::new()));
    /// let stream = GlocStream::new(S(5));
    /// let consumer = GlocConsumer::new(reactor, stream);
    /// assert_eq!(consumer.state().0, 5);
    /// ```
    pub fn state(&self) -> R::State {
        self.reactor.lock().unwrap().state().clone()
    }

    /// Calls a closure that mutates the reactor and pushes any state transition
    /// into the shared [`GlocStream`].
    ///
    /// The closure receives `&mut R` — the reactor itself — and can call any
    /// method on it. If the reactor's `emit()` produces a new state, the
    /// transition is propagated to the stream and all listeners are notified.
    ///
    /// # How it works
    ///
    /// 1. Records the state before the closure runs
    /// 2. Acquires the reactor mutex and runs the closure
    /// 3. Records the state after the closure
    /// 4. If old ≠ new, calls `stream.emit_transition(old, new)`
    ///
    /// # Parameters
    ///
    /// - `f` — a closure that accepts `&mut R` and calls one or more domain methods
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gloc_core::{Reactor, State};
    /// # use gloc_core::consumer::GlocConsumer;
    /// # use gloc_core::stream::GlocStream;
    /// # use std::sync::{Arc, Mutex};
    /// # #[derive(Clone, PartialEq, Debug)] struct S(i32);
    /// # struct R { s: S }
    /// # impl R { fn new() -> Self { Self { s: S(0) } } fn inc(&mut self) { self.emit(S(self.s.0 + 1)); } }
    /// # impl Reactor for R { type State = S; fn state(&self) -> &S { &self.s } fn emit(&mut self, s: S) { if s != self.s { self.s = s; } } }
    /// let reactor = Arc::new(Mutex::new(R::new()));
    /// let stream = GlocStream::new(S(0));
    /// let consumer = GlocConsumer::new(reactor, stream);
    /// consumer.update(|r| r.inc());
    /// assert_eq!(consumer.state().0, 1);
    /// ```
    pub fn update(&self, f: impl FnOnce(&mut R)) {
        let old = {
            let guard = self.reactor.lock().unwrap();
            guard.state().clone()
        };
        {
            let mut guard = self.reactor.lock().unwrap();
            f(&mut guard);
        }
        let new = {
            let guard = self.reactor.lock().unwrap();
            guard.state().clone()
        };
        if old != new {
            self.stream.emit_transition(&old, &new);
            // Notify global observer — formatted as Debug strings so the
            // type-erased observer trait needs no generic parameters.
            if let Some(obs) = crate::observer::observer() {
                obs.on_transition(
                    std::any::type_name::<R>(),
                    &format!("{old:?}"),
                    &format!("{new:?}"),
                );
            }
        }
    }

    /// Registers a closure listener on the underlying stream.
    ///
    /// The closure receives `(&old_state, &new_state)` on every real
    /// state transition — same semantics as [`GlocStream::listen`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gloc_core::{Reactor, State};
    /// # use gloc_core::consumer::GlocConsumer;
    /// # use gloc_core::stream::GlocStream;
    /// # use std::sync::{Arc, Mutex};
    /// # #[derive(Clone, PartialEq, Debug)] struct S(i32);
    /// # struct R { s: S }
    /// # impl R { fn new() -> Self { Self { s: S(0) } } fn inc(&mut self) { self.emit(S(self.s.0 + 1)); } }
    /// # impl Reactor for R { type State = S; fn state(&self) -> &S { &self.s } fn emit(&mut self, s: S) { if s != self.s { self.s = s; } } }
    /// let reactor = Arc::new(Mutex::new(R::new()));
    /// let stream = GlocStream::new(S(0));
    /// let consumer = GlocConsumer::new(reactor, stream);
    ///
    /// let log: std::sync::Arc<std::sync::Mutex<Vec<i32>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    /// let log_clone = log.clone();
    /// consumer.listen(move |_old, new| log_clone.lock().unwrap().push(new.0));
    ///
    /// consumer.update(|r| r.inc());
    /// consumer.update(|r| r.inc());
    ///
    /// assert_eq!(*log.lock().unwrap(), vec![1, 2]);
    /// ```
    pub fn listen(&self, f: impl Fn(&R::State, &R::State) + Send + 'static) {
        self.stream.listen(f);
    }

    /// Attaches a [`GlocListener`] implementation to this consumer.
    ///
    /// This is the trait-object version of [`listen`](Self::listen) — useful
    /// when you have a service struct that implements `GlocListener<R>` and
    /// want to attach it without converting it to a closure.
    ///
    /// # Parameters
    ///
    /// - `listener` — any value that implements `GlocListener<R> + Send + 'static`
    pub fn attach_listener<L>(&self, listener: L)
    where
        L: GlocListener<R> + Send + 'static,
    {
        self.stream.listen(move |old, new| {
            listener.on_transition(old, new);
        });
    }

    /// Returns a clone of the underlying [`GlocStream`].
    ///
    /// Use this to pass the raw stream to framework adapters.
    pub fn stream(&self) -> GlocStream<R::State> {
        self.stream.clone()
    }
}
