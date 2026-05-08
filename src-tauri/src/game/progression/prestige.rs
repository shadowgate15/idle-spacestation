//! Prestige profile, eligibility gate, reward formula, and run reset.
//!
//! Prestige is the meta-loop: when a run is "complete enough" the player can
//! voluntarily reset the run for permanent benefits. This module owns:
//!
//! - [`PrestigeProfile`] — the cross-run state. Carries discovered planets,
//!   owned doctrines, fragment balance, lifetime statistics, and user settings
//!   from one run to the next. Held alongside
//!   [`crate::game::sim::state::RunState`] on the global `GameState`
//!   (defined in the lib root) and persisted via [`crate::game::persistence`].
//! - [`evaluate_prestige_eligibility`] — the gate. Three constraints:
//!   station tier ≥ 4, ≥ 2 non-starter planets discovered, and net-power
//!   stable for at least [`POWER_STABILITY_TICKS_REQUIRED`] consecutive ticks.
//! - [`doctrine_fragment_reward`] — the currency formula. Computes the number
//!   of doctrine fragments awarded by a single prestige (capped at 6).
//! - [`execute_prestige`] — the reset. Folds the current run into the profile,
//!   awards fragments, then constructs a fresh starter
//!   [`crate::game::sim::state::RunState`] that inherits only the
//!   profile-tracked fields.
//!
//! ## Reset semantics
//!
//! A successful [`execute_prestige`] **carries over** (via [`PrestigeProfile`]):
//! - All discovered planet ids (sorted, deduplicated).
//! - All owned doctrine ids (sorted, deduplicated).
//! - The doctrine fragment balance plus the freshly awarded fragments.
//! - Lifetime statistics: `lifetime_ticks`, `lifetime_prestiges`,
//!   `lifetime_data_produced`, `fastest_prestige_ticks`.
//! - User settings (e.g. `autosave_enabled`).
//!
//! A successful [`execute_prestige`] **resets** (in the new
//! [`crate::game::sim::state::RunState`]):
//! - `tick_count`, all resources (materials, data, crew assignments),
//!   `survey_progress`, station tier, every system back to level 1, every
//!   service to inactive with no crew, and all autosave bookkeeping.
//! - The active planet is forced back to
//!   [`crate::game::content::planets::SOLSTICE_ANCHOR_ID`].
//! - Power generation is recomputed from the level-1 reactor and crew
//!   capacity from the level-1 habitat ring.
//!
//! See also: the `game_execute_prestige` Tauri command in
//! `crate::commands::progression` (private to the lib crate) which wraps
//! [`execute_prestige`].

#![allow(dead_code)]

use crate::game::content::planets::SOLSTICE_ANCHOR_ID;
use crate::game::content::services::SERVICES;
use crate::game::content::systems::{
    system_by_id, SystemProgression, HABITAT_RING_ID, LOGISTICS_SPINE_ID, REACTOR_CORE_ID,
    SURVEY_ARRAY_ID,
};
use crate::game::persistence::{SaveManager, SaveManagerError, SaveSettings};
use crate::game::sim::state::{
    RunState, ServiceState, StationState, SystemState, HOUSEKEEPING_POWER_PER_SECOND,
};

/// Required count of consecutive ticks with non-negative net power before the
/// prestige eligibility gate accepts power stability.
///
/// At the simulation's 250 ms tick cadence (4 Hz) this is approximately
/// 5 minutes of continuous stable power. Tracked on
/// `RunState::consecutive_stable_power_ticks` and updated each tick by
/// [`update_stable_power_ticks`].
pub const POWER_STABILITY_TICKS_REQUIRED: u32 = 1_200;

