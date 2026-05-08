pub(crate) mod inputs;
mod progression;
mod service;
mod snapshot_cmds;
mod system;

#[cfg(any(debug_assertions, test))]
pub(crate) mod devtools;

pub use progression::{game_execute_prestige, game_purchase_doctrine};
pub use service::{game_assign_service_crew, game_reprioritize_service, game_toggle_service};
pub use snapshot_cmds::{game_get_snapshot, game_request_load, game_request_save, game_start_survey};
pub use system::game_upgrade_system;

#[cfg(debug_assertions)]
pub use devtools::handlers::{
    game_devtools_advance_ticks, game_devtools_apply_crew, game_devtools_apply_progression,
    game_devtools_apply_resources, game_devtools_apply_services, game_devtools_apply_systems,
    game_devtools_get_state, game_devtools_reset_to_starter, game_devtools_set_visibility,
};

use crate::game::progression::PrestigeProfile;
use crate::game::sim::RunState;

pub(crate) fn action_response(
    run_state: &RunState,
    profile: &PrestigeProfile,
    ok: bool,
    reason_code: Option<&str>,
) -> crate::game::snapshot::ActionResponse {
    crate::game::snapshot::ActionResponse {
        ok,
        snapshot: crate::game::snapshot::build_snapshot(run_state, profile),
        reason_code: reason_code.map(str::to_string),
    }
}
