pub const SOLAR_HARVESTER_ID: &str = "solar-harvester";
pub const ORE_RECLAIMER_ID: &str = "ore-reclaimer";
pub const SURVEY_UPLINK_ID: &str = "survey-uplink";
pub const MAINTENANCE_BAY_ID: &str = "maintenance-bay";
pub const COMMAND_RELAY_ID: &str = "command-relay";
pub const FABRICATION_LOOP_ID: &str = "fabrication-loop";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceCategory {
    Production,
    Support,
    Conversion,
}

impl ServiceCategory {
    pub(crate) fn family(&self) -> &'static str {
        match self {
            ServiceCategory::Production => "production",
            ServiceCategory::Support => "support",
            ServiceCategory::Conversion => "conversion",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServiceDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub category: ServiceCategory,
    pub crew_required: u8,
    pub power_upkeep: f32,
    pub power_output: f32,
    pub materials_upkeep: f32,
    pub materials_output: f32,
    pub materials_input: f32,
    pub data_output: f32,
    pub survey_points: f32,
    pub global_service_power_modifier: f32,
    pub survey_speed_modifier: f32,
    pub service_priority_stability: i8,
}

pub const SERVICES: &[ServiceDefinition] = &[
    ServiceDefinition {
        id: SOLAR_HARVESTER_ID,
        label: "Solar Harvester",
        category: ServiceCategory::Production,
        crew_required: 2,
        power_upkeep: 0.0,
        power_output: 4.0,
        materials_upkeep: 0.0,
        materials_output: 0.0,
        materials_input: 0.0,
        data_output: 0.0,
        survey_points: 0.0,
        global_service_power_modifier: 0.0,
        survey_speed_modifier: 0.0,
        service_priority_stability: 0,
    },
    ServiceDefinition {
        id: ORE_RECLAIMER_ID,
        label: "Ore Reclaimer",
        category: ServiceCategory::Production,
        crew_required: 1,
        power_upkeep: 3.0,
        power_output: 0.0,
        materials_upkeep: 0.0,
        materials_output: 2.0,
        materials_input: 0.0,
        data_output: 0.0,
        survey_points: 0.0,
        global_service_power_modifier: 0.0,
        survey_speed_modifier: 0.0,
        service_priority_stability: 0,
    },
    ServiceDefinition {
        id: SURVEY_UPLINK_ID,
        label: "Survey Uplink",
        category: ServiceCategory::Production,
        crew_required: 1,
        power_upkeep: 2.0,
        power_output: 0.0,
        materials_upkeep: 0.0,
        materials_output: 0.0,
        materials_input: 0.0,
        data_output: 1.5,
        survey_points: 1.0,
        global_service_power_modifier: 0.0,
        survey_speed_modifier: 0.0,
        service_priority_stability: 0,
    },
    ServiceDefinition {
        id: MAINTENANCE_BAY_ID,
        label: "Maintenance Bay",
        category: ServiceCategory::Support,
        crew_required: 1,
        power_upkeep: 1.0,
        power_output: 0.0,
        materials_upkeep: 0.0,
        materials_output: 0.0,
        materials_input: 0.0,
        data_output: 0.0,
        survey_points: 0.0,
        global_service_power_modifier: -0.10,
        survey_speed_modifier: 0.0,
        service_priority_stability: 0,
    },
    ServiceDefinition {
        id: COMMAND_RELAY_ID,
        label: "Command Relay",
        category: ServiceCategory::Support,
        crew_required: 1,
        power_upkeep: 1.0,
        power_output: 0.0,
        materials_upkeep: 0.0,
        materials_output: 0.0,
        materials_input: 0.0,
        data_output: 0.0,
        survey_points: 0.0,
        global_service_power_modifier: 0.0,
        survey_speed_modifier: 0.10,
        service_priority_stability: 1,
    },
    ServiceDefinition {
        id: FABRICATION_LOOP_ID,
        label: "Fabrication Loop",
        category: ServiceCategory::Conversion,
        crew_required: 1,
        power_upkeep: 2.0,
        power_output: 0.0,
        materials_upkeep: 0.0,
        materials_output: 0.0,
        materials_input: -1.5,
        data_output: 2.0,
        survey_points: 0.0,
        global_service_power_modifier: 0.0,
        survey_speed_modifier: 0.0,
        service_priority_stability: 0,
    },
];

pub fn service_by_id(id: &str) -> Option<&'static ServiceDefinition> {
    SERVICES.iter().find(|service| service.id == id)
}

