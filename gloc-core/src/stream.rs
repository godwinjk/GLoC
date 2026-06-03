//! [`GlocStream`] — the reactive core of GLoC.
//!
//! A `GlocStream` broadcasts every real state transition to all registered
//! listeners synchronously. It supports fan-out (unlimited subscribers),
//! listener cancellation via [`ListenerHandle`], and a close signal that
//! fires when the owning reactor shuts down.
//!
//! # Design
//!
//! ```text
//! GlocStream<S>
//!   └── Arc<SharedState<S>>
//!         ├── current:        Mutex<S>
//!         ├── listeners:      Mutex<BTreeMap<u64, Listener<S>>>
//!         ├── close_listeners: Mutex<BTreeMap<u64, CloseListener>>
//!         ├── next_id:        AtomicU64
//!         └── closed:         AtomicBool
//!
//! emit_transition(old, new)
//!   1. update current  (current lock → released)
//!   2. snapshot listeners  (listeners lock → released)
//!   3. call each listener  (no lock held)
//!
//! close()
//!   1. set closed = true
//!   2. snapshot close_listeners  (lock → released)
//!   3. call each close_listener  (no lock held)
//!   4. clear all listeners
//! ```

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

use crate::state::State;

type Listener<S> = Arc<dyn Fn(&S, &S) + Send + Sync + 'static>;
type CloseListener = Arc<dyn Fn() + Send + Sync + 'static>;

// ---------------------------------------------------------------------------
// Internal shared state
// ---------------------------------------------------------------------------

struct SharedState<S: State> {
    current: Mutex<S>,
    listeners: Mutex<BTreeMap<u64, Listener<S>>>,
    close_listeners: Mutex<BTreeMap<u64, CloseListener>>,
    next_id: AtomicU64,
    closed: AtomicBool,
}

// ---------------------------------------------------------------------------
// ListenerHandle
// ---------------------------------------------------------------------------

/// A cancellable handle returned by [`GlocStream::listen`] and
/// [`GlocStream::on_close`].
///
/// When this handle is **dropped**, the associated listener is automatically
/// removed from the stream — no explicit cancel call needed. This is the RAII
/// pattern: store the handle for as long as you want the listener active.
///
/// Call [`forget`](Self::forget) to keep the listener active permanently
/// even after the handle goes out of scope.
///
/// # Example
///
/// ```rust
/// use gloc_core::stream::GlocStream;
///
/// let stream = GlocStream::new(0_i32);
///
/// {
///     let _handle = stream.listen(|_, new| println!("saw {new}"));
///     stream.emit_transition(&0, &1);  // prints: saw 1
/// } // handle dropped → listener cancelled
///
/// stream.emit_transition(&1, &2);  // no output — listener is gone
/// assert_eq!(stream.state(), 2);
/// ```
pub struct ListenerHandle {
    cancel: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
}

impl ListenerHandle {
    /// Explicitly cancel the listener now.
    pub fn cancel(&mut self) {
        if let Some(f) = self.cancel.take() {
            f();
        }
    }

    /// Detach from RAII — the listener stays active even after this handle
    /// is dropped.
    pub fn forget(mut self) {
        self.cancel = None;
    }
}

impl Clone for ListenerHandle {
    /// Clones the handle — both the original and the clone share the same
    /// cancel function (via `Arc`). The listener is cancelled when the
    /// **first** clone is dropped (subsequent cancels are no-ops).
    fn clone(&self) -> Self {
        Self {
            cancel: self.cancel.clone(),
        }
    }
}

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        self.cancel();
    }
}

// SAFETY: `Arc<dyn Fn() + Send + Sync>` is Send + Sync.
unsafe impl Send for ListenerHandle {}
unsafe impl Sync for ListenerHandle {}

// ---------------------------------------------------------------------------
// GlocStream
// ---------------------------------------------------------------------------

