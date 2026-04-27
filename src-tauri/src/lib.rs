mod game;

use std::thread;
use std::time::Duration;
use std::sync::Mutex;

use serde::Deserialize;
#[cfg(debug_assertions)]
use serde::Serialize;
#[cfg(debug_assertions)]
use tauri::Emitter;
#[cfg(debug_assertions)]
use tauri::menu::{MenuBuilder, MenuItem, SubmenuBuilder};
use tauri::Manager;
use crate::game::content::doctrines::doctrine_by_id;
use crate::game::content::planets::{AURORA_PIER_ID, CINDER_FORGE_ID};
use crate::game::content::services::service_by_id;
use crate::game::content::systems::{system_by_id, SystemProgression, HABITAT_RING_ID, REACTOR_CORE_ID, SYSTEMS};
use crate::game::progression::{execute_prestige, DoctrinePurchaseError, PrestigeExecutionError};
use crate::game::progression::PrestigeProfile;
use crate::game::snapshot::{build_snapshot, ActionResponse, RawGameSnapshot, SaveLoadResponse};
use crate::game::sim::{tick, RunState};


struct GameState(Mutex<(RunState, PrestigeProfile, u32)>);

#[cfg(debug_assertions)]
struct DevtoolsState(Mutex<bool>);

#[cfg(debug_assertions)]
const DEVTOOLS_TOGGLE_OVERLAY_MENU_ID: &str = "devtools-toggle-overlay";
#[cfg(debug_assertions)]
const DEVTOOLS_VISIBILITY_CHANGED_EVENT: &str = "devtools:visibility-changed";

#[cfg(debug_assertions)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevtoolsVisibilityInput {
    visible: bool,
}

#[cfg(debug_assertions)]
#[derive(Serialize)]
struct DevtoolsStateResponse {
    visible: bool,
    snapshot: RawGameSnapshot,
}

#[cfg(debug_assertions)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevtoolsApplyResourcesInput {
    materials: f32,
    data: f32,
}

#[cfg(debug_assertions)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevtoolsApplyCrewInput {
    crew_total: u8,
}

#[cfg(debug_assertions)]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct DevtoolsApplySystemEntry {
    id: String,
    level: u8,
}

#[cfg(debug_assertions)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevtoolsApplySystemsInput {
    systems: Vec<DevtoolsApplySystemEntry>,
}

#[cfg(debug_assertions)]
#[derive(Clone, Debug, PartialEq, Serialize)]
struct DevtoolsVisibilityChangedEvent {
    visible: bool,
}

#[cfg(debug_assertions)]
#[tauri::command]
fn game_devtools_get_state(
    game_state: tauri::State<GameState>,
    devtools_state: tauri::State<DevtoolsState>,
) -> Result<DevtoolsStateResponse, String> {
    Ok(current_devtools_state_response(&game_state, &devtools_state))
}

#[cfg(debug_assertions)]
#[tauri::command]
fn game_devtools_set_visibility(
    input: DevtoolsVisibilityInput,
    app: tauri::AppHandle,
) -> Result<DevtoolsStateResponse, String> {
    update_devtools_visibility(&app, input.visible)
}

#[cfg(debug_assertions)]
#[tauri::command]
fn game_devtools_apply_resources(
    input: DevtoolsApplyResourcesInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
) -> Result<serde_json::Value, String> {
    let mut guard = game_state.0.lock().expect("game state mutex poisoned");

    match apply_devtools_resources(&mut guard.0, input.materials, input.data) {
        Ok(()) => {
            refresh_runtime_state(&mut guard.0);
            Ok(devtools_action_success(&guard.0, &guard.1))
        }
        Err(reason_code) => Ok(devtools_action_failure(&guard.0, &guard.1, reason_code)),
    }
}

#[cfg(debug_assertions)]
#[tauri::command]
fn game_devtools_apply_crew(
    input: DevtoolsApplyCrewInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
) -> Result<serde_json::Value, String> {
    let mut guard = game_state.0.lock().expect("game state mutex poisoned");

    match apply_devtools_crew_total(&mut guard.0, input.crew_total) {
        Ok(()) => {
            refresh_runtime_state(&mut guard.0);
            Ok(devtools_action_success(&guard.0, &guard.1))
        }
        Err(reason_code) => Ok(devtools_action_failure(&guard.0, &guard.1, reason_code)),
    }
}

