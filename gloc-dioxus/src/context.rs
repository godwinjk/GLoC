use dioxus::prelude::*;
use gloc::{provider::GlocProvider, Reactor};

/// Internal per-reactor context stored in the Dioxus scope tree.
///
/// Both fields are `SyncSignal` (`Signal<T, SyncStorage>`) — `Send + Sync + Copy`
/// regardless of `T`. This lets the reactor's stream listener (which runs on
/// whatever thread calls `emit`) update the signal automatically, and lets
/// `GlocHandle` be `Copy` for use in multiple closures without cloning.
///
/// - `signal`   — drives re-renders; updated automatically by the stream listener.
/// - `provider` — shared reactor access for mutations and listener registration.
pub(crate) struct GlocCtx<R: Reactor>
where
    R: 'static,
    R::State: Send + Sync + 'static,
{
    pub(crate) signal: SyncSignal<R::State>,
    pub(crate) provider: SyncSignal<GlocProvider<R>>,
}

impl<R: Reactor> Clone for GlocCtx<R>
where
    R: 'static,
    R::State: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: Reactor> Copy for GlocCtx<R>
where
    R: 'static,
    R::State: Send + Sync + 'static,
{
}