/// Cross-run state that survives every prestige reset.
///
/// Held alongside [`RunState`] on the global `GameState` (defined in the lib
/// root) and persisted to disk by [`crate::game::persistence::SaveManager`].
/// Every field here is deliberately part of the carry-over set; anything that
/// should reset per run lives on [`RunState`] instead.
///
/// The `game_purchase_doctrine` Tauri command in
/// `crate::commands::progression` mutates this struct in place (debits
/// fragments, appends doctrine ids); the `game_execute_prestige` command
/// constructs a fresh value via [`execute_prestige`] and assigns it wholesale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrestigeProfile {
    /// Sorted, deduplicated list of every planet id ever discovered. Always
    /// includes [`SOLSTICE_ANCHOR_ID`] (the starter planet).
    pub discovered_planet_ids: Vec<String>,
    /// Sorted, deduplicated list of every owned doctrine id. Drives doctrine
    /// effects in subsequent runs.
    pub doctrine_ids: Vec<String>,
    /// Prestige fragment balance available for [`crate::game::progression::doctrines::purchase_doctrine`].
    /// One fragment is debited per purchase.
    pub doctrine_fragments: u32,
    /// Total ticks accumulated across every completed run (sum of each run's
    /// final `tick_count`). Saturating addition.
    pub lifetime_ticks: u64,
    /// Number of prestige resets the player has executed. Incremented by
    /// exactly 1 on each successful [`execute_prestige`].
    pub lifetime_prestiges: u32,
    /// Total `data` resource produced across every completed run. Saturating
    /// addition; feeds the [`doctrine_fragment_reward`] formula.
    pub lifetime_data_produced: u64,
    /// Smallest `tick_count` at which any run reached prestige. `None` until
    /// the first prestige completes; `Some(min(existing, current))` after.
    pub fastest_prestige_ticks: Option<u64>,
    /// Persisted user-level settings (e.g. autosave on/off) that travel with
    /// the profile across resets and across save/load.
    pub settings: SaveSettings,
}

impl Default for PrestigeProfile {
    fn default() -> Self {
        Self {
            discovered_planet_ids: vec![SOLSTICE_ANCHOR_ID.to_string()],
            doctrine_ids: Vec::new(),
            doctrine_fragments: 0,
            lifetime_ticks: 0,
            lifetime_prestiges: 0,
            lifetime_data_produced: 0,
            fastest_prestige_ticks: None,
            settings: SaveSettings::default(),
        }
    }
}

/// Reason a prestige attempt was rejected by the eligibility gate.
///
/// Each variant maps to a stable kebab-case string code (via [`Self::code`])
/// that the frontend converts into a typed `PrestigeRejectionCode`. Three
/// gates are evaluated in order: tier, planets, then power stability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrestigeIneligibleReason {
    /// Station tier (sum-of-system-levels minus 3, clamped 1..=4) is below 4.
    StationTierBelowFour,
    /// Fewer than two planets other than [`SOLSTICE_ANCHOR_ID`] are discovered.
    NeedsTwoNonStarterPlanets,
    /// Net power has not been non-negative for [`POWER_STABILITY_TICKS_REQUIRED`]
    /// consecutive ticks.
    UnstableNetPower,
}

impl PrestigeIneligibleReason {
    /// Stable kebab-case rejection code consumed by the frontend.
    ///
    /// The Tauri command layer surfaces this in `ActionResponse::reason_code`
    /// when [`execute_prestige`] returns
    /// [`PrestigeExecutionError::Ineligible`].
    pub(crate) fn code(&self) -> &'static str {
        match self {
            PrestigeIneligibleReason::StationTierBelowFour => "station-tier-below-four",
            PrestigeIneligibleReason::NeedsTwoNonStarterPlanets => "needs-two-non-starter-planets",
            PrestigeIneligibleReason::UnstableNetPower => "unstable-net-power",
        }
    }
}

/// Outcome of [`evaluate_prestige_eligibility`].
///
/// Always returns one of two shapes: `{ eligible: true, reason: None }` or
/// `{ eligible: false, reason: Some(_) }`. `eligible` and `reason` are never
/// inconsistent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrestigeEligibility {
    /// `true` when all three gates pass; `false` otherwise.
    pub eligible: bool,
    /// `None` when `eligible` is `true`, otherwise the first failing gate.
    pub reason: Option<PrestigeIneligibleReason>,
}

/// Failure modes for [`execute_prestige`] / [`execute_prestige_with_persistence`].
#[derive(Debug)]
pub enum PrestigeExecutionError {
    /// The eligibility gate rejected the attempt.
    Ineligible(PrestigeIneligibleReason),
    /// The pre-prestige save (only emitted by [`execute_prestige_with_persistence`])
    /// failed at the persistence layer.
    Save(SaveManagerError),
}

impl From<SaveManagerError> for PrestigeExecutionError {
    fn from(value: SaveManagerError) -> Self {
        Self::Save(value)
    }
}

