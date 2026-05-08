#![allow(dead_code)]

//! Mutable run-state model advanced by the simulation tick.
//!
//! The structs in this module are the authoritative in-memory game state. The
//! tick loop mutates them in place, command handlers inspect or adjust them, and
//! `game::snapshot` projects them into frontend-facing DTOs without owning
//! simulation rules.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use idle_spacestation_bit_eq_derive::BitHash;

use crate::game::bit_eq::BitHash as _;
use crate::game::content::doctrines::HARDENED_RELAYS_ID;
use crate::game::content::planets::{planet_by_id_required, SOLSTICE_ANCHOR_ID};
use crate::game::content::services::{
    ServiceDefinition, COMMAND_RELAY_ID, FABRICATION_LOOP_ID, MAINTENANCE_BAY_ID, ORE_RECLAIMER_ID,
    SERVICES, SOLAR_HARVESTER_ID, SURVEY_UPLINK_ID,
};
use crate::game::content::systems::{
    HABITAT_RING_ID, LOGISTICS_SPINE_ID, REACTOR_CORE_ID, SURVEY_ARRAY_ID,
};

/// Number of simulated seconds represented by one backend tick.
pub const SECONDS_PER_TICK: f32 = 0.25;
/// Baseline power reserved every second before any service upkeep is counted.
pub const HOUSEKEEPING_POWER_PER_SECOND: f32 = 2.0;
/// Tick interval for flagging autosave work on the run state.
pub const AUTOSAVE_CADENCE_TICKS: u64 = 60;
/// Survey progress required to discover Cinder Forge.
pub const CINDER_FORGE_SURVEY_THRESHOLD: f32 = 600.0;
/// Survey progress required to discover Aurora Pier.
pub const AURORA_PIER_SURVEY_THRESHOLD: f32 = 1400.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Gameplay reason a desired service could not remain active this tick.
pub enum ServicePauseReason {
    /// The Logistics Spine active-service slot cap was already filled.
    Capacity,
    /// The Habitat Ring crew pool could not staff the service.
    Crew,
    /// Power-deficit recovery shed the service after production planning.
    Deficit,
    /// Reactor service-power cap would be exceeded by the service upkeep.
    PowerCap,
}

impl crate::game::bit_eq::BitHash for ServicePauseReason {
    fn bit_hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}

impl ServicePauseReason {
    /// Stable lowercase code serialized into snapshots and rejection metadata.
    pub(crate) fn code(&self) -> &'static str {
        match self {
            ServicePauseReason::Capacity => "capacity",
            ServicePauseReason::Crew => "crew",
            ServicePauseReason::Deficit => "deficit",
            ServicePauseReason::PowerCap => "power-cap",
        }
    }
}

#[derive(BitHash, Debug, Clone, PartialEq)]
/// Authoritative mutable state for one active station run.
///
/// `RunState` is mutated in place by the six-phase tick loop and then projected
/// by `game::snapshot` into frontend DTOs. Field order for `BitHash` is explicit
/// so deterministic replay and push-event diffing remain stable across source
/// edits.
pub struct RunState {
    #[bit_hash(order = 0)]
    /// Number of completed simulation ticks in this run.
    pub tick_count: u64,
    #[bit_hash(order = 7)]
    /// Planet, doctrine, survey, and station-tier progression for the run.
    pub station: StationState,
    #[bit_hash(order = 13)]
    /// Current resource pools and derived per-tick power/crew accounting.
    pub resources: ResourceState,
    #[bit_hash(order = 20)]
    /// Service toggles, priorities, runtime activity, and crew assignments.
    pub services: Vec<ServiceState>,
    #[bit_hash(order = 19)]
    /// Installed station systems and their upgrade levels.
    pub systems: Vec<SystemState>,
    #[bit_hash(order = 5)]
    /// Count of consecutive ticks with non-negative net power for prestige checks.
    pub consecutive_stable_power_ticks: u32,
    #[bit_hash(order = 6)]
    /// Lifetime whole-data units produced during this run.
    pub lifetime_data_produced: u64,
    #[bit_hash(order = 1)]
    /// Whether the autosave cadence landed on the current tick.
    pub autosave_due: bool,
    #[bit_hash(order = 2)]
    /// Number of autosave cadence hits observed during this run.
    pub autosave_count: u32,
    #[bit_hash(order = 3)]
    /// Tick count at which the most recent autosave cadence hit occurred.
    pub last_autosave_tick: Option<u64>,
    #[bit_hash(order = 4)]
    /// Cached prestige availability after the latest tick stability evaluation.
    pub prestige_eligible: bool,
}

