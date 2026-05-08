//! Service-domain Tauri command handlers.
//!
//! Covers activation, crew assignment, and priority ordering for services.
//! All three commands mutate state and emit `game://state-changed` via
//! [`commit_and_emit`] when they succeed.
//!
//! See also: [`crate::commands`], [`crate::commands::inputs`].

use crate::commands::action_response;
use crate::commands::inputs::{
    AssignServiceCrewInput, ReprioritizeServiceInput, ServicePriorityDirection, ToggleServiceInput,
};
use crate::game::content::services::service_by_id_required;
use crate::game::sim::{RunState, ServicePauseReason};
use crate::game::snapshot::ActionResponse;
use crate::runtime::{active_service_slots, projected_power_after_toggle, refresh_runtime_state};
use crate::{commit_and_emit, GameState, LastEmittedSnapshot};

/// Activate or deactivate a service.
///
/// **Frontend alias**: `game_set_service_activation`
/// **Mutates state**: yes
/// **Emits `game://state-changed`**: yes (via [`commit_and_emit`]) on success
///
/// Deactivation always succeeds; activation must pass capacity, crew, and
/// power checks. When activation is rejected for capacity/crew/deficit
/// reasons the service is left in a paused state with the matching
/// [`ServicePauseReason`] so the UI can surface the cause.
///
/// # Errors
/// Returns an `ActionResponse { ok: false, reason_code: Some(_) }` (mapped to
/// `ServiceActivationRejectionCode` on the frontend) for one of:
/// - `unknown-service`: no service in `RunState` matches the requested id.
/// - `capacity-reached`: active service slots are exhausted for the station.
/// - `insufficient-crew`: not enough free crew to cover the service requirement.
/// - `power-deficit`: enabling the service would push projected power below 0.
#[tauri::command]
pub fn game_toggle_service(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: ToggleServiceInput,
    state: tauri::State<GameState>,
    last_emitted: tauri::State<LastEmittedSnapshot>,
) -> ActionResponse {
    let mut guard = state.lock();
    let result = apply_service_toggle(&mut guard.run, &input);
    if result.is_ok() {
        let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
    }

    action_response(&guard.run, &guard.profile, result.is_ok(), result.err())
}

fn apply_service_toggle(
    run: &mut RunState,
    input: &ToggleServiceInput,
) -> Result<(), &'static str> {
    let service_index = match run
        .services
        .iter()
        .position(|service| service.service_id == input.service_id)
    {
        Some(index) => index,
        None => return Err("unknown-service"),
    };

    if !input.active {
        run.services[service_index].deactivate();
        refresh_runtime_state(run);
        return Ok(());
    }

    let is_currently_active = run.services[service_index].is_active;
    let active_count = run
        .services
        .iter()
        .filter(|service| service.is_active)
        .count() as u8;
    if !is_currently_active && active_count >= active_service_slots(run) {
        run.services[service_index].pause_with(ServicePauseReason::Capacity);
        return Err("capacity-reached");
    }

    let required_crew = service_by_id_required(&input.service_id).crew_required;
    let additional_crew_needed =
        required_crew.saturating_sub(run.services[service_index].assigned_crew);
    if run.resources.crew_available < additional_crew_needed {
        run.services[service_index].pause_with(ServicePauseReason::Crew);
        return Err("insufficient-crew");
    }

    if projected_power_after_toggle(run, &input.service_id, is_currently_active) < 0.0 {
        run.services[service_index].pause_with(ServicePauseReason::Deficit);
        return Err("power-deficit");
    }

    run.services[service_index].activate(required_crew);
    refresh_runtime_state(run);
    Ok(())
}

/// Assign a specific crew count to a service.
///
/// **Frontend alias**: `game_assign_service_crew` (same as Rust name)
/// **Mutates state**: yes
/// **Emits `game://state-changed`**: yes (via [`commit_and_emit`]) on success
///
/// Replaces the service's `assigned_crew` with the requested value, then
/// re-projects runtime state. Negative values and assignments that exceed the
/// available free crew are rejected without mutation.
///
/// # Errors
/// Returns an `ActionResponse { ok: false, reason_code: Some(_) }` (mapped to
/// `ServiceCrewAssignmentRejectionCode` on the frontend) for one of:
/// - `unknown-service`: no service in `RunState` matches the requested id.
/// - `invalid-assignment`: the requested crew count is negative.
/// - `insufficient-crew`: assignment delta exceeds currently free crew.
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
        return action_response(
            &guard.run,
            &guard.profile,
            false,
            Some("invalid-assignment"),
        );
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

