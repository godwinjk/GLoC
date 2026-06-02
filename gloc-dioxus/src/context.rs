use dioxus::prelude::*;
use gloc::{provider::GlocProvider, Reactor};

/// Internal per-reactor context stored in the Dioxus scope tree.
///
/// Both fields are Dioxus `Signal`s — lightweight copy handles into a
/// generational arena. `GlocCtx<R>` is therefore `Copy`, satisfying
/// Dioxus's `use_context_provider` requirement cheaply.
///
/// - `state`    — updated on every real state transition; drives re-renders.
/// - `provider` — stable reference to the shared reactor + GlocStream.
pub(crate) struct GlocCtx<R: Reactor>
where
    R: 'static,
    R::State: Send + 'static,
{
    pub(crate) state: Signal<R::State>,
    pub(crate) provider: Signal<GlocProvider<R>>,
}

// `Signal<T>` is `Copy + Clone` regardless of whether `T` itself implements
// those traits. Using `#[derive]` would generate `R: Clone` / `R: Copy`
// bounds which are too restrictive, so we implement manually.
impl<R: Reactor> Clone for GlocCtx<R>
where
    R: 'static,
    R::State: Send + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: Reactor> Copy for GlocCtx<R>
where
    R: 'static,
    R::State: Send + 'static,
{
}
