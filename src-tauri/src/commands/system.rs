//! System-domain Tauri command handlers.
//!
//! Currently exposes [`game_upgrade_system`], which spends materials to bump a
//! station system by one level and refreshes runtime power/crew capacities.
//!
//! See also: [`crate::commands`].

use crate::commands::action_response;
use crate::commands::inputs::UpgradeSystemInput;
use crate::game::content::systems::system_by_id_required;
use crate::game::snapshot::ActionResponse;
use crate::runtime::refresh_runtime_state;
use crate::{commit_and_emit, GameState, LastEmittedSnapshot};

/// Upgrade a station system by one level, charging the upgrade's material cost.
///
/// **Frontend alias**: `game_upgrade_system` (same as Rust name)
/// **Mutates state**: yes
/// **Emits `game://state-changed`**: yes (via [`commit_and_emit`])
///
/// On success, decrements `resources.materials`, increments the system's
/// `level` (saturating), and re-projects runtime state via
/// [`refresh_runtime_state`] before emitting the change to the frontend.
///
/// # Errors
/// Returns an `ActionResponse { ok: false, reason_code: Some(_) }` (mapped to
/// `SystemUpgradeRejectionCode` on the frontend) for one of:
/// - `unknown-system`: no system in `RunState` matches the requested id.
/// - `max-level`: the system is already at its highest configured level.
/// - `insufficient-materials`: the player cannot afford the next-level cost.
#[tauri::command]
pub fn game_upgrade_system(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: UpgradeSystemInput,
    state: tauri::State<GameState>,
    last_emitted: tauri::State<LastEmittedSnapshot>,
) -> ActionResponse {
    let mut guard = state.lock();

    let system_index = match guard
        .run
        .systems
        .iter()
        .position(|system| system.system_id == input.system_id)
    {
        Some(index) => index,
        None => return action_response(&guard.run, &guard.profile, false, Some("unknown-system")),
    };

    let current_level = guard.run.systems[system_index].level;
    let upgrade_cost = system_by_id_required(&input.system_id)
        .progression
        .upgrade_cost_at(current_level);

    let upgrade_cost = match upgrade_cost {
        Some(cost) => cost,
        None => return action_response(&guard.run, &guard.profile, false, Some("max-level")),
    };

    if guard.run.resources.materials < upgrade_cost as f32 {
        return action_response(&guard.run, &guard.profile, false, Some("insufficient-materials"));
    }

    guard.run.resources.materials -= upgrade_cost as f32;
    guard.run.systems[system_index].level = guard.run.systems[system_index].level.saturating_add(1);
    refresh_runtime_state(&mut guard.run);
    let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
    action_response(&guard.run, &guard.profile, true, None)
}
