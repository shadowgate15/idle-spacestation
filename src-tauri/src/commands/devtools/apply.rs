//! Pure mutation helpers for the devtools commands.
//!
//! **Debug builds only** — the entire `commands::devtools` module is gated by
//! `#[cfg(debug_assertions)]` and stripped from release builds, so none of
//! these helpers are reachable from production code paths.
//!
//! Each helper validates its input first and returns `Err(&'static str)` with a
//! stable rejection code on failure (mapped to a typed
//! `Devtools*RejectionCode` in `gateway.ts`); on success it mutates the
//! supplied [`crate::game::sim::RunState`] (and, for progression,
//! [`crate::game::progression::PrestigeProfile`]) in place. **No locks** are
//! acquired here and **no events** are emitted — those concerns belong to
//! [`crate::commands::devtools::run_devtools_mutation`], which wraps every
//! handler and triggers `commit_and_emit` after a successful mutation.
//!
//! While a devtools input is focused, the frontend pauses snapshot apply via
//! `gameState.deferUntilBlur(true)` (see `src/routes/+layout.svelte`); the
//! emit fired post-mutation is therefore buffered until blur and the user's
//! in-flight edit is preserved.

use crate::commands::devtools::inputs::{
    DevtoolsApplyProgressionInput, DevtoolsApplySystemEntry, DevtoolsServiceEntry,
};
use crate::game::content::doctrines::doctrine_by_id;
use crate::game::content::planets::{planet_by_id, SOLSTICE_ANCHOR_ID};
use crate::game::content::services::service_by_id;
use crate::game::content::systems::system_by_id;
use crate::game::progression::PrestigeProfile;
use crate::game::sim::{tick, RunState};
use crate::runtime::habitat_crew_capacity;
use std::collections::HashSet;
use std::hash::Hash;

/// Validates that all items in an iterator are unique (no duplicates).
/// Returns an error code if any duplicate is found.
fn ensure_unique<T, I>(items: I) -> Result<(), &'static str>
where
    T: Eq + Hash,
    I: IntoIterator<Item = T>,
{
    let mut seen = HashSet::new();
    for item in items {
        if !seen.insert(item) {
            return Err("constraint_violation");
        }
    }
    Ok(())
}

/// Validates and overwrites the materials + data resource stockpiles.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes (writes [`RunState::resources`]).
/// **Emits event**: no — the wrapping
/// [`crate::commands::devtools::run_devtools_mutation`] handles that.
///
/// # Errors
/// - `invalid_range`: `materials < 0.0` or `data < 0.0` (also catches `NaN`,
///   since the comparisons use `>= 0.0` rather than `< 0.0`).
pub(crate) fn apply_devtools_resources(
    run_state: &mut RunState,
    materials: f32,
    data: f32,
) -> Result<(), &'static str> {
    if !(materials >= 0.0 && data >= 0.0) {
        return Err("invalid_range");
    }

    run_state.resources.materials = materials;
    run_state.resources.data = data;
    Ok(())
}

/// Validates and overwrites the crew headcount in [`RunState::resources`].
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes (writes `crew_total`).
/// **Emits event**: no.
///
/// Caller is expected to invoke [`crate::runtime::refresh_runtime_state`]
/// afterward so derived fields like `crew_available` reflect the new total —
/// [`crate::commands::devtools::run_devtools_mutation`] handles this.
///
/// # Errors
/// - `invalid_range`: `crew_total < 1` or `crew_total < crew_assigned`.
/// - `constraint_violation`: `crew_total > habitat_crew_capacity(run_state)`.
pub(crate) fn apply_devtools_crew_total(
    run_state: &mut RunState,
    crew_total: u8,
) -> Result<(), &'static str> {
    if crew_total < 1 || crew_total < run_state.resources.crew_assigned {
        return Err("invalid_range");
    }

    if crew_total > habitat_crew_capacity(run_state) {
        return Err("constraint_violation");
    }

    run_state.resources.crew_total = crew_total;
    Ok(())
}

/// Looks up the maximum level for a system id from the static content table.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: no.
/// Returns `None` if `system_id` is not in
/// [`crate::game::content::systems`].
pub(crate) fn system_max_level(system_id: &str) -> Option<u8> {
    system_by_id(system_id).map(|system| system.progression.max_level())
}

