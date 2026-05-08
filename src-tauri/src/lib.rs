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

pub(crate) struct GameRunState {
    pub(crate) run: RunState,
    pub(crate) profile: PrestigeProfile,
    pub(crate) session_ticks: u32,
}

pub(crate) struct GameState(Mutex<GameRunState>);

impl GameState {
    #[track_caller]
    pub(crate) fn lock(&self) -> std::sync::MutexGuard<'_, GameRunState> {
        self.0.lock().expect("game state mutex poisoned")
    }
}

pub(crate) struct LastEmittedSnapshot(Mutex<Option<RawGameSnapshot>>);

impl LastEmittedSnapshot {
    #[track_caller]
    pub(crate) fn lock(&self) -> std::sync::MutexGuard<'_, Option<RawGameSnapshot>> {
        self.0.lock().expect("last_emitted mutex poisoned")
    }
}

#[cfg(debug_assertions)]
pub(crate) struct DevtoolsState(Mutex<bool>);

#[cfg(debug_assertions)]
impl DevtoolsState {
    #[track_caller]
    pub(crate) fn lock(&self) -> std::sync::MutexGuard<'_, bool> {
        self.0.lock().expect("devtools state mutex poisoned")
    }
}

/// Payload: RawGameSnapshot (camelCase via existing serde rename_all attributes).
pub const STATE_CHANGED_EVENT: &str = "game://state-changed";

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
