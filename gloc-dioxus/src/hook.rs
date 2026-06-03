use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use gloc::{provider::GlocProvider, Reactor};

use crate::{context::GlocCtx, handle::GlocHandle};

/// Injects a reactor into the Dioxus component tree as shared reactive state.
///
/// Call once per reactor type near the root of the component tree. Every
/// descendant can then call [`use_gloc::<R>()`](use_gloc) to get a
/// [`GlocHandle<R>`] without prop drilling.
///
/// # How reactivity works
///
/// 1. The reactor's built-in stream is cloned (Arc-backed, cheap).
/// 2. A `SyncSignal<R::State>` is created — `Send + Sync`, safe to update
///    from the stream listener.
/// 3. A listener is registered on the stream: every `emit()` call automatically
///    calls `signal.set(new_state)` — no manual signal update needed anywhere.
/// 4. A `use_drop` hook calls `provider.close()` on component unmount, firing
///    `reactor.on_close()` and `GlocObserver::on_close()`.
///
/// ```text
/// reactor.method() → emit() → stream fires → signal.set() → Dioxus re-renders
/// ```
///
/// # Example
///
/// ```rust,ignore
/// fn App() -> Element {
///     use_gloc_provide(|| CounterReactor::new(CounterState { count: 0 }));
///     use_gloc_provide(|| ThemeReactor::new(Theme::Light));
///     rsx! { Router::<Route> {} }
/// }
/// ```
pub fn use_gloc_provide<R, F>(factory: F)
where
    R: Reactor + Send + 'static,
    R::State: Send + Sync + 'static,
    F: FnOnce() -> R + 'static,
{
    let ctx = use_context_provider(|| {
        let reactor = factory();
        let initial = reactor.state().clone();

        // Clone the reactor's built-in stream before moving the reactor into
        // Arc<Mutex>. GlocStream is Arc-backed — this clone is free and shares
        // the same listener list as the stream inside the reactor.
        let stream = reactor.stream();

        let arc = Arc::new(Mutex::new(reactor));
        let provider = GlocProvider::new(arc);

        // SyncSignal: Send + Sync + Copy — can be captured in the stream
        // listener which may fire from any thread.
        let signal = Signal::new_maybe_sync(initial);

        // THE BRIDGE — wired once here. Every emit() now automatically
        // updates the signal and schedules a Dioxus re-render.
        // SyncSignal is Copy — copy it into the closure, then call set()
        // on the copied handle (interior mutability via SyncStorage).
        // write_unchecked(&self) provides interior-mutability write access
        // without requiring &mut signal — correct for a Fn + Send listener.
        stream.listen(move |_, new| {
            *signal.write_unchecked() = new.clone();
        });

        let provider_signal = Signal::new_maybe_sync(provider);

        GlocCtx {
            signal,
            provider: provider_signal,
        }
    });

    // Lifecycle: when the providing component unmounts, fire on_close() on the
    // reactor and notify GlocObserver::on_close().
    use_drop(move || ctx.provider.read().close());
}

/// Returns a [`GlocHandle<R>`] for the reactor provided by the nearest ancestor
/// that called [`use_gloc_provide::<R>`](use_gloc_provide).
///
/// `GlocHandle<R>` is `Copy` — move it into multiple closures in `rsx!` without
/// `.clone()`.
///
/// # Panics
///
/// Panics if no ancestor called `use_gloc_provide::<R>()`.
///
/// # Example
///
/// ```rust,ignore
/// fn CounterPage() -> Element {
///     let counter = use_gloc::<CounterReactor>();
///     let count = counter.state().count;   // reactive
///
///     rsx! {
///         p { "{count}" }
///         button { onclick: move |_| counter.update(|r| r.increment()), "+" }
///     }
/// }
/// ```
pub fn use_gloc<R>() -> GlocHandle<R>
where
    R: Reactor + Send + 'static,
    R::State: Send + Sync + 'static,
{
    GlocHandle::from_ctx(use_context::<GlocCtx<R>>())
}