/// Validates and overwrites per-system levels in [`RunState::systems`].
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes (writes each `level` field).
/// **Emits event**: no.
///
/// Validation runs over the entire payload before any mutation, so a single
/// rejection leaves the run state untouched.
///
/// # Errors
/// - `constraint_violation`: duplicate id within `systems`.
/// - `unknown_id`: id not in [`crate::game::content::systems`] or absent from
///   the run state.
/// - `invalid_range`: `level < 1` or `level > system_max_level(id)`.
pub(crate) fn apply_devtools_system_levels(
    run_state: &mut RunState,
    systems: &[DevtoolsApplySystemEntry],
) -> Result<(), &'static str> {
    ensure_unique(systems.iter().map(|e| e.id.as_str()))?;

    for entry in systems {
        let Some(max_level) = system_max_level(&entry.id) else {
            return Err("unknown_id");
        };

        if entry.level < 1 || entry.level > max_level {
            return Err("invalid_range");
        }

        if !run_state
            .systems
            .iter()
            .any(|system| system.system_id == entry.id)
        {
            return Err("unknown_id");
        }
    }

    for entry in systems {
        let system = run_state
            .systems
            .iter_mut()
            .find(|system| system.system_id == entry.id)
            .expect("validated system id must exist in run state");
        system.level = entry.level;
    }

    Ok(())
}

/// Validates and applies a batch of per-service overrides to
/// [`RunState::services`].
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes (writes `desired_active`, `assigned_crew`,
/// `priority`, then re-sorts `services` by priority).
/// **Emits event**: no.
///
/// Validation is two-pass: per-entry checks (id, range, crew limit) run first,
/// then the helper simulates the full priority assignment across *all* services
/// (including those omitted from the payload) to detect priority collisions
/// before mutating anything.
///
/// # Errors
/// - `constraint_violation`: duplicate id, duplicate explicit priority, or a
///   priority collision after merging the partial payload with the unspecified
///   services that keep their existing priority.
/// - `unknown_id`: id not in [`crate::game::content::services`] or absent from
///   the run state.
/// - `invalid_range`: `assigned_crew > definition.crew_required`, or
///   `priority < 1` or `priority > run_state.services.len()`.
pub(crate) fn apply_devtools_services(
    run_state: &mut RunState,
    services: &[DevtoolsServiceEntry],
) -> Result<(), &'static str> {
    ensure_unique(services.iter().map(|e| e.id.as_str()))?;
    ensure_unique(services.iter().map(|e| e.priority))?;

    for entry in services {
        let Some(definition) = service_by_id(&entry.id) else {
            return Err("unknown_id");
        };

        if !run_state
            .services
            .iter()
            .any(|service| service.service_id == entry.id)
        {
            return Err("unknown_id");
        }

        if entry.assigned_crew > definition.crew_required {
            return Err("invalid_range");
        }

        if entry.priority < 1 || entry.priority > run_state.services.len() as u8 {
            return Err("invalid_range");
        }
    }

    let mut resulting_priorities = HashSet::new();
    for service in &run_state.services {
        let next_priority = services
            .iter()
            .find(|entry| entry.id == service.service_id)
            .map(|entry| entry.priority)
            .unwrap_or(service.priority);

        if !resulting_priorities.insert(next_priority) {
            return Err("constraint_violation");
        }
    }

    for entry in services {
        let service = run_state
            .services
            .iter_mut()
            .find(|service| service.service_id == entry.id)
            .expect("validated service id must exist in run state");
        service.desired_active = entry.desired_active;
        service.assigned_crew = entry.assigned_crew;
        service.priority = entry.priority;
    }

    run_state.services.sort_by_key(|service| service.priority);
    Ok(())
}

/// Collapses a per-planet survey-progress map into the single scalar
/// `survey_progress` field tracked on `RunState::station`.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: no — pure function.
///
/// Each entry is scaled by [`crate::game::content::planets::survey_threshold`]
/// to convert the `0.0..=1.0` per-planet progress into the absolute scale used
/// by the station counter, then the helper takes the running maximum so a
/// later, smaller survey can't *lower* the displayed progress. A floor is also
/// applied for any planet already in `discovered_planets`, ensuring discovered
/// planets always show at least their unlock threshold.
pub(crate) fn total_survey_progress_from_map(
    discovered_planets: &[String],
    survey_progress: &std::collections::HashMap<String, f32>,
) -> f32 {
    use crate::game::content::planets::survey_threshold;

    let discovered_floor = discovered_planets
        .iter()
        .filter_map(|planet_id| survey_threshold(planet_id))
        .fold(0.0, f32::max);

    survey_progress
        .iter()
        .filter_map(|(planet_id, progress)| {
            survey_threshold(planet_id).map(|threshold| progress * threshold)
        })
        .fold(discovered_floor, f32::max)
}

