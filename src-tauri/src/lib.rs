//! Library crate that powers the `idle-spacestation` Tauri backend.
//!
//! This crate owns the long-lived runtime: the [`tauri::Builder`] configured by
//! [`run`], the managed mutex-guarded `GameState` and `LastEmittedSnapshot`,
//! the background tick thread, and the single `tauri::generate_handler![]` that
//! registers every production and (debug-only) devtools command.
//!
//! # Architecture
//!
//! - `GameState` wraps the mutable `GameRunState` (the live `RunState`,
//!   the `PrestigeProfile`, and the per-session tick counter) behind a
//!   [`std::sync::Mutex`].
//! - `LastEmittedSnapshot` caches the last `RawGameSnapshot` emitted to the
//!   frontend so `commit_and_emit` can suppress redundant `game://state-changed`
//!   events when nothing actually changed.
//! - A daemon thread spawned in the [`tauri::Builder::setup`] callback ticks the
//!   simulation at roughly 4 Hz (every 250 ms) and pushes diffs through
//!   `commit_and_emit`.
//!
//! # Lock ordering
//!
//! Every callsite must acquire `GameState` **before** `LastEmittedSnapshot`
//! and must drop the `GameState` guard before invoking `commit_and_emit` (or
//! at minimum keep both guards in that order). Inverting this order risks a
//! deadlock if a future caller ever takes the snapshot lock first.
//!
//! # Devtools
//!
//! All `game_devtools_*` commands and the `DevtoolsState` resource are gated
//! by `#[cfg(debug_assertions)]` and stripped from release builds. The
//! `all_commands!` macro is the single source of truth for the handler list
//! and embeds the `#[cfg(debug_assertions)]` markers inline so production and
//! devtools commands never drift apart across two `generate_handler![]` calls.

mod commands;
mod game;
mod runtime;

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub(crate) use commands::*;
use game::progression::PrestigeProfile;
use game::snapshot::RawGameSnapshot;
use game::sim::{tick, RunState};
use tauri::{Emitter, Manager};

#[cfg(debug_assertions)]
use commands::devtools::install_debug_menu;

/// Mutable game-runtime payload guarded by [`GameState`].
///
/// Bundles the live simulation [`RunState`], the persistent [`PrestigeProfile`],
/// and a per-session tick counter so callers only need to hold a single mutex
/// to mutate the entire game world.
pub(crate) struct GameRunState {
    /// Live simulation state mutated by the tick loop and Tauri commands.
    pub(crate) run: RunState,
    /// Prestige profile carried across runs; persisted separately by save/load.
    pub(crate) profile: PrestigeProfile,
    /// Number of ticks executed since this process started (not persisted).
    pub(crate) session_ticks: u32,
}

/// Tauri-managed wrapper around the mutex protecting [`GameRunState`].
///
/// Acquired via [`GameState::lock`]. See the crate-level lock-ordering note —
/// [`GameState`] must be locked before [`LastEmittedSnapshot`].
pub(crate) struct GameState(Mutex<GameRunState>);

impl GameState {
    /// Locks the game-state mutex and returns the guard.
    ///
    /// # Panics
    ///
    /// Panics if the mutex is poisoned (i.e. another thread panicked while
    /// holding the guard). The panic carries the original caller location via
    /// `#[track_caller]`.
    #[track_caller]
    pub(crate) fn lock(&self) -> std::sync::MutexGuard<'_, GameRunState> {
        self.0.lock().expect("game state mutex poisoned")
    }
}

/// Tauri-managed cache of the most recently emitted [`RawGameSnapshot`].
///
/// [`commit_and_emit`] consults this cache to short-circuit `game://state-changed`
/// emission whenever the new snapshot bit-equals the previous one. Lock ordering:
/// always acquire [`GameState`] first, then this cache.
pub(crate) struct LastEmittedSnapshot(Mutex<Option<RawGameSnapshot>>);

impl LastEmittedSnapshot {
    /// Locks the snapshot cache mutex and returns the guard.
    ///
    /// # Panics
    ///
    /// Panics if the mutex is poisoned. Carries the caller location via
    /// `#[track_caller]`.
    #[track_caller]
    pub(crate) fn lock(&self) -> std::sync::MutexGuard<'_, Option<RawGameSnapshot>> {
        self.0.lock().expect("last_emitted mutex poisoned")
    }
}

/// Debug-only Tauri-managed boolean tracking devtools-overlay visibility.
///
/// Stripped from release builds via `#[cfg(debug_assertions)]`. Mutated by the
/// `game_devtools_set_visibility` command and read by `game_devtools_get_state`.
#[cfg(debug_assertions)]
pub(crate) struct DevtoolsState(Mutex<bool>);

#[cfg(debug_assertions)]
impl DevtoolsState {
    /// Locks the devtools-visibility mutex and returns the guard.
    ///
    /// # Panics
    ///
    /// Panics if the mutex is poisoned. Carries the caller location via
    /// `#[track_caller]`.
    #[track_caller]
    pub(crate) fn lock(&self) -> std::sync::MutexGuard<'_, bool> {
        self.0.lock().expect("devtools state mutex poisoned")
    }
}

