//! Snapshot, save/load, and survey-trigger Tauri command handlers.
//!
//! Hosts the read-only [`game_get_snapshot`] command used during frontend
//! bootstrap, the placeholder save/load commands, and [`game_start_survey`],
//! which marks the survey-uplink service as desired-active.
//!
//! See also: [`crate::commands`], [`crate::game::snapshot`].

use crate::commands::action_response;
use crate::game::content::planets::{AURORA_PIER_ID, CINDER_FORGE_ID};
use crate::game::snapshot::{build_snapshot, ActionResponse, RawGameSnapshot, SaveLoadResponse};
use crate::{commit_and_emit, GameState, LastEmittedSnapshot};

/// Return the current run + profile as a [`RawGameSnapshot`].
///
/// **Frontend alias**: `game_get_snapshot` (same as Rust name)
/// **Mutates state**: no
/// **Emits `game://state-changed`**: no
///
/// Used by the frontend's `gameState.init()` to seed the reactive store before
/// the first push event arrives.
#[tauri::command]
pub fn game_get_snapshot(state: tauri::State<GameState>) -> RawGameSnapshot {
    let guard = state.lock();
    build_snapshot(&guard.run, &guard.profile)
}

/// Mark the survey-uplink service as desired-active so the next tick begins
/// surveying for new planet discoveries.
///
/// **Frontend alias**: `game_start_survey` (same as Rust name)
/// **Mutates state**: yes
/// **Emits `game://state-changed`**: yes (via [`commit_and_emit`])
///
/// Sets `desired_active = true` on the `survey-uplink` service when present;
/// the simulation tick promotes it to fully active once capacity, crew, and
/// power constraints permit.
///
/// # Errors
/// Returns an `ActionResponse { ok: false, reason_code: Some(_) }` (mapped to
/// `SurveyStartRejectionCode` on the frontend) for:
/// - `all-planets-discovered`: both [`CINDER_FORGE_ID`] and [`AURORA_PIER_ID`]
///   are already in `discovered_planet_ids`, leaving nothing to survey.
#[tauri::command]
pub fn game_start_survey(
    app: tauri::AppHandle<impl tauri::Runtime>,
    state: tauri::State<GameState>,
    last_emitted: tauri::State<LastEmittedSnapshot>,
) -> ActionResponse {
    let mut guard = state.lock();

    if guard
        .run
        .station
        .discovered_planet_ids
        .iter()
        .any(|planet_id| planet_id == CINDER_FORGE_ID)
        && guard
            .run
            .station
            .discovered_planet_ids
            .iter()
            .any(|planet_id| planet_id == AURORA_PIER_ID)
    {
        return action_response(&guard.run, &guard.profile, false, Some("all-planets-discovered"));
    }

    if let Some(service) = guard.run.service_state_mut("survey-uplink") {
        service.desired_active = true;
    }
    let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
    action_response(&guard.run, &guard.profile, true, None)
}

/// Placeholder save command — returns `status: "saved"` plus a current
/// snapshot without persisting anything yet.
///
/// **Frontend alias**: `game_request_save` (same as Rust name)
/// **Mutates state**: no
/// **Emits `game://state-changed`**: no
///
/// The persistence subsystem under [`crate::game::persistence`] is scaffolded
/// but not yet wired into this command; the return shape is stable so the
/// frontend can integrate against it today.
#[tauri::command]
pub fn game_request_save(state: tauri::State<GameState>) -> SaveLoadResponse {
    let guard = state.lock();
    SaveLoadResponse {
        ok: true,
        status: "saved".to_string(),
        snapshot: build_snapshot(&guard.run, &guard.profile),
    }
}

/// Placeholder load command — returns `status: "loaded"` plus the current
/// snapshot and re-emits the live state via [`commit_and_emit`].
///
/// **Frontend alias**: `game_request_load` (same as Rust name)
/// **Mutates state**: no (current implementation does not replace state)
/// **Emits `game://state-changed`**: yes (via [`commit_and_emit`]) so the
/// frontend re-renders against the latest snapshot
///
/// The persistence subsystem under [`crate::game::persistence`] is scaffolded
/// but not yet wired into this command; the return shape is stable so the
/// frontend can integrate against it today.
#[tauri::command]
pub fn game_request_load(
    app: tauri::AppHandle<impl tauri::Runtime>,
    state: tauri::State<GameState>,
    last_emitted: tauri::State<LastEmittedSnapshot>,
) -> SaveLoadResponse {
    let guard = state.lock();
    let snapshot = build_snapshot(&guard.run, &guard.profile);
    let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
    SaveLoadResponse {
        ok: true,
        status: "loaded".to_string(),
        snapshot,
    }
}
