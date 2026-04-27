#![allow(dead_code)]

use crate::game::content::doctrines::{doctrine_by_id, DoctrineEffect};
use crate::game::content::planets::{PlanetModifierTarget, AURORA_PIER_ID, CINDER_FORGE_ID};
use crate::game::content::services::{service_by_id, ServiceCategory};
use crate::game::content::systems::{
    system_by_id, HabitatRingLevel, LogisticsSpineLevel, ReactorCoreLevel, SurveyArrayLevel,
    SystemProgression, HABITAT_RING_ID, LOGISTICS_SPINE_ID, REACTOR_CORE_ID, SURVEY_ARRAY_ID,
};
use crate::game::sim::deficit::{resolve_power_deficit, PendingServiceDelta};
use crate::game::sim::state::{
    AUTOSAVE_CADENCE_TICKS, AURORA_PIER_SURVEY_THRESHOLD, CINDER_FORGE_SURVEY_THRESHOLD,
    HOUSEKEEPING_POWER_PER_SECOND, RunState, SECONDS_PER_TICK, ServicePauseReason,
};

#[derive(Debug, Clone, Copy)]
struct AllocationState {
    reactor: ReactorCoreLevel,
    logistics: LogisticsSpineLevel,
    materials_capacity: f32,
}

pub fn tick(state: &mut RunState) {
    let allocation = allocation_phase(state);
    service_activation_phase(state, allocation);

    let mut pending = production_conversion_phase(state, allocation);
    upkeep_phase(state, &pending);
    resolve_power_deficit(state, &mut pending);
    surveys_unlocks_phase(state, allocation, &pending);
    autosave_prestige_check_phase(state);
}

fn allocation_phase(state: &mut RunState) -> AllocationState {
    let reactor = reactor_level(state);
    let habitat = habitat_level(state);
    let logistics = logistics_level(state);
    let _survey_array = survey_array_level(state);

    let materials_capacity = logistics.materials_capacity as f32;
    let crew_capacity = effective_crew_capacity(state, habitat.crew_capacity);

    state.resources.materials = state.resources.materials.clamp(0.0, materials_capacity);
    state.resources.crew_total = state.resources.crew_total.min(crew_capacity);
    state.resources.crew_assigned = state.resources.crew_assigned.min(state.resources.crew_total);
    state.resources.crew_available = state
        .resources
        .crew_total
        .saturating_sub(state.resources.crew_assigned);

    state.recalculate_station_tier();
    state.autosave_due = false;

    for service in &mut state.services {
        service.is_active = false;
        service.is_paused = false;
        service.pause_reason = None;
        service.assigned_crew = 0;
    }

    AllocationState {
        reactor,
        logistics,
        materials_capacity,
    }
}

fn service_activation_phase(state: &mut RunState, allocation: AllocationState) {
    let mut active_slots = 0u8;
    let mut available_crew = state.resources.crew_total;
    let mut support_discount_used = false;

    for index in state.ordered_service_indices() {
        let definition = state.services[index].definition();
        if !state.services[index].desired_active {
            continue;
        }

        if active_slots >= allocation.logistics.active_service_slots {
            mark_paused(&mut state.services[index], ServicePauseReason::Capacity);
            continue;
        }

        let crew_required = effective_crew_required(
            state,
            definition.category,
            definition.crew_required,
            support_discount_used,
        );

        if available_crew < crew_required {
            mark_paused(&mut state.services[index], ServicePauseReason::Crew);
            continue;
        }

        state.services[index].is_active = true;
        state.services[index].assigned_crew = crew_required;

        if matches!(definition.category, ServiceCategory::Support)
            && first_support_service_discount(state).is_some()
            && !support_discount_used
        {
            support_discount_used = true;
        }

        let total_upkeep = total_active_service_power_upkeep(state);
        if total_upkeep > allocation.reactor.service_power_cap as f32 + f32::EPSILON {
            mark_paused(&mut state.services[index], ServicePauseReason::PowerCap);
            continue;
        }

        active_slots += 1;
        available_crew = available_crew.saturating_sub(crew_required);
    }

    let assigned_crew: u16 = state
        .services
        .iter()
        .filter(|service| service.is_active)
        .map(|service| service.assigned_crew as u16)
        .sum();

    state.resources.crew_assigned = assigned_crew.min(state.resources.crew_total as u16) as u8;
    state.resources.crew_available = state
        .resources
        .crew_total
        .saturating_sub(state.resources.crew_assigned);
    state.resources.power_generated = allocation.reactor.power_output;
    state.resources.power_reserved =
        HOUSEKEEPING_POWER_PER_SECOND + total_active_service_power_upkeep(state);
    state.resources.power_available = state.resources.power_generated - state.resources.power_reserved;
}

