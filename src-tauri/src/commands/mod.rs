//! Tauri command surface exposed to the SvelteKit frontend.
//!
//! Production commands are split by domain into [`progression`], [`service`],
//! [`snapshot_cmds`], and [`system`]; their inputs live in [`inputs`]. Every
//! mutating command flows through the [`crate::commit_and_emit`] helper so the
//! frontend's reactive `gameState` store receives `game://state-changed`
//! events. Frontend callers reach these commands through the
//! `gameGateway` module in `src/lib/game/api/gateway.ts`, which wraps payloads
//! in a `{ input: payload }` envelope.
//!
//! Devtools commands are debug-only and live under [`devtools`]; they are
//! gated by `#[cfg(any(debug_assertions, test))]` and stripped from release
//! builds.

/// Strongly-typed command input DTOs deserialized from the Tauri
/// `{ input: payload }` envelope.
pub(crate) mod inputs;
mod progression;
mod service;
mod snapshot_cmds;
mod system;

/// Debug-only devtools command surface; absent from release builds.
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

/// Build the standard [`crate::game::snapshot::ActionResponse`] returned by
/// every mutating command.
///
/// Bundles the success flag, an optional rejection code, and a freshly built
/// snapshot of the current run + prestige state so the frontend can apply the
/// result locally without waiting for the next backend event.
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
