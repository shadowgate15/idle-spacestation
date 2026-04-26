pub mod doctrines;
pub mod prestige;
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