/// Validates and overwrites every progression-related field in
/// [`RunState::station`] and the [`PrestigeProfile`].
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes (writes both `RunState::station` and
/// `PrestigeProfile`'s doctrine + planet fields).
/// **Emits event**: no.
///
/// Doctrine and planet id lists are de-duplicated and sorted before being
/// written so the canonical form is stable for the diff cache used by
/// [`crate::commit_and_emit`].
///
/// # Errors
/// - `constraint_violation`: duplicate doctrine or planet id; the starter
///   planet [`crate::game::content::planets::SOLSTICE_ANCHOR_ID`] is missing
///   from `discovered_planets`; or `active_planet` is not in
///   `discovered_planets`.
/// - `unknown_id`: doctrine, planet, active-planet, or survey-key id missing
///   from the static content tables.
/// - `invalid_range`: a `survey_progress` value falls outside `0.0..=1.0`.
pub(crate) fn apply_devtools_progression(
    run_state: &mut RunState,
    profile: &mut PrestigeProfile,
    input: &DevtoolsApplyProgressionInput,
) -> Result<(), &'static str> {
    ensure_unique(input.unlocked_doctrines.iter().map(|s| s.as_str()))?;
    for doctrine_id in &input.unlocked_doctrines {
        if doctrine_by_id(doctrine_id).is_none() {
            return Err("unknown_id");
        }
    }

    ensure_unique(input.discovered_planets.iter().map(|s| s.as_str()))?;
    for planet_id in &input.discovered_planets {
        if planet_by_id(planet_id).is_none() {
            return Err("unknown_id");
        }
    }

    if !input
        .discovered_planets
        .iter()
        .any(|planet_id| planet_id == SOLSTICE_ANCHOR_ID)
    {
        return Err("constraint_violation");
    }

    if planet_by_id(&input.active_planet).is_none() {
        return Err("unknown_id");
    }

    if !input
        .discovered_planets
        .iter()
        .any(|planet_id| planet_id == &input.active_planet)
    {
        return Err("constraint_violation");
    }

    for (planet_id, progress) in &input.survey_progress {
        if planet_by_id(planet_id).is_none() {
            return Err("unknown_id");
        }

        if !(*progress >= 0.0 && *progress <= 1.0) {
            return Err("invalid_range");
        }
    }

    let mut unlocked_doctrines = input.unlocked_doctrines.clone();
    unlocked_doctrines.sort();

    let mut discovered_planets = input.discovered_planets.clone();
    discovered_planets.sort();

    run_state.station.doctrine_fragments = input.doctrine_fragments;
    run_state.station.doctrine_ids = unlocked_doctrines.clone();
    run_state.station.discovered_planet_ids = discovered_planets.clone();
    run_state.station.active_planet_id = input.active_planet.clone();
    run_state.station.survey_progress =
        total_survey_progress_from_map(&discovered_planets, &input.survey_progress);

    profile.doctrine_fragments = input.doctrine_fragments;
    profile.doctrine_ids = unlocked_doctrines;
    profile.discovered_planet_ids = discovered_planets;

    Ok(())
}

/// Validates `count` and runs the production [`tick`](crate::game::sim::tick())
/// `count` times.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes (delegates to `tick`, which mutates the run state in
/// the same way the background tick thread does).
/// **Emits event**: no — the wrapping
/// [`crate::commands::devtools::run_devtools_mutation`] fires a single
/// `game://state-changed` after the loop completes.
///
/// # Errors
/// - `invalid_range`: `count` is outside `1..=240` (one minute at the
///   4 Hz tick cadence in `lib.rs::TICK_INTERVAL_MS`).
pub(crate) fn apply_devtools_advance_ticks(
    run_state: &mut RunState,
    count: u32,
) -> Result<(), &'static str> {
    if !(1..=240).contains(&count) {
        return Err("invalid_range");
    }

    for _ in 0..count {
        tick(run_state);
    }

    Ok(())
}

