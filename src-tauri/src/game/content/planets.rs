//! Static data catalog for orbiting planets.
//!
//! These definitions are read-only once loaded at startup; they are never mutated at runtime.
//! See [`crate::game::sim::state::RunState`] for the mutable game state that consumes this data.

/// Stable string ID for Solstice Anchor, the starter balanced planet.
pub const SOLSTICE_ANCHOR_ID: &str = "solstice-anchor";
/// Stable string ID for Cinder Forge, the high-yield industrial planet.
pub const CINDER_FORGE_ID: &str = "cinder-forge";
/// Stable string ID for Aurora Pier, the data-focused research planet.
pub const AURORA_PIER_ID: &str = "aurora-pier";

/// Identifies which station stat a [`PlanetModifier`] applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetModifierTarget {
    /// Scales crew productivity (affects resource output per assigned crew).
    CrewEfficiency,
    /// Scales the amount of Data produced per tick.
    DataOutput,
    /// Scales the amount of Materials produced per tick.
    MaterialsOutput,
    /// Scales the power upkeep cost of all active services.
    ServicePowerUpkeep,
    /// Scales the maximum crew the station can house.
    CrewCapacity,
}

impl PlanetModifierTarget {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            PlanetModifierTarget::CrewEfficiency => "crew-efficiency",
            PlanetModifierTarget::DataOutput => "data-output",
            PlanetModifierTarget::MaterialsOutput => "materials-output",
            PlanetModifierTarget::ServicePowerUpkeep => "service-power-upkeep",
            PlanetModifierTarget::CrewCapacity => "crew-capacity",
        }
    }

    pub(crate) fn label(&self) -> &'static str {
        match self {
            PlanetModifierTarget::CrewEfficiency => "Crew efficiency",
            PlanetModifierTarget::DataOutput => "Data output",
            PlanetModifierTarget::MaterialsOutput => "Materials output",
            PlanetModifierTarget::ServicePowerUpkeep => "Service power upkeep",
            PlanetModifierTarget::CrewCapacity => "Crew capacity",
        }
    }
}

/// A single percentage-based modifier applied by a planet to a station stat.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanetModifier {
    /// The station stat this modifier affects.
    pub target: PlanetModifierTarget,
    /// Additive percentage adjustment (e.g. `0.10` = +10%, `-0.15` = -15%).
    pub percent: f32,
}

/// Defines a planet that the player can orbit during a run, including its stat modifiers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanetDefinition {
    /// Unique kebab-case identifier used to track the active planet in [`crate::game::sim::state::RunState`].
    pub id: &'static str,
    /// Short display name shown in the planet picker.
    pub label: &'static str,
    /// Human-readable summary of the planet's trade-offs.
    pub description: &'static str,
    /// Stat modifiers applied while the station orbits this planet.
    pub modifiers: &'static [PlanetModifier],
}

const SOLSTICE_ANCHOR_MODIFIERS: &[PlanetModifier] = &[
    PlanetModifier {
        target: PlanetModifierTarget::CrewEfficiency,
        percent: 0.10,
    },
    PlanetModifier {
        target: PlanetModifierTarget::DataOutput,
        percent: -0.10,
    },
];

const CINDER_FORGE_MODIFIERS: &[PlanetModifier] = &[
    PlanetModifier {
        target: PlanetModifierTarget::MaterialsOutput,
        percent: 0.25,
    },
    PlanetModifier {
        target: PlanetModifierTarget::ServicePowerUpkeep,
        percent: 0.20,
    },
];

const AURORA_PIER_MODIFIERS: &[PlanetModifier] = &[
    PlanetModifier {
        target: PlanetModifierTarget::DataOutput,
        percent: 0.30,
    },
    PlanetModifier {
        target: PlanetModifierTarget::CrewCapacity,
        percent: -0.15,
    },
];

/// Ordered catalog of all available planets; use the `id` field for stable lookups.
pub const PLANETS: &[PlanetDefinition] = &[
    PlanetDefinition {
        id: SOLSTICE_ANCHOR_ID,
        label: "Solstice Anchor",
        description: "Starter balanced planet with efficient crews but weaker research output.",
        modifiers: SOLSTICE_ANCHOR_MODIFIERS,
    },
    PlanetDefinition {
        id: CINDER_FORGE_ID,
        label: "Cinder Forge",
        description: "Industrial planet tuned for material throughput at higher power cost.",
        modifiers: CINDER_FORGE_MODIFIERS,
    },
    PlanetDefinition {
        id: AURORA_PIER_ID,
        label: "Aurora Pier",
        description: "Research planet with stronger data returns and lower crew capacity.",
        modifiers: AURORA_PIER_MODIFIERS,
    },
];

