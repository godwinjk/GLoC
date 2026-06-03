//! [`GlocProvider`] — shared ownership and lifecycle management for reactors.
//!
//! A `GlocProvider` wraps a reactor in `Arc<Mutex<R>>` for shared mutable
//! access across threads or async contexts. The reactor's built-in stream
//! (from `Reactor::stream()`) is used directly — no separate stream is stored
//! in the provider.
//!
//! # Example
//!
//! ```rust
//! use gloc_core::{Reactor, State};
//! use gloc_core::stream::GlocStream;
//! use gloc_core::provider::GlocProvider;
//! use std::sync::{Arc, Mutex};
//!
//! #[derive(Clone, PartialEq, Debug)]
//! struct CounterState { pub count: i32 }
//!
//! struct CounterReactor { state: CounterState, stream: GlocStream<CounterState> }
//! impl CounterReactor {
//!     fn new() -> Self {
//!         let state = CounterState { count: 0 };
//!         Self { stream: GlocStream::new(state.clone()), state }
//!     }
//!     fn increment(&mut self) {
//!         let next = self.state().count + 1;
//!         self.emit(CounterState { count: next });
//!     }
//! }
//! impl Reactor for CounterReactor {
//!     type State = CounterState;
//!     fn state(&self) -> &CounterState { &self.state }
//!     fn emit(&mut self, next: CounterState) {
//!         if next != self.state {
//!             let old = self.state.clone();
//!             self.state = next;
//!             self.stream.emit_transition(&old, &self.state);
//!         }
//!     }
//!     fn stream(&self) -> GlocStream<CounterState> { self.stream.clone() }
//! }
//!
//! let provider = GlocProvider::new(Arc::new(Mutex::new(CounterReactor::new())));
//! provider.update(|r| r.increment());
//! assert_eq!(provider.state().count, 1);
//! ```

use std::sync::{Arc, Mutex};

use crate::listener::GlocListener;
use crate::reactor::Reactor;
use crate::stream::ListenerHandle;

/// Provides shared read/write access to a reactor with explicit lifecycle management.
///
/// `GlocProvider<R>` wraps a reactor in `Arc<Mutex<R>>` for multi-owner, multi-thread
/// access. The reactor's own built-in stream (from [`Reactor::stream`]) carries all
/// transitions — the provider itself stores no stream.
///
/// # Cloning
///
/// Cloning is free — only increments the `Arc` reference count.
///
/// # Thread safety
///
/// `GlocProvider<R>` is `Send + Sync` when `R: Send`.
pub struct GlocProvider<R: Reactor>
where
    R::State: Send,
{
    reactor: Arc<Mutex<R>>,
}

impl<R: Reactor> Clone for GlocProvider<R>
where
    R::State: Send,
{
    fn clone(&self) -> Self {
        Self {
            reactor: Arc::clone(&self.reactor),
        }
    }
}