#[derive(BitHash, Debug, Clone, PartialEq)]
/// Station-level progression that is not tied to individual resources or services.
pub struct StationState {
    #[bit_hash(order = 0)]
    /// Planet whose modifiers currently affect the run.
    pub active_planet_id: String,
    #[bit_hash(order = 1, sort)]
    /// Planets discovered through accumulated survey progress.
    pub discovered_planet_ids: Vec<String>,
    #[bit_hash(order = 2, sort)]
    /// Doctrine identifiers unlocked for this run.
    pub doctrine_ids: Vec<String>,
    #[bit_hash(order = 3)]
    /// Spendable prestige currency available during the run.
    pub doctrine_fragments: u32,
    #[bit_hash(order = 5)]
    /// Accumulated survey progress toward planet discovery thresholds.
    pub survey_progress: f32,
    #[bit_hash(order = 4)]
    /// Derived station tier computed from system levels.
    pub station_tier: u8,
}

#[derive(BitHash, Debug, Clone, PartialEq)]
/// Resource pools and derived allocation totals for the current tick.
pub struct ResourceState {
    /// Power produced by the reactor baseline before service output.
    pub power_generated: f32,
    /// Power reserved by housekeeping and active-service upkeep.
    pub power_reserved: f32,
    /// Net power remaining after reservation and service output.
    pub power_available: f32,
    /// Materials stockpile clamped by Logistics Spine capacity.
    pub materials: f32,
    /// Research data accumulated by active services.
    pub data: f32,
    /// Total crew population currently available to the station.
    pub crew_total: u8,
    /// Crew assigned to services after activation and deficit recovery.
    pub crew_assigned: u8,
    /// Crew not assigned to services after activation and deficit recovery.
    pub crew_available: u8,
}

#[derive(BitHash, Debug, Clone, PartialEq)]
/// Runtime state for a catalog service within a run.
pub struct ServiceState {
    /// Catalog identifier for the service definition.
    pub service_id: String,
    /// Player intent to run the service when capacity allows.
    pub desired_active: bool,
    /// Whether the service survived this tick's activation and deficit phases.
    pub is_active: bool,
    /// Whether a desired service is paused by a concrete pause reason.
    pub is_paused: bool,
    /// Most recent reason the service could not remain active.
    pub pause_reason: Option<ServicePauseReason>,
    /// Lower values activate first and shed later during deficit recovery.
    pub priority: u8,
    /// Crew committed to this service while active.
    pub assigned_crew: u8,
}

#[derive(BitHash, Debug, Clone, PartialEq, Eq)]
/// Upgrade state for one station system.
pub struct SystemState {
    /// Catalog identifier for the system definition.
    pub system_id: String,
    /// Current upgrade level for the system.
    pub level: u8,
}

/// Backward-compatible alias for the active run-state type.
pub type SimState = RunState;

impl RunState {
    /// Builds the deterministic starter run used by a new game and tests.
    ///
    /// Seeds Solstice Anchor, baseline resources, starter systems, and the
    /// canonical service list so tick output and snapshot ordering are stable.
    pub fn starter_fixture() -> Self {
        Self {
            tick_count: 0,
            station: StationState {
                active_planet_id: SOLSTICE_ANCHOR_ID.to_string(),
                discovered_planet_ids: vec![SOLSTICE_ANCHOR_ID.to_string()],
                doctrine_ids: Vec::new(),
                doctrine_fragments: 0,
                survey_progress: 0.0,
                station_tier: 1,
            },
            resources: ResourceState {
                power_generated: 8.0,
                power_reserved: HOUSEKEEPING_POWER_PER_SECOND,
                power_available: 6.0,
                materials: 120.0,
                data: 0.0,
                crew_total: 6,
                crew_assigned: 2,
                crew_available: 4,
            },
            services: vec![
                ServiceState::new(SOLAR_HARVESTER_ID, true, 1),
                ServiceState::new(ORE_RECLAIMER_ID, false, 2),
                ServiceState::new(SURVEY_UPLINK_ID, false, 3),
                ServiceState::new(MAINTENANCE_BAY_ID, false, 4),
                ServiceState::new(COMMAND_RELAY_ID, false, 5),
                ServiceState::new(FABRICATION_LOOP_ID, false, 6),
            ],
            systems: vec![
                SystemState::new(REACTOR_CORE_ID, 1),
                SystemState::new(HABITAT_RING_ID, 1),
                SystemState::new(LOGISTICS_SPINE_ID, 1),
                SystemState::new(SURVEY_ARRAY_ID, 1),
            ],
            consecutive_stable_power_ticks: 0,
            lifetime_data_produced: 0,
            autosave_due: false,
            autosave_count: 0,
            last_autosave_tick: None,
            prestige_eligible: false,
        }
    }

