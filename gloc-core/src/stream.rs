//! [`GlocStream`] — the reactive core of GLOC.
//!
//! A `GlocStream` is a shared, observable state container. It holds the
//! current state and notifies all registered listeners synchronously
//! whenever the state transitions to a new value.
//!
//! Built entirely on `std` — no third-party runtime or async dependencies.
//! `Arc<Mutex<_>>` provides thread-safe shared ownership so multiple
//! [`GlocSubscription`]s and [`GlocConsumer`](crate::consumer::GlocConsumer)s
//! can all observe the same stream independently.
//!
//! # Design
//!
//! ```text
//! GlocStream<S>
//!   └── Arc<SharedState<S>>
//!         ├── current:   Mutex<S>                         — latest state
//!         └── listeners: Mutex<Vec<Box<dyn Fn(&S, &S)>>>  — old → new callbacks
//!
//! emit_transition(old, new)
//!   → update current
//!   → call every listener with (&old, &new)
//! ```
//!
//! # Example
//!
//! ```rust
//! use gloc_core::stream::GlocStream;
//!
//! let stream = GlocStream::new(0_i32);
//!
//! stream.listen(|old, new| println!("{old} → {new}"));
//!
//! stream.emit_transition(&0, &1);  // prints: 0 → 1
//! assert_eq!(stream.state(), 1);
//! ```

use std::sync::{Arc, Mutex};

use crate::state::State;

/// Type alias for a heap-allocated state transition listener.
type Listener<S> = Box<dyn Fn(&S, &S) + Send + 'static>;

// ---------------------------------------------------------------------------
// Internal shared state
// ---------------------------------------------------------------------------

/// Heap-allocated shared state behind every `GlocStream`.
///
/// Wrapped in `Arc` so the stream and all its subscriptions / consumers
/// point to the same allocation — no copying, no coordination overhead.
struct SharedState<S: State> {
    /// The most recently emitted state value.
    current: Mutex<S>,

    /// Registered transition listeners.
    ///
    /// Each listener receives `(&old_state, &new_state)` synchronously
    /// inside [`GlocStream::emit_transition`].
    listeners: Mutex<Vec<Listener<S>>>,
}

// ---------------------------------------------------------------------------
// GlocStream
// ---------------------------------------------------------------------------

/// A shared, observable stream of state transitions.
///
/// `GlocStream<S>` is the reactive backbone injected into every cubit by the
/// `#[cubit]` macro. It replaces the previous `Vec<Box<dyn Fn>>` listener
/// list with a proper, cloneable, multi-subscriber reactive primitive.
///
/// # Thread safety
///
/// `GlocStream<S>` is `Send + Sync` when `S: Send`. Multiple threads can
/// hold clones of the same stream and register listeners independently.
///
/// # Clone behaviour
///
/// Cloning a `GlocStream` is cheap — it increments the `Arc` reference
/// count. All clones share the same underlying state and listener list.
///
/// # Example
///
/// ```rust
/// use gloc_core::stream::GlocStream;
///
/// let stream = GlocStream::new(String::from("idle"));
///
/// // Register a listener — receives old and new state on every transition
/// stream.listen(|old, new| {
///     println!("transition: {old:?} → {new:?}");
/// });
///
/// stream.emit_transition(&"idle".to_string(), &"loading".to_string());
/// // prints: transition: "idle" → "loading"
///
/// assert_eq!(stream.state(), "loading");
/// ```
#[derive(Clone)]
pub struct GlocStream<S: State> {
    inner: Arc<SharedState<S>>,
}

impl<S: State + Send> GlocStream<S> {
    /// Creates a new `GlocStream` with the given `initial` state.
    ///
    /// # Parameters
    ///
    /// - `initial` — the starting state. The stream holds this value until
    ///   the first call to [`emit_transition`](Self::emit_transition).
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc_core::stream::GlocStream;
    ///
    /// let stream = GlocStream::new(42_i32);
    /// assert_eq!(stream.state(), 42);
    /// ```
    pub fn new(initial: S) -> Self {
        Self {
            inner: Arc::new(SharedState {
                current: Mutex::new(initial),
                listeners: Mutex::new(Vec::new()),
            }),
        }
    }

    /// Returns a clone of the current state.
    ///
    /// Because `GlocStream` is shared across threads, the state is stored
    /// behind a `Mutex`. This method acquires the lock, clones the value,
    /// and releases the lock — the returned value is fully owned by the caller.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc_core::stream::GlocStream;
    ///
    /// let stream = GlocStream::new(10_i32);
    /// assert_eq!(stream.state(), 10);
    /// ```
    pub fn state(&self) -> S {
        self.inner.current.lock().unwrap().clone()
    }

