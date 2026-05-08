//! Doctrine inventory, purchase rules, and doctrine-driven runtime modifiers.
//!
//! Doctrines are permanent per-profile unlocks bought with prestige fragments.
//! Once owned they apply for every subsequent run via the helper functions in
//! this module (e.g. [`survey_progress_doctrine_multiplier`],
//! [`apply_frontier_charters_starting_level`]).
//!
//! The inventory abstraction (`DoctrineInventory`) lets `purchase_doctrine`
//! mutate either the live [`PrestigeProfile`] or the persisted
//! [`crate::game::persistence::save::ProfileState`] through the same code
//! path, keeping save/load and runtime in sync.
//!
//! See also: the `game_purchase_doctrine` Tauri command in
//! `crate::commands::progression` (private to the lib crate) which wraps
//! [`purchase_doctrine`], and [`crate::game::content::doctrines`] for the
//! static doctrine catalog.

#![allow(dead_code)]

use crate::game::content::doctrines::{doctrine_by_id, DoctrineEffect, FRONTIER_CHARTERS_ID};
use crate::game::content::planets::SOLSTICE_ANCHOR_ID;
use crate::game::persistence::save::ProfileState;
use crate::game::progression::prestige::PrestigeProfile;

/// Reasons a [`purchase_doctrine`] call can fail.
///
/// These map 1:1 to the `reason_code` strings returned by the
/// `game_purchase_doctrine` Tauri command in `crate::commands::progression`
/// (kebab-case form).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoctrinePurchaseError {
    /// The requested doctrine id does not exist in the static catalog
    /// ([`crate::game::content::doctrines`]).
    UnknownDoctrine,
    /// The inventory already owns this doctrine; doctrines cannot be bought twice.
    AlreadyUnlocked,
    /// The inventory has zero fragments, so the unit-cost purchase cannot proceed.
    InsufficientFragments,
}

/// Storage-agnostic doctrine inventory accessor.
///
/// Both the in-memory [`PrestigeProfile`] (live game state) and the persisted
/// [`ProfileState`] (save file) satisfy this trait, allowing the same purchase
/// routine to keep the live inventory and the on-disk inventory in lockstep.
pub trait DoctrineInventory {
    /// Sorted list of owned doctrine ids.
    fn doctrine_ids(&self) -> &[String];
    /// Mutable owned-doctrines list. [`purchase_doctrine`] keeps this sorted.
    fn doctrine_ids_mut(&mut self) -> &mut Vec<String>;
    /// Current prestige fragment balance available for purchases.
    fn doctrine_fragments(&self) -> u32;
    /// Mutable fragment balance. Each successful purchase debits exactly 1.
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

/// Spend one fragment to permanently unlock `doctrine_id` on `inventory`.
///
/// On success debits exactly one fragment, appends the id to the owned list,
/// and re-sorts the list so equality checks against persisted inventories stay
/// stable.
///
/// # Errors
/// - [`DoctrinePurchaseError::UnknownDoctrine`] if the id is not in the catalog.
/// - [`DoctrinePurchaseError::AlreadyUnlocked`] if the inventory already owns it.
/// - [`DoctrinePurchaseError::InsufficientFragments`] if the balance is zero.
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

/// Multiplicative survey-progress modifier from owned doctrines for a service.
///
/// Walks the owned doctrines, keeps every
/// [`DoctrineEffect::SurveyProgressMultiplier`] whose
/// `source_service_id` matches `service_id`, and folds them with `*`. Returns
/// `1.0` when no matching effects exist (the no-op multiplier). Used by
/// [`crate::game::progression::survey::accumulate_survey_progress`] to scale
/// per-tick progress.
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

/// Compute the starting system level for a newly discovered planet, honoring
/// the Frontier Charters doctrine.
///
/// Returns `current_level` unchanged for the starter planet
/// ([`SOLSTICE_ANCHOR_ID`]) and whenever Frontier Charters
/// ([`FRONTIER_CHARTERS_ID`]) is not owned. Otherwise scans every
/// [`DoctrineEffect::NewlyDiscoveredPlanetsStartWithSystemLevel`] effect whose
/// `system_id` matches and returns the maximum of `current_level` and the
/// effect's prescribed level. Multiple Frontier-style effects compose by
/// `max`, never by sum.
pub fn apply_frontier_charters_starting_level(
    doctrine_ids: &[String],
    planet_id: &str,
    system_id: &str,
    current_level: u8,
) -> u8 {
    if planet_id == SOLSTICE_ANCHOR_ID || !doctrine_ids.iter().any(|id| id == FRONTIER_CHARTERS_ID)
    {
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
        let doctrine_ids = vec![
            DEEP_SURVEY_PROTOCOLS_ID.to_string(),
            FRONTIER_CHARTERS_ID.to_string(),
        ];

        assert!(
            (survey_progress_doctrine_multiplier(&doctrine_ids, "survey-uplink") - 1.2).abs()
                < 0.000_001
        );
        assert_eq!(
            apply_frontier_charters_starting_level(
                &doctrine_ids,
                "cinder-forge",
                REACTOR_CORE_ID,
                1
            ),
            2
        );
        assert_eq!(
            apply_frontier_charters_starting_level(
                &doctrine_ids,
                SOLSTICE_ANCHOR_ID,
                REACTOR_CORE_ID,
                1
            ),
            1
        );
    }
}
