use crate::commands::action_response;
use crate::commands::inputs::{AssignServiceCrewInput, ReprioritizeServiceInput, ServicePriorityDirection, ToggleServiceInput};
use crate::game::content::services::service_by_id_required;
use crate::game::snapshot::ActionResponse;
use crate::runtime::{
    active_service_power_output, active_service_slots, effective_service_power_upkeep,
    reactor_power_output, refresh_runtime_state,
};
use crate::{commit_and_emit, GameState, LastEmittedSnapshot};

#[tauri::command]
pub fn game_toggle_service(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: ToggleServiceInput,
    state: tauri::State<GameState>,
    last_emitted: tauri::State<LastEmittedSnapshot>,
) -> ActionResponse {
    let mut guard = state.lock();

    let service_index = match guard
        .run
        .services
        .iter()
        .position(|service| service.service_id == input.service_id)
    {
        Some(index) => index,
        None => return action_response(&guard.run, &guard.profile, false, Some("unknown-service")),
    };

    if !input.active {
        let service = &mut guard.run.services[service_index];
        service.desired_active = false;
        service.is_active = false;
        service.is_paused = false;
        service.pause_reason = None;
        service.assigned_crew = 0;
        refresh_runtime_state(&mut guard.run);
        let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
        return action_response(&guard.run, &guard.profile, true, None);
    }

    let active_slots = active_service_slots(&guard.run);
    let is_currently_active = guard.run.services[service_index].is_active;
    let active_count = guard.run.services.iter().filter(|service| service.is_active).count() as u8;
    if !is_currently_active && active_count >= active_slots {
        let service = &mut guard.run.services[service_index];
        service.desired_active = true;
        service.is_active = false;
        service.is_paused = true;
        service.pause_reason = Some(crate::game::sim::ServicePauseReason::Capacity);
        return action_response(&guard.run, &guard.profile, false, Some("capacity-reached"));
    }

    let required_crew = service_by_id_required(&input.service_id).crew_required;
    let additional_crew_needed =
        required_crew.saturating_sub(guard.run.services[service_index].assigned_crew);
    if guard.run.resources.crew_available < additional_crew_needed {
        let service = &mut guard.run.services[service_index];
        service.desired_active = true;
        service.is_active = false;
        service.is_paused = true;
        service.pause_reason = Some(crate::game::sim::ServicePauseReason::Crew);
        return action_response(&guard.run, &guard.profile, false, Some("insufficient-crew"));
    }

    let projected_reserved = guard.run.resources.power_reserved
        + effective_service_power_upkeep(&guard.run, &input.service_id)
        - if guard.run.services[service_index].is_active {
            effective_service_power_upkeep(&guard.run, &input.service_id)
        } else {
            0.0
        };
    let projected_available = reactor_power_output(&guard.run)
        - projected_reserved
        + active_service_power_output(&guard.run)
        + if guard.run.services[service_index].is_active {
            0.0
        } else {
            service_by_id_required(&input.service_id).power_output
        };
    if projected_available < 0.0 {
        let service = &mut guard.run.services[service_index];
        service.desired_active = true;
        service.is_active = false;
        service.is_paused = true;
        service.pause_reason = Some(crate::game::sim::ServicePauseReason::Deficit);
        return action_response(&guard.run, &guard.profile, false, Some("power-deficit"));
    }

    let service = &mut guard.run.services[service_index];
    service.desired_active = true;
    service.is_active = true;
    service.is_paused = false;
    service.pause_reason = None;
    service.assigned_crew = required_crew;
    refresh_runtime_state(&mut guard.run);
    let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
    action_response(&guard.run, &guard.profile, true, None)
}

#[tauri::command]
pub fn game_assign_service_crew(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: AssignServiceCrewInput,
    state: tauri::State<GameState>,
    last_emitted: tauri::State<LastEmittedSnapshot>,
) -> ActionResponse {
    let mut guard = state.lock();

    let service_index = match guard
        .run
        .services
        .iter()
        .position(|service| service.service_id == input.service_id)
    {
        Some(index) => index,
        None => return action_response(&guard.run, &guard.profile, false, Some("unknown-service")),
    };
    if input.assigned_crew < 0 {
        return action_response(&guard.run, &guard.profile, false, Some("invalid-assignment"));
    }

    let next_assigned_crew = input.assigned_crew as u8;
    let current_assigned_crew = guard.run.services[service_index].assigned_crew;
    let delta = input.assigned_crew - current_assigned_crew as i32;
    if delta > guard.run.resources.crew_available as i32 {
        return action_response(&guard.run, &guard.profile, false, Some("insufficient-crew"));
    }

    guard.run.services[service_index].assigned_crew = next_assigned_crew;
    refresh_runtime_state(&mut guard.run);
    let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
    action_response(&guard.run, &guard.profile, true, None)
}

#[tauri::command]
pub fn game_reprioritize_service(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: ReprioritizeServiceInput,
    state: tauri::State<GameState>,
    last_emitted: tauri::State<LastEmittedSnapshot>,
) -> ActionResponse {
    let mut guard = state.lock();
    let mut ordered_indices: Vec<_> = (0..guard.run.services.len()).collect();
    ordered_indices.sort_by_key(|index| guard.run.services[*index].priority);

    let current_order_index = match ordered_indices
        .iter()
        .position(|index| guard.run.services[*index].service_id == input.service_id)
    {
        Some(index) => index,
        None => return action_response(&guard.run, &guard.profile, false, Some("unknown-service")),
    };

    let swap_order_index = match input.direction {
        ServicePriorityDirection::Up if current_order_index > 0 => current_order_index - 1,
        ServicePriorityDirection::Down if current_order_index + 1 < ordered_indices.len() => {
            current_order_index + 1
        }
        _ => return action_response(&guard.run, &guard.profile, false, Some("priority-limit")),
    };

    let current_index = ordered_indices[current_order_index];
    let swap_index = ordered_indices[swap_order_index];
    let current_priority = guard.run.services[current_index].priority;
    guard.run.services[current_index].priority = guard.run.services[swap_index].priority;
    guard.run.services[swap_index].priority = current_priority;
    let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
    action_response(&guard.run, &guard.profile, true, None)
}
