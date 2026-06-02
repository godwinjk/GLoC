//! # GLoC Space Game — Bevy Example
//!
//! A headless Bevy app that demonstrates GLoC reactive state management in a
//! game-engine context.  Two reactors model distinct domains — player stats and
//! enemy-wave management — and are wired into Bevy's ECS via [`GlocPlugin`] /
//! [`GlocResource`].
//!
//! ## Features demonstrated
//!
//! | Pattern | Where |
//! |---------|-------|
//! | `#[reactor(state = T, neutrons = E)]` | `PlayerReactor` + `PlayerEvent` |
//! | `#[reactor(state = T)]` direct methods | `WaveReactor` |
//! | `#[reactor_state]` | `PlayerState`, `WaveState` |
//! | `GlocPlugin<R>` — reactor as Bevy Resource | Both reactors |
//! | `GlocResource<R>` — ECS access | All systems |
//! | `listen` transition observers | `setup_observers` |
//! | Auto-wave-end on enemies-remaining == 0 | `WaveReactor::enemy_defeated` |
//!
//! ## Running
//!
//! ```sh
//! cargo run -p gloc-example-bevy
//! ```

use bevy::app::{App, Startup, Update};
use bevy::ecs::system::{Local, Res};
use bevy::prelude::*;
use gloc::{reactor, reactor_state, Reactor};
use gloc_bevy::{GlocPlugin, GlocResource};

// ---------------------------------------------------------------------------
// PlayerState
// ---------------------------------------------------------------------------

/// Complete snapshot of the player's stats at a single point in time.
///
/// All fields are updated atomically by `PlayerReactor`; observers always see
/// a consistent picture rather than partial mutations.
#[reactor_state]
pub struct PlayerState {
    /// Current hit-points.  Clamped to `[0, max_health]` by the reactor.
    pub health: i32,
    /// The ceiling for health restoration — cannot be exceeded by `Heal`.
    pub max_health: i32,
    /// Cumulative score earned this session.
    pub score: u32,
    /// Whether the player is still alive (`health > 0`).
    pub alive: bool,
    /// Remaining shield charges.  Shield absorbs incoming damage first.
    pub shield: i32,
}