/// Hard-resets the run state, the prestige profile, and the session-tick
/// counter back to the starter fixture used at first launch.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes (overwrites all three arguments wholesale).
/// **Emits event**: no.
///
/// Always succeeds — there is no validation to perform — so the helper returns
/// `()` rather than `Result`.
pub(crate) fn reset_devtools_session(
    run_state: &mut RunState,
    profile: &mut PrestigeProfile,
    session_ticks: &mut u32,
) {
    *run_state = RunState::starter_fixture();
    *profile = PrestigeProfile::default();
    *session_ticks = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::content::systems::{HABITAT_RING_ID, REACTOR_CORE_ID};
    use crate::runtime::refresh_runtime_state;

    #[test]
    fn apply_resources_success() {
        let mut run_state = RunState::starter_fixture();

        apply_devtools_resources(&mut run_state, 250.0, 15.5).expect("resources should apply");

        assert_eq!(run_state.resources.materials, 250.0);
        assert_eq!(run_state.resources.data, 15.5);
    }

    #[test]
    fn apply_resources_rejects_negative() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_resources(&mut run_state, -1.0, 0.0)
            .expect_err("negative resources should be rejected");

        assert_eq!(error, "invalid_range");
        assert_eq!(run_state.resources.materials, 120.0);
    }

    #[test]
    fn apply_crew_success() {
        let mut run_state = RunState::starter_fixture();
        run_state.services[0].assigned_crew = 2;

        apply_devtools_crew_total(&mut run_state, 4).expect("crew total should apply");
        refresh_runtime_state(&mut run_state);

        assert_eq!(run_state.resources.crew_total, 4);
        assert_eq!(run_state.resources.crew_available, 2);
    }

    #[test]
    fn apply_crew_rejects_below_assigned() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_crew_total(&mut run_state, 1)
            .expect_err("crew below assigned should be rejected");

        assert_eq!(error, "invalid_range");
        assert_eq!(run_state.resources.crew_total, 6);
    }

    #[test]
    fn apply_systems_sets_reactor_level() {
        let mut run_state = RunState::starter_fixture();

        apply_devtools_system_levels(
            &mut run_state,
            &[DevtoolsApplySystemEntry {
                id: REACTOR_CORE_ID.to_string(),
                level: 2,
            }],
        )
        .expect("system levels should apply");
        refresh_runtime_state(&mut run_state);

        assert_eq!(run_state.system_level(REACTOR_CORE_ID), Some(2));
        assert_eq!(run_state.resources.power_generated, 12.0);
    }

    #[test]
    fn apply_systems_rejects_unknown_id() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_system_levels(
            &mut run_state,
            &[DevtoolsApplySystemEntry {
                id: "unknown-system".to_string(),
                level: 1,
            }],
        )
        .expect_err("unknown system should be rejected");

        assert_eq!(error, "unknown_id");
    }

    #[test]
    fn apply_systems_rejects_out_of_range_level() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_system_levels(
            &mut run_state,
            &[DevtoolsApplySystemEntry {
                id: HABITAT_RING_ID.to_string(),
                level: 5,
            }],
        )
        .expect_err("out of range level should be rejected");

        assert_eq!(error, "invalid_range");
    }

    #[test]
    fn apply_systems_rejects_duplicate_ids() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_system_levels(
            &mut run_state,
            &[
                DevtoolsApplySystemEntry {
                    id: REACTOR_CORE_ID.to_string(),
                    level: 2,
                },
                DevtoolsApplySystemEntry {
                    id: REACTOR_CORE_ID.to_string(),
                    level: 3,
                },
            ],
        )
        .expect_err("duplicate system ids should be rejected");

        assert_eq!(error, "constraint_violation");
        assert_eq!(run_state.system_level(REACTOR_CORE_ID), Some(1));
    }

    #[test]
    fn apply_services_success() {
        let mut run_state = RunState::starter_fixture();

        apply_devtools_services(
            &mut run_state,
            &[
                DevtoolsServiceEntry {
                    id: "solar-harvester".to_string(),
                    desired_active: false,
                    assigned_crew: 0,
                    priority: 6,
                },
                DevtoolsServiceEntry {
                    id: "ore-reclaimer".to_string(),
                    desired_active: true,
                    assigned_crew: 1,
                    priority: 1,
                },
                DevtoolsServiceEntry {
                    id: "survey-uplink".to_string(),
                    desired_active: true,
                    assigned_crew: 1,
                    priority: 2,
                },
                DevtoolsServiceEntry {
                    id: "maintenance-bay".to_string(),
                    desired_active: false,
                    assigned_crew: 0,
                    priority: 3,
                },
                DevtoolsServiceEntry {
                    id: "command-relay".to_string(),
                    desired_active: true,
                    assigned_crew: 1,
                    priority: 4,
                },
                DevtoolsServiceEntry {
                    id: "fabrication-loop".to_string(),
                    desired_active: false,
                    assigned_crew: 0,
                    priority: 5,
                },
            ],
        )
        .expect("services should apply");

        assert_eq!(run_state.services[0].service_id, "ore-reclaimer");
        assert_eq!(run_state.services[0].priority, 1);
        assert!(run_state.services[0].desired_active);
        assert_eq!(run_state.services[0].assigned_crew, 1);
        assert_eq!(run_state.services[5].service_id, "solar-harvester");
        assert_eq!(run_state.services[5].priority, 6);
        assert!(!run_state.services[5].desired_active);
    }

    #[test]
    fn apply_services_rejects_unknown_id() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_services(
            &mut run_state,
            &[DevtoolsServiceEntry {
                id: "nonexistent-service".to_string(),
                desired_active: true,
                assigned_crew: 0,
                priority: 1,
            }],
        )
        .expect_err("unknown service should be rejected");

        assert_eq!(error, "unknown_id");
    }

    #[test]
    fn apply_services_rejects_duplicate_priorities() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_services(
            &mut run_state,
            &[
                DevtoolsServiceEntry {
                    id: "solar-harvester".to_string(),
                    desired_active: true,
                    assigned_crew: 2,
                    priority: 1,
                },
                DevtoolsServiceEntry {
                    id: "ore-reclaimer".to_string(),
                    desired_active: true,
                    assigned_crew: 1,
                    priority: 1,
                },
            ],
        )
        .expect_err("duplicate priorities should be rejected");

        assert_eq!(error, "constraint_violation");
    }

    #[test]
    fn apply_services_rejects_invalid_crew_assignment() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_services(
            &mut run_state,
            &[DevtoolsServiceEntry {
                id: "ore-reclaimer".to_string(),
                desired_active: true,
                assigned_crew: 2,
                priority: 2,
            }],
        )
        .expect_err("crew assignment above requirement should be rejected");

        assert_eq!(error, "invalid_range");
    }

    #[test]
    fn apply_progression_success() {
        let mut run_state = RunState::starter_fixture();
        let mut profile = PrestigeProfile::default();
        let mut survey_progress = std::collections::HashMap::new();
        survey_progress.insert("cinder-forge".to_string(), 1.0);
        survey_progress.insert("aurora-pier".to_string(), 0.5);

        apply_devtools_progression(
            &mut run_state,
            &mut profile,
            &DevtoolsApplyProgressionInput {
                doctrine_fragments: 3,
                unlocked_doctrines: vec![
                    "hardened-relays".to_string(),
                    "efficient-shifts".to_string(),
                ],
                discovered_planets: vec!["cinder-forge".to_string(), "solstice-anchor".to_string()],
                active_planet: "cinder-forge".to_string(),
                survey_progress,
            },
        )
        .expect("progression should apply");

        assert_eq!(run_state.station.doctrine_fragments, 3);
        assert_eq!(
            run_state.station.doctrine_ids,
            vec![
                "efficient-shifts".to_string(),
                "hardened-relays".to_string()
            ]
        );
        assert_eq!(
            run_state.station.discovered_planet_ids,
            vec!["cinder-forge".to_string(), "solstice-anchor".to_string()]
        );
        assert_eq!(run_state.station.active_planet_id, "cinder-forge");
        assert!((run_state.station.survey_progress - 700.0).abs() < 0.000_001);
        assert_eq!(profile.doctrine_fragments, 3);
        assert_eq!(profile.doctrine_ids, run_state.station.doctrine_ids);
        assert_eq!(
            profile.discovered_planet_ids,
            run_state.station.discovered_planet_ids
        );
    }

    #[test]
    fn apply_progression_rejects_active_planet_not_in_discovered() {
        let mut run_state = RunState::starter_fixture();
        let mut profile = PrestigeProfile::default();

        let error = apply_devtools_progression(
            &mut run_state,
            &mut profile,
            &DevtoolsApplyProgressionInput {
                doctrine_fragments: 0,
                unlocked_doctrines: Vec::new(),
                discovered_planets: vec!["solstice-anchor".to_string()],
                active_planet: "cinder-forge".to_string(),
                survey_progress: std::collections::HashMap::new(),
            },
        )
        .expect_err("active planet outside discovered set should be rejected");

        assert_eq!(error, "constraint_violation");
    }

    #[test]
    fn apply_progression_rejects_missing_solstice_anchor() {
        let mut run_state = RunState::starter_fixture();
        let mut profile = PrestigeProfile::default();

        let error = apply_devtools_progression(
            &mut run_state,
            &mut profile,
            &DevtoolsApplyProgressionInput {
                doctrine_fragments: 0,
                unlocked_doctrines: Vec::new(),
                discovered_planets: vec!["cinder-forge".to_string()],
                active_planet: "cinder-forge".to_string(),
                survey_progress: std::collections::HashMap::new(),
            },
        )
        .expect_err("starter planet must remain discovered");

        assert_eq!(error, "constraint_violation");
    }

    #[test]
    fn apply_progression_rejects_unknown_doctrine_id() {
        let mut run_state = RunState::starter_fixture();
        let mut profile = PrestigeProfile::default();

        let error = apply_devtools_progression(
            &mut run_state,
            &mut profile,
            &DevtoolsApplyProgressionInput {
                doctrine_fragments: 1,
                unlocked_doctrines: vec!["made-up-doctrine".to_string()],
                discovered_planets: vec!["solstice-anchor".to_string()],
                active_planet: "solstice-anchor".to_string(),
                survey_progress: std::collections::HashMap::new(),
            },
        )
        .expect_err("unknown doctrine should be rejected");

        assert_eq!(error, "unknown_id");
    }

    #[test]
    fn advance_ticks_success_boundary_low() {
        let mut run_state = RunState::starter_fixture();

        apply_devtools_advance_ticks(&mut run_state, 1).expect("one tick should be accepted");

        assert_eq!(run_state.tick_count, 1);
    }

    #[test]
    fn advance_ticks_success_boundary_high() {
        let mut run_state = RunState::starter_fixture();

        apply_devtools_advance_ticks(&mut run_state, 240).expect("240 ticks should be accepted");

        assert_eq!(run_state.tick_count, 240);
    }

    #[test]
    fn advance_ticks_rejects_zero() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_advance_ticks(&mut run_state, 0)
            .expect_err("zero ticks should be rejected");

        assert_eq!(error, "invalid_range");
        assert_eq!(run_state.tick_count, 0);
    }

    #[test]
    fn advance_ticks_rejects_over_240() {
        let mut run_state = RunState::starter_fixture();

        let error = apply_devtools_advance_ticks(&mut run_state, 241)
            .expect_err("counts above 240 should be rejected");

        assert_eq!(error, "invalid_range");
        assert_eq!(run_state.tick_count, 0);
    }

    #[test]
    fn reset_to_starter_restores_starter_fixture() {
        let mut run_state = RunState::starter_fixture();
        let mut profile = PrestigeProfile::default();
        let mut session_ticks = 99;

        run_state.tick_count = 42;
        run_state.station.active_planet_id = "cinder-forge".to_string();
        run_state.station.discovered_planet_ids =
            vec!["solstice-anchor".to_string(), "cinder-forge".to_string()];
        run_state.station.doctrine_ids = vec!["efficient-shifts".to_string()];
        run_state.station.doctrine_fragments = 4;
        run_state.station.survey_progress = 800.0;
        profile.discovered_planet_ids =
            vec!["solstice-anchor".to_string(), "aurora-pier".to_string()];
        profile.doctrine_ids = vec!["hardened-relays".to_string()];
        profile.doctrine_fragments = 2;

        reset_devtools_session(&mut run_state, &mut profile, &mut session_ticks);

        assert_eq!(run_state, RunState::starter_fixture());
        assert_eq!(profile, PrestigeProfile::default());
        assert_eq!(session_ticks, 0);
    }
}