/// Tick-by-tick update for the consecutive-stable-power counter.
///
/// Returns `current_ticks + 1` (saturating at `u32::MAX`) when `net_power` is
/// non-negative, or `0` when net power has gone negative this tick. Called
/// once per simulation tick; the resulting count is the input to
/// [`evaluate_prestige_eligibility`]'s power-stability gate.
pub fn update_stable_power_ticks(current_ticks: u32, net_power: f32) -> u32 {
    if net_power >= 0.0 {
        current_ticks.saturating_add(1)
    } else {
        0
    }
}

/// Compute the station tier (`1..=4`) from the current system levels.
///
/// Defined as `(sum_of_system_levels − 3).clamp(1, 4)`. With the canonical
/// four systems all at level 1 the sum is 4 and the tier is 1; raising every
/// system to level 2 produces tier 4 (the prestige prerequisite).
pub fn calculate_station_tier(run_state: &RunState) -> u8 {
    let total_levels = run_state
        .systems
        .iter()
        .map(|system| system.level as i16)
        .sum::<i16>();
    (total_levels - 3).clamp(1, 4) as u8
}

/// Evaluate whether the player can prestige right now.
///
/// Checks three gates **in order** and short-circuits on the first failure,
/// so the returned `reason` is always the first failing constraint:
/// 1. Station tier ≥ 4 — see [`calculate_station_tier`].
/// 2. At least two non-starter planets discovered (planets other than
///    [`SOLSTICE_ANCHOR_ID`]).
/// 3. `consecutive_stable_power_ticks ≥ POWER_STABILITY_TICKS_REQUIRED`.
///
/// `consecutive_stable_power_ticks` is sourced from
/// `run_state.consecutive_stable_power_ticks` by the caller (kept as a
/// parameter so the simulation tick can pass an updated value within the
/// same tick).
pub fn evaluate_prestige_eligibility(
    run_state: &RunState,
    consecutive_stable_power_ticks: u32,
) -> PrestigeEligibility {
    if calculate_station_tier(run_state) < 4 {
        return PrestigeEligibility {
            eligible: false,
            reason: Some(PrestigeIneligibleReason::StationTierBelowFour),
        };
    }

    let non_starter_planet_count = run_state
        .station
        .discovered_planet_ids
        .iter()
        .filter(|planet_id| planet_id.as_str() != SOLSTICE_ANCHOR_ID)
        .count();
    if non_starter_planet_count < 2 {
        return PrestigeEligibility {
            eligible: false,
            reason: Some(PrestigeIneligibleReason::NeedsTwoNonStarterPlanets),
        };
    }

    if consecutive_stable_power_ticks < POWER_STABILITY_TICKS_REQUIRED {
        return PrestigeEligibility {
            eligible: false,
            reason: Some(PrestigeIneligibleReason::UnstableNetPower),
        };
    }

    PrestigeEligibility {
        eligible: true,
        reason: None,
    }
}

/// Compute the doctrine-fragment reward for a single prestige.
///
/// Formula:
///
/// ```text
/// fragments = min(6, 1 + (discovered_planet_count − 1) + (lifetime_data_produced / 1500))
/// ```
///
/// Components:
/// - **Base**: every prestige is worth at least `1` fragment.
/// - **Discovery**: one extra fragment per non-starter planet discovered.
///   Saturating subtraction guards against an empty discovered list.
/// - **Data**: one extra fragment per `1_500` units of lifetime data produced
///   (integer division; remainder is dropped).
///
/// The total is capped at `6` to prevent runaway scaling. `lifetime_data_produced`
/// is the post-update profile value (i.e. **including** the current run's
/// contribution), matching what [`execute_prestige`] passes in.
pub fn doctrine_fragment_reward(
    discovered_planet_count: usize,
    lifetime_data_produced: u64,
) -> u32 {
    let discovered_component = discovered_planet_count.saturating_sub(1) as u32;
    let data_component = (lifetime_data_produced / 1_500) as u32;
    (1 + discovered_component + data_component).min(6)
}

