//! Static data catalog for station resources.
//!
//! These definitions are read-only once loaded at startup; they are never mutated at runtime.
//! See [`crate::game::sim::state::RunState`] for the mutable game state that consumes this data.

/// Stable string ID for the Power resource (per-tick throughput).
pub const POWER_ID: &str = "power";
/// Stable string ID for the Materials resource (stockpiled construction material).
pub const MATERIALS_ID: &str = "materials";
/// Stable string ID for the Data resource (stockpiled research currency).
pub const DATA_ID: &str = "data";
/// Stable string ID for the Crew resource (assignment pool, not a spendable currency).
pub const CREW_ID: &str = "crew";

/// Describes the sub-fields tracked for an assignment-pool resource like Crew.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrewPoolSemantics {
    /// Named sub-fields exposed on the pool (e.g. `["total", "assigned", "available"]`).
    pub fields: &'static [&'static str],
    /// Whether the pool can be permanently spent like a currency (`false` for Crew).
    pub spendable_currency: bool,
}

/// Classifies how a resource is tracked and consumed by the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceModel {
    /// Produced and consumed within a single tick; excess is lost unless `storable` is true.
    ThroughputBudget { storable: bool },
    /// Accumulates over time and can be spent on upgrades or actions.
    Stockpile,
    /// Tracked as a partitioned pool (total / assigned / available) rather than a simple counter.
    AssignmentPool(CrewPoolSemantics),
}

/// Metadata describing a single resource in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceMetadata {
    /// Unique kebab-case identifier referenced throughout the simulation and IPC layer.
    pub id: &'static str,
    /// Short display name shown in the UI.
    pub label: &'static str,
    /// How the simulation tracks and consumes this resource.
    pub model: ResourceModel,
    /// Human-readable description of the resource's role in the station economy.
    pub description: &'static str,
}

/// Ordered catalog of all tracked resources; use the `id` field for stable lookups.
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

/// Looks up a resource metadata entry by its stable string ID. Returns `None` if not found.
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
        assert_eq!(
            resource_by_id(MATERIALS_ID).unwrap().model,
            ResourceModel::Stockpile
        );
        assert_eq!(
            resource_by_id(DATA_ID).unwrap().model,
            ResourceModel::Stockpile
        );
        assert_eq!(
            resource_by_id(CREW_ID).unwrap().model,
            ResourceModel::AssignmentPool(CrewPoolSemantics {
                fields: &["total", "assigned", "available"],
                spendable_currency: false,
            })
        );
    }
}
