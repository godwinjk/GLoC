use dioxus::prelude::*;
use gloc::{provider::GlocProvider, stream::ListenerHandle, Reactor};

use crate::context::GlocCtx;

/// A reactive handle to a GLoC reactor provided via [`use_gloc_provide`].
///
/// `GlocHandle<R>` is `Copy` — both fields are `SyncSignal`, which is `Copy`
/// regardless of `R`. Move the same handle into as many closures in `rsx!`
/// as you like without `.clone()`.
///
/// # Reactivity
///
/// The underlying signal is updated automatically by the reactor's built-in
/// stream. Every call to `reactor.emit()` (via any domain method) fires the
/// stream, which fires the listener registered in [`use_gloc_provide`], which
/// calls `signal.set(new_state)`. Dioxus then schedules a re-render for every
/// component that called `state()` on this handle.
///
/// No manual `signal.set()` is ever needed.
///
/// # Example
///
/// ```rust,ignore
/// fn Counter() -> Element {
///     let counter = use_gloc::<CounterReactor>();
///     let count = counter.state().count;   // reactive
///
///     rsx! {
///         p { "{count}" }
///         button { onclick: move |_| counter.update(|r| r.increment()), "+" }
///         button { onclick: move |_| counter.update(|r| r.decrement()), "−" }
///     }
/// }
/// ```
///
/// [`use_gloc_provide`]: crate::use_gloc_provide
pub struct GlocHandle<R: Reactor>
where
    R: 'static,
    R::State: Send + Sync + 'static,
{
    signal: SyncSignal<R::State>,
    provider: SyncSignal<GlocProvider<R>>,
}

impl<R: Reactor> Copy for GlocHandle<R>
where
    R: 'static,
    R::State: Send + Sync + 'static,
{
}

impl<R: Reactor> Clone for GlocHandle<R>
where
    R: 'static,
    R::State: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: Reactor> GlocHandle<R>
where
    R: Send + 'static,
    R::State: Send + Sync + 'static,
{
    pub(crate) fn from_ctx(ctx: GlocCtx<R>) -> Self {
        Self {
            signal: ctx.signal,
            provider: ctx.provider,
        }
    }

    /// Returns a clone of the current reactor state.
    ///
    /// Calling this inside a component's render body subscribes the component
    /// to the signal — any real state transition triggers a re-render
    /// automatically via the reactor's stream.
    pub fn state(&self) -> R::State {
        self.signal.cloned()
    }

    /// Calls `f` with `&mut R`.
    ///
    /// `emit()` inside `f` fires the reactor's built-in stream, which fires
    /// the signal listener registered at provide-time, which updates the signal
    /// and schedules a Dioxus re-render — all automatically.
    ///
    /// ```text
    /// update(|r| r.increment())
    ///   └─ emit(new_state)
    ///         └─ stream fires
    ///               └─ signal.set(new_state)   ← automatic
    ///                     └─ Dioxus re-renders  ✓
    /// ```
    pub fn update(&self, f: impl FnOnce(&mut R)) {
        self.provider.read().update(f);
    }

    /// Registers a closure that fires on every real state transition.
    ///
    /// Returns a [`ListenerHandle`] — drop it to cancel the listener, or
    /// call [`ListenerHandle::forget`] to keep it active permanently.
    pub fn listen(
        &self,
        f: impl Fn(&R::State, &R::State) + Send + Sync + 'static,
    ) -> ListenerHandle {
        self.provider.read().listen(f)
    }
}