/// Execute a prestige reset.
///
/// On success returns `(reset_run_state, next_profile, fragments_awarded)`:
/// - `reset_run_state` is a brand-new starter [`RunState`] derived from
///   `next_profile` (see [`reset_run_state`] for what carries over).
/// - `next_profile` folds the current run into `profile`: extends discovered
///   planets and doctrines (sorted/deduplicated), saturating-adds
///   `tick_count` and `lifetime_data_produced`, increments
///   `lifetime_prestiges`, refreshes `fastest_prestige_ticks` to
///   `min(existing, run_state.tick_count)`, and credits `fragments_awarded`
///   on top of the current run's fragment balance.
/// - `fragments_awarded` is the [`doctrine_fragment_reward`] for this run,
///   computed against the **post-update** profile.
///
/// # Errors
/// Returns [`PrestigeExecutionError::Ineligible`] with the first failing
/// reason from [`evaluate_prestige_eligibility`]. This function never returns
/// [`PrestigeExecutionError::Save`] — that is reserved for
/// [`execute_prestige_with_persistence`].
pub fn execute_prestige(
    run_state: &RunState,
    profile: &PrestigeProfile,
    consecutive_stable_power_ticks: u32,
) -> Result<(RunState, PrestigeProfile, u32), PrestigeExecutionError> {
    let eligibility = evaluate_prestige_eligibility(run_state, consecutive_stable_power_ticks);
    if let Some(reason) = eligibility.reason {
        return Err(PrestigeExecutionError::Ineligible(reason));
    }

    let mut next_profile = profile.clone();
    next_profile.discovered_planet_ids =
        sorted_unique(run_state.station.discovered_planet_ids.clone());
    next_profile.doctrine_ids = sorted_unique(run_state.station.doctrine_ids.clone());
    next_profile.lifetime_ticks = next_profile
        .lifetime_ticks
        .saturating_add(run_state.tick_count);
    next_profile.lifetime_prestiges = next_profile.lifetime_prestiges.saturating_add(1);
    next_profile.lifetime_data_produced = next_profile
        .lifetime_data_produced
        .saturating_add(run_state.lifetime_data_produced);
    next_profile.fastest_prestige_ticks = match next_profile.fastest_prestige_ticks {
        Some(existing) => Some(existing.min(run_state.tick_count)),
        None => Some(run_state.tick_count),
    };

    let fragments_awarded = doctrine_fragment_reward(
        next_profile.discovered_planet_ids.len(),
        next_profile.lifetime_data_produced,
    );
    next_profile.doctrine_fragments = run_state
        .station
        .doctrine_fragments
        .saturating_add(fragments_awarded);

    let reset_run_state = reset_run_state(&next_profile);
    Ok((reset_run_state, next_profile, fragments_awarded))
}

/// Persist a pre-prestige snapshot, then call [`execute_prestige`].
///
/// The persistence step (`save_before_prestige`) writes a recovery save so
/// the player can roll back if the reset is undesired. On any persistence
/// failure the function returns [`PrestigeExecutionError::Save`] **without**
/// performing the reset; the run is left untouched.
///
/// # Errors
/// - [`PrestigeExecutionError::Save`] when the save layer rejects the
///   pre-prestige write (the persistence subsystem is scaffolded; production
///   commands currently bypass this entry point).
/// - [`PrestigeExecutionError::Ineligible`] when the gate rejects (forwarded
///   from [`execute_prestige`]).
pub fn execute_prestige_with_persistence(
    save_manager: &SaveManager,
    run_state: &RunState,
    profile: &PrestigeProfile,
    consecutive_stable_power_ticks: u32,
) -> Result<(RunState, PrestigeProfile, u32), PrestigeExecutionError> {
    save_manager.save_before_prestige(
        run_state,
        &crate::game::persistence::ProfileState {
            discovered_planet_ids: profile.discovered_planet_ids.clone(),
            doctrine_ids: profile.doctrine_ids.clone(),
            doctrine_fragments: profile.doctrine_fragments,
            lifetime_ticks: profile.lifetime_ticks,
            lifetime_prestiges: profile.lifetime_prestiges,
        },
        &profile.settings,
    )?;

    execute_prestige(run_state, profile, consecutive_stable_power_ticks)
}

