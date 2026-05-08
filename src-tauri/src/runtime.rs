use crate::game::content::planets::PlanetModifierTarget;
use crate::game::content::services::service_by_id_required;
use crate::game::content::systems::{system_by_id_required, SystemProgression, HABITAT_RING_ID, REACTOR_CORE_ID};
use crate::game::sim::RunState;

pub(crate) fn refresh_runtime_state(run_state: &mut RunState) {
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
    run_state.resources.power_available =
        run_state.resources.power_generated - run_state.resources.power_reserved + active_service_power_output(run_state);
    run_state.station.station_tier = crate::game::progression::calculate_station_tier(run_state);
    let eligibility = crate::game::progression::evaluate_prestige_eligibility(
        run_state,
        run_state.consecutive_stable_power_ticks,
    );
    run_state.prestige_eligible = eligibility.eligible;
}

pub(crate) fn active_service_slots(run_state: &RunState) -> u8 {
    match system_by_id_required("logistics-spine").progression {
        SystemProgression::LogisticsSpine(levels) => {
            let level = run_state
                .system_level("logistics-spine")
                .unwrap_or(1)
                .clamp(1, levels.len() as u8);
            levels[(level - 1) as usize].active_service_slots
        }
        _ => unreachable!("logistics-spine progression must use logistics levels"),
    }
}

pub(crate) fn habitat_crew_capacity(run_state: &RunState) -> u8 {
    let base_capacity = match system_by_id_required(HABITAT_RING_ID).progression {
        SystemProgression::HabitatRing(levels) => {
            let level = run_state
                .system_level(HABITAT_RING_ID)
                .unwrap_or(1)
                .clamp(1, levels.len() as u8);
            levels[(level - 1) as usize].crew_capacity
        }
        _ => unreachable!("habitat-ring progression must use habitat levels"),
    };
    let planet_modifier = run_state
        .active_planet_definition()
        .modifiers
        .iter()
        .filter(|modifier| matches!(modifier.target, PlanetModifierTarget::CrewCapacity))
        .map(|modifier| modifier.percent)
        .sum::<f32>();

    ((base_capacity as f32) * (1.0 + planet_modifier)).floor().max(1.0) as u8
}

pub(crate) fn reactor_power_output(run_state: &RunState) -> f32 {
    match system_by_id_required(REACTOR_CORE_ID).progression {
        SystemProgression::ReactorCore(levels) => {
            let level = run_state
                .system_level(REACTOR_CORE_ID)
                .unwrap_or(1)
                .clamp(1, levels.len() as u8);
            levels[(level - 1) as usize].power_output
        }
        _ => unreachable!("reactor-core progression must use reactor levels"),
    }
}

pub(crate) fn active_service_power_output(run_state: &RunState) -> f32 {
    run_state
        .services
        .iter()
        .filter(|service| service.is_active)
        .map(|service| service_by_id_required(&service.service_id).power_output)
        .sum()
}

pub(crate) fn effective_service_power_upkeep(run_state: &RunState, service_id: &str) -> f32 {
    let definition = service_by_id_required(service_id);
    let planet_modifier = run_state
        .active_planet_definition()
        .modifiers
        .iter()
        .filter(|modifier| matches!(modifier.target, PlanetModifierTarget::ServicePowerUpkeep))
        .map(|modifier| modifier.percent)
        .sum::<f32>();

    (definition.power_upkeep * (1.0 + planet_modifier + active_service_power_modifier(run_state))).max(0.0)
}

pub(crate) fn active_service_power_modifier(run_state: &RunState) -> f32 {
    run_state
        .services
        .iter()
        .filter(|service| service.is_active)
        .map(|service| service_by_id_required(&service.service_id).global_service_power_modifier)
        .sum::<f32>()
}
