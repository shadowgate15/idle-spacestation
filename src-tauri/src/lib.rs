mod game;

use std::sync::Mutex;

use serde::Deserialize;
use crate::game::content::doctrines::doctrine_by_id;
use crate::game::content::planets::{planet_by_id, AURORA_PIER_ID, CINDER_FORGE_ID};
use crate::game::content::services::service_by_id;
use crate::game::content::systems::{system_by_id, SystemProgression, HABITAT_RING_ID, REACTOR_CORE_ID};
use crate::game::progression::{execute_prestige, DoctrinePurchaseError, PrestigeExecutionError};
use crate::game::progression::PrestigeProfile;
use crate::game::snapshot::{build_snapshot, ActionResponse, RawGameSnapshot, SaveLoadResponse};
use crate::game::sim::RunState;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

struct GameState(Mutex<(RunState, PrestigeProfile, u32)>);

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
struct SelectPlanetInput {
    planet_id: String,
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
fn game_select_planet(input: SelectPlanetInput, state: tauri::State<GameState>) -> ActionResponse {
    let mut guard = state.0.lock().expect("game state mutex poisoned");

    if planet_by_id(&input.planet_id).is_none() {
        return action_response(&guard.0, &guard.1, false, Some("unknown-planet"));
    }
    if !guard
        .0
        .station
        .discovered_planet_ids
        .iter()
        .any(|planet_id| planet_id == &input.planet_id)
    {
        return action_response(&guard.0, &guard.1, false, Some("planet-undiscovered"));
    }
    if guard.0.station.active_planet_id == input.planet_id {
        return action_response(&guard.0, &guard.1, false, Some("planet-not-selectable"));
    }

    guard.0.station.active_planet_id = input.planet_id;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .manage(GameState(Mutex::new((
            RunState::starter_fixture(),
            PrestigeProfile::default(),
            0u32,
        ))))
        .invoke_handler(tauri::generate_handler![
            greet,
            game_get_snapshot,
            game_toggle_service,
            game_upgrade_system,
            game_assign_service_crew,
            game_reprioritize_service,
            game_select_planet,
            game_start_survey,
            game_purchase_doctrine,
            game_execute_prestige,
            game_request_save,
            game_request_load,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
