//! [`GlocObserver`] — global interceptor for all reactor transitions.
//!
//! A `GlocObserver` is registered once at application startup and receives
//! every state transition from every reactor that uses a consumer. It is the
//! central place for cross-cutting concerns: logging, analytics, crash
//! reporting, and debugging.
//!
//! # Design
//!
//! - **Optional** — if no observer is registered the system runs with zero overhead.
//! - **Global** — one observer for the whole application, not per-reactor.
//! - **Type-erased** — observes any reactor; states are passed as `Debug`
//!   strings so the trait needs no generic parameters.
//! - **Thread-safe** — the registry uses `OnceLock<RwLock<_>>` from `std`.
//!
//! # Lifecycle events
//!
//! | Method | When it fires |
//! |---|---|
//! | `on_create` | A reactor enters a managed scope |
//! | `on_transition` | A real state change is emitted (`old != new`) |
//! | `on_close` | A reactor leaves its managed scope |
//!
//! # Example
//!
//! ```rust
//! use gloc_core::observer::{GlocObserver, set_observer};
//!
//! struct AppLogger;
//!
//! impl GlocObserver for AppLogger {
//!     fn on_transition(&self, reactor: &str, old: &str, new: &str) {
//!         println!("[{reactor}] {old} → {new}");
//!     }
//!
//!     fn on_create(&self, reactor: &str) {
//!         println!("[{reactor}] created");
//!     }
//!
//!     fn on_close(&self, reactor: &str) {
//!         println!("[{reactor}] closed");
//!     }
//! }
//!
//! // Called once at the start of main() — before any managed reactor is created.
//! set_observer(AppLogger);
//! ```

use std::sync::{Arc, OnceLock, RwLock};

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// A global observer that receives lifecycle and transition events from every
/// managed reactor in the application.
///
/// Implement this trait on a single application-level struct and register it
/// with [`set_observer`]. All three methods have default no-op implementations
/// so you only override what you need.
///
/// # Contract
///
/// - Methods are called synchronously on the thread that caused the event.
/// - Implementations must not block or call `emit()` — doing so will deadlock.
/// - All methods must be `Send + Sync` since the observer is shared globally.
pub trait GlocObserver: Send + Sync + 'static {
    /// Called on every real state transition with **Debug-formatted strings**.
    ///
    /// Implement this for simple logging and string-based diagnostics.
    ///
    /// # Parameters
    ///
    /// - `reactor_name` — fully-qualified Rust type name, e.g.
    ///   `"my_app::counter::CounterReactor"`. Use `.split("::").last()` for
    ///   the short name.
    /// - `old` — previous state as `"{:?}"`
    /// - `new` — new state as `"{:?}"`
    fn on_transition(&self, reactor_name: &str, old: &str, new: &str) {
        let _ = (reactor_name, old, new);
    }

    /// Called on every real state transition with the **actual state values**.
    ///
    /// Implement this for structured logging, analytics, or pattern matching
    /// on real state types — downcast with `old.downcast_ref::<MyState>()`.
    ///
    /// Both `on_transition` (strings) and `on_change` (typed) fire on every
    /// transition. Implement whichever fits your use case, or both.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc_core::observer::GlocObserver;
    ///
    /// #[derive(Clone, PartialEq, Debug)]
    /// struct CounterState { count: i32 }
    ///
    /// struct Analytics;
    ///
    /// impl GlocObserver for Analytics {
    ///     fn on_change(&self, reactor_name: &str, old: &dyn std::any::Any, new: &dyn std::any::Any) {
    ///         if let (Some(o), Some(n)) = (
    ///             old.downcast_ref::<CounterState>(),
    ///             new.downcast_ref::<CounterState>(),
    ///         ) {
    ///             println!("{reactor_name}: {} → {}", o.count, n.count);
    ///         }
    ///     }
    /// }
    /// ```
    fn on_change(&self, reactor_name: &str, old: &dyn std::any::Any, new: &dyn std::any::Any) {
        let _ = (reactor_name, old, new);
    }

    /// Called when a reactor is created. Default is a no-op.
    fn on_create(&self, reactor_name: &str) {
        let _ = reactor_name;
    }

    /// Called when a reactor is closed. Default is a no-op.
    fn on_close(&self, reactor_name: &str) {
        let _ = reactor_name;
    }
}

// ---------------------------------------------------------------------------
// Global registry
// ---------------------------------------------------------------------------

/// The global observer slot.
///
/// `OnceLock` initialises the `RwLock` lazily on first access. The `RwLock`
/// holds an `Option` so the observer can be set or cleared (useful in tests).
static OBSERVER: OnceLock<RwLock<Option<Arc<dyn GlocObserver>>>> = OnceLock::new();

/// Returns a reference to the global `RwLock`, initialising it on first call.
fn registry() -> &'static RwLock<Option<Arc<dyn GlocObserver>>> {
    OBSERVER.get_or_init(|| RwLock::new(None))
}

/// Registers `observer` as the global transition interceptor.
///
/// Must be called **once** at application startup, before any managed
/// reactor is created.
///
/// Calling this a second time replaces the previous observer.
///
/// # Example
///
/// ```rust
/// use gloc_core::observer::{GlocObserver, set_observer};
///
/// struct Logger;
/// impl GlocObserver for Logger {
///     fn on_transition(&self, reactor: &str, old: &str, new: &str) {
///         println!("[{reactor}] {old} → {new}");
///     }
/// }
///
/// set_observer(Logger);
/// ```
pub fn set_observer(observer: impl GlocObserver) {
    *registry().write().unwrap() = Some(Arc::new(observer));
}

/// Returns the current global observer, if one has been registered.
///
/// Returns `None` with zero cost if no observer has been set — the
/// `OnceLock` is still uninitialised in that case.
///
/// Used internally by `GlocProvider` and the `#[reactor]` macro.
pub fn observer() -> Option<Arc<dyn GlocObserver>> {
    OBSERVER.get()?.read().unwrap().clone()
}

/// Removes the currently registered observer.
///
/// Primarily useful in tests to reset state between test cases. Not
/// intended for production use.
pub fn clear_observer() {
    *registry().write().unwrap() = None;
}