    /// Returns the catalog definition for the active planet.
    pub fn active_planet_definition(&self) -> &'static crate::game::content::PlanetDefinition {
        planet_by_id_required(&self.station.active_planet_id)
    }

    /// Returns whether the run currently has the given doctrine id.
    pub fn has_doctrine(&self, doctrine_id: &str) -> bool {
        self.station
            .doctrine_ids
            .iter()
            .any(|candidate| candidate == doctrine_id)
    }

    /// Returns the same-tick power-upkeep refund ratio from Hardened Relays.
    pub fn hardened_relays_refund_ratio(&self) -> f32 {
        if self.has_doctrine(HARDENED_RELAYS_ID) {
            0.5
        } else {
            0.0
        }
    }

    /// Looks up the current upgrade level for a station system.
    pub fn system_level(&self, system_id: &str) -> Option<u8> {
        self.systems
            .iter()
            .find(|system| system.system_id == system_id)
            .map(|system| system.level)
    }

    /// Returns immutable runtime state for a service id.
    pub fn service_state(&self, service_id: &str) -> Option<&ServiceState> {
        self.services
            .iter()
            .find(|service| service.service_id == service_id)
    }

    /// Returns mutable runtime state for a service id.
    pub fn service_state_mut(&mut self, service_id: &str) -> Option<&mut ServiceState> {
        self.services
            .iter_mut()
            .find(|service| service.service_id == service_id)
    }

    /// Returns service indices in activation order.
    ///
    /// Services sort by player priority and then by canonical catalog order to
    /// keep tick behavior deterministic when priorities tie.
    pub fn ordered_service_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.services.len()).collect();
        indices.sort_by_key(|&index| {
            (
                self.services[index].priority,
                catalog_service_order(&self.services[index].service_id),
            )
        });
        indices
    }

    /// Produces a deterministic bit-level hash of the current run state.
    pub fn state_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.bit_hash(&mut hasher);
        hasher.finish()
    }

    /// Recomputes the derived station tier from system upgrade levels.
    ///
    /// The allocation phase calls this before snapshots and prestige checks so
    /// route projections see a tier consistent with the latest system state.
    pub fn recalculate_station_tier(&mut self) {
        let tier = self
            .systems
            .iter()
            .map(|system| system.level as i16)
            .sum::<i16>()
            - 3;
        self.station.station_tier = tier.clamp(1, 4) as u8;
    }
}

impl StationState {
    /// Returns whether a planet id is present in the discovered list.
    pub fn has_discovered(&self, planet_id: &str) -> bool {
        self.discovered_planet_ids.iter().any(|id| id == planet_id)
    }
}

impl ServiceState {
    /// Creates runtime state for a catalog service with a desired toggle and priority.
    pub fn new(service_id: &str, desired_active: bool, priority: u8) -> Self {
        Self {
            service_id: service_id.to_string(),
            desired_active,
            is_active: desired_active,
            is_paused: false,
            pause_reason: None,
            priority,
            assigned_crew: 0,
        }
    }

    /// Returns the catalog definition referenced by this service state.
    pub fn definition(&self) -> &'static ServiceDefinition {
        SERVICES
            .iter()
            .find(|service| service.id == self.service_id)
            .expect("service state must reference catalog service")
    }

    /// Marks the service as desired but paused for the given reason.
    pub fn pause_with(&mut self, reason: ServicePauseReason) {
        self.desired_active = true;
        self.is_active = false;
        self.is_paused = true;
        self.pause_reason = Some(reason);
    }

    /// Clears player intent and all runtime activation state for the service.
    pub fn deactivate(&mut self) {
        self.desired_active = false;
        self.is_active = false;
        self.is_paused = false;
        self.pause_reason = None;
        self.assigned_crew = 0;
    }

    /// Marks the service as desired and active with an assigned crew count.
    pub fn activate(&mut self, assigned_crew: u8) {
        self.desired_active = true;
        self.is_active = true;
        self.is_paused = false;
        self.pause_reason = None;
        self.assigned_crew = assigned_crew;
    }
}

impl SystemState {
    /// Creates upgrade state for a station system.
    pub fn new(system_id: &str, level: u8) -> Self {
        Self {
            system_id: system_id.to_string(),
            level,
        }
    }
}

/// Returns the canonical catalog position for stable service tie-breaking.
pub fn catalog_service_order(service_id: &str) -> usize {
    SERVICES
        .iter()
        .position(|service| service.id == service_id)
        .expect("service must exist in canonical catalog order")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::content::planets::{AURORA_PIER_ID, CINDER_FORGE_ID};

    #[test]
    fn station_has_discovered_returns_true_for_starter_planet() {
        let run = RunState::starter_fixture();
        assert!(run.station.has_discovered(SOLSTICE_ANCHOR_ID));
    }

    #[test]
    fn station_has_discovered_returns_false_for_undiscovered() {
        let run = RunState::starter_fixture();
        assert!(!run.station.has_discovered(CINDER_FORGE_ID));
        assert!(!run.station.has_discovered(AURORA_PIER_ID));
    }

    #[test]
    fn station_has_discovered_returns_true_when_discovered() {
        let mut run = RunState::starter_fixture();
        run.station
            .discovered_planet_ids
            .push(CINDER_FORGE_ID.to_string());
        assert!(run.station.has_discovered(CINDER_FORGE_ID));
    }
}
