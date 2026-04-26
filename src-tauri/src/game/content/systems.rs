pub const REACTOR_CORE_ID: &str = "reactor-core";
pub const HABITAT_RING_ID: &str = "habitat-ring";
pub const LOGISTICS_SPINE_ID: &str = "logistics-spine";
pub const SURVEY_ARRAY_ID: &str = "survey-array";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReactorCoreLevel {
    pub power_output: f32,
    pub service_power_cap: u8,
    pub upgrade_cost_materials: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HabitatRingLevel {
    pub crew_capacity: u8,
    pub recovery_ceiling_per_minute: f32,
    pub upgrade_cost_materials: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogisticsSpineLevel {
    pub active_service_slots: u8,
    pub materials_capacity: u32,
    pub upgrade_cost_materials: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurveyArrayLevel {
    pub data_multiplier: f32,
    pub survey_multiplier: f32,
    pub upgrade_cost_materials: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemProgression {
    ReactorCore(&'static [ReactorCoreLevel]),
    HabitatRing(&'static [HabitatRingLevel]),
    LogisticsSpine(&'static [LogisticsSpineLevel]),
    SurveyArray(&'static [SurveyArrayLevel]),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemDefinition {
    pub id: &'static str,
    pub label: &'static str,
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

pub fn system_by_id(id: &str) -> Option<&'static SystemDefinition> {
    SYSTEMS.iter().find(|system| system.id == id)
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
}
