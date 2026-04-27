use super::services::SURVEY_UPLINK_ID;
use super::systems::REACTOR_CORE_ID;

pub const EFFICIENT_SHIFTS_ID: &str = "efficient-shifts";
pub const DEEP_SURVEY_PROTOCOLS_ID: &str = "deep-survey-protocols";
pub const HARDENED_RELAYS_ID: &str = "hardened-relays";
pub const FRONTIER_CHARTERS_ID: &str = "frontier-charters";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DoctrineEffect {
    FirstSupportServiceCrewDiscount { reduction: u8, minimum_crew: u8 },
    SurveyProgressMultiplier {
        source_service_id: &'static str,
        multiplier: f32,
    },
    SameTickPowerRefundOnDisable { refund_ratio: f32 },
    NewlyDiscoveredPlanetsStartWithSystemLevel { system_id: &'static str, level: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoctrineDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub effect: DoctrineEffect,
}

pub const DOCTRINES: &[DoctrineDefinition] = &[
    DoctrineDefinition {
        id: EFFICIENT_SHIFTS_ID,
        label: "Efficient Shifts",
        description: "The first support service needs 1 less Crew, to a minimum of 1.",
        effect: DoctrineEffect::FirstSupportServiceCrewDiscount {
            reduction: 1,
            minimum_crew: 1,
        },
    },
    DoctrineDefinition {
        id: DEEP_SURVEY_PROTOCOLS_ID,
        label: "Deep Survey Protocols",
        description: "Survey Uplink grants 20% more survey progress.",
        effect: DoctrineEffect::SurveyProgressMultiplier {
            source_service_id: SURVEY_UPLINK_ID,
            multiplier: 1.20,
        },
    },
    DoctrineDefinition {
        id: HARDENED_RELAYS_ID,
        label: "Hardened Relays",
        description: "Disabled services refund 50% of current-tick power upkeep back to the same tick.",
        effect: DoctrineEffect::SameTickPowerRefundOnDisable { refund_ratio: 0.50 },
    },
    DoctrineDefinition {
        id: FRONTIER_CHARTERS_ID,
        label: "Frontier Charters",
        description: "Newly discovered planets begin with Reactor Core level 2.",
        effect: DoctrineEffect::NewlyDiscoveredPlanetsStartWithSystemLevel {
            system_id: REACTOR_CORE_ID,
            level: 2,
        },
    },
];

pub fn doctrine_by_id(id: &str) -> Option<&'static DoctrineDefinition> {
    DOCTRINES.iter().find(|doctrine| doctrine.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_catalogs_doctrine_ids_exist() {
        assert_eq!(DOCTRINES.len(), 4);
        for id in [
            EFFICIENT_SHIFTS_ID,
            DEEP_SURVEY_PROTOCOLS_ID,
            HARDENED_RELAYS_ID,
            FRONTIER_CHARTERS_ID,
        ] {
            assert!(doctrine_by_id(id).is_some(), "missing doctrine: {id}");
        }
    }

    #[test]
    fn content_catalogs_doctrine_effects_match_plan() {
        assert_eq!(
            doctrine_by_id(EFFICIENT_SHIFTS_ID).unwrap().effect,
            DoctrineEffect::FirstSupportServiceCrewDiscount {
                reduction: 1,
                minimum_crew: 1,
            }
        );
        assert_eq!(
            doctrine_by_id(DEEP_SURVEY_PROTOCOLS_ID).unwrap().effect,
            DoctrineEffect::SurveyProgressMultiplier {
                source_service_id: SURVEY_UPLINK_ID,
                multiplier: 1.20,
            }
        );
        assert_eq!(
            doctrine_by_id(HARDENED_RELAYS_ID).unwrap().effect,
            DoctrineEffect::SameTickPowerRefundOnDisable { refund_ratio: 0.50 }
        );
        assert_eq!(
            doctrine_by_id(FRONTIER_CHARTERS_ID).unwrap().effect,
            DoctrineEffect::NewlyDiscoveredPlanetsStartWithSystemLevel {
                system_id: REACTOR_CORE_ID,
                level: 2,
            }
        );
    }
}