/// Build a fresh starter [`RunState`] from a post-prestige `profile`.
///
/// All run-scoped fields are initialized to their starter values: `tick_count`
/// is `0`, every system is rebuilt at level 1
/// ([`REACTOR_CORE_ID`], [`HABITAT_RING_ID`], [`LOGISTICS_SPINE_ID`],
/// [`SURVEY_ARRAY_ID`]), every service is inactive with no crew, and
/// resources are seeded from the level-1 reactor power output and level-1
/// habitat crew capacity (with [`HOUSEKEEPING_POWER_PER_SECOND`] reserved for
/// housekeeping). Only the profile-tracked fields (planets, doctrines,
/// fragments) are copied onto the new station; everything else resets.
fn reset_run_state(profile: &PrestigeProfile) -> RunState {
    let reactor_level = level_one_reactor_power_output();
    let crew_total = level_one_habitat_crew_capacity();

    RunState {
        tick_count: 0,
        station: StationState {
            active_planet_id: SOLSTICE_ANCHOR_ID.to_string(),
            discovered_planet_ids: profile.discovered_planet_ids.clone(),
            doctrine_ids: profile.doctrine_ids.clone(),
            doctrine_fragments: profile.doctrine_fragments,
            survey_progress: 0.0,
            station_tier: 1,
        },
        resources: crate::game::sim::ResourceState {
            power_generated: reactor_level,
            power_reserved: HOUSEKEEPING_POWER_PER_SECOND,
            power_available: reactor_level - HOUSEKEEPING_POWER_PER_SECOND,
            materials: 0.0,
            data: 0.0,
            crew_total,
            crew_assigned: 0,
            crew_available: crew_total,
        },
        services: SERVICES
            .iter()
            .enumerate()
            .map(|(index, service)| ServiceState {
                service_id: service.id.to_string(),
                desired_active: false,
                is_active: false,
                is_paused: false,
                pause_reason: None,
                priority: (index + 1) as u8,
                assigned_crew: 0,
            })
            .collect(),
        systems: vec![
            SystemState::new(REACTOR_CORE_ID, 1),
            SystemState::new(HABITAT_RING_ID, 1),
            SystemState::new(LOGISTICS_SPINE_ID, 1),
            SystemState::new(SURVEY_ARRAY_ID, 1),
        ],
        consecutive_stable_power_ticks: 0,
        lifetime_data_produced: 0,
        autosave_due: false,
        autosave_count: 0,
        last_autosave_tick: None,
        prestige_eligible: false,
    }
}

/// Power output of the reactor at level 1, looked up from the static system
/// catalog. Used by [`reset_run_state`] to seed the post-prestige run.
///
/// # Panics
/// Panics via `expect` if the reactor-core system is missing from the
/// catalog, or via `unreachable!` if its progression variant is not
/// [`SystemProgression::ReactorCore`]. Both indicate a content bug.
fn level_one_reactor_power_output() -> f32 {
    match system_by_id(REACTOR_CORE_ID)
        .expect("reactor-core system must exist")
        .progression
    {
        SystemProgression::ReactorCore(levels) => levels[0].power_output,
        _ => unreachable!("reactor-core progression must use reactor levels"),
    }
}

/// Crew capacity of the habitat ring at level 1, looked up from the static
/// system catalog. Used by [`reset_run_state`] to seed the post-prestige run.
///
/// # Panics
/// Panics via `expect` if the habitat-ring system is missing from the
/// catalog, or via `unreachable!` if its progression variant is not
/// [`SystemProgression::HabitatRing`]. Both indicate a content bug.
fn level_one_habitat_crew_capacity() -> u8 {
    match system_by_id(HABITAT_RING_ID)
        .expect("habitat-ring system must exist")
        .progression
    {
        SystemProgression::HabitatRing(levels) => levels[0].crew_capacity,
        _ => unreachable!("habitat-ring progression must use habitat levels"),
    }
}