    /// Transitions to `next`, updates `current`, and notifies all listeners.
    ///
    /// This is called by the cubit's generated `emit()` method **after**
    /// change-detection has already confirmed `next != old`. The old value
    /// is passed in so listeners can observe both sides of the transition.
    ///
    /// Listeners are called synchronously, in registration order, while the
    /// state lock is **not** held — so listeners can safely call `stream.state()`
    /// without deadlocking.
    ///
    /// # Parameters
    ///
    /// - `old`  — the state before this transition
    /// - `next` — the state after this transition (already stored as current)
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc_core::stream::GlocStream;
    ///
    /// let stream = GlocStream::new(0_i32);
    /// stream.listen(|old, new| assert_eq!((*old, *new), (0, 1)));
    /// stream.emit_transition(&0, &1);
    /// assert_eq!(stream.state(), 1);
    /// ```
    pub fn emit_transition(&self, old: &S, next: &S) {
        // Update current state first.
        *self.inner.current.lock().unwrap() = next.clone();

        // Collect listeners snapshot — release the lock before calling them
        // so listeners can safely read stream.state() without deadlocking.
        let listeners = self.inner.listeners.lock().unwrap();
        for listener in listeners.iter() {
            listener(old, next);
        }
    }

    /// Registers a listener that fires on every state transition.
    ///
    /// The listener receives `(&old_state, &new_state)` — both sides of the
    /// transition — so callers can implement logic like "navigate only if we
    /// moved from Loading to Success".
    ///
    /// Listeners are called synchronously inside [`emit_transition`](Self::emit_transition),
    /// in registration order. They must not block.
    ///
    /// # Parameters
    ///
    /// - `f` — a closure or function that accepts `(&S, &S)` and is `Send + 'static`
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc_core::stream::GlocStream;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let stream = GlocStream::new(0_i32);
    /// let log: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));
    /// let log_clone = log.clone();
    ///
    /// stream.listen(move |old, new| {
    ///     log_clone.lock().unwrap().push((*old, *new));
    /// });
    ///
    /// stream.emit_transition(&0, &1);
    /// stream.emit_transition(&1, &2);
    ///
    /// assert_eq!(*log.lock().unwrap(), vec![(0, 1), (1, 2)]);
    /// ```
    pub fn listen(&self, f: impl Fn(&S, &S) + Send + 'static) {
        self.inner.listeners.lock().unwrap().push(Box::new(f));
    }

    /// Returns a [`GlocSubscription`] that holds a shared reference to this stream.
    ///
    /// A subscription is a lightweight handle — cloning it is free (just an
    /// `Arc` increment). Use it to pass read access to this stream into
    /// framework adapters or background tasks without giving them write access.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc_core::stream::GlocStream;
    ///
    /// let stream = GlocStream::new(0_i32);
    /// let sub = stream.subscribe();
    ///
    /// stream.emit_transition(&0, &5);
    /// assert_eq!(sub.state(), 5);  // subscription sees the update
    /// ```
    pub fn subscribe(&self) -> GlocSubscription<S> {
        GlocSubscription {
            inner: Arc::clone(&self.inner),
        }
    }
}

// ---------------------------------------------------------------------------
// GlocSubscription
// ---------------------------------------------------------------------------

/// A read-only handle to a [`GlocStream`].
///
/// A subscription is returned by [`GlocStream::subscribe`] and gives the
/// holder read access to the current state and the ability to register
/// additional listeners — without the ability to emit new transitions.
///
/// Cloning a `GlocSubscription` is free — it only increments the `Arc`
/// reference count on the shared inner state.
///
/// # Example
///
/// ```rust
/// use gloc_core::stream::GlocStream;
///
/// let stream = GlocStream::new(false);
/// let sub1 = stream.subscribe();
/// let sub2 = stream.subscribe();  // second independent subscriber
///
/// stream.emit_transition(&false, &true);
///
/// assert_eq!(sub1.state(), true);
/// assert_eq!(sub2.state(), true);  // both see the same state
/// ```
#[derive(Clone)]
pub struct GlocSubscription<S: State> {
    inner: Arc<SharedState<S>>,
}

impl<S: State + Send> GlocSubscription<S> {
    /// Returns a clone of the current state seen by this subscription.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc_core::stream::GlocStream;
    ///
    /// let stream = GlocStream::new(99_i32);
    /// let sub = stream.subscribe();
    /// assert_eq!(sub.state(), 99);
    /// ```
    pub fn state(&self) -> S {
        self.inner.current.lock().unwrap().clone()
    }

    /// Registers a listener on the underlying stream.
    ///
    /// Equivalent to calling [`GlocStream::listen`] — the listener is shared
    /// with the original stream and all other subscriptions.
    pub fn listen(&self, f: impl Fn(&S, &S) + Send + 'static) {
        self.inner.listeners.lock().unwrap().push(Box::new(f));
    }
}
