//! Runtime projections derived from the live [`RunState`].
//!
//! Functions in this module compute values that depend on the *current* shape
//! of the game world but are not themselves persisted: power totals, crew
//! capacity, active service slot counts, and projected post-toggle power.
//! Every helper here is read-mostly — the sole exception is
//! [`refresh_runtime_state`], which writes back into the cached resource and
//! prestige fields of [`RunState`] so the rest of the simulation can read them
//! without recomputing.
//!
//! These helpers are invoked from the tick loop in [`fn@crate::game::sim::tick`]
//! and from individual Tauri commands that need an authoritative
//! "what-would-happen-if" projection (e.g. service-toggle UI hints).
//!
//! See also: [`crate::game::sim`], [`crate::game::content`].

use crate::game::content::planets::PlanetModifierTarget;
use crate::game::content::services::service_by_id_required;
use crate::game::content::systems::{
    system_by_id_required, SystemProgression, HABITAT_RING_ID, REACTOR_CORE_ID,
};
use crate::game::sim::{effective_service_power_upkeep, RunState};

/// Recomputes derived runtime fields on `run_state` in place.
///
/// Updates the cached crew totals (clamped against habitat capacity), reactor
/// power output, reserved/available power budget, station tier, and prestige
/// eligibility flag based on the latest content data and progression rules.
/// Call this whenever a mutation could invalidate a derived field — service
/// activation, system upgrades, crew assignment, doctrine purchases, etc. —
/// before the next snapshot is built.
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

/// Returns the number of service slots unlocked by the current logistics-spine level.
///
/// Indexes into the `LogisticsSpine` progression table for the system's current
/// level (clamped to the table bounds, defaulting to level 1 if the system is
/// missing) and returns its `active_service_slots` entry.
///
/// # Panics
///
/// Panics via `unreachable!` if the `logistics-spine` system definition is not
/// configured with a `LogisticsSpine` progression — that would indicate a
/// content-data bug, not a runtime condition.
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

/// Returns the maximum crew the station can house at the current habitat level.
///
/// Reads the base capacity from the habitat-ring progression table, then
/// applies the sum of all active planet modifiers targeting
/// [`PlanetModifierTarget::CrewCapacity`] (treated as a percent multiplier).
/// The result is floored and clamped to a minimum of 1.
///
/// # Panics
///
/// Panics via `unreachable!` if the `habitat-ring` system definition is not
/// configured with a `HabitatRing` progression.
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

    ((base_capacity as f32) * (1.0 + planet_modifier))
        .floor()
        .max(1.0) as u8
}

/// Returns the reactor's raw power generation at its current level.
///
/// Indexes into the reactor-core progression table for the system's current
/// level (clamped to the table bounds, defaulting to level 1 if missing) and
/// returns its `power_output`. Does not subtract upkeep — see
/// [`projected_power_after_toggle`] or [`refresh_runtime_state`] for net values.
///
/// # Panics
///
/// Panics via `unreachable!` if the `reactor-core` system definition is not
/// configured with a `ReactorCore` progression.
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

/// Sums the `power_output` of every currently active service.
///
/// Only services with `is_active == true` contribute. Iterates the run-state's
/// service list and looks each up by id in the static service catalog.
pub(crate) fn active_service_power_output(run_state: &RunState) -> f32 {
    run_state
        .services
        .iter()
        .filter(|service| service.is_active)
        .map(|service| service_by_id_required(&service.service_id).power_output)
        .sum()
}

/// Computes the net power balance the station would have if `service_id`'s
/// activation flipped from `currently_active` to its opposite.
///
/// Used by service-toggle UI/command paths to show "available power if you
/// flip this service" without actually mutating run-state. Combines the upkeep
/// delta (subtract if turning off, add if turning on), the output delta (same
/// sign convention applied to `power_output`), the existing reserved budget,
/// the reactor's current output, and the active-service power output.
///
/// # Panics
///
/// Panics via the lookup helpers if `service_id` is not present in the static
/// service catalog or any underlying system definition is misconfigured.
pub(crate) fn projected_power_after_toggle(
    run_state: &RunState,
    service_id: &str,
    currently_active: bool,
) -> f32 {
    let upkeep = effective_service_power_upkeep(run_state, service_id);
    let definition = service_by_id_required(service_id);
    let upkeep_delta = if currently_active { -upkeep } else { upkeep };
    let output_delta = if currently_active {
        -definition.power_output
    } else {
        definition.power_output
    };
    let new_reserved = run_state.resources.power_reserved + upkeep_delta;
    reactor_power_output(run_state) - new_reserved
        + active_service_power_output(run_state)
        + output_delta
}
