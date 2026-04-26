pub const SOLSTICE_ANCHOR_ID: &str = "solstice-anchor";
pub const CINDER_FORGE_ID: &str = "cinder-forge";
pub const AURORA_PIER_ID: &str = "aurora-pier";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetModifierTarget {
    CrewEfficiency,
    DataOutput,
    MaterialsOutput,
    ServicePowerUpkeep,
    CrewCapacity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanetModifier {
    pub target: PlanetModifierTarget,
    pub percent: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanetDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
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

pub fn planet_by_id(id: &str) -> Option<&'static PlanetDefinition> {
    PLANETS.iter().find(|planet| planet.id == id)
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
}
