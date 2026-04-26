#![allow(dead_code)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::game::content::doctrines::HARDENED_RELAYS_ID;
use crate::game::content::planets::{planet_by_id, SOLSTICE_ANCHOR_ID};
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

#[derive(Debug, Clone, PartialEq)]
pub struct RunState {
    pub tick_count: u64,
    pub station: StationState,
    pub resources: ResourceState,
    pub services: Vec<ServiceState>,
    pub systems: Vec<SystemState>,
    pub autosave_due: bool,
    pub autosave_count: u32,
    pub last_autosave_tick: Option<u64>,
    pub prestige_eligible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StationState {
    pub active_planet_id: String,
    pub discovered_planet_ids: Vec<String>,
    pub doctrine_ids: Vec<String>,
    pub doctrine_fragments: u32,
    pub survey_progress: f32,
    pub station_tier: u8,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceState {
    pub service_id: String,
    pub desired_active: bool,
    pub is_active: bool,
    pub is_paused: bool,
    pub pause_reason: Option<ServicePauseReason>,
    pub priority: u8,
    pub assigned_crew: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
            autosave_due: false,
            autosave_count: 0,
            last_autosave_tick: None,
            prestige_eligible: false,
        }
    }

    pub fn active_planet_definition(&self) -> &'static crate::game::content::PlanetDefinition {
        planet_by_id(&self.station.active_planet_id).expect("active planet must exist in catalog")
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

        self.tick_count.hash(&mut hasher);
        self.autosave_due.hash(&mut hasher);
        self.autosave_count.hash(&mut hasher);
        self.last_autosave_tick.hash(&mut hasher);
        self.prestige_eligible.hash(&mut hasher);

        self.station.active_planet_id.hash(&mut hasher);
        sorted_clone(&self.station.discovered_planet_ids).hash(&mut hasher);
        sorted_clone(&self.station.doctrine_ids).hash(&mut hasher);
        self.station.doctrine_fragments.hash(&mut hasher);
        self.station.station_tier.hash(&mut hasher);
        self.station.survey_progress.to_bits().hash(&mut hasher);

        self.resources.power_generated.to_bits().hash(&mut hasher);
        self.resources.power_reserved.to_bits().hash(&mut hasher);
        self.resources.power_available.to_bits().hash(&mut hasher);
        self.resources.materials.to_bits().hash(&mut hasher);
        self.resources.data.to_bits().hash(&mut hasher);
        self.resources.crew_total.hash(&mut hasher);
        self.resources.crew_assigned.hash(&mut hasher);
        self.resources.crew_available.hash(&mut hasher);

        for system in &self.systems {
            system.system_id.hash(&mut hasher);
            system.level.hash(&mut hasher);
        }

        for service in &self.services {
            service.service_id.hash(&mut hasher);
            service.desired_active.hash(&mut hasher);
            service.is_active.hash(&mut hasher);
            service.is_paused.hash(&mut hasher);
            service.pause_reason.hash(&mut hasher);
            service.priority.hash(&mut hasher);
            service.assigned_crew.hash(&mut hasher);
        }

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
}

impl SystemState {
    pub fn new(system_id: &str, level: u8) -> Self {
        Self {
            system_id: system_id.to_string(),
            level,
        }
    }
}

fn sorted_clone(values: &[String]) -> Vec<String> {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted
}

pub fn catalog_service_order(service_id: &str) -> usize {
    SERVICES
        .iter()
        .position(|service| service.id == service_id)
        .expect("service must exist in canonical catalog order")
}