/// A shared, observable stream of state transitions.
///
/// Every `#[reactor]`-generated reactor carries one built-in `GlocStream`.
/// Adapters, UI components, and other reactors subscribe to it via
/// [`listen`](Self::listen). Each subscriber gets `(&old, &new)` on every
/// real state transition.
///
/// # Fan-out
///
/// Any number of listeners can be registered — they all fire on every
/// `emit_transition` call.
///
/// # Cancellation
///
/// [`listen`] returns a [`ListenerHandle`]. Dropping the handle cancels the
/// listener automatically (RAII). Call [`ListenerHandle::forget`] to keep
/// the listener alive permanently.
///
/// # Close signal
///
/// [`on_close`](Self::on_close) registers a callback that fires once when
/// the stream is closed via [`close`](Self::close). Useful for reactor-to-reactor
/// subscriptions where the subscribing reactor needs to clean up when the
/// source reactor shuts down.
///
/// # Thread safety
///
/// `GlocStream<S>` is `Send + Sync` when `S: Send`. Cloning is free — all
/// clones share the same `Arc<SharedState<S>>`.
#[derive(Clone)]
pub struct GlocStream<S: State> {
    inner: Arc<SharedState<S>>,
}

impl<S: State + Send + 'static> GlocStream<S> {
    /// Creates a new `GlocStream` initialised with `initial`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc_core::stream::GlocStream;
    ///
    /// let stream = GlocStream::new(0_i32);
    /// assert_eq!(stream.state(), 0);
    /// ```
    pub fn new(initial: S) -> Self {
        Self {
            inner: Arc::new(SharedState {
                current: Mutex::new(initial),
                listeners: Mutex::new(BTreeMap::new()),
                close_listeners: Mutex::new(BTreeMap::new()),
                next_id: AtomicU64::new(0),
                closed: AtomicBool::new(false),
            }),
        }
    }

    /// Returns a clone of the current state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc_core::stream::GlocStream;
    ///
    /// let stream = GlocStream::new(42_i32);
    /// assert_eq!(stream.state(), 42);
    /// ```
    pub fn state(&self) -> S {
        self.inner.current.lock().unwrap().clone()
    }

    /// Returns `true` if [`close`](Self::close) has been called.
    pub fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::Acquire)
    }

    /// Updates `current` to `next` and notifies all registered listeners.
    ///
    /// Does nothing if the stream is already closed.
    ///
    /// Both the `current` lock and the `listeners` lock are fully released
    /// before any listener fires — so listeners can safely call
    /// [`listen`](Self::listen), [`state`](Self::state), or
    /// [`on_close`](Self::on_close) without deadlocking.
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
        if self.inner.closed.load(Ordering::Acquire) {
            return;
        }

        // 1. Update current — lock acquired and released immediately.
        *self.inner.current.lock().unwrap() = next.clone();

        // 2. Snapshot listeners — lock released before calling any of them.
        let snapshot: Vec<Listener<S>> = self
            .inner
            .listeners
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect();

        // 3. Call every listener with no lock held.
        for listener in &snapshot {
            listener(old, next);
        }
    }

    /// Registers a listener that fires on every state transition.
    ///
    /// Returns a [`ListenerHandle`] — drop it to cancel the listener
    /// automatically, or call [`ListenerHandle::forget`] to make it permanent.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc_core::stream::GlocStream;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let stream = GlocStream::new(0_i32);
    /// let log: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(vec![]));
    /// let log2 = log.clone();
    ///
    /// let handle = stream.listen(move |_, new| log2.lock().unwrap().push(*new));
    ///
    /// stream.emit_transition(&0, &1);
    /// stream.emit_transition(&1, &2);
    /// assert_eq!(*log.lock().unwrap(), vec![1, 2]);
    ///
    /// handle.forget();  // keep listening permanently
    /// ```
    pub fn listen(&self, f: impl Fn(&S, &S) + Send + Sync + 'static) -> ListenerHandle {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        self.inner.listeners.lock().unwrap().insert(id, Arc::new(f));

        let inner: Weak<SharedState<S>> = Arc::downgrade(&self.inner);
        ListenerHandle {
            cancel: Some(Arc::new(move || {
                if let Some(inner) = inner.upgrade() {
                    inner.listeners.lock().unwrap().remove(&id);
                }
            })),
        }
    }

    /// Registers a callback that fires **once** when this stream is closed.
    ///
    /// Use this for reactor-to-reactor subscriptions — when reactor A subscribes
    /// to reactor B's stream, A can register an `on_close` callback to clean up
    /// when B shuts down.
    ///
    /// Returns a [`ListenerHandle`] — drop it to cancel the close callback
    /// before it fires.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc_core::stream::GlocStream;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let stream = GlocStream::new(0_i32);
    /// let closed = Arc::new(Mutex::new(false));
    /// let closed2 = closed.clone();
    ///
    /// let handle = stream.on_close(move || { *closed2.lock().unwrap() = true; });
    ///
    /// assert!(!*closed.lock().unwrap());
    /// stream.close();
    /// assert!(*closed.lock().unwrap());
    ///
    /// handle.forget();
    /// ```
    pub fn on_close(&self, f: impl Fn() + Send + Sync + 'static) -> ListenerHandle {
        // If already closed, fire immediately and return a no-op handle.
        if self.inner.closed.load(Ordering::Acquire) {
            f();
            return ListenerHandle { cancel: None };
        }

        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        self.inner
            .close_listeners
            .lock()
            .unwrap()
            .insert(id, Arc::new(f));

        let inner: Weak<SharedState<S>> = Arc::downgrade(&self.inner);
        ListenerHandle {
            cancel: Some(Arc::new(move || {
                if let Some(inner) = inner.upgrade() {
                    inner.close_listeners.lock().unwrap().remove(&id);
                }
            })),
        }
    }

    /// Closes the stream — fires all close listeners, then clears all
    /// transition listeners.
    ///
    /// After closing:
    /// - [`emit_transition`](Self::emit_transition) is a no-op.
    /// - [`is_closed`](Self::is_closed) returns `true`.
    /// - Any new [`on_close`](Self::on_close) callback fires immediately.
    ///
    /// Called automatically by [`GlocProvider::close`](crate::provider::GlocProvider::close)
    /// when a reactor shuts down.
    pub fn close(&self) {
        // Mark closed first — subsequent emit_transition calls are no-ops.
        self.inner.closed.store(true, Ordering::Release);

        // Snapshot and clear close listeners.
        let close_snapshot: Vec<CloseListener> = {
            let mut guard = self.inner.close_listeners.lock().unwrap();
            let snap = guard.values().cloned().collect();
            guard.clear();
            snap
        };

        // Fire close callbacks with no lock held.
        for f in &close_snapshot {
            f();
        }

        // Clear transition listeners — stream is done.
        self.inner.listeners.lock().unwrap().clear();
    }

    /// Returns a [`GlocSubscription`] — a read-only handle to this stream.
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
/// Returned by [`GlocStream::subscribe`]. Gives the holder read access and
/// the ability to register listeners, without the ability to emit transitions.
///
/// # Example
///
/// ```rust
/// use gloc_core::stream::GlocStream;
///
/// let stream = GlocStream::new(false);
/// let sub1 = stream.subscribe();
/// let sub2 = stream.subscribe();
///
/// stream.emit_transition(&false, &true);
/// assert_eq!(sub1.state(), true);
/// assert_eq!(sub2.state(), true);
/// ```
#[derive(Clone)]
pub struct GlocSubscription<S: State> {
    inner: Arc<SharedState<S>>,
}

