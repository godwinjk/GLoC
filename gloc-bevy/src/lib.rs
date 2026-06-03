//! # gloc-bevy
//!
//! Bevy integration for GLoC — bridge GLoC reactors into Bevy's ECS as
//! [`Resource`]s so that game systems can read and mutate domain state
//! using the same reactor API used everywhere else in the application.
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use gloc::Reactor;
//! use gloc_bevy::{GlocPlugin, GlocResource};
//!
//! #[derive(Clone, PartialEq, Debug)]
//! struct CounterState { count: i32 }
//!
//! struct CounterReactor { state: CounterState }
//! // ... impl Reactor for CounterReactor ...
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(GlocPlugin::new(CounterReactor::new()))
//!         .add_systems(Update, increment_system)
//!         .run();
//! }
//!
//! fn increment_system(counter: Res<GlocResource<CounterReactor>>) {
//!     counter.update(|r| r.increment());
//! }
//! ```

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use bevy::app::{App, Plugin};
use bevy::ecs::system::Resource;
use gloc::{GlocProvider, Reactor};

// ---------------------------------------------------------------------------
// GlocResource
// ---------------------------------------------------------------------------

/// A Bevy [`Resource`] wrapper around a [`GlocProvider`].
///
/// Bevy's orphan rule prevents implementing `Resource` directly on
/// `GlocProvider`, so this newtype exists purely to satisfy that constraint
/// while preserving the full `GlocProvider` API via [`Deref`] / [`DerefMut`].
///
/// # Usage
///
/// Insert this into a Bevy app via [`GlocPlugin`], then access it in systems
/// with `Res<GlocResource<R>>` or `ResMut<GlocResource<R>>`.
pub struct GlocResource<R: Reactor>(pub GlocProvider<R>)
where
    R::State: Send;

/// Satisfies the Bevy `Resource` bound so `GlocResource<R>` can be inserted
/// into the Bevy world. The `Send + Sync + 'static` requirements come from
/// Bevy's ECS, which must be able to move resources across threads.
impl<R: Reactor + Send + Sync + 'static> Resource for GlocResource<R> where R::State: Send {}

impl<R: Reactor> Deref for GlocResource<R>
where
    R::State: Send,
{
    type Target = GlocProvider<R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R: Reactor> DerefMut for GlocResource<R>
where
    R::State: Send,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// ---------------------------------------------------------------------------
// GlocPlugin
// ---------------------------------------------------------------------------

/// A Bevy [`Plugin`] that inserts a [`GlocResource`] into the Bevy app.
///
/// Using a plugin rather than a bare `app.insert_resource(...)` call keeps
/// the setup declarative and lets the reactor's initial state live alongside
/// the plugin configuration rather than scattered across `main`.
///
/// # Type Parameters
///
/// - `R` — the reactor type to manage. Must be `Send + Sync + 'static` so it
///   can be safely held by Bevy's `World` and accessed from parallel systems.
///
/// # Example
///
/// ```rust,ignore
/// App::new()
///     .add_plugins(GlocPlugin::new(CounterReactor::new()))
///     .run();
/// ```
pub struct GlocPlugin<R: Reactor>
where
    R::State: Send,
{
    /// The reactor instance to wrap and insert. Stored as `Option` so
    /// `build()` can move it out of `&self` without requiring `Clone`.
    reactor: Mutex<Option<R>>,
}

impl<R: Reactor + Send + Sync + 'static> GlocPlugin<R>
where
    R::State: Send,
{
    /// Creates a new `GlocPlugin` that will insert the given `reactor` as a
    /// [`GlocResource`] when the plugin is built into a Bevy [`App`].
    ///
    /// # Parameters
    ///
    /// - `reactor` — the initial reactor instance; owns the starting state.
    pub fn new(reactor: R) -> Self {
        Self {
            reactor: Mutex::new(Some(reactor)),
        }
    }
}

impl<R: Reactor + Send + Sync + 'static> Plugin for GlocPlugin<R>
where
    R::State: Send + Clone + 'static,
{
    /// Builds the plugin by constructing a [`GlocProvider`] from the reactor
    /// and inserting a [`GlocResource`] into the Bevy world.
    ///
    /// The reactor is moved out on the first call. Bevy guarantees `build` is
    /// called exactly once per plugin instance, so the `Option` unwrap is safe.
    fn build(&self, app: &mut App) {
        let reactor = self
            .reactor
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .expect("GlocPlugin::build called more than once — each plugin instance must be used with exactly one App");

        let shared = Arc::new(Mutex::new(reactor));
        let consumer = GlocProvider::new(shared);

        app.insert_resource(GlocResource(consumer));
    }
}
