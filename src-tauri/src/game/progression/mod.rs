//! Run-spanning progression systems: doctrines, prestige, and survey discovery.
//!
//! This module groups the three subsystems whose state outlives a single tick
//! cycle and (in the case of [`prestige`]) outlives the run itself:
//!
//! - [`doctrines`]: per-profile permanent unlocks bought with prestige fragments
//!   that modify simulation behavior (survey multipliers, planet starting levels).
//! - [`prestige`]: the meta-loop. Defines [`prestige::PrestigeProfile`] (what
//!   carries across resets), the eligibility gate, the fragment reward formula,
//!   and the reset that produces a fresh [`crate::game::sim::state::RunState`].
//! - [`survey`]: accumulates survey progress while the survey-uplink service
//!   runs and unlocks discoverable planets at fixed thresholds.
//!
//! All three are consumed by the simulation tick loop and by the
//! progression-domain Tauri commands (see `crate::commands::progression`,
//! private to the lib crate) and by [`crate::game::sim::state::RunState`] for
//! the per-run state these helpers mutate.

/// Doctrine inventory + purchase rules and the doctrine-driven runtime
/// modifiers (survey multipliers, planet starting levels).
pub mod doctrines;
/// Prestige profile, eligibility gate, reward formula, and run reset.
pub mod prestige;
/// Survey progress accumulation and threshold-driven planet discovery.
pub mod survey;

#[allow(unused_imports)]
pub use doctrines::{
    apply_frontier_charters_starting_level, purchase_doctrine, survey_progress_doctrine_multiplier,
    DoctrinePurchaseError,
};
#[allow(unused_imports)]
pub use prestige::{
    calculate_station_tier, doctrine_fragment_reward, evaluate_prestige_eligibility,
    execute_prestige, update_stable_power_ticks, PrestigeEligibility, PrestigeExecutionError,
    PrestigeIneligibleReason, PrestigeProfile, POWER_STABILITY_TICKS_REQUIRED,
};
#[allow(unused_imports)]
pub use survey::{accumulate_survey_progress, SurveyOutcome};
