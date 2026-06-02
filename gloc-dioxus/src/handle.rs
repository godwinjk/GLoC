use dioxus::prelude::*;
use gloc::{provider::GlocProvider, Reactor};

use crate::context::GlocCtx;

/// A reactive handle to a GLoC reactor injected via [`use_gloc_provide`].
///
/// `GlocHandle<R>` is `Copy` — it holds two Dioxus `Signal` indices, both of
/// which are `Copy` regardless of `R`. This means the handle can be moved into
/// multiple closures in `rsx!` without requiring explicit `.clone()` calls.
///
/// # Reading state
///
/// Call [`state`](Self::state) inside a component's render body to read the
/// current state. Dioxus tracks this signal access and automatically
/// re-renders the component on the next transition.
///
/// ```rust,ignore
/// let counter = use_gloc::<CounterReactor>();
/// let count = counter.state().count;   // reactive — re-renders on change
/// ```
///
/// # Mutating state
///
/// Call [`update`](Self::update) with a closure that calls one or more methods
/// on `&mut R`. After the closure returns, any state change is propagated
/// through the [`GlocStream`](gloc::GlocStream) and into the Dioxus signal,
/// scheduling a re-render.
///
/// ```rust,ignore
/// button {
///     onclick: move |_| counter.update(|r| r.increment()),
///     "+"
/// }
/// ```
///
/// For neutron dispatch, pass the `fire()` call inside `update`:
///
/// ```rust,ignore
/// button {
///     onclick: move |_| counter.update(|r| r.fire(CounterEvent::Increment)),
///     "+"
/// }
/// ```
///
/// # Observing transitions
///
/// Use [`listen`](Self::listen) to register a side-effect listener that fires
/// on every real transition. This is called on the same thread as the mutation,
/// synchronously, so keep it non-blocking.
///
/// [`use_gloc_provide`]: crate::use_gloc_provide
pub struct GlocHandle<R: Reactor>
where
    R: 'static,
    R::State: Send + 'static,
{
    state: Signal<R::State>,
    provider: Signal<GlocProvider<R>>,
}

// `Signal<T>` is `Copy` regardless of `T`, so `GlocHandle<R>` is `Copy`
// as long as both fields are signals. Derive generates overly restrictive
// bounds (R: Copy), so we implement it manually.
impl<R: Reactor> Copy for GlocHandle<R>
where
    R: 'static,
    R::State: Send + 'static,
{
}

impl<R: Reactor> Clone for GlocHandle<R>
where
    R: 'static,
    R::State: Send + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: Reactor> GlocHandle<R>
where
    R: Send + 'static,
    R::State: Send + 'static,
{
    pub(crate) fn from_ctx(ctx: GlocCtx<R>) -> Self {
        Self {
            state: ctx.state,
            provider: ctx.provider,
        }
    }

    /// Returns a clone of the current reactor state.
    ///
    /// Calling this inside a component's render body subscribes the component
    /// to the underlying signal — any real state transition triggers a
    /// re-render automatically.
    pub fn state(&self) -> R::State {
        self.state.cloned()
    }

    /// Calls `f` with `&mut R` and propagates any resulting state change.
    ///
    /// The full chain on a real transition:
    ///
    /// 1. `f(&mut reactor)` — domain method runs, calls `emit(next)`
    /// 2. `GlocProvider::update` — emits to `GlocStream` + notifies global observer
    /// 3. The Dioxus state signal is set to the new state
    /// 4. Dioxus schedules a re-render for every component that read `state()`
    ///
    /// Must be called on the Dioxus UI thread (i.e. inside an event handler).
    /// `Signal<T>` uses `UnsyncStorage` which is not `Send`.
    ///
    /// If the state did not change (`Reactor::emit` change-detection), the
    /// signal is still set but Dioxus dedups equal writes so no re-render fires.
    pub fn update(&self, f: impl FnOnce(&mut R)) {
        let provider = self.provider.read();
        provider.update(f);
        let new_state = provider.state();
        // Release the provider read guard before writing the state signal.
        drop(provider);
        // Signal is Copy — copy the handle so we can call set() on &mut Signal.
        let mut state = self.state;
        state.set(new_state);
    }

    /// Registers a closure that fires on every real state transition.
    ///
    /// The closure receives `(&old_state, &new_state)` synchronously on the
    /// thread that called [`update`](Self::update). Must not block or call
    /// `emit()`.
    pub fn listen(&self, f: impl Fn(&R::State, &R::State) + Send + 'static) {
        self.provider.read().listen(f);
    }
}