/// Returns a service definition by ID, panicking if not found.
///
/// # Panics
/// Panics with "service must exist in catalog" if the ID is not found.
#[track_caller]
pub fn service_by_id_required(id: &str) -> &'static ServiceDefinition {
    service_by_id(id).expect("service must exist in catalog")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_catalogs_service_ids_exist() {
        assert_eq!(SERVICES.len(), 6);
        for id in [
            SOLAR_HARVESTER_ID,
            ORE_RECLAIMER_ID,
            SURVEY_UPLINK_ID,
            MAINTENANCE_BAY_ID,
            COMMAND_RELAY_ID,
            FABRICATION_LOOP_ID,
        ] {
            assert!(service_by_id(id).is_some(), "missing service: {id}");
        }
    }

    #[test]
    fn content_catalogs_service_runtime_tables_match_plan() {
        assert_eq!(
            service_by_id(SOLAR_HARVESTER_ID).unwrap(),
            &ServiceDefinition {
                id: SOLAR_HARVESTER_ID,
                label: "Solar Harvester",
                category: ServiceCategory::Production,
                crew_required: 2,
                power_upkeep: 0.0,
                power_output: 4.0,
                materials_upkeep: 0.0,
                materials_output: 0.0,
                materials_input: 0.0,
                data_output: 0.0,
                survey_points: 0.0,
                global_service_power_modifier: 0.0,
                survey_speed_modifier: 0.0,
                service_priority_stability: 0,
            }
        );

        assert_eq!(
            service_by_id(ORE_RECLAIMER_ID).unwrap(),
            &ServiceDefinition {
                id: ORE_RECLAIMER_ID,
                label: "Ore Reclaimer",
                category: ServiceCategory::Production,
                crew_required: 1,
                power_upkeep: 3.0,
                power_output: 0.0,
                materials_upkeep: 0.0,
                materials_output: 2.0,
                materials_input: 0.0,
                data_output: 0.0,
                survey_points: 0.0,
                global_service_power_modifier: 0.0,
                survey_speed_modifier: 0.0,
                service_priority_stability: 0,
            }
        );

        assert_eq!(
            service_by_id(SURVEY_UPLINK_ID).unwrap(),
            &ServiceDefinition {
                id: SURVEY_UPLINK_ID,
                label: "Survey Uplink",
                category: ServiceCategory::Production,
                crew_required: 1,
                power_upkeep: 2.0,
                power_output: 0.0,
                materials_upkeep: 0.0,
                materials_output: 0.0,
                materials_input: 0.0,
                data_output: 1.5,
                survey_points: 1.0,
                global_service_power_modifier: 0.0,
                survey_speed_modifier: 0.0,
                service_priority_stability: 0,
            }
        );

        assert_eq!(
            service_by_id(MAINTENANCE_BAY_ID).unwrap(),
            &ServiceDefinition {
                id: MAINTENANCE_BAY_ID,
                label: "Maintenance Bay",
                category: ServiceCategory::Support,
                crew_required: 1,
                power_upkeep: 1.0,
                power_output: 0.0,
                materials_upkeep: 0.0,
                materials_output: 0.0,
                materials_input: 0.0,
                data_output: 0.0,
                survey_points: 0.0,
                global_service_power_modifier: -0.10,
                survey_speed_modifier: 0.0,
                service_priority_stability: 0,
            }
        );

        assert_eq!(
            service_by_id(COMMAND_RELAY_ID).unwrap(),
            &ServiceDefinition {
                id: COMMAND_RELAY_ID,
                label: "Command Relay",
                category: ServiceCategory::Support,
                crew_required: 1,
                power_upkeep: 1.0,
                power_output: 0.0,
                materials_upkeep: 0.0,
                materials_output: 0.0,
                materials_input: 0.0,
                data_output: 0.0,
                survey_points: 0.0,
                global_service_power_modifier: 0.0,
                survey_speed_modifier: 0.10,
                service_priority_stability: 1,
            }
        );

        assert_eq!(
            service_by_id(FABRICATION_LOOP_ID).unwrap(),
            &ServiceDefinition {
                id: FABRICATION_LOOP_ID,
                label: "Fabrication Loop",
                category: ServiceCategory::Conversion,
                crew_required: 1,
                power_upkeep: 2.0,
                power_output: 0.0,
                materials_upkeep: 0.0,
                materials_output: 0.0,
                materials_input: -1.5,
                data_output: 2.0,
                survey_points: 0.0,
                global_service_power_modifier: 0.0,
                survey_speed_modifier: 0.0,
                service_priority_stability: 0,
            }
        );
    }

    #[test]
    #[should_panic(expected = "service must exist in catalog")]
    fn service_by_id_required_panics_on_unknown() {
        service_by_id_required("nonexistent-service-that-does-not-exist");
    }
}
