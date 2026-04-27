pub const POWER_ID: &str = "power";
pub const MATERIALS_ID: &str = "materials";
pub const DATA_ID: &str = "data";
pub const CREW_ID: &str = "crew";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrewPoolSemantics {
    pub fields: &'static [&'static str],
    pub spendable_currency: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceModel {
    ThroughputBudget { storable: bool },
    Stockpile,
    AssignmentPool(CrewPoolSemantics),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceMetadata {
    pub id: &'static str,
    pub label: &'static str,
    pub model: ResourceModel,
    pub description: &'static str,
}

pub const RESOURCES: &[ResourceMetadata] = &[
    ResourceMetadata {
        id: POWER_ID,
        label: "Power",
        model: ResourceModel::ThroughputBudget { storable: false },
        description: "Per-tick throughput budget that cannot be stockpiled.",
    },
    ResourceMetadata {
        id: MATERIALS_ID,
        label: "Materials",
        model: ResourceModel::Stockpile,
        description: "Stockpile used for construction, upgrades, and upkeep.",
    },
    ResourceMetadata {
        id: DATA_ID,
        label: "Data",
        model: ResourceModel::Stockpile,
        description: "Stockpile used for surveys, doctrines, and prestige acceleration.",
    },
    ResourceMetadata {
        id: CREW_ID,
        label: "Crew",
        model: ResourceModel::AssignmentPool(CrewPoolSemantics {
            fields: &["total", "assigned", "available"],
            spendable_currency: false,
        }),
        description: "Assignment pool tracked as total, assigned, and available workers.",
    },
];

pub fn resource_by_id(id: &str) -> Option<&'static ResourceMetadata> {
    RESOURCES.iter().find(|resource| resource.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_catalogs_resource_ids_exist() {
        assert_eq!(RESOURCES.len(), 4);
        for id in [POWER_ID, MATERIALS_ID, DATA_ID, CREW_ID] {
            assert!(resource_by_id(id).is_some(), "missing resource: {id}");
        }
    }

    #[test]
    fn content_catalogs_resource_semantics_match_plan() {
        assert_eq!(
            resource_by_id(POWER_ID).unwrap().model,
            ResourceModel::ThroughputBudget { storable: false }
        );
        assert_eq!(resource_by_id(MATERIALS_ID).unwrap().model, ResourceModel::Stockpile);
        assert_eq!(resource_by_id(DATA_ID).unwrap().model, ResourceModel::Stockpile);
        assert_eq!(
            resource_by_id(CREW_ID).unwrap().model,
            ResourceModel::AssignmentPool(CrewPoolSemantics {
                fields: &["total", "assigned", "available"],
                spendable_currency: false,
            })
        );
    }
}
