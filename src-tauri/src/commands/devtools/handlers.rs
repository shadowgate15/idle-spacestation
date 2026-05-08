use crate::commands::devtools::apply::{
    apply_devtools_advance_ticks, apply_devtools_crew_total, apply_devtools_progression,
    apply_devtools_resources, apply_devtools_services, apply_devtools_system_levels,
    reset_devtools_session,
};
use crate::commands::devtools::inputs::{
    DevtoolsAdvanceTicksInput, DevtoolsApplyCrewInput, DevtoolsApplyProgressionInput,
    DevtoolsApplyResourcesInput, DevtoolsApplyServicesInput, DevtoolsApplySystemsInput,
    DevtoolsStateResponse, DevtoolsVisibilityInput,
};
use crate::commands::devtools::{
    current_devtools_state_response, devtools_action_failure, devtools_action_success,
    update_devtools_visibility,
};
use crate::runtime::refresh_runtime_state;
use crate::{commit_and_emit, DevtoolsState, GameRunState, GameState, LastEmittedSnapshot};

#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_get_state(
    game_state: tauri::State<GameState>,
    devtools_state: tauri::State<DevtoolsState>,
) -> Result<DevtoolsStateResponse, String> {
    Ok(current_devtools_state_response(&game_state, &devtools_state))
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_set_visibility(
    input: DevtoolsVisibilityInput,
    app: tauri::AppHandle,
) -> Result<DevtoolsStateResponse, String> {
    update_devtools_visibility(&app, input.visible)
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_apply_resources(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: DevtoolsApplyResourcesInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    let mut guard = game_state.lock();

    match apply_devtools_resources(&mut guard.run, input.materials, input.data) {
        Ok(()) => {
            refresh_runtime_state(&mut guard.run);
            let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
            Ok(devtools_action_success(&guard.run, &guard.profile))
        }
        Err(reason_code) => Ok(devtools_action_failure(&guard.run, &guard.profile, reason_code)),
    }
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_apply_crew(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: DevtoolsApplyCrewInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    let mut guard = game_state.lock();

    match apply_devtools_crew_total(&mut guard.run, input.crew_total) {
        Ok(()) => {
            refresh_runtime_state(&mut guard.run);
            let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
            Ok(devtools_action_success(&guard.run, &guard.profile))
        }
        Err(reason_code) => Ok(devtools_action_failure(&guard.run, &guard.profile, reason_code)),
    }
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_apply_systems(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: DevtoolsApplySystemsInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    let mut guard = game_state.lock();

    match apply_devtools_system_levels(&mut guard.run, &input.systems) {
        Ok(()) => {
            refresh_runtime_state(&mut guard.run);
            let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
            Ok(devtools_action_success(&guard.run, &guard.profile))
        }
        Err(reason_code) => Ok(devtools_action_failure(&guard.run, &guard.profile, reason_code)),
    }
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_apply_services(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: DevtoolsApplyServicesInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    let mut guard = game_state.lock();

    match apply_devtools_services(&mut guard.run, &input.services) {
        Ok(()) => {
            refresh_runtime_state(&mut guard.run);
            let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
            Ok(devtools_action_success(&guard.run, &guard.profile))
        }
        Err(reason_code) => Ok(devtools_action_failure(&guard.run, &guard.profile, reason_code)),
    }
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_apply_progression(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: DevtoolsApplyProgressionInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    let mut guard = game_state.lock();
    let GameRunState { run: run_state, profile, .. } = &mut *guard;

    match apply_devtools_progression(run_state, profile, &input) {
        Ok(()) => {
            refresh_runtime_state(run_state);
            let _ = commit_and_emit(&app, run_state, profile, &last_emitted);
            Ok(devtools_action_success(run_state, profile))
        }
        Err(reason_code) => Ok(devtools_action_failure(run_state, profile, reason_code)),
    }
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_advance_ticks(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: DevtoolsAdvanceTicksInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    let mut guard = game_state.lock();

    // apply_devtools_advance_ticks runs the tick loop internally; we emit ONCE
    // after all ticks complete (not per-tick) to avoid flooding the frontend.
    match apply_devtools_advance_ticks(&mut guard.run, input.count) {
        Ok(()) => {
            guard.session_ticks = guard.session_ticks.saturating_add(input.count);
            let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
            Ok(devtools_action_success(&guard.run, &guard.profile))
        }
        Err(reason_code) => Ok(devtools_action_failure(&guard.run, &guard.profile, reason_code)),
    }
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_reset_to_starter(
    app: tauri::AppHandle<impl tauri::Runtime>,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    let mut guard = game_state.lock();
    let GameRunState { run: run_state, profile, session_ticks } = &mut *guard;

    reset_devtools_session(run_state, profile, session_ticks);
    refresh_runtime_state(run_state);
    let _ = commit_and_emit(&app, run_state, profile, &last_emitted);
    Ok(devtools_action_success(run_state, profile))
}
