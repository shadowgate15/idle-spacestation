use crate::commands::action_response;
use crate::game::content::planets::{AURORA_PIER_ID, CINDER_FORGE_ID};
use crate::game::snapshot::{build_snapshot, ActionResponse, RawGameSnapshot, SaveLoadResponse};
use crate::{commit_and_emit, GameState, LastEmittedSnapshot};

#[tauri::command]
pub fn game_get_snapshot(state: tauri::State<GameState>) -> RawGameSnapshot {
    let guard = state.lock();
    build_snapshot(&guard.run, &guard.profile)
}

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

#[tauri::command]
pub fn game_request_save(state: tauri::State<GameState>) -> SaveLoadResponse {
    let guard = state.lock();
    SaveLoadResponse {
        ok: true,
        status: "saved".to_string(),
        snapshot: build_snapshot(&guard.run, &guard.profile),
    }
}

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