/// Looks up a planet definition by its stable string ID. Returns `None` if not found.
pub fn planet_by_id(id: &str) -> Option<&'static PlanetDefinition> {
    PLANETS.iter().find(|planet| planet.id == id)
}

/// Returns a planet definition by ID, panicking if not found.
///
/// # Panics
/// Panics with "planet must exist in catalog" if the ID is not found.
#[track_caller]
pub fn planet_by_id_required(id: &str) -> &'static PlanetDefinition {
    planet_by_id(id).expect("planet must exist in catalog")
}

/// Returns the survey threshold for a given planet ID, or `None` if the planet is not surveyable.
///
/// SOLSTICE_ANCHOR has no survey threshold (returns `None`).
/// CINDER_FORGE returns `Some(600.0)`.
/// AURORA_PIER returns `Some(1400.0)`.
/// Unknown planets return `None`.
pub fn survey_threshold(planet_id: &str) -> Option<f32> {
    use crate::game::sim::state::{CINDER_FORGE_SURVEY_THRESHOLD, AURORA_PIER_SURVEY_THRESHOLD};
    
    match planet_id {
        SOLSTICE_ANCHOR_ID => None,
        CINDER_FORGE_ID => Some(CINDER_FORGE_SURVEY_THRESHOLD),
        AURORA_PIER_ID => Some(AURORA_PIER_SURVEY_THRESHOLD),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_catalogs_planet_ids_exist() {
        assert_eq!(PLANETS.len(), 3);
        assert!(planet_by_id(SOLSTICE_ANCHOR_ID).is_some());
        assert!(planet_by_id(CINDER_FORGE_ID).is_some());
        assert!(planet_by_id(AURORA_PIER_ID).is_some());
    }

    #[test]
    fn content_catalogs_planet_modifiers_match_plan() {
        let solstice_anchor = planet_by_id(SOLSTICE_ANCHOR_ID).unwrap();
        assert_eq!(
            solstice_anchor.modifiers,
            &[
                PlanetModifier {
                    target: PlanetModifierTarget::CrewEfficiency,
                    percent: 0.10,
                },
                PlanetModifier {
                    target: PlanetModifierTarget::DataOutput,
                    percent: -0.10,
                },
            ]
        );

        let cinder_forge = planet_by_id(CINDER_FORGE_ID).unwrap();
        assert_eq!(
            cinder_forge.modifiers,
            &[
                PlanetModifier {
                    target: PlanetModifierTarget::MaterialsOutput,
                    percent: 0.25,
                },
                PlanetModifier {
                    target: PlanetModifierTarget::ServicePowerUpkeep,
                    percent: 0.20,
                },
            ]
        );

        let aurora_pier = planet_by_id(AURORA_PIER_ID).unwrap();
        assert_eq!(
            aurora_pier.modifiers,
            &[
                PlanetModifier {
                    target: PlanetModifierTarget::DataOutput,
                    percent: 0.30,
                },
                PlanetModifier {
                    target: PlanetModifierTarget::CrewCapacity,
                    percent: -0.15,
                },
            ]
        );
    }

    #[test]
    #[should_panic(expected = "planet must exist in catalog")]
    fn planet_by_id_required_panics_on_unknown() {
        planet_by_id_required("nonexistent-planet-that-does-not-exist");
    }

    #[test]
    fn survey_threshold_semantics() {
        assert_eq!(survey_threshold(SOLSTICE_ANCHOR_ID), None, "SOLSTICE has no threshold");
        assert_eq!(survey_threshold(CINDER_FORGE_ID), Some(600.0), "Cinder Forge threshold");
        assert_eq!(survey_threshold(AURORA_PIER_ID), Some(1400.0), "Aurora Pier threshold");
        assert_eq!(survey_threshold("unknown-planet"), None, "Unknown planet has no threshold");
    }
}