#[cfg(debug_assertions)]
#[tauri::command]
fn game_devtools_apply_systems(
    input: DevtoolsApplySystemsInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
) -> Result<serde_json::Value, String> {
    let mut guard = game_state.0.lock().expect("game state mutex poisoned");

    match apply_devtools_system_levels(&mut guard.0, &input.systems) {
        Ok(()) => {
            refresh_runtime_state(&mut guard.0);
            Ok(devtools_action_success(&guard.0, &guard.1))
        }
        Err(reason_code) => Ok(devtools_action_failure(&guard.0, &guard.1, reason_code)),
    }
}

#[cfg(debug_assertions)]
fn read_devtools_visibility(devtools_state: &DevtoolsState) -> bool {
    *devtools_state
        .0
        .lock()
        .expect("devtools state mutex poisoned")
}

#[cfg(debug_assertions)]
fn set_devtools_visibility_state(devtools_state: &DevtoolsState, visible: bool) -> bool {
    let mut guard = devtools_state
        .0
        .lock()
        .expect("devtools state mutex poisoned");
    *guard = visible;
    *guard
}

#[cfg(debug_assertions)]
#[cfg_attr(not(test), allow(dead_code))]
fn toggle_devtools_visibility_state(devtools_state: &DevtoolsState) -> bool {
    let mut guard = devtools_state
        .0
        .lock()
        .expect("devtools state mutex poisoned");
    *guard = !*guard;
    *guard
}

#[cfg(debug_assertions)]
fn devtools_visibility_payload(visible: bool) -> DevtoolsVisibilityChangedEvent {
    DevtoolsVisibilityChangedEvent { visible }
}

#[cfg(debug_assertions)]
fn build_devtools_state_response(game_state: &GameState, visible: bool) -> DevtoolsStateResponse {
    let guard = game_state.0.lock().expect("game state mutex poisoned");
    DevtoolsStateResponse {
        visible,
        snapshot: build_snapshot(&guard.0, &guard.1),
    }
}

#[cfg(debug_assertions)]
fn current_devtools_state_response(
    game_state: &GameState,
    devtools_state: &DevtoolsState,
) -> DevtoolsStateResponse {
    build_devtools_state_response(game_state, read_devtools_visibility(devtools_state))
}

#[cfg(debug_assertions)]
fn emit_devtools_visibility_changed<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    visible: bool,
) -> Result<(), String> {
    app.emit(
        DEVTOOLS_VISIBILITY_CHANGED_EVENT,
        devtools_visibility_payload(visible),
    )
    .map_err(|error| error.to_string())
}

#[cfg(debug_assertions)]
fn update_devtools_visibility<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    visible: bool,
) -> Result<DevtoolsStateResponse, String> {
    let devtools_state = app.state::<DevtoolsState>();
    let visible = set_devtools_visibility_state(&devtools_state, visible);
    let game_state = app.state::<GameState>();
    let response = build_devtools_state_response(&game_state, visible);
    emit_devtools_visibility_changed(app, visible)?;
    Ok(response)
}

#[cfg(debug_assertions)]
fn toggle_devtools_visibility<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<DevtoolsStateResponse, String> {
    let devtools_state = app.state::<DevtoolsState>();
    let visible = !read_devtools_visibility(&devtools_state);
    update_devtools_visibility(app, visible)
}

#[cfg(debug_assertions)]
fn install_debug_menu<R: tauri::Runtime>(app: &mut tauri::App<R>) -> tauri::Result<()> {
    let toggle_overlay = MenuItem::with_id(
        app,
        DEVTOOLS_TOGGLE_OVERLAY_MENU_ID,
        "Toggle Game State Overlay",
        true,
        None::<&str>,
    )?;
    let debug_menu = SubmenuBuilder::new(app, "Debug")
        .item(&toggle_overlay)
        .build()?;
    let menu = MenuBuilder::new(app).items(&[&debug_menu]).build()?;

    app.set_menu(menu)?;
    app.on_menu_event(|app_handle, event| {
        if event.id().0.as_str() == DEVTOOLS_TOGGLE_OVERLAY_MENU_ID {
            let _ = toggle_devtools_visibility(app_handle);
        }
    });

    Ok(())
}