fn production_conversion_phase(
    state: &RunState,
    allocation: AllocationState,
) -> Vec<PendingServiceDelta> {
    let mut pending = vec![PendingServiceDelta::default(); state.services.len()];
    let mut shadow_materials = state.resources.materials;

    for index in state.ordered_service_indices() {
        let service_state = &state.services[index];
        if !service_state.is_active {
            continue;
        }

        let definition = service_state.definition();
        let materials_drain_per_tick =
            (definition.materials_upkeep + (-definition.materials_input).max(0.0)) * SECONDS_PER_TICK;

        let scale = if materials_drain_per_tick > 0.0 {
            (shadow_materials / materials_drain_per_tick).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let materials_output = definition.materials_output
            * SECONDS_PER_TICK
            * effective_materials_output_multiplier(state)
            * scale;
        let materials_delta = materials_output - (materials_drain_per_tick * scale);
        let max_material_gain = allocation.materials_capacity - shadow_materials;
        let clamped_material_delta = if materials_delta.is_sign_positive() {
            materials_delta.min(max_material_gain.max(0.0))
        } else {
            materials_delta.max(-shadow_materials)
        };

        shadow_materials = (shadow_materials + clamped_material_delta).clamp(0.0, allocation.materials_capacity);

        pending[index] = PendingServiceDelta {
            power_upkeep: effective_service_power_upkeep(state, &service_state.service_id),
            power_output: definition.power_output,
            materials_delta: clamped_material_delta,
            data_delta: definition.data_output
                * SECONDS_PER_TICK
                * effective_data_output_multiplier(state)
                * scale,
            survey_delta: definition.survey_points
                * SECONDS_PER_TICK
                * effective_survey_output_multiplier(state, &service_state.service_id),
        };
    }

    pending
}

fn upkeep_phase(state: &mut RunState, pending: &[PendingServiceDelta]) {
    let power_delta: f32 = pending.iter().map(|service| service.power_output).sum();
    state.resources.power_available += power_delta;
}

fn surveys_unlocks_phase(
    state: &mut RunState,
    allocation: AllocationState,
    pending: &[PendingServiceDelta],
) {
    state.resources.materials = (state.resources.materials
        + pending.iter().map(|service| service.materials_delta).sum::<f32>())
        .clamp(0.0, allocation.materials_capacity);
    state.resources.data += pending.iter().map(|service| service.data_delta).sum::<f32>();

    let data_delta: f32 = pending.iter().map(|service| service.data_delta).sum();
    if data_delta > 0.0 {
        state.lifetime_data_produced = state
            .lifetime_data_produced
            .saturating_add(data_delta.floor() as u64);
    }

    state.station.survey_progress += pending.iter().map(|service| service.survey_delta).sum::<f32>();

    unlock_planet_if_ready(state, CINDER_FORGE_ID, CINDER_FORGE_SURVEY_THRESHOLD);
    unlock_planet_if_ready(state, AURORA_PIER_ID, AURORA_PIER_SURVEY_THRESHOLD);
}

fn autosave_prestige_check_phase(state: &mut RunState) {
    state.tick_count += 1;
    state.autosave_due = state.tick_count % AUTOSAVE_CADENCE_TICKS == 0;
    if state.autosave_due {
        state.autosave_count += 1;
        state.last_autosave_tick = Some(state.tick_count);
    }

    let net_power = state.resources.power_available;
    state.consecutive_stable_power_ticks =
        crate::game::progression::prestige::update_stable_power_ticks(
            state.consecutive_stable_power_ticks,
            net_power,
        );

    let eligibility = crate::game::progression::prestige::evaluate_prestige_eligibility(
        state,
        state.consecutive_stable_power_ticks,
    );
    state.prestige_eligible = eligibility.eligible;
}

fn unlock_planet_if_ready(state: &mut RunState, planet_id: &str, threshold: f32) {
    if state.station.survey_progress + f32::EPSILON < threshold {
        return;
    }

    if !state
        .station
        .discovered_planet_ids
        .iter()
        .any(|candidate| candidate == planet_id)
    {
        state.station.discovered_planet_ids.push(planet_id.to_string());
        state.station.discovered_planet_ids.sort();
    }
}

fn effective_crew_capacity(state: &RunState, base_capacity: u8) -> u8 {
    let modifier = planet_modifier_total(state, PlanetModifierTarget::CrewCapacity);
    ((base_capacity as f32) * (1.0 + modifier)).floor().max(1.0) as u8
}

fn effective_crew_required(
    state: &RunState,
    category: ServiceCategory,
    base_required: u8,
    support_discount_used: bool,
) -> u8 {
    let crew_efficiency = planet_modifier_total(state, PlanetModifierTarget::CrewEfficiency);
    let discounted = if matches!(category, ServiceCategory::Support) && !support_discount_used {
        if let Some((reduction, minimum)) = first_support_service_discount(state) {
            base_required.saturating_sub(reduction).max(minimum)
        } else {
            base_required
        }
    } else {
        base_required
    };

    ((discounted as f32) / (1.0 + crew_efficiency))
        .ceil()
        .max(1.0) as u8
}

fn total_active_service_power_upkeep(state: &RunState) -> f32 {
    state
        .services
        .iter()
        .filter(|service| service.is_active)
        .map(|service| effective_service_power_upkeep(state, &service.service_id))
        .sum()
}

fn effective_service_power_upkeep(state: &RunState, service_id: &str) -> f32 {
    let definition = service_by_id(service_id).expect("service must exist in catalog");
    let modifier = planet_modifier_total(state, PlanetModifierTarget::ServicePowerUpkeep)
        + state
            .services
            .iter()
            .filter(|service| service.is_active)
            .map(|service| service.definition().global_service_power_modifier)
            .sum::<f32>();

    (definition.power_upkeep * (1.0 + modifier)).max(0.0)
}

fn effective_materials_output_multiplier(state: &RunState) -> f32 {
    1.0 + planet_modifier_total(state, PlanetModifierTarget::MaterialsOutput)
}

fn effective_data_output_multiplier(state: &RunState) -> f32 {
    survey_array_level(state).data_multiplier * (1.0 + planet_modifier_total(state, PlanetModifierTarget::DataOutput))
}

fn effective_survey_output_multiplier(state: &RunState, service_id: &str) -> f32 {
    let doctrine_multiplier = state
        .station
        .doctrine_ids
        .iter()
        .filter_map(|doctrine_id| doctrine_by_id(doctrine_id))
        .filter_map(|doctrine| match doctrine.effect {
            DoctrineEffect::SurveyProgressMultiplier {
                source_service_id,
                multiplier,
            } if source_service_id == service_id => Some(multiplier),
            _ => None,
        })
        .fold(1.0, |acc, multiplier| acc * multiplier);

    let service_multiplier = 1.0
        + state
            .services
            .iter()
            .filter(|service| service.is_active)
            .map(|service| service.definition().survey_speed_modifier)
            .sum::<f32>();

    survey_array_level(state).survey_multiplier * service_multiplier * doctrine_multiplier
}

fn planet_modifier_total(state: &RunState, target: PlanetModifierTarget) -> f32 {
    state
        .active_planet_definition()
        .modifiers
        .iter()
        .filter(|modifier| modifier.target == target)
        .map(|modifier| modifier.percent)
        .sum()
}

fn first_support_service_discount(state: &RunState) -> Option<(u8, u8)> {
    state.station
        .doctrine_ids
        .iter()
        .filter_map(|doctrine_id| doctrine_by_id(doctrine_id))
        .find_map(|doctrine| match doctrine.effect {
            DoctrineEffect::FirstSupportServiceCrewDiscount { reduction, minimum_crew } => {
                Some((reduction, minimum_crew))
            }
            _ => None,
        })
}

fn reactor_level(state: &RunState) -> ReactorCoreLevel {
    match system_by_id(REACTOR_CORE_ID)
        .expect("reactor-core system must exist")
        .progression
    {
        SystemProgression::ReactorCore(levels) => levels[(state.system_level(REACTOR_CORE_ID).unwrap_or(1) - 1) as usize],
        _ => unreachable!("reactor-core progression must be reactor levels"),
    }
}

fn habitat_level(state: &RunState) -> HabitatRingLevel {
    match system_by_id(HABITAT_RING_ID)
        .expect("habitat-ring system must exist")
        .progression
    {
        SystemProgression::HabitatRing(levels) => levels[(state.system_level(HABITAT_RING_ID).unwrap_or(1) - 1) as usize],
        _ => unreachable!("habitat-ring progression must be habitat levels"),
    }
}

fn logistics_level(state: &RunState) -> LogisticsSpineLevel {
    match system_by_id(LOGISTICS_SPINE_ID)
        .expect("logistics-spine system must exist")
        .progression
    {
        SystemProgression::LogisticsSpine(levels) => {
            levels[(state.system_level(LOGISTICS_SPINE_ID).unwrap_or(1) - 1) as usize]
        }
        _ => unreachable!("logistics-spine progression must be logistics levels"),
    }
}

fn survey_array_level(state: &RunState) -> SurveyArrayLevel {
    match system_by_id(SURVEY_ARRAY_ID)
        .expect("survey-array system must exist")
        .progression
    {
        SystemProgression::SurveyArray(levels) => levels[(state.system_level(SURVEY_ARRAY_ID).unwrap_or(1) - 1) as usize],
        _ => unreachable!("survey-array progression must be survey-array levels"),
    }
}

fn mark_paused(service: &mut crate::game::sim::state::ServiceState, reason: ServicePauseReason) {
    service.is_active = false;
    service.is_paused = true;
    service.pause_reason = Some(reason);
    service.assigned_crew = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::content::doctrines::HARDENED_RELAYS_ID;
    use crate::game::content::services::{
        FABRICATION_LOOP_ID, ORE_RECLAIMER_ID, SOLAR_HARVESTER_ID, SURVEY_UPLINK_ID,
    };

    fn advance_ticks(state: &mut RunState, ticks: usize) {
        for _ in 0..ticks {
            tick(state);
        }
    }

    fn deficit_fixture() -> RunState {
        let mut state = RunState::starter_fixture();
        state.station.active_planet_id = "solstice-anchor".to_string();
        state.station.discovered_planet_ids = vec!["solstice-anchor".to_string()];
        state.resources.crew_total = 10;
        state.resources.crew_assigned = 3;
        state.resources.crew_available = 7;
        state.systems = vec![
            crate::game::sim::state::SystemState::new(REACTOR_CORE_ID, 1),
            crate::game::sim::state::SystemState::new(HABITAT_RING_ID, 3),
            crate::game::sim::state::SystemState::new(LOGISTICS_SPINE_ID, 3),
            crate::game::sim::state::SystemState::new(SURVEY_ARRAY_ID, 1),
        ];
        for service in &mut state.services {
            service.desired_active = false;
            service.is_active = false;
        }

        state.service_state_mut(ORE_RECLAIMER_ID).unwrap().desired_active = true;
        state.service_state_mut(ORE_RECLAIMER_ID).unwrap().priority = 1;
        state.service_state_mut(SURVEY_UPLINK_ID).unwrap().desired_active = true;
        state.service_state_mut(SURVEY_UPLINK_ID).unwrap().priority = 2;
        state.service_state_mut(FABRICATION_LOOP_ID).unwrap().desired_active = true;
        state.service_state_mut(FABRICATION_LOOP_ID).unwrap().priority = 3;
        state.service_state_mut(SOLAR_HARVESTER_ID).unwrap().desired_active = false;

        state
    }

    #[test]
    fn simulation_core_starter_fixture_matches_expected_totals_after_40_ticks() {
        let mut state = RunState::starter_fixture();
        advance_ticks(&mut state, 40);

        assert_eq!(state.tick_count, 40);
        assert_eq!(state.resources.materials, 120.0);
        assert_eq!(state.resources.data, 0.0);
        assert_eq!(state.resources.power_generated, 8.0);
        assert_eq!(state.resources.power_reserved, 2.0);
        assert_eq!(state.resources.power_available, 10.0);
        assert_eq!(state.resources.crew_total, 6);
        assert_eq!(state.resources.crew_assigned, 2);
        assert_eq!(state.resources.crew_available, 4);
        assert_eq!(state.station.active_planet_id, "solstice-anchor");
        assert_eq!(state.station.survey_progress, 0.0);
    }

    #[test]
    fn simulation_core_autosave_cadence_triggers_every_60_ticks() {
        let mut state = RunState::starter_fixture();
        advance_ticks(&mut state, 240);

        assert_eq!(state.tick_count, 240);
        assert_eq!(state.autosave_count, 4);
        assert_eq!(state.last_autosave_tick, Some(240));
        assert!(state.autosave_due);
    }

    #[test]
    fn simulation_core_deterministic_2400_tick_run_matches_state_hash() {
        let mut left = RunState::starter_fixture();
        let mut right = RunState::starter_fixture();

        advance_ticks(&mut left, 2400);
        advance_ticks(&mut right, 2400);

        println!(
            "determinism: ticks={} hash={} materials={} data={} power_available={}",
            left.tick_count,
            left.state_hash(),
            left.resources.materials,
            left.resources.data,
            left.resources.power_available
        );

        assert_eq!(left.state_hash(), right.state_hash());
        assert_eq!(left.resources.materials, 120.0);
        assert_eq!(left.resources.data, 0.0);
        assert_eq!(left.resources.power_available, 10.0);
        assert_eq!(left.autosave_count, 40);
        assert_eq!(left.last_autosave_tick, Some(2400));
    }

    #[test]
    fn simulation_core_power_deficit_shuts_down_lower_priority_services_first() {
        let mut state = deficit_fixture();
        tick(&mut state);

        println!(
            "deficit-order: ore_active={} survey_active={} fabrication_active={} power_available={} materials={} data={}",
            state.service_state(ORE_RECLAIMER_ID).unwrap().is_active,
            state.service_state(SURVEY_UPLINK_ID).unwrap().is_active,
            state.service_state(FABRICATION_LOOP_ID).unwrap().is_active,
            state.resources.power_available,
            state.resources.materials,
            state.resources.data
        );

        assert!(state.service_state(ORE_RECLAIMER_ID).unwrap().is_active);
        assert!(state.service_state(SURVEY_UPLINK_ID).unwrap().is_active);
        assert!(!state.service_state(FABRICATION_LOOP_ID).unwrap().is_active);
        assert!(state.service_state(FABRICATION_LOOP_ID).unwrap().is_paused);
        assert_eq!(
            state.service_state(FABRICATION_LOOP_ID).unwrap().pause_reason,
            Some(ServicePauseReason::Deficit)
        );
        assert!(state.resources.power_available >= 0.0);
    }

    #[test]
    fn simulation_core_paused_services_produce_no_output() {
        let mut state = deficit_fixture();
        tick(&mut state);

        assert_eq!(state.resources.materials, 120.5);
        assert!((state.resources.data - 0.3375).abs() < 0.000_001);
        assert_eq!(state.station.survey_progress, 0.25);
    }

    #[test]
    fn simulation_core_hardened_relays_refund_is_applied_to_same_tick_power_pool() {
        let mut without_relays = deficit_fixture();
        let mut with_relays = deficit_fixture();
        with_relays
            .station
            .doctrine_ids
            .push(HARDENED_RELAYS_ID.to_string());

        tick(&mut without_relays);
        tick(&mut with_relays);

        let fabrication_upkeep = 2.0;
        let expected_refund = fabrication_upkeep * 0.50;

        assert_eq!(
            with_relays.resources.power_available,
            without_relays.resources.power_available + expected_refund
        );
    }
}