impl PlayerState {
    /// Returns a fresh player in full health with no score and no shield.
    pub fn initial() -> Self {
        Self {
            health: 100,
            max_health: 100,
            score: 0,
            alive: true,
            shield: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// PlayerEvent
// ---------------------------------------------------------------------------

/// All commands that external systems can send to `PlayerReactor` via the
/// event bus.
///
/// Using an event enum lets Bevy systems fire-and-forget without holding a
/// mutable borrow of the reactor directly.
#[derive(Debug)]
pub enum PlayerEvent {
    /// Inflict `i32` points of damage.  Shield absorbs first.
    TakeDamage(i32),
    /// Restore `i32` hit-points, capped at `max_health`.
    Heal(i32),
    /// Add `u32` points to the cumulative score.
    AddScore(u32),
    /// Grant a shield with `i32` charges.
    ActivateShield(i32),
    /// Reset the player to full health (keeps score).
    Respawn,
}

// ---------------------------------------------------------------------------
// PlayerReactor
// ---------------------------------------------------------------------------

/// Manages player stats and enforces all combat rules.
///
/// `#[reactor(state = PlayerState, neutrons = PlayerEvent)]` generates:
/// - `impl Reactor for PlayerReactor` — `state()`, `emit()` with change-detection
/// - `pub fn new(initial: PlayerState) -> Self`
/// - `pub fn fire(&mut self, neutron: PlayerEvent)` — calls `self.on_event`
/// - `subscribe()` / `attach_listener()`
#[reactor(state = PlayerState, neutrons = PlayerEvent)]
pub struct PlayerReactor {}

impl PlayerReactor {
    /// Routes each `PlayerEvent` variant to the appropriate handler logic.
    ///
    /// The macro generates the public `fire()` entry-point; the user owns
    /// this implementation so domain rules live in one clear place.
    fn on_event(&mut self, event: PlayerEvent) {
        match event {
            PlayerEvent::TakeDamage(dmg) => self.take_damage(dmg),
            PlayerEvent::Heal(hp) => self.heal(hp),
            PlayerEvent::AddScore(pts) => self.add_score(pts),
            PlayerEvent::ActivateShield(charges) => self.activate_shield(charges),
            PlayerEvent::Respawn => self.respawn(),
        }
    }

    /// Applies `dmg` points of damage, with shield absorbing first.
    ///
    /// Clamps `health` to zero and sets `alive = false` when health is
    /// exhausted — never goes negative.
    fn take_damage(&mut self, dmg: i32) {
        let mut next = self.state().clone();
        let remaining_dmg = if next.shield > 0 {
            let absorbed = next.shield.min(dmg);
            next.shield -= absorbed;
            dmg - absorbed
        } else {
            dmg
        };
        next.health = (next.health - remaining_dmg).max(0);
        next.alive = next.health > 0;
        self.emit(next);
    }

    /// Restores `hp` hit-points, capped at `max_health`.
    fn heal(&mut self, hp: i32) {
        let mut next = self.state().clone();
        next.health = (next.health + hp).min(next.max_health);
        self.emit(next);
    }

    /// Adds `pts` to the cumulative score.
    fn add_score(&mut self, pts: u32) {
        let mut next = self.state().clone();
        next.score += pts;
        self.emit(next);
    }

    /// Grants a shield with the given number of absorption charges.
    fn activate_shield(&mut self, charges: i32) {
        let mut next = self.state().clone();
        next.shield = charges;
        self.emit(next);
    }

    /// Resets the player to full health and marks them alive.
    ///
    /// Score is intentionally preserved across respawns.
    fn respawn(&mut self) {
        let mut next = self.state().clone();
        next.health = next.max_health;
        next.alive = true;
        next.shield = 0;
        self.emit(next);
    }
}

// ---------------------------------------------------------------------------
// WaveState
// ---------------------------------------------------------------------------

/// Snapshot of the current enemy-wave status.
///
/// `wave_active` is the gate used by combat and score systems to determine
/// whether gameplay is in progress.
#[reactor_state]
pub struct WaveState {
    /// The current wave number, starting at 0 (no wave started yet).
    pub wave: u32,
    /// How many enemies are still alive in the active wave.
    pub enemies_remaining: u32,
    /// Whether a wave is currently in progress.
    pub wave_active: bool,
}

impl WaveState {
    /// Returns the initial state with no active wave.
    pub fn initial() -> Self {
        Self {
            wave: 0,
            enemies_remaining: 0,
            wave_active: false,
        }
    }
}

// ---------------------------------------------------------------------------
// WaveReactor
// ---------------------------------------------------------------------------

/// Manages enemy-wave lifecycle using direct method calls (no events).
///
/// `#[reactor(state = WaveState)]` generates the `Reactor` impl, constructor,
/// `subscribe`, and `attach_listener`.
#[reactor(state = WaveState)]
pub struct WaveReactor {}

impl WaveReactor {
    /// Begins a new wave with `enemy_count` enemies.
    ///
    /// Increments the wave counter and marks the wave as active.  Has no
    /// effect if a wave is already in progress.
    pub fn start_wave(&mut self, enemy_count: u32) {
        if self.state().wave_active {
            return;
        }
        let mut next = self.state().clone();
        next.wave += 1;
        next.enemies_remaining = enemy_count;
        next.wave_active = true;
        println!(
            "[WaveReactor] Wave {} started — {} enemies incoming",
            next.wave, next.enemies_remaining
        );
        self.emit(next);
    }

    /// Records the defeat of one enemy and auto-ends the wave when the last
    /// enemy falls.
    ///
    /// Silently ignores calls when no wave is active or no enemies remain.
    pub fn enemy_defeated(&mut self) {
        let current = self.state().clone();
        if !current.wave_active || current.enemies_remaining == 0 {
            return;
        }
        let mut next = current;
        next.enemies_remaining -= 1;
        if next.enemies_remaining == 0 {
            next.wave_active = false;
            println!("[WaveReactor] Wave {} complete!", next.wave);
        }
        self.emit(next);
    }

    /// Forcibly ends the current wave regardless of remaining enemies.
    ///
    /// Useful for game-over or level-transition scenarios.
    pub fn end_wave(&mut self) {
        let mut next = self.state().clone();
        next.wave_active = false;
        self.emit(next);
    }
}

// ---------------------------------------------------------------------------
// Bevy timer resources
// ---------------------------------------------------------------------------

/// Accumulates delta time so `wave_system` can fire on a fixed cadence.
///
/// Using a `Local<f32>` inside the system avoids a separate `Resource` struct
/// for each timer, keeping the ECS world lean.
type WaveTimer = f32;

/// Accumulates time since the player died for the respawn delay.
type RespawnTimer = f32;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Registers `listen` observers on both reactors at startup.
///
/// Wiring observers here rather than inside `Update` systems prevents
/// duplicate registrations.  Observers log every state transition so the
/// console shows GLoC change-detection in action.
fn setup_observers(player: Res<GlocResource<PlayerReactor>>, wave: Res<GlocResource<WaveReactor>>) {
    player.listen(|old, new| {
        println!(
            "[PlayerReactor] hp: {} → {}  score: {} → {}  shield: {} → {}  alive: {} → {}",
            old.health,
            new.health,
            old.score,
            new.score,
            old.shield,
            new.shield,
            old.alive,
            new.alive,
        );
    });

    wave.listen(|old, new| {
        println!(
            "[WaveReactor] wave: {} → {}  enemies: {} → {}  active: {} → {}",
            old.wave,
            new.wave,
            old.enemies_remaining,
            new.enemies_remaining,
            old.wave_active,
            new.wave_active,
        );
    });
}

/// Starts a new wave every 5 seconds when no wave is currently active.
///
/// Uses a `Local<WaveTimer>` accumulator to measure elapsed time without
/// requiring a dedicated `Resource`.  Enemy count scales with the wave number
/// to give the game a simple difficulty curve.
fn wave_system(wave: Res<GlocResource<WaveReactor>>, time: Res<Time>, mut timer: Local<WaveTimer>) {
    *timer += time.delta_secs();

    if *timer >= 5.0 {
        *timer = 0.0;
        let current_wave = wave.state().wave;
        let enemy_count = 3 + current_wave * 2; // scale difficulty
        wave.update(|r| r.start_wave(enemy_count));
    }
}

/// Drives combat each frame when a wave is active.
///
/// - Dispatches `TakeDamage(5)` to the player every frame.
/// - Kills one enemy per frame via `enemy_defeated()`.
///
/// The rapid cadence (every frame) is intentional for demo purposes —
/// it drives visible transitions quickly without sleeping.
fn combat_system(player: Res<GlocResource<PlayerReactor>>, wave: Res<GlocResource<WaveReactor>>) {
    let wave_state = wave.state();
    if !wave_state.wave_active {
        return;
    }

    let player_state = player.state();
    if !player_state.alive {
        return;
    }

    // Player takes 5 damage each combat frame
    player.update(|r| r.fire(PlayerEvent::TakeDamage(5)));

    // One enemy is defeated each combat frame
    wave.update(|r| r.enemy_defeated());
}

/// Handles respawning after the player dies.
///
/// Counts down a 3-second delay using a `Local<RespawnTimer>`.  Resets
/// the timer after each respawn so subsequent deaths are handled correctly.
fn respawn_system(
    player: Res<GlocResource<PlayerReactor>>,
    time: Res<Time>,
    mut timer: Local<RespawnTimer>,
) {
    let state = player.state();
    if state.alive {
        // Player is alive — reset any accumulated respawn time
        *timer = 0.0;
        return;
    }

    *timer += time.delta_secs();
    if *timer >= 3.0 {
        println!("[combat] Player respawning after 3 s...");
        *timer = 0.0;
        player.update(|r| r.fire(PlayerEvent::Respawn));
    }
}

/// Awards 100 points for each enemy defeated since the last frame.
///
/// Compares the previous `enemies_remaining` count (held in a `Local`)
/// against the current value to determine how many enemies fell this frame.
fn score_system(
    player: Res<GlocResource<PlayerReactor>>,
    wave: Res<GlocResource<WaveReactor>>,
    mut prev_enemies: Local<u32>,
) {
    let wave_state = wave.state();
    let current_enemies = wave_state.enemies_remaining;

    if current_enemies < *prev_enemies {
        let defeated = *prev_enemies - current_enemies;
        let points = defeated * 100;
        player.update(|r| r.fire(PlayerEvent::AddScore(points)));
    }

    *prev_enemies = current_enemies;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

/// Entry point — builds and runs the headless Bevy space-game app.
///
/// Uses `MinimalPlugins` so no window or renderer is created.  `ScheduleRunnerPlugin`
/// limits execution to a fixed number of updates so the demo terminates on its own
/// rather than running forever.
fn main() {
    println!("=== GLoC Space Game (Bevy headless demo) ===");
    println!("Watch state transitions logged by the listen observers.");
    println!();

    App::new()
        .add_plugins(
            MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(
                std::time::Duration::from_millis(100),
            )),
        )
        .add_plugins(GlocPlugin::new(PlayerReactor::new(PlayerState::initial())))
        .add_plugins(GlocPlugin::new(WaveReactor::new(WaveState::initial())))
        .add_systems(Startup, setup_observers)
        .add_systems(
            Update,
            (wave_system, combat_system, respawn_system, score_system),
        )
        .run();
}