#[cfg(debug_assertions)]
#[cfg_attr(not(test), allow(dead_code))]
fn devtools_enabled() -> bool {
    true
}

#[cfg(not(debug_assertions))]
fn devtools_enabled() -> bool {
    false
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToggleServiceInput {
    service_id: String,
    active: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpgradeSystemInput {
    system_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PurchaseDoctrineInput {
    doctrine_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfirmPrestigeInput {
    confirm: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssignServiceCrewInput {
    service_id: String,
    assigned_crew: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReprioritizeServiceInput {
    service_id: String,
    direction: ServicePriorityDirection,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum ServicePriorityDirection {
    Up,
    Down,
}

#[tauri::command]
fn game_get_snapshot(state: tauri::State<GameState>) -> RawGameSnapshot {
    let guard = state.0.lock().expect("game state mutex poisoned");
    build_snapshot(&guard.0, &guard.1)
}

#[tauri::command]
fn game_toggle_service(input: ToggleServiceInput, state: tauri::State<GameState>) -> ActionResponse {
    let mut guard = state.0.lock().expect("game state mutex poisoned");

    let service_index = match guard
        .0
        .services
        .iter()
        .position(|service| service.service_id == input.service_id)
    {
        Some(index) => index,
        None => return action_response(&guard.0, &guard.1, false, Some("unknown-service")),
    };

    if !input.active {
        let service = &mut guard.0.services[service_index];
        service.desired_active = false;
        service.is_active = false;
        service.is_paused = false;
        service.pause_reason = None;
        service.assigned_crew = 0;
        refresh_runtime_state(&mut guard.0);
        return action_response(&guard.0, &guard.1, true, None);
    }

    let active_slots = active_service_slots(&guard.0);
    let is_currently_active = guard.0.services[service_index].is_active;
    let active_count = guard.0.services.iter().filter(|service| service.is_active).count() as u8;
    if !is_currently_active && active_count >= active_slots {
        let service = &mut guard.0.services[service_index];
        service.desired_active = true;
        service.is_active = false;
        service.is_paused = true;
        service.pause_reason = Some(crate::game::sim::ServicePauseReason::Capacity);
        return action_response(&guard.0, &guard.1, false, Some("capacity-reached"));
    }

    let required_crew = service_by_id(&input.service_id)
        .expect("service must exist in catalog")
        .crew_required;
    let additional_crew_needed = required_crew.saturating_sub(guard.0.services[service_index].assigned_crew);
    if guard.0.resources.crew_available < additional_crew_needed {
        let service = &mut guard.0.services[service_index];
        service.desired_active = true;
        service.is_active = false;
        service.is_paused = true;
        service.pause_reason = Some(crate::game::sim::ServicePauseReason::Crew);
        return action_response(&guard.0, &guard.1, false, Some("insufficient-crew"));
    }

    let projected_reserved = guard.0.resources.power_reserved
        + effective_service_power_upkeep(&guard.0, &input.service_id)
        - if guard.0.services[service_index].is_active {
            effective_service_power_upkeep(&guard.0, &input.service_id)
        } else {
            0.0
        };
    let projected_available = reactor_power_output(&guard.0)
        - projected_reserved
        + active_service_power_output(&guard.0)
        + if guard.0.services[service_index].is_active {
            0.0
        } else {
            service_by_id(&input.service_id)
                .expect("service must exist in catalog")
                .power_output
        };
    if projected_available < 0.0 {
        let service = &mut guard.0.services[service_index];
        service.desired_active = true;
        service.is_active = false;
        service.is_paused = true;
        service.pause_reason = Some(crate::game::sim::ServicePauseReason::Deficit);
        return action_response(&guard.0, &guard.1, false, Some("power-deficit"));
    }

    let service = &mut guard.0.services[service_index];
    service.desired_active = true;
    service.is_active = true;
    service.is_paused = false;
    service.pause_reason = None;
    service.assigned_crew = required_crew;
    refresh_runtime_state(&mut guard.0);
    action_response(&guard.0, &guard.1, true, None)
}

#[tauri::command]
fn game_upgrade_system(input: UpgradeSystemInput, state: tauri::State<GameState>) -> ActionResponse {
    let mut guard = state.0.lock().expect("game state mutex poisoned");

    let system_index = match guard
        .0
        .systems
        .iter()
        .position(|system| system.system_id == input.system_id)
    {
        Some(index) => index,
        None => return action_response(&guard.0, &guard.1, false, Some("unknown-system")),
    };

    let current_level = guard.0.systems[system_index].level;
    let upgrade_cost = match system_by_id(&input.system_id)
        .expect("system must exist in catalog")
        .progression
    {
        SystemProgression::ReactorCore(levels) => levels[(current_level - 1) as usize].upgrade_cost_materials,
        SystemProgression::HabitatRing(levels) => levels[(current_level - 1) as usize].upgrade_cost_materials,
        SystemProgression::LogisticsSpine(levels) => levels[(current_level - 1) as usize].upgrade_cost_materials,
        SystemProgression::SurveyArray(levels) => levels[(current_level - 1) as usize].upgrade_cost_materials,
    };

    let upgrade_cost = match upgrade_cost {
        Some(cost) => cost,
        None => return action_response(&guard.0, &guard.1, false, Some("max-level")),
    };

    if guard.0.resources.materials < upgrade_cost as f32 {
        return action_response(&guard.0, &guard.1, false, Some("insufficient-materials"));
    }

    guard.0.resources.materials -= upgrade_cost as f32;
    guard.0.systems[system_index].level = guard.0.systems[system_index].level.saturating_add(1);
    refresh_runtime_state(&mut guard.0);
    action_response(&guard.0, &guard.1, true, None)
}

#[tauri::command]
fn game_purchase_doctrine(input: PurchaseDoctrineInput, state: tauri::State<GameState>) -> ActionResponse {
    let mut guard = state.0.lock().expect("game state mutex poisoned");

    if doctrine_by_id(&input.doctrine_id).is_none() {
        return action_response(&guard.0, &guard.1, false, Some("unknown-doctrine"));
    }

    match crate::game::progression::purchase_doctrine(&mut guard.1, &input.doctrine_id) {
        Ok(()) => {
            guard.0.station.doctrine_ids = guard.1.doctrine_ids.clone();
            guard.0.station.doctrine_fragments = guard.1.doctrine_fragments;
            action_response(&guard.0, &guard.1, true, None)
        }
        Err(DoctrinePurchaseError::UnknownDoctrine) => {
            action_response(&guard.0, &guard.1, false, Some("unknown-doctrine"))
        }
        Err(DoctrinePurchaseError::AlreadyUnlocked) => {
            action_response(&guard.0, &guard.1, false, Some("already-unlocked"))
        }
        Err(DoctrinePurchaseError::InsufficientFragments) => {
            action_response(&guard.0, &guard.1, false, Some("insufficient-fragments"))
        }
    }
}

#[tauri::command]
fn game_execute_prestige(input: ConfirmPrestigeInput, state: tauri::State<GameState>) -> ActionResponse {
    let mut guard = state.0.lock().expect("game state mutex poisoned");

    if !input.confirm {
        return action_response(&guard.0, &guard.1, false, Some("confirmation-required"));
    }

    match execute_prestige(&guard.0, &guard.1, guard.0.consecutive_stable_power_ticks) {
        Ok((run_state, profile, stable_ticks)) => {
            guard.0 = run_state;
            guard.1 = profile;
            guard.2 = stable_ticks;
            refresh_runtime_state(&mut guard.0);
            action_response(&guard.0, &guard.1, true, None)
        }
        Err(PrestigeExecutionError::Ineligible(reason)) => action_response(
            &guard.0,
            &guard.1,
            false,
            Some(match reason {
                crate::game::progression::PrestigeIneligibleReason::StationTierBelowFour => {
                    "station-tier-below-four"
                }
                crate::game::progression::PrestigeIneligibleReason::NeedsTwoNonStarterPlanets => {
                    "needs-two-non-starter-planets"
                }
                crate::game::progression::PrestigeIneligibleReason::UnstableNetPower => {
                    "unstable-net-power"
                }
            }),
        ),
        Err(PrestigeExecutionError::Save(_)) => {
            action_response(&guard.0, &guard.1, false, Some("not-implemented"))
        }
    }
}

#[tauri::command]
fn game_assign_service_crew(
    input: AssignServiceCrewInput,
    state: tauri::State<GameState>,
) -> ActionResponse {
    let mut guard = state.0.lock().expect("game state mutex poisoned");

    let service_index = match guard
        .0
        .services
        .iter()
        .position(|service| service.service_id == input.service_id)
    {
        Some(index) => index,
        None => return action_response(&guard.0, &guard.1, false, Some("unknown-service")),
    };
    if input.assigned_crew < 0 {
        return action_response(&guard.0, &guard.1, false, Some("invalid-assignment"));
    }

    let next_assigned_crew = input.assigned_crew as u8;
    let current_assigned_crew = guard.0.services[service_index].assigned_crew;
    let delta = input.assigned_crew - current_assigned_crew as i32;
    if delta > guard.0.resources.crew_available as i32 {
        return action_response(&guard.0, &guard.1, false, Some("insufficient-crew"));
    }

    guard.0.services[service_index].assigned_crew = next_assigned_crew;
    refresh_runtime_state(&mut guard.0);
    action_response(&guard.0, &guard.1, true, None)
}

#[tauri::command]
fn game_reprioritize_service(
    input: ReprioritizeServiceInput,
    state: tauri::State<GameState>,
) -> ActionResponse {
    let mut guard = state.0.lock().expect("game state mutex poisoned");
    let mut ordered_indices: Vec<_> = (0..guard.0.services.len()).collect();
    ordered_indices.sort_by_key(|index| guard.0.services[*index].priority);

    let current_order_index = match ordered_indices
        .iter()
        .position(|index| guard.0.services[*index].service_id == input.service_id)
    {
        Some(index) => index,
        None => return action_response(&guard.0, &guard.1, false, Some("unknown-service")),
    };

    let swap_order_index = match input.direction {
        ServicePriorityDirection::Up if current_order_index > 0 => current_order_index - 1,
        ServicePriorityDirection::Down if current_order_index + 1 < ordered_indices.len() => {
            current_order_index + 1
        }
        _ => return action_response(&guard.0, &guard.1, false, Some("priority-limit")),
    };

    let current_index = ordered_indices[current_order_index];
    let swap_index = ordered_indices[swap_order_index];
    let current_priority = guard.0.services[current_index].priority;
    guard.0.services[current_index].priority = guard.0.services[swap_index].priority;
    guard.0.services[swap_index].priority = current_priority;
    action_response(&guard.0, &guard.1, true, None)
}

#[tauri::command]
fn game_start_survey(state: tauri::State<GameState>) -> ActionResponse {
    let mut guard = state.0.lock().expect("game state mutex poisoned");

    if guard
        .0
        .station
        .discovered_planet_ids
        .iter()
        .any(|planet_id| planet_id == CINDER_FORGE_ID)
        && guard
            .0
            .station
            .discovered_planet_ids
            .iter()
            .any(|planet_id| planet_id == AURORA_PIER_ID)
    {
        return action_response(&guard.0, &guard.1, false, Some("all-planets-discovered"));
    }

    if let Some(service) = guard.0.service_state_mut("survey-uplink") {
        service.desired_active = true;
    }
    action_response(&guard.0, &guard.1, true, None)
}

#[tauri::command]
fn game_request_save(state: tauri::State<GameState>) -> SaveLoadResponse {
    let guard = state.0.lock().expect("game state mutex poisoned");
    SaveLoadResponse {
        ok: true,
        status: "saved".to_string(),
        snapshot: build_snapshot(&guard.0, &guard.1),
    }
}

#[tauri::command]
fn game_request_load(state: tauri::State<GameState>) -> SaveLoadResponse {
    let guard = state.0.lock().expect("game state mutex poisoned");
    SaveLoadResponse {
        ok: true,
        status: "loaded".to_string(),
        snapshot: build_snapshot(&guard.0, &guard.1),
    }
}

fn action_response(
    run_state: &RunState,
    profile: &PrestigeProfile,
    ok: bool,
    reason_code: Option<&str>,
) -> ActionResponse {
    ActionResponse {
        ok,
        snapshot: build_snapshot(run_state, profile),
        reason_code: reason_code.map(str::to_string),
    }
}

#[cfg(debug_assertions)]
fn devtools_action_success(run_state: &RunState, profile: &PrestigeProfile) -> serde_json::Value {
    serde_json::json!({
        "ok": true,
        "snapshot": build_snapshot(run_state, profile),
    })
}

#[cfg(debug_assertions)]
fn devtools_action_failure(
    run_state: &RunState,
    profile: &PrestigeProfile,
    reason_code: &str,
) -> serde_json::Value {
    serde_json::json!({
        "ok": false,
        "reasonCode": reason_code,
        "snapshot": build_snapshot(run_state, profile),
    })
}

#[cfg(any(debug_assertions, test))]
fn apply_devtools_resources(
    run_state: &mut RunState,
    materials: f32,
    data: f32,
) -> Result<(), &'static str> {
    if !(materials >= 0.0 && data >= 0.0) {
        return Err("invalid_range");
    }

    run_state.resources.materials = materials;
    run_state.resources.data = data;
    Ok(())
}

#[cfg(any(debug_assertions, test))]
fn apply_devtools_crew_total(run_state: &mut RunState, crew_total: u8) -> Result<(), &'static str> {
    if crew_total < 1 || crew_total < run_state.resources.crew_assigned {
        return Err("invalid_range");
    }

    if crew_total > habitat_crew_capacity(run_state) {
        return Err("constraint_violation");
    }

    run_state.resources.crew_total = crew_total;
    Ok(())
}

#[cfg(any(debug_assertions, test))]
fn system_max_level(system_id: &str) -> Option<u8> {
    SYSTEMS
        .iter()
        .find(|system| system.id == system_id)
        .map(|system| match system.progression {
            SystemProgression::ReactorCore(levels) => levels.len() as u8,
            SystemProgression::HabitatRing(levels) => levels.len() as u8,
            SystemProgression::LogisticsSpine(levels) => levels.len() as u8,
            SystemProgression::SurveyArray(levels) => levels.len() as u8,
        })
}

#[cfg(any(debug_assertions, test))]
fn apply_devtools_system_levels(
    run_state: &mut RunState,
    systems: &[DevtoolsApplySystemEntry],
) -> Result<(), &'static str> {
    for entry in systems {
        let Some(max_level) = system_max_level(&entry.id) else {
            return Err("unknown_id");
        };

        if entry.level < 1 || entry.level > max_level {
            return Err("invalid_range");
        }

        if !run_state.systems.iter().any(|system| system.system_id == entry.id) {
            return Err("unknown_id");
        }
    }

    for entry in systems {
        let system = run_state
            .systems
            .iter_mut()
            .find(|system| system.system_id == entry.id)
            .expect("validated system id must exist in run state");
        system.level = entry.level;
    }

    Ok(())
}

fn refresh_runtime_state(run_state: &mut RunState) {
    let crew_capacity = habitat_crew_capacity(run_state);
    run_state.resources.crew_total = run_state.resources.crew_total.min(crew_capacity);
    run_state.resources.crew_assigned = run_state
        .services
        .iter()
        .map(|service| service.assigned_crew as u16)
        .sum::<u16>()
        .min(run_state.resources.crew_total as u16) as u8;
    run_state.resources.crew_available = run_state
        .resources
        .crew_total
        .saturating_sub(run_state.resources.crew_assigned);
    run_state.resources.power_generated = reactor_power_output(run_state);
    run_state.resources.power_reserved = crate::game::sim::state::HOUSEKEEPING_POWER_PER_SECOND
        + run_state
            .services
            .iter()
            .filter(|service| service.is_active)
            .map(|service| effective_service_power_upkeep(run_state, &service.service_id))
            .sum::<f32>();
    run_state.resources.power_available = run_state.resources.power_generated
        - run_state.resources.power_reserved
        + active_service_power_output(run_state);
    run_state.station.station_tier = crate::game::progression::calculate_station_tier(run_state);
    let eligibility = crate::game::progression::evaluate_prestige_eligibility(
        run_state,
        run_state.consecutive_stable_power_ticks,
    );
    run_state.prestige_eligible = eligibility.eligible;
}

fn active_service_slots(run_state: &RunState) -> u8 {
    match system_by_id("logistics-spine")
        .expect("logistics-spine system must exist")
        .progression
    {
        SystemProgression::LogisticsSpine(levels) => {
            let level = run_state.system_level("logistics-spine").unwrap_or(1).clamp(1, levels.len() as u8);
            levels[(level - 1) as usize].active_service_slots
        }
        _ => unreachable!("logistics-spine progression must use logistics levels"),
    }
}

fn habitat_crew_capacity(run_state: &RunState) -> u8 {
    let base_capacity = match system_by_id(HABITAT_RING_ID)
        .expect("habitat-ring system must exist")
        .progression
    {
        SystemProgression::HabitatRing(levels) => {
            let level = run_state.system_level(HABITAT_RING_ID).unwrap_or(1).clamp(1, levels.len() as u8);
            levels[(level - 1) as usize].crew_capacity
        }
        _ => unreachable!("habitat-ring progression must use habitat levels"),
    };
    let planet_modifier = run_state
        .active_planet_definition()
        .modifiers
        .iter()
        .filter(|modifier| matches!(modifier.target, crate::game::content::planets::PlanetModifierTarget::CrewCapacity))
        .map(|modifier| modifier.percent)
        .sum::<f32>();

    ((base_capacity as f32) * (1.0 + planet_modifier)).floor().max(1.0) as u8
}

fn reactor_power_output(run_state: &RunState) -> f32 {
    match system_by_id(REACTOR_CORE_ID)
        .expect("reactor-core system must exist")
        .progression
    {
        SystemProgression::ReactorCore(levels) => {
            let level = run_state.system_level(REACTOR_CORE_ID).unwrap_or(1).clamp(1, levels.len() as u8);
            levels[(level - 1) as usize].power_output
        }
        _ => unreachable!("reactor-core progression must use reactor levels"),
    }
}

fn active_service_power_output(run_state: &RunState) -> f32 {
    run_state
        .services
        .iter()
        .filter(|service| service.is_active)
        .map(|service| {
            service_by_id(&service.service_id)
                .expect("service must exist in catalog")
                .power_output
        })
        .sum()
}

fn effective_service_power_upkeep(run_state: &RunState, service_id: &str) -> f32 {
    let definition = service_by_id(service_id).expect("service must exist in catalog");
    let planet_modifier = run_state
        .active_planet_definition()
        .modifiers
        .iter()
        .filter(|modifier| {
            matches!(
                modifier.target,
                crate::game::content::planets::PlanetModifierTarget::ServicePowerUpkeep
            )
        })
        .map(|modifier| modifier.percent)
        .sum::<f32>();
    let service_modifier = run_state
        .services
        .iter()
        .filter(|service| service.is_active)
        .map(|service| {
            service_by_id(&service.service_id)
                .expect("service must exist in catalog")
                .global_service_power_modifier
        })
        .sum::<f32>();

    (definition.power_upkeep * (1.0 + planet_modifier + service_modifier)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_service_input_accepts_camel_case_payload() {
        let input: ToggleServiceInput = serde_json::from_str(
            r#"{"serviceId":"solar-harvester","active":true}"#,
        )
        .expect("camelCase payload should deserialize");

        assert_eq!(input.service_id, "solar-harvester");
        assert!(input.active);
    }

    #[test]
    fn upgrade_system_input_accepts_camel_case_payload() {
        let input: UpgradeSystemInput =
            serde_json::from_str(r#"{"systemId":"reactor-core"}"#)
                .expect("camelCase payload should deserialize");

        assert_eq!(input.system_id, "reactor-core");
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

    #[test]
    fn devtools_visibility_toggle_flips_separate_mutex_state() {
        #[cfg(debug_assertions)]
        {
            let devtools_state = DevtoolsState(Mutex::new(false));

            assert!(!read_devtools_visibility(&devtools_state));
            assert!(toggle_devtools_visibility_state(&devtools_state));
            assert!(read_devtools_visibility(&devtools_state));
            assert!(!toggle_devtools_visibility_state(&devtools_state));
            assert!(!read_devtools_visibility(&devtools_state));
            assert!(set_devtools_visibility_state(&devtools_state, true));
            assert!(read_devtools_visibility(&devtools_state));
        }
    }

    #[test]
    fn devtools_visibility_event_payload_matches_frontend_contract() {
        #[cfg(debug_assertions)]
        {
            let payload = serde_json::to_value(devtools_visibility_payload(true))
                .expect("event payload should serialize");

            assert_eq!(payload, serde_json::json!({ "visible": true }));
        }
    }

    #[test]
    fn devtools_enabled_helper_matches_build_gating() {
        assert_eq!(devtools_enabled(), cfg!(debug_assertions));
    }

    #[test]
    fn apply_resources_success() {
        let mut run_state = RunState::starter_fixture();

        apply_devtools_resources(&mut run_state, 250.0, 15.5).expect("resources should apply");

        assert_eq!(run_state.resources.materials, 250.0);
        assert_eq!(run_state.resources.data, 15.5);
    }

    #[test]
    fn apply_resources_rejects_negative() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_resources(&mut run_state, -1.0, 0.0)
            .expect_err("negative resources should be rejected");

        assert_eq!(error, "invalid_range");
        assert_eq!(run_state.resources.materials, 120.0);
    }

    #[test]
    fn apply_crew_success() {
        let mut run_state = RunState::starter_fixture();
        run_state.services[0].assigned_crew = 2;

        apply_devtools_crew_total(&mut run_state, 4).expect("crew total should apply");
        refresh_runtime_state(&mut run_state);

        assert_eq!(run_state.resources.crew_total, 4);
        assert_eq!(run_state.resources.crew_available, 2);
    }

    #[test]
    fn apply_crew_rejects_below_assigned() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_crew_total(&mut run_state, 1)
            .expect_err("crew below assigned should be rejected");

        assert_eq!(error, "invalid_range");
        assert_eq!(run_state.resources.crew_total, 6);
    }

    #[test]
    fn apply_systems_sets_reactor_level() {
        let mut run_state = RunState::starter_fixture();

        apply_devtools_system_levels(
            &mut run_state,
            &[DevtoolsApplySystemEntry {
                id: REACTOR_CORE_ID.to_string(),
                level: 2,
            }],
        )
        .expect("system levels should apply");
        refresh_runtime_state(&mut run_state);

        assert_eq!(run_state.system_level(REACTOR_CORE_ID), Some(2));
        assert_eq!(run_state.resources.power_generated, 12.0);
    }

    #[test]
    fn apply_systems_rejects_unknown_id() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_system_levels(
            &mut run_state,
            &[DevtoolsApplySystemEntry {
                id: "unknown-system".to_string(),
                level: 1,
            }],
        )
        .expect_err("unknown system should be rejected");

        assert_eq!(error, "unknown_id");
    }

    #[test]
    fn apply_systems_rejects_out_of_range_level() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_system_levels(
            &mut run_state,
            &[DevtoolsApplySystemEntry {
                id: HABITAT_RING_ID.to_string(),
                level: 5,
            }],
        )
        .expect_err("out of range level should be rejected");

        assert_eq!(error, "invalid_range");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    let builder = builder.setup(|app| {
        app.manage(GameState(Mutex::new((
            RunState::starter_fixture(),
            PrestigeProfile::default(),
            0u32,
        ))));

        #[cfg(debug_assertions)]
        {
            app.manage(DevtoolsState(Mutex::new(false)));
            install_debug_menu(app)?;
        }

        let app_handle = app.handle().clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(250));

            let state = app_handle.state::<GameState>();
            let mut guard = state.0.lock().expect("game state mutex poisoned");
            tick(&mut guard.0);
        });

        Ok(())
    });

    #[cfg(debug_assertions)]
    let builder = builder.invoke_handler(tauri::generate_handler![
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
        game_devtools_get_state,
        game_devtools_set_visibility,
        game_devtools_apply_resources,
        game_devtools_apply_crew,
        game_devtools_apply_systems,
    ]);

    #[cfg(not(debug_assertions))]
    let builder = builder.invoke_handler(tauri::generate_handler![
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
    ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