/// Tauri event name fired whenever the game snapshot changes.
///
/// Subscribed to by the frontend `gameState` store. Emitted exclusively by the
/// crate-internal `commit_and_emit` helper; callers must never invoke
/// `app.emit(STATE_CHANGED_EVENT, ...)` directly.
///
/// Payload: RawGameSnapshot (camelCase via existing serde rename_all attributes).
pub const STATE_CHANGED_EVENT: &str = "game://state-changed";

/// Builds the current snapshot, diffs it against [`LastEmittedSnapshot`], and
/// emits [`STATE_CHANGED_EVENT`] when something actually changed.
///
/// This is the single chokepoint through which the backend pushes state to the
/// frontend. Every mutating Tauri command and the background tick loop must
/// route through it; bypassing the cache leaves clients stale until the next
/// tick fires (or, worse, floods them with redundant events).
///
/// # Errors
///
/// Returns `Err(String)` carrying [`tauri::Error`]'s display message when the
/// underlying [`tauri::Emitter::emit`] call fails (typically because the app
/// handle has already shut down).
///
/// # Panics
///
/// Panics if [`LastEmittedSnapshot`]'s mutex is poisoned. **Lock-order
/// invariant:** the caller must have already dropped (or never held) the
/// [`GameState`] mutex before invoking this helper, since `commit_and_emit`
/// acquires [`LastEmittedSnapshot`] and any caller that holds both in the
/// reverse order risks a deadlock against a future call site that takes them
/// in the canonical order.
pub(crate) fn commit_and_emit<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    run: &crate::game::sim::RunState,
    profile: &crate::game::progression::PrestigeProfile,
    last_emitted: &LastEmittedSnapshot,
) -> Result<(), String> {
    use crate::game::snapshot::{build_snapshot, state_equals};
    let new_snapshot = build_snapshot(run, profile);
    // Lock order: callers acquire GameState first, then this helper acquires
    // LastEmittedSnapshot. Keep every push-based caller in that order to avoid deadlocks.
    let mut cache = last_emitted.lock();

    if let Some(ref previous_snapshot) = *cache {
        if state_equals(previous_snapshot, &new_snapshot) {
            return Ok(());
        }
    }

    *cache = Some(new_snapshot.clone());
    drop(cache);

    app.emit(STATE_CHANGED_EVENT, &new_snapshot)
        .map_err(|error| error.to_string())
}

/// Single source of truth for the Tauri command list.
///
/// Production commands are always registered. Debug-only devtools commands carry
/// `#[cfg(debug_assertions)]` and are stripped in release builds. This eliminates
/// the footgun of maintaining two parallel handler-registration call sites.
///
/// # Invariants
///
/// - Each `#[cfg(debug_assertions)]` attribute must appear **immediately before**
///   the devtools command identifier it gates (not after, and not on its own
///   line wrapping multiple commands). The macro relies on the attribute
///   binding to the next token in the comma-separated list inside
///   [`tauri::generate_handler!`].
/// - The macro must expand to exactly **one** `tauri::generate_handler![]`
///   invocation. Splitting it into two (e.g. one for production commands and
///   another for devtools commands) breaks Tauri's handler registration, which
///   replaces — rather than merges — the previously installed handler set.
macro_rules! all_commands {
    () => {
        tauri::generate_handler![
            game_get_snapshot,
            game_toggle_service,
            game_upgrade_system,
            game_assign_service_crew,
            game_reprioritize_service,
            game_start_survey,
            game_purchase_doctrine,
            game_execute_prestige,
            game_request_save,
            game_request_load,
            #[cfg(debug_assertions)]
            game_devtools_get_state,
            #[cfg(debug_assertions)]
            game_devtools_set_visibility,
            #[cfg(debug_assertions)]
            game_devtools_apply_resources,
            #[cfg(debug_assertions)]
            game_devtools_apply_crew,
            #[cfg(debug_assertions)]
            game_devtools_apply_systems,
            #[cfg(debug_assertions)]
            game_devtools_apply_services,
            #[cfg(debug_assertions)]
            game_devtools_apply_progression,
            #[cfg(debug_assertions)]
            game_devtools_advance_ticks,
            #[cfg(debug_assertions)]
            game_devtools_reset_to_starter,
        ]
    };
}

