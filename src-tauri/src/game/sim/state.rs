#![allow(dead_code)]

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

pub const SECONDS_PER_TICK: f32 = 0.25;
pub const HOUSEKEEPING_POWER_PER_SECOND: f32 = 2.0;
pub const AUTOSAVE_CADENCE_TICKS: u64 = 60;
pub const CINDER_FORGE_SURVEY_THRESHOLD: f32 = 600.0;
pub const AURORA_PIER_SURVEY_THRESHOLD: f32 = 1400.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServicePauseReason {
    Capacity,
    Crew,
    Deficit,
    PowerCap,
}

impl crate::game::bit_eq::BitHash for ServicePauseReason {
    fn bit_hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}

impl ServicePauseReason {
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
pub struct RunState {
    #[bit_hash(order = 0)]
    pub tick_count: u64,
    #[bit_hash(order = 7)]
    pub station: StationState,
    #[bit_hash(order = 13)]
    pub resources: ResourceState,
    #[bit_hash(order = 20)]
    pub services: Vec<ServiceState>,
    #[bit_hash(order = 19)]
    pub systems: Vec<SystemState>,
    #[bit_hash(order = 5)]
    pub consecutive_stable_power_ticks: u32,
    #[bit_hash(order = 6)]
    pub lifetime_data_produced: u64,
    #[bit_hash(order = 1)]
    pub autosave_due: bool,
    #[bit_hash(order = 2)]
    pub autosave_count: u32,
    #[bit_hash(order = 3)]
    pub last_autosave_tick: Option<u64>,
    #[bit_hash(order = 4)]
    pub prestige_eligible: bool,
}

#[derive(BitHash, Debug, Clone, PartialEq)]
pub struct StationState {
    #[bit_hash(order = 0)]
    pub active_planet_id: String,
    #[bit_hash(order = 1, sort)]
    pub discovered_planet_ids: Vec<String>,
    #[bit_hash(order = 2, sort)]
    pub doctrine_ids: Vec<String>,
    #[bit_hash(order = 3)]
    pub doctrine_fragments: u32,
    #[bit_hash(order = 5)]
    pub survey_progress: f32,
    #[bit_hash(order = 4)]
    pub station_tier: u8,
}

#[derive(BitHash, Debug, Clone, PartialEq)]
pub struct ResourceState {
    pub power_generated: f32,
    pub power_reserved: f32,
    pub power_available: f32,
    pub materials: f32,
    pub data: f32,
    pub crew_total: u8,
    pub crew_assigned: u8,
    pub crew_available: u8,
}

#[derive(BitHash, Debug, Clone, PartialEq)]
pub struct ServiceState {
    pub service_id: String,
    pub desired_active: bool,
    pub is_active: bool,
    pub is_paused: bool,
    pub pause_reason: Option<ServicePauseReason>,
    pub priority: u8,
    pub assigned_crew: u8,
}

#[derive(BitHash, Debug, Clone, PartialEq, Eq)]
pub struct SystemState {
    pub system_id: String,
    pub level: u8,
}

pub type SimState = RunState;

impl RunState {
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

    pub fn active_planet_definition(&self) -> &'static crate::game::content::PlanetDefinition {
        planet_by_id_required(&self.station.active_planet_id)
    }

    pub fn has_doctrine(&self, doctrine_id: &str) -> bool {
        self.station
            .doctrine_ids
            .iter()
            .any(|candidate| candidate == doctrine_id)
    }

    pub fn hardened_relays_refund_ratio(&self) -> f32 {
        if self.has_doctrine(HARDENED_RELAYS_ID) {
            0.5
        } else {
            0.0
        }
    }

    pub fn system_level(&self, system_id: &str) -> Option<u8> {
        self.systems
            .iter()
            .find(|system| system.system_id == system_id)
            .map(|system| system.level)
    }

    pub fn service_state(&self, service_id: &str) -> Option<&ServiceState> {
        self.services
            .iter()
            .find(|service| service.service_id == service_id)
    }

    pub fn service_state_mut(&mut self, service_id: &str) -> Option<&mut ServiceState> {
        self.services
            .iter_mut()
            .find(|service| service.service_id == service_id)
    }

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

    pub fn state_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.bit_hash(&mut hasher);
        hasher.finish()
    }

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
    pub fn has_discovered(&self, planet_id: &str) -> bool {
        self.discovered_planet_ids.iter().any(|id| id == planet_id)
    }
}

impl ServiceState {
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

    pub fn definition(&self) -> &'static ServiceDefinition {
        SERVICES
            .iter()
            .find(|service| service.id == self.service_id)
            .expect("service state must reference catalog service")
    }

    pub fn pause_with(&mut self, reason: ServicePauseReason) {
        self.desired_active = true;
        self.is_active = false;
        self.is_paused = true;
        self.pause_reason = Some(reason);
    }

    pub fn deactivate(&mut self) {
        self.desired_active = false;
        self.is_active = false;
        self.is_paused = false;
        self.pause_reason = None;
        self.assigned_crew = 0;
    }

    pub fn activate(&mut self, assigned_crew: u8) {
        self.desired_active = true;
        self.is_active = true;
        self.is_paused = false;
        self.pause_reason = None;
        self.assigned_crew = assigned_crew;
    }
}

impl SystemState {
    pub fn new(system_id: &str, level: u8) -> Self {
        Self {
            system_id: system_id.to_string(),
            level,
        }
    }
}

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