/// Reactive builder hook — calls `builder` with the current state and
/// re-runs it on every state transition.
///
/// This is the GLoC equivalent of Flutter's `BlocBuilder` (without `buildWhen`).
/// Use [`use_gloc_builder_when`] to gate rebuilds on a predicate.
pub fn use_gloc_builder<R, F>(builder: F) -> Element
where
    R: Reactor + Send + 'static,
    R::State: Send + Sync + 'static,
    F: Fn(&R::State) -> Element,
{
    use_gloc_builder_when::<R, F, _>(|_, _| true, builder)
}

/// Builder hook with a `build_when` guard — only updates the UI when
/// `when(old, new)` returns `true`.
///
/// This is the GLoC equivalent of Flutter's `BlocBuilder(buildWhen:)`.
pub fn use_gloc_builder_when<R, F, W>(when: W, builder: F) -> Element
where
    R: Reactor + Send + 'static,
    R::State: Send + Sync + 'static,
    F: Fn(&R::State) -> Element,
    W: Fn(&R::State, &R::State) -> bool + 'static,
{
    let ctx = use_context::<GlocCtx<R>>();
    let new_state = ctx.signal.cloned();

    let mut shown = use_signal(|| ctx.signal.peek().clone());
    let old_state = shown.peek().clone();

    if when(&old_state, &new_state) {
        shown.set(new_state.clone());
        builder(&new_state)
    } else {
        builder(&old_state)
    }
}

/// Side-effect listener hook — calls `listener(old, new)` on every real
/// transition without causing the component to re-render.
///
/// This is the GLoC equivalent of Flutter's `BlocListener`.
/// Use [`use_gloc_listener_when`] to gate the listener on a predicate.
pub fn use_gloc_listener<R, F>(listener: F)
where
    R: Reactor + Send + 'static,
    R::State: Send + Sync + 'static,
    F: Fn(&R::State, &R::State) + Send + Sync + 'static,
{
    use_gloc_listener_when::<R, F, _>(|_, _| true, listener);
}

/// Side-effect listener hook with a `listen_when` guard — calls `listener`
/// only when `when(old, new)` returns `true`.
///
/// This is the GLoC equivalent of Flutter's `BlocListener(listenWhen:)`.
pub fn use_gloc_listener_when<R, F, W>(when: W, listener: F)
where
    R: Reactor + Send + 'static,
    R::State: Send + Sync + 'static,
    F: Fn(&R::State, &R::State) + Send + Sync + 'static,
    W: Fn(&R::State, &R::State) -> bool + Send + Sync + 'static,
{
    let ctx = use_context::<GlocCtx<R>>();
    // Store the ListenerHandle in use_hook — it lives for the component's
    // lifetime and auto-cancels (via RAII Drop) when the component unmounts.
    use_hook(move || {
        ctx.provider.peek().listen(move |old, new| {
            if when(old, new) {
                listener(old, new);
            }
        })
    });
}

/// Combined builder + listener hook — re-renders the UI on state change
/// AND runs side effects on transitions, in one call.
///
/// This is the GLoC equivalent of Flutter's `BlocConsumer`.
pub fn use_gloc_consumer<R, B, L>(builder: B, listener: L) -> Element
where
    R: Reactor + Send + 'static,
    R::State: Send + Sync + 'static,
    B: Fn(&R::State) -> Element,
    L: Fn(&R::State, &R::State) + Send + Sync + 'static,
{
    use_gloc_consumer_when::<R, B, L, _, _>(|_, _| true, builder, |_, _| true, listener)
}

/// Combined builder + listener hook with `build_when` and `listen_when` guards.
///
/// This is the GLoC equivalent of Flutter's `BlocConsumer(buildWhen:, listenWhen:)`.
pub fn use_gloc_consumer_when<R, B, L, BW, LW>(
    build_when: BW,
    builder: B,
    listen_when: LW,
    listener: L,
) -> Element
where
    R: Reactor + Send + 'static,
    R::State: Send + Sync + 'static,
    B: Fn(&R::State) -> Element,
    L: Fn(&R::State, &R::State) + Send + Sync + 'static,
    BW: Fn(&R::State, &R::State) -> bool + 'static,
    LW: Fn(&R::State, &R::State) -> bool + Send + Sync + 'static,
{
    use_gloc_listener_when::<R, L, LW>(listen_when, listener);
    use_gloc_builder_when::<R, B, BW>(build_when, builder)
}