/// Boots the Tauri runtime: configures plugins, registers managed state,
/// spawns the background tick thread, installs the command handler, and runs.
///
/// Called from both the desktop `main` entrypoint and the mobile entrypoint via
/// the `#[cfg_attr(mobile, tauri::mobile_entry_point)]` attribute.
///
/// # Setup steps
///
/// 1. Build a [`tauri::Builder`] with the opener plugin (always) and the
///    MCP-bridge plugin (debug builds only).
/// 2. Inside the [`tauri::Builder::setup`] callback:
///    - Manage `GameState` seeded from `RunState::starter_fixture` and a
///      default `PrestigeProfile`.
///    - Manage an empty `LastEmittedSnapshot`.
///    - In debug builds, manage `DevtoolsState` (defaulting to hidden) and
///      install the debug menu via `install_debug_menu`.
///    - Spawn a background **daemon thread** that runs an infinite loop:
///      sleep 250 ms (≈4 Hz cadence), lock `GameState`, advance the
///      simulation via `tick`, then call `commit_and_emit` to push any
///      diff to the frontend. Errors from `commit_and_emit` are logged via
///      `eprintln!` and the loop continues — **the tick loop never panics**.
/// 3. Register every Tauri command via the `all_commands!` macro.
/// 4. Hand control to [`tauri::App::run`].
///
/// # Panics
///
/// Panics if [`tauri::Builder::run`] returns an error during application
/// startup; this is the standard Tauri pattern since a failure here means the
/// app cannot launch.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    #[cfg(debug_assertions)]
    let builder = builder.plugin(tauri_plugin_mcp_bridge::init());

    let builder = builder.setup(|app| {
        app.manage(GameState(Mutex::new(GameRunState {
            run: RunState::starter_fixture(),
            profile: PrestigeProfile::default(),
            session_ticks: 0u32,
        })));
        app.manage(LastEmittedSnapshot(Mutex::new(None)));

        #[cfg(debug_assertions)]
        {
            app.manage(DevtoolsState(Mutex::new(false)));
            install_debug_menu(app)?;
        }

        let app_handle = app.handle().clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(250));

            let game_state = app_handle.state::<GameState>();
            let last_emitted = app_handle.state::<LastEmittedSnapshot>();
            let mut guard = game_state.lock();
            tick(&mut guard.run);
            // Emit state-changed if game state changed. Log error, never panic.
            if let Err(err) = commit_and_emit(&app_handle, &guard.run, &guard.profile, &last_emitted)
            {
                eprintln!("[tick_loop] commit_and_emit error: {err}");
            }
            drop(guard);
        });

        Ok(())
    });

    let builder = builder.invoke_handler(all_commands!());

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::snapshot::build_snapshot;

    mod commit_and_emit {
        use super::*;

        fn should_emit(cache: &LastEmittedSnapshot, snapshot: &RawGameSnapshot) -> bool {
            use crate::game::snapshot::state_equals;
            let cache_guard = cache.lock();
            match &*cache_guard {
                Some(previous_snapshot) => !state_equals(previous_snapshot, snapshot),
                None => true,
            }
        }

        fn update_cache(cache: &LastEmittedSnapshot, snapshot: RawGameSnapshot) {
            let mut cache_guard = cache.lock();
            *cache_guard = Some(snapshot);
        }
        #[test]
        fn cache_starts_empty() {
            let cache = LastEmittedSnapshot(Mutex::new(None));
            assert!(cache.lock().is_none(), "cache should start empty");
        }
        #[test]
        fn first_call_emits() {
            let cache = LastEmittedSnapshot(Mutex::new(None));
            let snapshot = build_snapshot(&RunState::starter_fixture(), &PrestigeProfile::default());
            assert!(should_emit(&cache, &snapshot), "first call should always emit");
        }
        #[test]
        fn unchanged_skips() {
            let cache = LastEmittedSnapshot(Mutex::new(None));
            let snapshot = build_snapshot(&RunState::starter_fixture(), &PrestigeProfile::default());
            update_cache(&cache, snapshot.clone());
            assert!(!should_emit(&cache, &snapshot), "second call with same state should skip emit");
        }
        #[test]
        fn changed_emits() {
            let cache = LastEmittedSnapshot(Mutex::new(None));
            let snapshot = build_snapshot(&RunState::starter_fixture(), &PrestigeProfile::default());
            update_cache(&cache, snapshot);
            let mut changed_run = RunState::starter_fixture();
            changed_run.resources.materials += 100.0;
            let changed_snapshot = build_snapshot(&changed_run, &PrestigeProfile::default());
            assert!(should_emit(&cache, &changed_snapshot), "changed state should emit");
        }
        #[test]
        #[cfg(debug_assertions)]
        fn advance_ticks_commit_and_emit_called_once_not_n_times() {
            let cache = LastEmittedSnapshot(Mutex::new(None));
            let mut run = RunState::starter_fixture();
            let profile = PrestigeProfile::default();
            crate::commands::devtools::apply::apply_devtools_advance_ticks(&mut run, 5)
                .expect("5 ticks should be accepted");
            let final_snapshot = build_snapshot(&run, &profile);
            update_cache(&cache, final_snapshot.clone());
            let cache_guard = cache.lock();
            let cached = cache_guard
                .as_ref()
                .expect("cache should hold final post-loop snapshot");
            assert!(
                crate::game::snapshot::state_equals(cached, &final_snapshot),
                "cache should contain the final post-N-ticks snapshot, not an intermediate one"
            );
        }
    }

    #[test]
    fn background_tick_target_advances_run_state() {
        let mut run_state = RunState::starter_fixture();
        let initial_tick_count = run_state.tick_count;
        let initial_materials = run_state.resources.materials;
        tick(&mut run_state);
        assert_eq!(run_state.tick_count, initial_tick_count + 1);
        assert!(run_state.resources.materials >= initial_materials);
    }
}
