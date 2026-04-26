#![allow(dead_code)]

use crate::game::content::doctrines::{doctrine_by_id, DoctrineEffect, FRONTIER_CHARTERS_ID};
use crate::game::content::planets::SOLSTICE_ANCHOR_ID;
use crate::game::persistence::save::ProfileState;
use crate::game::progression::prestige::PrestigeProfile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoctrinePurchaseError {
    UnknownDoctrine,
    AlreadyUnlocked,
    InsufficientFragments,
}

pub trait DoctrineInventory {
    fn doctrine_ids(&self) -> &[String];
    fn doctrine_ids_mut(&mut self) -> &mut Vec<String>;
    fn doctrine_fragments(&self) -> u32;
    fn doctrine_fragments_mut(&mut self) -> &mut u32;
}

impl DoctrineInventory for PrestigeProfile {
    fn doctrine_ids(&self) -> &[String] {
        &self.doctrine_ids
    }

    fn doctrine_ids_mut(&mut self) -> &mut Vec<String> {
        &mut self.doctrine_ids
    }

    fn doctrine_fragments(&self) -> u32 {
        self.doctrine_fragments
    }

    fn doctrine_fragments_mut(&mut self) -> &mut u32 {
        &mut self.doctrine_fragments
    }
}

impl DoctrineInventory for ProfileState {
    fn doctrine_ids(&self) -> &[String] {
        &self.doctrine_ids
    }

    fn doctrine_ids_mut(&mut self) -> &mut Vec<String> {
        &mut self.doctrine_ids
    }

    fn doctrine_fragments(&self) -> u32 {
        self.doctrine_fragments
    }

    fn doctrine_fragments_mut(&mut self) -> &mut u32 {
        &mut self.doctrine_fragments
    }
}

pub fn purchase_doctrine<T: DoctrineInventory>(
    inventory: &mut T,
    doctrine_id: &str,
) -> Result<(), DoctrinePurchaseError> {
    if doctrine_by_id(doctrine_id).is_none() {
        return Err(DoctrinePurchaseError::UnknownDoctrine);
    }

    if inventory
        .doctrine_ids()
        .iter()
        .any(|owned_id| owned_id == doctrine_id)
    {
        return Err(DoctrinePurchaseError::AlreadyUnlocked);
    }

    if inventory.doctrine_fragments() == 0 {
        return Err(DoctrinePurchaseError::InsufficientFragments);
    }

    *inventory.doctrine_fragments_mut() -= 1;
    inventory.doctrine_ids_mut().push(doctrine_id.to_string());
    inventory.doctrine_ids_mut().sort();

    Ok(())
}

pub fn survey_progress_doctrine_multiplier(doctrine_ids: &[String], service_id: &str) -> f32 {
    doctrine_ids
        .iter()
        .filter_map(|doctrine_id| doctrine_by_id(doctrine_id))
        .filter_map(|doctrine| match doctrine.effect {
            DoctrineEffect::SurveyProgressMultiplier {
                source_service_id,
                multiplier,
            } if source_service_id == service_id => Some(multiplier),
            _ => None,
        })
        .fold(1.0, |acc, multiplier| acc * multiplier)
}

pub fn apply_frontier_charters_starting_level(
    doctrine_ids: &[String],
    planet_id: &str,
    system_id: &str,
    current_level: u8,
) -> u8 {
    if planet_id == SOLSTICE_ANCHOR_ID || !doctrine_ids.iter().any(|id| id == FRONTIER_CHARTERS_ID) {
        return current_level;
    }

    doctrine_ids
        .iter()
        .filter_map(|doctrine_id| doctrine_by_id(doctrine_id))
        .filter_map(|doctrine| match doctrine.effect {
            DoctrineEffect::NewlyDiscoveredPlanetsStartWithSystemLevel {
                system_id: target_system_id,
                level,
            } if target_system_id == system_id => Some(level),
            _ => None,
        })
        .fold(current_level, u8::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::content::doctrines::{DEEP_SURVEY_PROTOCOLS_ID, EFFICIENT_SHIFTS_ID};
    use crate::game::content::systems::REACTOR_CORE_ID;

    #[test]
    fn progression_doctrines_purchase_cannot_exceed_fragment_count() {
        let mut profile = PrestigeProfile {
            doctrine_fragments: 0,
            ..PrestigeProfile::default()
        };

        assert_eq!(
            purchase_doctrine(&mut profile, EFFICIENT_SHIFTS_ID),
            Err(DoctrinePurchaseError::InsufficientFragments)
        );

        profile.doctrine_fragments = 1;
        purchase_doctrine(&mut profile, EFFICIENT_SHIFTS_ID)
            .expect("purchase should spend available fragment");

        assert_eq!(profile.doctrine_fragments, 0);
        assert_eq!(profile.doctrine_ids, vec![EFFICIENT_SHIFTS_ID.to_string()]);
        assert_eq!(
            purchase_doctrine(&mut profile, EFFICIENT_SHIFTS_ID),
            Err(DoctrinePurchaseError::AlreadyUnlocked)
        );
    }

    #[test]
    fn progression_doctrines_effect_helpers_apply_expected_modifiers() {
        let doctrine_ids = vec![DEEP_SURVEY_PROTOCOLS_ID.to_string(), FRONTIER_CHARTERS_ID.to_string()];

        assert!((survey_progress_doctrine_multiplier(&doctrine_ids, "survey-uplink") - 1.2).abs() < 0.000_001);
        assert_eq!(
            apply_frontier_charters_starting_level(&doctrine_ids, "cinder-forge", REACTOR_CORE_ID, 1),
            2
        );
        assert_eq!(
            apply_frontier_charters_starting_level(&doctrine_ids, SOLSTICE_ANCHOR_ID, REACTOR_CORE_ID, 1),
            1
        );
    }
}