impl<R: Reactor> GlocProvider<R>
where
    R::State: Send,
{
    /// Wraps a reactor in a `GlocProvider` for shared access.
    pub fn new(reactor: Arc<Mutex<R>>) -> Self {
        Self { reactor }
    }

    /// Returns a clone of the current state.
    pub fn state(&self) -> R::State {
        self.reactor.lock().unwrap().state().clone()
    }

    /// Calls `f` with `&mut R`.
    ///
    /// `emit()` inside `f` fires the reactor's built-in stream automatically —
    /// no manual stream management needed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gloc_core::{Reactor, State};
    /// # use gloc_core::provider::GlocProvider;
    /// # use gloc_core::stream::GlocStream;
    /// # use std::sync::{Arc, Mutex};
    /// # #[derive(Clone, PartialEq, Debug)] struct S(i32);
    /// # struct R { s: S, stream: GlocStream<S> }
    /// # impl R { fn new() -> Self { Self { s: S(0), stream: GlocStream::new(S(0)) } } fn inc(&mut self) { self.emit(S(self.s.0 + 1)); } }
    /// # impl Reactor for R { type State = S; fn state(&self) -> &S { &self.s } fn emit(&mut self, s: S) { if s != self.s { let o = self.s.clone(); self.s = s.clone(); self.stream.emit_transition(&o, &s); } } fn stream(&self) -> GlocStream<S> { self.stream.clone() } }
    /// let provider = GlocProvider::new(Arc::new(Mutex::new(R::new())));
    /// provider.update(|r| r.inc());
    /// assert_eq!(provider.state().0, 1);
    /// ```
    pub fn update(&self, f: impl FnOnce(&mut R)) {
        let mut guard = self.reactor.lock().unwrap();
        f(&mut guard);
        // emit() inside f already fired the stream and notified the observer.
    }

    /// Registers a listener on the reactor's built-in stream.
    ///
    /// The listener receives `(&old, &new)` on every real state transition,
    /// in registration order, synchronously.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gloc_core::{Reactor, State};
    /// # use gloc_core::provider::GlocProvider;
    /// # use gloc_core::stream::GlocStream;
    /// # use std::sync::{Arc, Mutex};
    /// # #[derive(Clone, PartialEq, Debug)] struct S(i32);
    /// # struct R { s: S, stream: GlocStream<S> }
    /// # impl R { fn new() -> Self { Self { s: S(0), stream: GlocStream::new(S(0)) } } fn inc(&mut self) { self.emit(S(self.s.0 + 1)); } }
    /// # impl Reactor for R { type State = S; fn state(&self) -> &S { &self.s } fn emit(&mut self, s: S) { if s != self.s { let o = self.s.clone(); self.s = s.clone(); self.stream.emit_transition(&o, &s); } } fn stream(&self) -> GlocStream<S> { self.stream.clone() } }
    /// let provider = GlocProvider::new(Arc::new(Mutex::new(R::new())));
    /// let log = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    /// let log2 = log.clone();
    /// let _h = provider.listen(move |_old, new| log2.lock().unwrap().push(new.0));
    /// provider.update(|r| r.inc());
    /// provider.update(|r| r.inc());
    /// assert_eq!(*log.lock().unwrap(), vec![1, 2]);
    /// ```
    /// Registers a listener on the reactor's stream. Returns a [`ListenerHandle`]
    /// — drop it to cancel, or call [`ListenerHandle::forget`] to make permanent.
    pub fn listen(&self, f: impl Fn(&R::State, &R::State) + Send + Sync + 'static) -> ListenerHandle
    where
        R::State: 'static,
    {
        self.reactor.lock().unwrap().stream().listen(f)
    }

    /// Attaches a [`GlocListener`] to the reactor's stream.
    /// Returns a [`ListenerHandle`] for cancellation.
    pub fn attach_listener<L>(&self, listener: L) -> ListenerHandle
    where
        L: GlocListener<R> + Send + Sync + 'static,
        R::State: 'static,
    {
        self.reactor
            .lock()
            .unwrap()
            .stream()
            .listen(move |old, new| {
                listener.on_transition(old, new);
            })
    }

    /// Returns a clone of the reactor's built-in stream.
    pub fn stream(&self) -> crate::stream::GlocStream<R::State> {
        self.reactor.lock().unwrap().stream()
    }

    /// Fires the close lifecycle:
    /// 1. Calls `reactor.on_close()` — user cleanup hook.
    /// 2. Notifies the global `GlocObserver::on_close`.
    /// 3. Closes the reactor's built-in stream — fires close listeners,
    ///    then clears all transition listeners.
    pub fn close(&self)
    where
        R::State: 'static,
    {
        if let Ok(mut guard) = self.reactor.lock() {
            guard.on_close();
            if let Some(obs) = crate::observer::observer() {
                obs.on_close(std::any::type_name::<R>());
            }
            let stream = guard.stream();
            drop(guard); // release reactor lock before closing stream
            stream.close();
        }
    }

    /// Releases this provider — calls `close()` then drops.
    pub fn release(self)
    where
        R::State: 'static,
    {
        self.close();
    }
}
