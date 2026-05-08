//! Static data catalog for upgradeable station systems.
//!
//! Each system has a fixed progression of four levels. Systems are upgraded by spending
//! Materials; the cost for each transition is stored in the level's `upgrade_cost_materials`
//! field (`None` at the maximum level).
//!
//! These definitions are read-only once loaded at startup; they are never mutated at runtime.
//! See [`crate::game::sim::state::RunState`] for the mutable game state that consumes this data.

/// Stable string ID for the Reactor Core system (power and service-power-cap progression).
pub const REACTOR_CORE_ID: &str = "reactor-core";
/// Stable string ID for the Habitat Ring system (crew capacity and recovery progression).
pub const HABITAT_RING_ID: &str = "habitat-ring";
/// Stable string ID for the Logistics Spine system (service slots and materials-cap progression).
pub const LOGISTICS_SPINE_ID: &str = "logistics-spine";
/// Stable string ID for the Survey Array system (data and survey multiplier progression).
pub const SURVEY_ARRAY_ID: &str = "survey-array";

/// Stats for one level of the Reactor Core upgrade path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReactorCoreLevel {
    /// Total power output contributed by the reactor at this level.
    pub power_output: f32,
    /// Maximum power that can be allocated to services at this level.
    pub service_power_cap: u8,
    /// Materials cost to upgrade from this level to the next; `None` at the maximum level.
    pub upgrade_cost_materials: Option<u32>,
}

/// Stats for one level of the Habitat Ring upgrade path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HabitatRingLevel {
    /// Maximum crew the station can house at this level.
    pub crew_capacity: u8,
    /// Maximum crew recovery rate per real-time minute at this level.
    pub recovery_ceiling_per_minute: f32,
    /// Materials cost to upgrade from this level to the next; `None` at the maximum level.
    pub upgrade_cost_materials: Option<u32>,
}

/// Stats for one level of the Logistics Spine upgrade path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogisticsSpineLevel {
    /// Number of service slots available at this level.
    pub active_service_slots: u8,
    /// Maximum materials the station can stockpile at this level.
    pub materials_capacity: u32,
    /// Materials cost to upgrade from this level to the next; `None` at the maximum level.
    pub upgrade_cost_materials: Option<u32>,
}

/// Stats for one level of the Survey Array upgrade path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurveyArrayLevel {
    /// Multiplier applied to all data generation at this level.
    pub data_multiplier: f32,
    /// Multiplier applied to all survey progress at this level.
    pub survey_multiplier: f32,
    /// Materials cost to upgrade from this level to the next; `None` at the maximum level.
    pub upgrade_cost_materials: Option<u32>,
}

