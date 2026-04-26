use serde::{Deserialize, Serialize};

use crate::game::sim::{RunState, ServicePauseReason};

pub const SAVE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileState {
    pub discovered_planet_ids: Vec<String>,
    pub doctrine_ids: Vec<String>,
    pub doctrine_fragments: u32,
    pub lifetime_ticks: u64,
    pub lifetime_prestiges: u32,
}

impl Default for ProfileState {
    fn default() -> Self {
        Self {
            discovered_planet_ids: Vec::new(),
            doctrine_ids: Vec::new(),
            doctrine_fragments: 0,
            lifetime_ticks: 0,
            lifetime_prestiges: 0,
        }
    }
}

impl ProfileState {
    pub fn from_run_state(run_state: &RunState) -> Self {
        let mut profile = Self::default();
        profile.sync_from_run_state(run_state);
        profile
    }

    pub fn sync_from_run_state(&mut self, run_state: &RunState) {
        self.discovered_planet_ids = sorted_unique(run_state.station.discovered_planet_ids.clone());
        self.doctrine_ids = sorted_unique(run_state.station.doctrine_ids.clone());
        self.doctrine_fragments = run_state.station.doctrine_fragments;
        self.lifetime_ticks = self.lifetime_ticks.max(run_state.tick_count);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSettings {
    pub autosave_enabled: bool,
}

impl Default for SaveSettings {
    fn default() -> Self {
        Self {
            autosave_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveData {
    pub save_version: u32,
    pub run_state: RunStateSnapshot,
    pub profile_state: ProfileState,
    pub settings: SaveSettings,
}

impl SaveData {
    pub fn from_runtime(
        run_state: &RunState,
        profile_state: &ProfileState,
        settings: &SaveSettings,
    ) -> Self {
        let mut synced_profile = profile_state.clone();
        synced_profile.sync_from_run_state(run_state);

        Self {
            save_version: SAVE_VERSION,
            run_state: RunStateSnapshot::from(run_state),
            profile_state: synced_profile,
            settings: settings.clone(),
        }
    }

    pub fn into_runtime(self) -> (RunState, ProfileState, SaveSettings) {
        (self.run_state.into(), self.profile_state, self.settings)
    }

    pub fn fresh() -> Self {
        let run_state = RunState::starter_fixture();
        let profile_state = ProfileState::from_run_state(&run_state);
        Self::from_runtime(&run_state, &profile_state, &SaveSettings::default())
    }
}

pub fn serialize_save(data: &SaveData) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(data)
}

pub(crate) fn deserialize_v1(data: &str) -> Result<SaveData, serde_json::Error> {
    serde_json::from_str(data)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStateSnapshot {
    pub tick_count: u64,
    pub station: StationStateSnapshot,
    pub resources: ResourceStateSnapshot,
    pub services: Vec<ServiceStateSnapshot>,
    pub systems: Vec<SystemStateSnapshot>,
    pub autosave_due: bool,
    pub autosave_count: u32,
    pub last_autosave_tick: Option<u64>,
    pub prestige_eligible: bool,
}

impl From<&RunState> for RunStateSnapshot {
    fn from(value: &RunState) -> Self {
        Self {
            tick_count: value.tick_count,
            station: StationStateSnapshot::from(&value.station),
            resources: ResourceStateSnapshot::from(&value.resources),
            services: value.services.iter().map(ServiceStateSnapshot::from).collect(),
            systems: value.systems.iter().map(SystemStateSnapshot::from).collect(),
            autosave_due: value.autosave_due,
            autosave_count: value.autosave_count,
            last_autosave_tick: value.last_autosave_tick,
            prestige_eligible: value.prestige_eligible,
        }
    }
}

impl From<RunStateSnapshot> for RunState {
    fn from(value: RunStateSnapshot) -> Self {
        Self {
            tick_count: value.tick_count,
            station: value.station.into(),
            resources: value.resources.into(),
            services: value.services.into_iter().map(Into::into).collect(),
            systems: value.systems.into_iter().map(Into::into).collect(),
            autosave_due: value.autosave_due,
            autosave_count: value.autosave_count,
            last_autosave_tick: value.last_autosave_tick,
            prestige_eligible: value.prestige_eligible,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationStateSnapshot {
    pub active_planet_id: String,
    pub discovered_planet_ids: Vec<String>,
    pub doctrine_ids: Vec<String>,
    pub doctrine_fragments: u32,
    pub survey_progress: f32,
    pub station_tier: u8,
}

impl From<&crate::game::sim::StationState> for StationStateSnapshot {
    fn from(value: &crate::game::sim::StationState) -> Self {
        Self {
            active_planet_id: value.active_planet_id.clone(),
            discovered_planet_ids: value.discovered_planet_ids.clone(),
            doctrine_ids: value.doctrine_ids.clone(),
            doctrine_fragments: value.doctrine_fragments,
            survey_progress: value.survey_progress,
            station_tier: value.station_tier,
        }
    }
}

impl From<StationStateSnapshot> for crate::game::sim::StationState {
    fn from(value: StationStateSnapshot) -> Self {
        Self {
            active_planet_id: value.active_planet_id,
            discovered_planet_ids: value.discovered_planet_ids,
            doctrine_ids: value.doctrine_ids,
            doctrine_fragments: value.doctrine_fragments,
            survey_progress: value.survey_progress,
            station_tier: value.station_tier,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceStateSnapshot {
    pub power_generated: f32,
    pub power_reserved: f32,
    pub power_available: f32,
    pub materials: f32,
    pub data: f32,
    pub crew_total: u8,
    pub crew_assigned: u8,
    pub crew_available: u8,
}

impl From<&crate::game::sim::ResourceState> for ResourceStateSnapshot {
    fn from(value: &crate::game::sim::ResourceState) -> Self {
        Self {
            power_generated: value.power_generated,
            power_reserved: value.power_reserved,
            power_available: value.power_available,
            materials: value.materials,
            data: value.data,
            crew_total: value.crew_total,
            crew_assigned: value.crew_assigned,
            crew_available: value.crew_available,
        }
    }
}

impl From<ResourceStateSnapshot> for crate::game::sim::ResourceState {
    fn from(value: ResourceStateSnapshot) -> Self {
        Self {
            power_generated: value.power_generated,
            power_reserved: value.power_reserved,
            power_available: value.power_available,
            materials: value.materials,
            data: value.data,
            crew_total: value.crew_total,
            crew_assigned: value.crew_assigned,
            crew_available: value.crew_available,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStateSnapshot {
    pub service_id: String,
    pub desired_active: bool,
    pub is_active: bool,
    pub is_paused: bool,
    pub pause_reason: Option<ServicePauseReasonSnapshot>,
    pub priority: u8,
    pub assigned_crew: u8,
}

impl From<&crate::game::sim::ServiceState> for ServiceStateSnapshot {
    fn from(value: &crate::game::sim::ServiceState) -> Self {
        Self {
            service_id: value.service_id.clone(),
            desired_active: value.desired_active,
            is_active: value.is_active,
            is_paused: value.is_paused,
            pause_reason: value.pause_reason.map(Into::into),
            priority: value.priority,
            assigned_crew: value.assigned_crew,
        }
    }
}

impl From<ServiceStateSnapshot> for crate::game::sim::ServiceState {
    fn from(value: ServiceStateSnapshot) -> Self {
        Self {
            service_id: value.service_id,
            desired_active: value.desired_active,
            is_active: value.is_active,
            is_paused: value.is_paused,
            pause_reason: value.pause_reason.map(Into::into),
            priority: value.priority,
            assigned_crew: value.assigned_crew,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServicePauseReasonSnapshot {
    Capacity,
    Crew,
    Deficit,
    PowerCap,
}

impl From<ServicePauseReason> for ServicePauseReasonSnapshot {
    fn from(value: ServicePauseReason) -> Self {
        match value {
            ServicePauseReason::Capacity => Self::Capacity,
            ServicePauseReason::Crew => Self::Crew,
            ServicePauseReason::Deficit => Self::Deficit,
            ServicePauseReason::PowerCap => Self::PowerCap,
        }
    }
}

impl From<ServicePauseReasonSnapshot> for ServicePauseReason {
    fn from(value: ServicePauseReasonSnapshot) -> Self {
        match value {
            ServicePauseReasonSnapshot::Capacity => Self::Capacity,
            ServicePauseReasonSnapshot::Crew => Self::Crew,
            ServicePauseReasonSnapshot::Deficit => Self::Deficit,
            ServicePauseReasonSnapshot::PowerCap => Self::PowerCap,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStateSnapshot {
    pub system_id: String,
    pub level: u8,
}

impl From<&crate::game::sim::SystemState> for SystemStateSnapshot {
    fn from(value: &crate::game::sim::SystemState) -> Self {
        Self {
            system_id: value.system_id.clone(),
            level: value.level,
        }
    }
}

impl From<SystemStateSnapshot> for crate::game::sim::SystemState {
    fn from(value: SystemStateSnapshot) -> Self {
        Self {
            system_id: value.system_id,
            level: value.level,
        }
    }
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}