/// Move a service one slot up or down in the priority order.
///
/// **Frontend alias**: `game_reprioritize_service` (same as Rust name)
/// **Mutates state**: yes
/// **Emits `game://state-changed`**: yes (via [`commit_and_emit`]) on success
///
/// Swaps the priority value of the target service with its neighbour in the
/// requested direction. The relative priority of all other services is
/// preserved.
///
/// # Errors
/// Returns an `ActionResponse { ok: false, reason_code: Some(_) }` (mapped to
/// `ServicePriorityRejectionCode` on the frontend) for one of:
/// - `unknown-service`: no service in `RunState` matches the requested id.
/// - `priority-limit`: the service is already at the top/bottom of the order.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::content::services::{ORE_RECLAIMER_ID, SOLAR_HARVESTER_ID, SURVEY_UPLINK_ID};
    use crate::game::sim::ServiceState;

    fn toggle_input(service_id: &str, active: bool) -> ToggleServiceInput {
        ToggleServiceInput {
            service_id: service_id.to_string(),
            active,
        }
    }

    #[test]
    fn service_state_deactivate_clears_all_fields() {
        let mut service = ServiceState::new(ORE_RECLAIMER_ID, true, 2);
        service.pause_with(ServicePauseReason::Crew);
        service.assigned_crew = 3;

        service.deactivate();

        assert!(!service.desired_active);
        assert!(!service.is_active);
        assert!(!service.is_paused);
        assert_eq!(service.pause_reason, None);
        assert_eq!(service.assigned_crew, 0);
    }

    #[test]
    fn service_state_pause_with_sets_correct_fields() {
        let mut service = ServiceState::new(ORE_RECLAIMER_ID, true, 2);
        service.activate(2);

        service.pause_with(ServicePauseReason::Deficit);

        assert!(service.desired_active);
        assert!(!service.is_active);
        assert!(service.is_paused);
        assert_eq!(service.pause_reason, Some(ServicePauseReason::Deficit));
        assert_eq!(service.assigned_crew, 2);
    }

    #[test]
    fn service_state_activate_sets_correct_fields() {
        let mut service = ServiceState::new(ORE_RECLAIMER_ID, false, 2);
        service.pause_with(ServicePauseReason::Capacity);

        service.activate(1);

        assert!(service.desired_active);
        assert!(service.is_active);
        assert!(!service.is_paused);
        assert_eq!(service.pause_reason, None);
        assert_eq!(service.assigned_crew, 1);
    }

    #[test]
    fn toggle_service_off_clears_state() {
        let mut run = RunState::starter_fixture();
        let service = run.service_state_mut(ORE_RECLAIMER_ID).unwrap();
        service.activate(1);
        service.pause_with(ServicePauseReason::Crew);
        refresh_runtime_state(&mut run);

        let result = apply_service_toggle(&mut run, &toggle_input(ORE_RECLAIMER_ID, false));

        let service = run.service_state(ORE_RECLAIMER_ID).unwrap();
        assert_eq!(result, Ok(()));
        assert!(!service.desired_active);
        assert!(!service.is_active);
        assert!(!service.is_paused);
        assert_eq!(service.pause_reason, None);
        assert_eq!(service.assigned_crew, 0);
    }

    #[test]
    fn toggle_service_unknown_service_returns_reason_code() {
        let mut run = RunState::starter_fixture();

        let result = apply_service_toggle(&mut run, &toggle_input("unknown-service-id", true));

        assert_eq!(result, Err("unknown-service"));
    }

    #[test]
    fn toggle_service_capacity_reached_returns_reason_code() {
        let mut run = RunState::starter_fixture();
        run.service_state_mut(ORE_RECLAIMER_ID).unwrap().activate(1);
        refresh_runtime_state(&mut run);

        let result = apply_service_toggle(&mut run, &toggle_input(SURVEY_UPLINK_ID, true));

        let service = run.service_state(SURVEY_UPLINK_ID).unwrap();
        assert_eq!(result, Err("capacity-reached"));
        assert_eq!(service.pause_reason, Some(ServicePauseReason::Capacity));
    }

    #[test]
    fn toggle_service_insufficient_crew_returns_reason_code() {
        let mut run = RunState::starter_fixture();
        run.resources.crew_available = 0;

        let result = apply_service_toggle(&mut run, &toggle_input(ORE_RECLAIMER_ID, true));

        let service = run.service_state(ORE_RECLAIMER_ID).unwrap();
        assert_eq!(result, Err("insufficient-crew"));
        assert_eq!(service.pause_reason, Some(ServicePauseReason::Crew));
    }

    #[test]
    fn toggle_service_power_deficit_returns_reason_code() {
        let mut run = RunState::starter_fixture();
        run.resources.crew_available = 4;
        run.resources.power_reserved = 100.0;

        let result = apply_service_toggle(&mut run, &toggle_input(ORE_RECLAIMER_ID, true));

        let service = run.service_state(ORE_RECLAIMER_ID).unwrap();
        assert_eq!(result, Err("power-deficit"));
        assert_eq!(service.pause_reason, Some(ServicePauseReason::Deficit));
    }

    #[test]
    fn toggle_service_active_service_keeps_rejection_order_after_capacity() {
        let mut run = RunState::starter_fixture();
        run.service_state_mut(ORE_RECLAIMER_ID).unwrap().activate(1);
        refresh_runtime_state(&mut run);
        run.resources.crew_available = 0;
        run.resources.power_reserved = 100.0;

        let result = apply_service_toggle(&mut run, &toggle_input(SURVEY_UPLINK_ID, true));

        assert_eq!(result, Err("capacity-reached"));
    }

    #[test]
    fn toggle_service_on_activates_and_refreshes() {
        let mut run = RunState::starter_fixture();

        let result = apply_service_toggle(&mut run, &toggle_input(ORE_RECLAIMER_ID, true));

        let service = run.service_state(ORE_RECLAIMER_ID).unwrap();
        assert_eq!(result, Ok(()));
        assert!(service.is_active);
        assert_eq!(service.assigned_crew, 1);
        assert_eq!(run.resources.crew_assigned, 1);
    }

    #[test]
    fn toggle_service_off_allows_starter_solar_to_clear_state() {
        let mut run = RunState::starter_fixture();

        let result = apply_service_toggle(&mut run, &toggle_input(SOLAR_HARVESTER_ID, false));

        let service = run.service_state(SOLAR_HARVESTER_ID).unwrap();
        assert_eq!(result, Ok(()));
        assert!(!service.is_active);
        assert_eq!(service.assigned_crew, 0);
    }
}