impl<S: State + Send + 'static> GlocSubscription<S> {
    /// Returns a clone of the current state.
    pub fn state(&self) -> S {
        self.inner.current.lock().unwrap().clone()
    }

    /// Registers a listener. Returns a [`ListenerHandle`] for cancellation.
    pub fn listen(&self, f: impl Fn(&S, &S) + Send + Sync + 'static) -> ListenerHandle {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        self.inner.listeners.lock().unwrap().insert(id, Arc::new(f));

        let inner: Weak<SharedState<S>> = Arc::downgrade(&self.inner);
        ListenerHandle {
            cancel: Some(Arc::new(move || {
                if let Some(inner) = inner.upgrade() {
                    inner.listeners.lock().unwrap().remove(&id);
                }
            })),
        }
    }

    /// Registers a close callback. Returns a [`ListenerHandle`] for cancellation.
    pub fn on_close(&self, f: impl Fn() + Send + Sync + 'static) -> ListenerHandle {
        if self.inner.closed.load(Ordering::Acquire) {
            f();
            return ListenerHandle { cancel: None };
        }
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        self.inner
            .close_listeners
            .lock()
            .unwrap()
            .insert(id, Arc::new(f));

        let inner: Weak<SharedState<S>> = Arc::downgrade(&self.inner);
        ListenerHandle {
            cancel: Some(Arc::new(move || {
                if let Some(inner) = inner.upgrade() {
                    inner.close_listeners.lock().unwrap().remove(&id);
                }
            })),
        }
    }
}
