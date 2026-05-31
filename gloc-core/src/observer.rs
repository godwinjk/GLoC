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
    /// Called on every real state transition (`old != new`).
    ///
    /// # Parameters
    ///
    /// - `reactor_name` — the fully-qualified Rust type name of the reactor,
    ///   e.g. `"my_app::counter::CounterReactor"`. Use `split("::").last()`
    ///   if you only want the short name.
    /// - `old` — the previous state formatted with `{:?}`
    /// - `new` — the new state formatted with `{:?}`
    fn on_transition(&self, reactor_name: &str, old: &str, new: &str);

    /// Called when a reactor enters a managed scope.
    ///
    /// Default implementation is a no-op.
    fn on_create(&self, reactor_name: &str) {
        let _ = reactor_name;
    }

    /// Called when a reactor leaves its managed scope.
    ///
    /// Default implementation is a no-op.
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
/// Used internally by `GlocConsumer` and the `#[reactor]` macro.
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