/// Sort `values` lexicographically and remove consecutive duplicates.
///
/// Used by [`execute_prestige`] to canonicalise carried-over planet and
/// doctrine id lists so equality comparisons against persisted state are
/// stable regardless of insertion order.
fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::content::doctrines::EFFICIENT_SHIFTS_ID;
    use crate::game::content::planets::{AURORA_PIER_ID, CINDER_FORGE_ID};

    fn eligible_fixture() -> RunState {
        let mut run_state = RunState::starter_fixture();
        run_state.tick_count = 1_440;
        run_state.resources.data = 3_200.0;
        run_state.lifetime_data_produced = 3_200;
        run_state.resources.materials = 275.0;
        run_state.resources.crew_assigned = 5;
        run_state.resources.crew_available = 1;
        run_state.resources.power_generated = 20.0;
        run_state.resources.power_reserved = 18.0;
        run_state.resources.power_available = 2.0;
        run_state.station.discovered_planet_ids = vec![
            AURORA_PIER_ID.to_string(),
            CINDER_FORGE_ID.to_string(),
            SOLSTICE_ANCHOR_ID.to_string(),
        ];
        run_state.station.doctrine_ids = vec![EFFICIENT_SHIFTS_ID.to_string()];
        run_state.station.doctrine_fragments = 2;
        run_state.station.survey_progress = 1_550.0;
        run_state.services[0].desired_active = true;
        run_state.services[0].is_active = true;
        run_state.services[0].assigned_crew = 2;
        run_state.services[1].desired_active = true;
        run_state.services[1].is_active = true;
        run_state.services[1].assigned_crew = 1;
        run_state.services[2].desired_active = true;
        run_state.services[2].is_active = true;
        run_state.services[2].assigned_crew = 1;
        run_state.services[3].desired_active = true;
        run_state.services[3].is_active = true;
        run_state.services[3].assigned_crew = 1;
        run_state.systems = vec![
            SystemState::new(REACTOR_CORE_ID, 2),
            SystemState::new(HABITAT_RING_ID, 2),
            SystemState::new(LOGISTICS_SPINE_ID, 2),
            SystemState::new(SURVEY_ARRAY_ID, 1),
        ];
        run_state.station.station_tier = calculate_station_tier(&run_state);
        run_state.prestige_eligible = true;
        run_state
    }

    #[test]
    fn progression_prestige_gate_uses_exact_contract() {
        let eligible = eligible_fixture();
        let mut below_tier = eligible_fixture();
        below_tier.systems = vec![
            SystemState::new(REACTOR_CORE_ID, 1),
            SystemState::new(HABITAT_RING_ID, 2),
            SystemState::new(LOGISTICS_SPINE_ID, 2),
            SystemState::new(SURVEY_ARRAY_ID, 1),
        ];

        let mut only_starter = eligible_fixture();
        only_starter.station.discovered_planet_ids = vec![SOLSTICE_ANCHOR_ID.to_string()];

        let unstable_power = eligible_fixture();

        let eligible_result =
            evaluate_prestige_eligibility(&eligible, POWER_STABILITY_TICKS_REQUIRED);
        let tier_result =
            evaluate_prestige_eligibility(&below_tier, POWER_STABILITY_TICKS_REQUIRED);
        let planet_result =
            evaluate_prestige_eligibility(&only_starter, POWER_STABILITY_TICKS_REQUIRED);
        let power_result =
            evaluate_prestige_eligibility(&unstable_power, POWER_STABILITY_TICKS_REQUIRED - 1);

        println!(
            "prestige-gate\nfixture=eligible eligible={} reason={:?}\nfixture=below-tier eligible={} reason={:?}\nfixture=starter-only eligible={} reason={:?}\nfixture=unstable-power eligible={} reason={:?}",
            eligible_result.eligible,
            eligible_result.reason,
            tier_result.eligible,
            tier_result.reason,
            planet_result.eligible,
            planet_result.reason,
            power_result.eligible,
            power_result.reason,
        );

        assert_eq!(
            eligible_result,
            PrestigeEligibility {
                eligible: true,
                reason: None
            }
        );
        assert_eq!(
            tier_result.reason,
            Some(PrestigeIneligibleReason::StationTierBelowFour)
        );
        assert_eq!(
            planet_result.reason,
            Some(PrestigeIneligibleReason::NeedsTwoNonStarterPlanets)
        );
        assert_eq!(
            power_result.reason,
            Some(PrestigeIneligibleReason::UnstableNetPower)
        );
    }

    #[test]
    fn progression_prestige_reward_formula_matches_plan_and_caps_at_six() {
        assert_eq!(doctrine_fragment_reward(1, 0), 1);
        assert_eq!(doctrine_fragment_reward(3, 1_499), 3);
        assert_eq!(doctrine_fragment_reward(3, 3_000), 5);
        assert_eq!(doctrine_fragment_reward(3, 15_000), 6);
    }

    #[test]
    fn progression_prestige_reset_persists_only_profile_fields() {
        let run_state = eligible_fixture();
        let profile = PrestigeProfile {
            discovered_planet_ids: vec![SOLSTICE_ANCHOR_ID.to_string()],
            doctrine_ids: Vec::new(),
            doctrine_fragments: 0,
            lifetime_ticks: 900,
            lifetime_prestiges: 2,
            lifetime_data_produced: 1_600,
            fastest_prestige_ticks: Some(2_000),
            settings: SaveSettings {
                autosave_enabled: false,
            },
        };

        let (reset_run_state, next_profile, fragments_awarded) =
            execute_prestige(&run_state, &profile, POWER_STABILITY_TICKS_REQUIRED)
                .expect("eligible fixture should prestige");

        println!(
            "prestige-reset-table\nfield\tbefore\tafter\tstatus\ndiscovered_planets\t{:?}\t{:?}\tpersisted\ndoctrines\t{:?}\t{:?}\tpersisted\ndoctrine_fragments\t{}\t{}\tpersisted+reward\nlifetime_ticks\t{}\t{}\tpersisted\nlifetime_prestiges\t{}\t{}\tpersisted\nlifetime_data_produced\t{}\t{}\tpersisted\nfastest_prestige_ticks\t{:?}\t{:?}\tpersisted\nsettings.autosave_enabled\t{}\t{}\tpersisted\nrun.tick_count\t{}\t{}\treset\nrun.materials\t{}\t{}\treset\nrun.data\t{}\t{}\treset\nrun.survey_progress\t{}\t{}\treset\nrun.active_services\t{}\t{}\treset\nrun.crew_assigned\t{}\t{}\treset\nrun.autosave_count\t{}\t{}\treset",
            run_state.station.discovered_planet_ids,
            next_profile.discovered_planet_ids,
            run_state.station.doctrine_ids,
            next_profile.doctrine_ids,
            run_state.station.doctrine_fragments,
            next_profile.doctrine_fragments,
            profile.lifetime_ticks,
            next_profile.lifetime_ticks,
            profile.lifetime_prestiges,
            next_profile.lifetime_prestiges,
            profile.lifetime_data_produced,
            next_profile.lifetime_data_produced,
            profile.fastest_prestige_ticks,
            next_profile.fastest_prestige_ticks,
            profile.settings.autosave_enabled,
            next_profile.settings.autosave_enabled,
            run_state.tick_count,
            reset_run_state.tick_count,
            run_state.resources.materials,
            reset_run_state.resources.materials,
            run_state.resources.data,
            reset_run_state.resources.data,
            run_state.station.survey_progress,
            reset_run_state.station.survey_progress,
            run_state.services.iter().filter(|service| service.is_active).count(),
            reset_run_state.services.iter().filter(|service| service.is_active).count(),
            run_state.resources.crew_assigned,
            reset_run_state.resources.crew_assigned,
            run_state.autosave_count,
            reset_run_state.autosave_count,
        );

        assert_eq!(fragments_awarded, 6);
        assert_eq!(
            next_profile.discovered_planet_ids,
            vec![
                AURORA_PIER_ID.to_string(),
                CINDER_FORGE_ID.to_string(),
                SOLSTICE_ANCHOR_ID.to_string(),
            ]
        );
        assert_eq!(
            next_profile.doctrine_ids,
            vec![EFFICIENT_SHIFTS_ID.to_string()]
        );
        assert_eq!(next_profile.doctrine_fragments, 8);
        assert_eq!(next_profile.lifetime_ticks, 2_340);
        assert_eq!(next_profile.lifetime_prestiges, 3);
        assert_eq!(next_profile.lifetime_data_produced, 4_800);
        assert_eq!(next_profile.fastest_prestige_ticks, Some(1_440));
        assert!(!next_profile.settings.autosave_enabled);

        assert_eq!(reset_run_state.tick_count, 0);
        assert_eq!(reset_run_state.station.active_planet_id, SOLSTICE_ANCHOR_ID);
        assert_eq!(reset_run_state.station.survey_progress, 0.0);
        assert_eq!(reset_run_state.station.station_tier, 1);
        assert_eq!(reset_run_state.resources.materials, 0.0);
        assert_eq!(reset_run_state.resources.data, 0.0);
        assert_eq!(reset_run_state.resources.crew_assigned, 0);
        assert_eq!(
            reset_run_state.resources.crew_available,
            reset_run_state.resources.crew_total
        );
        assert_eq!(reset_run_state.autosave_count, 0);
        assert_eq!(reset_run_state.last_autosave_tick, None);
        assert!(reset_run_state.services.iter().all(|service| {
            !service.desired_active
                && !service.is_active
                && !service.is_paused
                && service.assigned_crew == 0
        }));
        assert!(reset_run_state
            .systems
            .iter()
            .all(|system| system.level == 1));
    }
}