/// Typed progression table for a system, discriminated by system kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemProgression {
    /// Level table for the Reactor Core system.
    ReactorCore(&'static [ReactorCoreLevel]),
    /// Level table for the Habitat Ring system.
    HabitatRing(&'static [HabitatRingLevel]),
    /// Level table for the Logistics Spine system.
    LogisticsSpine(&'static [LogisticsSpineLevel]),
    /// Level table for the Survey Array system.
    SurveyArray(&'static [SurveyArrayLevel]),
}

impl SystemProgression {
    /// Returns the maximum level for this progression (i.e. the number of defined levels).
    pub fn max_level(&self) -> u8 {
        match self {
            Self::ReactorCore(levels) => levels.len() as u8,
            Self::HabitatRing(levels) => levels.len() as u8,
            Self::LogisticsSpine(levels) => levels.len() as u8,
            Self::SurveyArray(levels) => levels.len() as u8,
        }
    }

    /// Returns the materials cost to upgrade FROM the given level (1-based).
    /// Returns `None` for the max level (no upgrade available) or for an out-of-range level.
    pub fn upgrade_cost_at(&self, level: u8) -> Option<u32> {
        let idx = level.saturating_sub(1) as usize;
        match self {
            Self::ReactorCore(levels) => levels.get(idx).and_then(|l| l.upgrade_cost_materials),
            Self::HabitatRing(levels) => levels.get(idx).and_then(|l| l.upgrade_cost_materials),
            Self::LogisticsSpine(levels) => levels.get(idx).and_then(|l| l.upgrade_cost_materials),
            Self::SurveyArray(levels) => levels.get(idx).and_then(|l| l.upgrade_cost_materials),
        }
    }
}

/// Defines a station system with its full upgrade progression table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemDefinition {
    /// Unique kebab-case identifier used throughout the simulation and IPC layer.
    pub id: &'static str,
    /// Short display name shown in the UI.
    pub label: &'static str,
    /// The typed level table for this system's upgrade path.
    pub progression: SystemProgression,
}

const REACTOR_CORE_LEVELS: &[ReactorCoreLevel] = &[
    ReactorCoreLevel {
        power_output: 8.0,
        service_power_cap: 8,
        upgrade_cost_materials: Some(40),
    },
    ReactorCoreLevel {
        power_output: 12.0,
        service_power_cap: 12,
        upgrade_cost_materials: Some(80),
    },
    ReactorCoreLevel {
        power_output: 16.0,
        service_power_cap: 16,
        upgrade_cost_materials: Some(140),
    },
    ReactorCoreLevel {
        power_output: 20.0,
        service_power_cap: 20,
        upgrade_cost_materials: None,
    },
];

const HABITAT_RING_LEVELS: &[HabitatRingLevel] = &[
    HabitatRingLevel {
        crew_capacity: 6,
        recovery_ceiling_per_minute: 1.0,
        upgrade_cost_materials: Some(35),
    },
    HabitatRingLevel {
        crew_capacity: 8,
        recovery_ceiling_per_minute: 1.5,
        upgrade_cost_materials: Some(75),
    },
    HabitatRingLevel {
        crew_capacity: 10,
        recovery_ceiling_per_minute: 2.0,
        upgrade_cost_materials: Some(130),
    },
    HabitatRingLevel {
        crew_capacity: 12,
        recovery_ceiling_per_minute: 2.5,
        upgrade_cost_materials: None,
    },
];

const LOGISTICS_SPINE_LEVELS: &[LogisticsSpineLevel] = &[
    LogisticsSpineLevel {
        active_service_slots: 2,
        materials_capacity: 250,
        upgrade_cost_materials: Some(30),
    },
    LogisticsSpineLevel {
        active_service_slots: 3,
        materials_capacity: 400,
        upgrade_cost_materials: Some(70),
    },
    LogisticsSpineLevel {
        active_service_slots: 4,
        materials_capacity: 600,
        upgrade_cost_materials: Some(120),
    },
    LogisticsSpineLevel {
        active_service_slots: 5,
        materials_capacity: 850,
        upgrade_cost_materials: None,
    },
];

const SURVEY_ARRAY_LEVELS: &[SurveyArrayLevel] = &[
    SurveyArrayLevel {
        data_multiplier: 1.00,
        survey_multiplier: 1.00,
        upgrade_cost_materials: Some(50),
    },
    SurveyArrayLevel {
        data_multiplier: 1.20,
        survey_multiplier: 1.15,
        upgrade_cost_materials: Some(95),
    },
    SurveyArrayLevel {
        data_multiplier: 1.40,
        survey_multiplier: 1.30,
        upgrade_cost_materials: Some(155),
    },
    SurveyArrayLevel {
        data_multiplier: 1.65,
        survey_multiplier: 1.50,
        upgrade_cost_materials: None,
    },
];

/// Ordered catalog of all station systems; use the `id` field for stable lookups.
pub const SYSTEMS: &[SystemDefinition] = &[
    SystemDefinition {
        id: REACTOR_CORE_ID,
        label: "Reactor Core",
        progression: SystemProgression::ReactorCore(REACTOR_CORE_LEVELS),
    },
    SystemDefinition {
        id: HABITAT_RING_ID,
        label: "Habitat Ring",
        progression: SystemProgression::HabitatRing(HABITAT_RING_LEVELS),
    },
    SystemDefinition {
        id: LOGISTICS_SPINE_ID,
        label: "Logistics Spine",
        progression: SystemProgression::LogisticsSpine(LOGISTICS_SPINE_LEVELS),
    },
    SystemDefinition {
        id: SURVEY_ARRAY_ID,
        label: "Survey Array",
        progression: SystemProgression::SurveyArray(SURVEY_ARRAY_LEVELS),
    },
];

/// Looks up a system definition by its stable string ID. Returns `None` if not found.
pub fn system_by_id(id: &str) -> Option<&'static SystemDefinition> {
    SYSTEMS.iter().find(|system| system.id == id)
}

/// Returns a system definition by ID, panicking if not found.
///
/// # Panics
/// Panics with "system must exist in catalog" if the ID is not found.
#[track_caller]
pub fn system_by_id_required(id: &str) -> &'static SystemDefinition {
    system_by_id(id).expect("system must exist in catalog")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_catalogs_system_ids_exist_with_four_levels_each() {
        assert_eq!(SYSTEMS.len(), 4);

        let reactor_core = system_by_id(REACTOR_CORE_ID).unwrap();
        let habitat_ring = system_by_id(HABITAT_RING_ID).unwrap();
        let logistics_spine = system_by_id(LOGISTICS_SPINE_ID).unwrap();
        let survey_array = system_by_id(SURVEY_ARRAY_ID).unwrap();

        match reactor_core.progression {
            SystemProgression::ReactorCore(levels) => assert_eq!(levels.len(), 4),
            _ => panic!("wrong progression for reactor-core"),
        }

        match habitat_ring.progression {
            SystemProgression::HabitatRing(levels) => assert_eq!(levels.len(), 4),
            _ => panic!("wrong progression for habitat-ring"),
        }

        match logistics_spine.progression {
            SystemProgression::LogisticsSpine(levels) => assert_eq!(levels.len(), 4),
            _ => panic!("wrong progression for logistics-spine"),
        }

        match survey_array.progression {
            SystemProgression::SurveyArray(levels) => assert_eq!(levels.len(), 4),
            _ => panic!("wrong progression for survey-array"),
        }
    }

    #[test]
    fn content_catalogs_system_progression_matches_plan() {
        let reactor_core = system_by_id(REACTOR_CORE_ID).unwrap();
        match reactor_core.progression {
            SystemProgression::ReactorCore(levels) => assert_eq!(
                levels,
                &[
                    ReactorCoreLevel {
                        power_output: 8.0,
                        service_power_cap: 8,
                        upgrade_cost_materials: Some(40),
                    },
                    ReactorCoreLevel {
                        power_output: 12.0,
                        service_power_cap: 12,
                        upgrade_cost_materials: Some(80),
                    },
                    ReactorCoreLevel {
                        power_output: 16.0,
                        service_power_cap: 16,
                        upgrade_cost_materials: Some(140),
                    },
                    ReactorCoreLevel {
                        power_output: 20.0,
                        service_power_cap: 20,
                        upgrade_cost_materials: None,
                    },
                ]
            ),
            _ => panic!("wrong progression for reactor-core"),
        }

        let habitat_ring = system_by_id(HABITAT_RING_ID).unwrap();
        match habitat_ring.progression {
            SystemProgression::HabitatRing(levels) => assert_eq!(
                levels,
                &[
                    HabitatRingLevel {
                        crew_capacity: 6,
                        recovery_ceiling_per_minute: 1.0,
                        upgrade_cost_materials: Some(35),
                    },
                    HabitatRingLevel {
                        crew_capacity: 8,
                        recovery_ceiling_per_minute: 1.5,
                        upgrade_cost_materials: Some(75),
                    },
                    HabitatRingLevel {
                        crew_capacity: 10,
                        recovery_ceiling_per_minute: 2.0,
                        upgrade_cost_materials: Some(130),
                    },
                    HabitatRingLevel {
                        crew_capacity: 12,
                        recovery_ceiling_per_minute: 2.5,
                        upgrade_cost_materials: None,
                    },
                ]
            ),
            _ => panic!("wrong progression for habitat-ring"),
        }

        let logistics_spine = system_by_id(LOGISTICS_SPINE_ID).unwrap();
        match logistics_spine.progression {
            SystemProgression::LogisticsSpine(levels) => assert_eq!(
                levels,
                &[
                    LogisticsSpineLevel {
                        active_service_slots: 2,
                        materials_capacity: 250,
                        upgrade_cost_materials: Some(30),
                    },
                    LogisticsSpineLevel {
                        active_service_slots: 3,
                        materials_capacity: 400,
                        upgrade_cost_materials: Some(70),
                    },
                    LogisticsSpineLevel {
                        active_service_slots: 4,
                        materials_capacity: 600,
                        upgrade_cost_materials: Some(120),
                    },
                    LogisticsSpineLevel {
                        active_service_slots: 5,
                        materials_capacity: 850,
                        upgrade_cost_materials: None,
                    },
                ]
            ),
            _ => panic!("wrong progression for logistics-spine"),
        }

        let survey_array = system_by_id(SURVEY_ARRAY_ID).unwrap();
        match survey_array.progression {
            SystemProgression::SurveyArray(levels) => assert_eq!(
                levels,
                &[
                    SurveyArrayLevel {
                        data_multiplier: 1.00,
                        survey_multiplier: 1.00,
                        upgrade_cost_materials: Some(50),
                    },
                    SurveyArrayLevel {
                        data_multiplier: 1.20,
                        survey_multiplier: 1.15,
                        upgrade_cost_materials: Some(95),
                    },
                    SurveyArrayLevel {
                        data_multiplier: 1.40,
                        survey_multiplier: 1.30,
                        upgrade_cost_materials: Some(155),
                    },
                    SurveyArrayLevel {
                        data_multiplier: 1.65,
                        survey_multiplier: 1.50,
                        upgrade_cost_materials: None,
                    },
                ]
            ),
            _ => panic!("wrong progression for survey-array"),
        }
    }

    #[test]
    #[should_panic(expected = "system must exist in catalog")]
    fn system_by_id_required_panics_on_unknown() {
        system_by_id_required("nonexistent-system-that-does-not-exist");
    }
}
