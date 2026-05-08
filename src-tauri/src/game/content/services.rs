//! Static data catalog for station services (the activatable modules that drive production).
//!
//! These definitions are read-only once loaded at startup; they are never mutated at runtime.
//! See [`crate::game::sim::state::RunState`] for the mutable game state that consumes this data.

/// Stable string ID for the Solar Harvester service (primary power producer).
pub const SOLAR_HARVESTER_ID: &str = "solar-harvester";
/// Stable string ID for the Ore Reclaimer service (materials producer).
pub const ORE_RECLAIMER_ID: &str = "ore-reclaimer";
/// Stable string ID for the Survey Uplink service (data and survey-point producer).
pub const SURVEY_UPLINK_ID: &str = "survey-uplink";
/// Stable string ID for the Maintenance Bay service (global power upkeep reducer).
pub const MAINTENANCE_BAY_ID: &str = "maintenance-bay";
/// Stable string ID for the Command Relay service (survey speed booster and priority stabiliser).
pub const COMMAND_RELAY_ID: &str = "command-relay";
/// Stable string ID for the Fabrication Loop service (materials-to-data converter).
pub const FABRICATION_LOOP_ID: &str = "fabrication-loop";

/// Broad category that groups services by their primary economic role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceCategory {
    /// Generates primary resources (power, materials, data, or survey points).
    Production,
    /// Modifies station-wide multipliers without direct resource generation.
    Support,
    /// Consumes one resource to produce another (e.g. materials → data).
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

/// Full static definition of a service module that can be slotted and activated on the station.
///
/// All numeric fields are per-tick rates unless noted otherwise. Positive values are outputs;
/// negative values are inputs consumed by the service.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServiceDefinition {
    /// Unique kebab-case identifier used throughout the simulation and IPC layer.
    pub id: &'static str,
    /// Short display name shown in the UI.
    pub label: &'static str,
    /// Broad economic role of this service.
    pub category: ServiceCategory,
    /// Number of crew members the service requires to run each tick.
    pub crew_required: u8,
    /// Power consumed per tick while active (subtracted from the power budget).
    pub power_upkeep: f32,
    /// Power generated per tick while active (added to the power budget).
    pub power_output: f32,
    /// Materials consumed per tick as ongoing upkeep (non-conversion cost).
    pub materials_upkeep: f32,
    /// Materials generated per tick while active.
    pub materials_output: f32,
    /// Materials consumed per tick as conversion input (negative = consumed).
    pub materials_input: f32,
    /// Data generated per tick while active.
    pub data_output: f32,
    /// Survey progress points contributed per tick while active.
    pub survey_points: f32,
    /// Fractional additive modifier applied to every other service's power upkeep (e.g. `-0.10` = −10%).
    pub global_service_power_modifier: f32,
    /// Fractional additive modifier applied to the global survey speed (e.g. `0.10` = +10%).
    pub survey_speed_modifier: f32,
    /// Signed adjustment to this service's scheduling priority for stability tiebreaking.
    pub service_priority_stability: i8,
}

/// Ordered catalog of all available services; use the `id` field for stable lookups.
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

/// Looks up a service definition by its stable string ID. Returns `None` if not found.
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
