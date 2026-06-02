use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use gloc::{observer, provider::GlocProvider, stream::GlocStream, Reactor};

use crate::{context::GlocCtx, handle::GlocHandle};

/// Injects a reactor into the Dioxus component tree as shared reactive state.
///
/// Call this once per reactor type near the root of the component tree (e.g.
/// in your `App` component). Every descendant can then call
/// [`use_gloc::<R>()`](use_gloc) to get a [`GlocHandle<R>`] without any prop
/// drilling.
///
/// `factory` is called exactly once — on the first render of the component
/// where this hook is placed. Subsequent re-renders of that component reuse
/// the context that was already established.
///
/// # Reactive bridge
///
/// Internally this hook:
///
/// 1. Creates a `GlocProvider<R>` backed by `Arc<Mutex<R>>` + `GlocStream`.
/// 2. Creates a Dioxus `Signal<R::State>` initialised with the starting state.
/// 3. Registers a `GlocStream` listener that calls `signal.set(new)` on every
///    real transition — this is what makes descendant components re-render.
/// 4. Notifies the global [`GlocObserver`](gloc::GlocObserver) (`on_create`).
///
/// # Example
///
/// ```rust,ignore
/// fn App() -> Element {
///     use_gloc_provide(|| CounterReactor::new(CounterState::new(0)));
///     use_gloc_provide(|| ThemeReactor::new(Theme::Light));
///
///     rsx! { Router::<Route> {} }
/// }
/// ```
///
/// For multiple reactors just call the hook once per type. Call order must be
/// stable across renders (same rule as all Dioxus hooks).
pub fn use_gloc_provide<R, F>(factory: F)
where
    R: Reactor + Send + 'static,
    R::State: Send + 'static,
    F: FnOnce() -> R + 'static,
{
    use_context_provider(|| {
        let reactor = factory();
        let initial = reactor.state().clone();

        let arc = Arc::new(Mutex::new(reactor));
        let stream = GlocStream::new(initial.clone());
        let provider = GlocProvider::new(arc, stream);

        // Notify global observer — same lifecycle event as GlocProvider::new
        // in other adapters.
        if let Some(obs) = observer() {
            obs.on_create(std::any::type_name::<R>());
        }

        // Dioxus signal that drives re-renders. Updated explicitly in
        // GlocHandle::update() rather than via a stream listener, because
        // Signal<T> uses UnsyncStorage (not Send) and cannot be captured in a
        // GlocStream listener (which requires Send + 'static).
        let state_signal: Signal<R::State> = Signal::new(initial);

        let provider_signal: Signal<GlocProvider<R>> = Signal::new(provider);

        GlocCtx {
            state: state_signal,
            provider: provider_signal,
        }
    });
}

/// Returns a [`GlocHandle<R>`] for the reactor provided by the nearest
/// ancestor that called [`use_gloc_provide::<R>`](use_gloc_provide).
///
/// `GlocHandle<R>` is `Copy`, so it can be moved into multiple closures in
/// `rsx!` without explicit cloning.
///
/// # Panics
///
/// Panics if no ancestor called `use_gloc_provide::<R>()` — same behaviour as
/// Dioxus's own `use_context::<T>()`.
///
/// # Example
///
/// ```rust,ignore
/// fn CounterPage() -> Element {
///     let counter = use_gloc::<CounterReactor>();
///     let count   = counter.state().count;   // reactive
///
///     rsx! {
///         p { "{count}" }
///         button { onclick: move |_| counter.update(|r| r.increment()), "+" }
///         button { onclick: move |_| counter.update(|r| r.decrement()), "−" }
///     }
/// }
/// ```
pub fn use_gloc<R>() -> GlocHandle<R>
where
    R: Reactor + Send + 'static,
    R::State: Send + 'static,
{
    GlocHandle::from_ctx(use_context::<GlocCtx<R>>())
}
