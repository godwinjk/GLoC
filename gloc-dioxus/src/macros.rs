/// Reactive builder macro — shorthand for [`use_gloc_builder`] /
/// [`use_gloc_builder_when`].
///
/// Eliminates the turbofish (`use_gloc_builder::<R, _>(...)`) and provides
/// optional `buildWhen` guard support in a single call site — mirroring
/// Flutter's `BlocBuilder(buildWhen: ..., builder: ...)`.
///
/// # Without guard — rebuilds on every state change
///
/// ```rust,ignore
/// fn CounterView() -> Element {
///     let counter = use_gloc::<CounterReactor>();
///     gloc_builder!(CounterReactor, |state| rsx! {
///         p { "{state.count}" }
///         button { onclick: move |_| counter.update(|r| r.increment()), "+" }
///     })
/// }
/// ```
///
/// # With `buildWhen` guard — rebuilds only when predicate returns `true`
///
/// ```rust,ignore
/// gloc_builder!(CounterReactor,
///     when: |old, new| old.count != new.count,
///     |state| rsx! { p { "{state.count}" } }
/// )
/// ```
///
/// [`use_gloc_builder`]: crate::use_gloc_builder
/// [`use_gloc_builder_when`]: crate::use_gloc_builder_when
#[macro_export]
macro_rules! gloc_builder {
    ($R:ty, $builder:expr) => {
        $crate::use_gloc_builder::<$R, _>($builder)
    };
    ($R:ty, when: $when:expr, $builder:expr) => {
        $crate::use_gloc_builder_when::<$R, _, _>($when, $builder)
    };
}

/// Side-effect listener macro — shorthand for [`use_gloc_listener`] /
/// [`use_gloc_listener_when`].
///
/// Does **not** return an `Element` and does not cause re-renders. Use it for
/// navigation, logging, analytics, snackbars, etc.
///
/// # Without guard — fires on every state transition
///
/// ```rust,ignore
/// fn CartPage() -> Element {
///     gloc_listener!(CartReactor, |old, new| {
///         if new.status == CartStatus::CheckedOut {
///             navigator().push(Route::Confirmation);
///         }
///     });
///     rsx! { /* ... */ }
/// }
/// ```
///
/// # With `listenWhen` guard — fires only when predicate returns `true`
///
/// ```rust,ignore
/// gloc_listener!(CartReactor,
///     when: |old, new| old.status != new.status,
///     |old, new| { navigator().push(Route::Confirmation); }
/// )
/// ```
///
/// [`use_gloc_listener`]: crate::use_gloc_listener
/// [`use_gloc_listener_when`]: crate::use_gloc_listener_when
#[macro_export]
macro_rules! gloc_listener {
    ($R:ty, $listener:expr) => {
        $crate::use_gloc_listener::<$R, _>($listener)
    };
    ($R:ty, when: $when:expr, $listener:expr) => {
        $crate::use_gloc_listener_when::<$R, _, _>($when, $listener)
    };
}

/// Combined builder + listener macro — shorthand for [`use_gloc_consumer`] /
/// [`use_gloc_consumer_when`].
///
/// Hooks are always called listener-first, then builder — the same order as
/// [`use_gloc_consumer`]. Hook call order must be stable across renders.
///
/// # Without guards — always rebuilds and always listens
///
/// ```rust,ignore
/// fn CartView() -> Element {
///     let cart = use_gloc::<CartReactor>();
///     gloc_consumer!(CartReactor,
///         build: |state| rsx! {
///             p { "Total: ${state.total:.2}" }
///             button { onclick: move |_| cart.update(|r| r.checkout()), "Checkout" }
///         },
///         listen: |_old, new| {
///             if new.status == CartStatus::CheckedOut { println!("order placed"); }
///         }
///     )
/// }
/// ```
///
/// # With `build_when` only
///
/// ```rust,ignore
/// gloc_consumer!(CartReactor,
///     build_when: |old, new| old.total != new.total,
///     build:  |state| rsx! { p { "${state.total:.2}" } },
///     listen: |_old, new| { println!("transition"); }
/// )
/// ```
///
/// # With `listen_when` only
///
/// ```rust,ignore
/// gloc_consumer!(CartReactor,
///     build: |state| rsx! { p { "${state.total:.2}" } },
///     listen_when: |old, new| old.status != new.status,
///     listen: |_old, new| { navigator().push(Route::Confirmation); }
/// )
/// ```
///
/// # With both guards
///
/// ```rust,ignore
/// gloc_consumer!(CartReactor,
///     build_when:  |old, new| old.total  != new.total,
///     build:       |state| rsx! { p { "${state.total:.2}" } },
///     listen_when: |old, new| old.status != new.status,
///     listen:      |_old, new| { navigator().push(Route::Confirmation); }
/// )
/// ```
///
/// [`use_gloc_consumer`]: crate::use_gloc_consumer
/// [`use_gloc_consumer_when`]: crate::use_gloc_consumer_when
#[macro_export]
macro_rules! gloc_consumer {
    // No guards
    (
        $R:ty,
        build:  $builder:expr,
        listen: $listener:expr
    ) => {
        $crate::use_gloc_consumer::<$R, _, _>($builder, $listener)
    };

    // build_when only
    (
        $R:ty,
        build_when: $bw:expr,
        build:      $builder:expr,
        listen:     $listener:expr
    ) => {
        $crate::use_gloc_consumer_when::<$R, _, _, _, _>($bw, $builder, |_, _| true, $listener)
    };

    // listen_when only
    (
        $R:ty,
        build:       $builder:expr,
        listen_when: $lw:expr,
        listen:      $listener:expr
    ) => {
        $crate::use_gloc_consumer_when::<$R, _, _, _, _>(|_, _| true, $builder, $lw, $listener)
    };

    // Both guards
    (
        $R:ty,
        build_when:  $bw:expr,
        build:       $builder:expr,
        listen_when: $lw:expr,
        listen:      $listener:expr
    ) => {
        $crate::use_gloc_consumer_when::<$R, _, _, _, _>($bw, $builder, $lw, $listener)
    };
}
