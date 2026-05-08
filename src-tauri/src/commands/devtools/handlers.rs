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
    current_devtools_state_response, run_devtools_mutation, update_devtools_visibility,
};
use crate::{DevtoolsState, GameState, LastEmittedSnapshot};

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
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, _, _| {
        apply_devtools_resources(run, input.materials, input.data)
    })
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
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, _, _| {
        apply_devtools_crew_total(run, input.crew_total)
    })
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
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, _, _| {
        apply_devtools_system_levels(run, &input.systems)
    })
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
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, _, _| {
        apply_devtools_services(run, &input.services)
    })
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
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, profile, _| {
        apply_devtools_progression(run, profile, &input)
    })
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
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, _, session_ticks| {
        apply_devtools_advance_ticks(run, input.count)?;
        *session_ticks = session_ticks.saturating_add(input.count);
        Ok(())
    })
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_reset_to_starter(
    app: tauri::AppHandle<impl tauri::Runtime>,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, profile, session_ticks| {
        reset_devtools_session(run, profile, session_ticks);
        Ok(())
    })
}
