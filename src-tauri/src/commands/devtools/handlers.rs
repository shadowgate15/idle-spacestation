//! Tauri command handlers for the devtools overlay.
//!
//! **Debug builds only** — every `#[tauri::command]` in this file is gated by
//! `#[cfg(debug_assertions)]` and stripped from release builds, so none of these
//! functions appear in the production IPC surface. They are registered into the
//! handler bundle by the `all_commands!` macro in `src-tauri/src/lib.rs`.
//!
//! Mutating handlers funnel through
//! [`crate::commands::devtools::run_devtools_mutation`] so they share the
//! lock-order discipline (`GameState` then `LastEmittedSnapshot`) and the
//! `commit_and_emit` event-diff pipeline. While a devtools input is focused,
//! the frontend pauses snapshot apply via `gameState.deferUntilBlur(true)`
//! (see `src/routes/+layout.svelte`), so a successful mutation's emitted
//! snapshot is buffered until blur and the user's draft is preserved.

use crate::commands::devtools::apply::{
    apply_devtools_advance_ticks, apply_devtools_crew_total, apply_devtools_progression,
    apply_devtools_resources, apply_devtools_services, apply_devtools_system_levels,
    reset_devtools_session,
};
use crate::commands::devtools::inputs::{
    DevtoolsAdvanceTicksInput, DevtoolsApplyCrewInput, DevtoolsApplyProgressionInput,
    DevtoolsApplyResourcesInput, DevtoolsApplyServicesInput, DevtoolsApplySystemsInput,
    DevtoolsStateResponse, DevtoolsVisibilityInput,
};
use crate::commands::devtools::{
    current_devtools_state_response, run_devtools_mutation, update_devtools_visibility,
};
use crate::{DevtoolsState, GameState, LastEmittedSnapshot};

/// Returns the current overlay visibility plus a fresh game-state snapshot.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Frontend alias**: `gameGateway.getDevtoolsState()` (same Rust name).
/// **Mutates state**: no.
/// **Emits `game://state-changed`**: no.
///
/// Used by `DevtoolsOverlay.svelte` on mount to seed its initial state.
///
/// # Errors
/// Returns `Err(String)` only if the underlying state lock is poisoned (the
/// helper itself never fails today; the `Result` shape preserves room for
/// future failure modes).
#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_get_state(
    game_state: tauri::State<GameState>,
    devtools_state: tauri::State<DevtoolsState>,
) -> Result<DevtoolsStateResponse, String> {
    Ok(current_devtools_state_response(&game_state, &devtools_state))
}

/// Sets the overlay visibility flag and emits `devtools:visibility-changed`.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Frontend alias**: `gameGateway.setDevtoolsVisibility(input)` (same Rust name).
/// **Mutates state**: yes ([`crate::DevtoolsState`] only — game state is left
/// untouched, so `game://state-changed` is *not* fired).
/// **Emits `game://state-changed`**: no. **Emits `devtools:visibility-changed`**: yes.
///
/// # Errors
/// Returns `Err(String)` if the runtime cannot deliver the visibility-changed
/// event (bubbles up from
/// [`crate::commands::devtools::emit_devtools_visibility_changed`]).
#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_set_visibility(
    input: DevtoolsVisibilityInput,
    app: tauri::AppHandle,
) -> Result<DevtoolsStateResponse, String> {
    update_devtools_visibility(&app, input.visible)
}

/// Overwrites the materials and data resource stockpiles in [`crate::game::sim::RunState`].
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Frontend alias**: `gameGateway.applyDevtoolsResources(input)` — used by
/// the Resources panel (`src/lib/components/devtools/ResourcesPanel.svelte`).
/// **Mutates state**: yes. **Emits `game://state-changed`**: yes (via
/// [`crate::commands::devtools::run_devtools_mutation`] →
/// [`crate::commit_and_emit`]).
///
/// While a Resources panel input is focused, the frontend defers applying the
/// emitted snapshot until blur (`gameState.deferUntilBlur(true)`) so the
/// user's in-flight edit is preserved.
///
/// # Errors
/// Returns `Err(String)` only if the lock acquisition / event emit pipeline
/// fails. Validation rejections are surfaced through the action-response
/// envelope (`{ ok: false, reasonCode }`) rather than as `Err`. Possible
/// `reasonCode`s — see
/// [`crate::commands::devtools::apply::apply_devtools_resources`]:
/// - `invalid_range`: any field is negative.
#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_apply_resources(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: DevtoolsApplyResourcesInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, _, _| {
        apply_devtools_resources(run, input.materials, input.data)
    })
}

/// Sets the crew headcount in [`crate::game::sim::RunState::resources`].
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Frontend alias**: `gameGateway.applyDevtoolsCrew(input)` — used by the
/// Crew panel (`src/lib/components/devtools/CrewPanel.svelte`).
/// **Mutates state**: yes. **Emits `game://state-changed`**: yes.
///
/// Focus-deferral applies: while a Crew panel input is focused, snapshot apply
/// is paused via `gameState.deferUntilBlur(true)`.
///
/// # Errors
/// Validation rejections are returned in the action-response envelope. Possible
/// `reasonCode`s — see
/// [`crate::commands::devtools::apply::apply_devtools_crew_total`]:
/// - `invalid_range`: requested `crew_total < 1` or below current
///   `crew_assigned`.
/// - `constraint_violation`: requested `crew_total` exceeds
///   [`crate::runtime::habitat_crew_capacity`].
#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_apply_crew(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: DevtoolsApplyCrewInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, _, _| {
        apply_devtools_crew_total(run, input.crew_total)
    })
}

/// Sets per-system levels in [`crate::game::sim::RunState::systems`].
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Frontend alias**: `gameGateway.applyDevtoolsSystems(input)` — used by
/// the Systems panel (`src/lib/components/devtools/SystemsPanel.svelte`).
/// **Mutates state**: yes. **Emits `game://state-changed`**: yes.
///
/// Focus-deferral applies: while a Systems panel level input is focused,
/// snapshot apply is paused via `gameState.deferUntilBlur(true)`.
///
/// # Errors
/// Validation rejections are returned in the action-response envelope. Possible
/// `reasonCode`s — see
/// [`crate::commands::devtools::apply::apply_devtools_system_levels`]:
/// - `constraint_violation`: duplicate system id in the payload.
/// - `unknown_id`: id is not in [`crate::game::content::systems`] or not in the
///   current run state.
/// - `invalid_range`: `level < 1` or above the system's
///   [`crate::game::content::systems::SystemDefinition::progression`] max.
#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_apply_systems(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: DevtoolsApplySystemsInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, _, _| {
        apply_devtools_system_levels(run, &input.systems)
    })
}

/// Sets per-service desired_active / assigned_crew / priority in
/// [`crate::game::sim::RunState::services`].
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Frontend alias**: `gameGateway.applyDevtoolsServices(input)` — used by
/// the Services panel (`src/lib/components/devtools/ServicesPanel.svelte`).
/// **Mutates state**: yes. **Emits `game://state-changed`**: yes.
///
/// Focus-deferral applies: while a Services panel input is focused, snapshot
/// apply is paused via `gameState.deferUntilBlur(true)`.
///
/// # Errors
/// Validation rejections are returned in the action-response envelope. Possible
/// `reasonCode`s — see
/// [`crate::commands::devtools::apply::apply_devtools_services`]:
/// - `constraint_violation`: duplicate id, duplicate priority, or a priority
///   collision after merging the partial payload with unspecified services.
/// - `unknown_id`: id is not in [`crate::game::content::services`] or not in
///   the current run state.
/// - `invalid_range`: `assigned_crew` exceeds the service's `crew_required`,
///   or `priority` falls outside `1..=run_state.services.len()`.
#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_apply_services(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: DevtoolsApplyServicesInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, _, _| {
        apply_devtools_services(run, &input.services)
    })
}

/// Sets the full progression state — doctrine fragments, unlocked doctrines,
/// discovered planets, the active planet, and per-planet survey progress —
/// across both [`crate::game::sim::RunState::station`] and the
/// [`crate::game::progression::PrestigeProfile`].
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Frontend alias**: `gameGateway.applyDevtoolsProgression(input)` — used by
/// the Progression panel
/// (`src/lib/components/devtools/ProgressionPanel.svelte`).
/// **Mutates state**: yes (both run state and prestige profile).
/// **Emits `game://state-changed`**: yes.
///
/// Focus-deferral applies: while a Progression panel input is focused,
/// snapshot apply is paused via `gameState.deferUntilBlur(true)`.
///
/// # Errors
/// Validation rejections are returned in the action-response envelope. Possible
/// `reasonCode`s — see
/// [`crate::commands::devtools::apply::apply_devtools_progression`]:
/// - `constraint_violation`: duplicate doctrine/planet ids; the starter planet
///   `solstice-anchor` is missing from `discovered_planets`; or `active_planet`
///   is not in `discovered_planets`.
/// - `unknown_id`: doctrine, planet, or active-planet id not in the static
///   content tables ([`crate::game::content::doctrines`],
///   [`crate::game::content::planets`]).
/// - `invalid_range`: a `survey_progress` value falls outside `0.0..=1.0`.
#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_apply_progression(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: DevtoolsApplyProgressionInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, profile, _| {
        apply_devtools_progression(run, profile, &input)
    })
}

/// Fast-forwards the simulation by running [`tick`](crate::game::sim::tick())
/// N times and bumps the session-tick counter by the same amount.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Frontend alias**: `gameGateway.advanceDevtoolsTicks(input)` — used by the
/// Session panel (`src/lib/components/devtools/SessionPanel.svelte`).
/// **Mutates state**: yes (runs the production tick loop in-line).
/// **Emits `game://state-changed`**: yes (one final emit after all ticks
/// complete; the inner `tick` calls do not emit individually).
///
/// Focus-deferral applies: while the Session panel input is focused, snapshot
/// apply is paused via `gameState.deferUntilBlur(true)`.
///
/// # Errors
/// Validation rejections are returned in the action-response envelope. Possible
/// `reasonCode`s — see
/// [`crate::commands::devtools::apply::apply_devtools_advance_ticks`]:
/// - `invalid_range`: `count` is `0` or greater than `240` (one minute at the
///   4 Hz tick cadence).
#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_advance_ticks(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: DevtoolsAdvanceTicksInput,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, _, session_ticks| {
        apply_devtools_advance_ticks(run, input.count)?;
        *session_ticks = session_ticks.saturating_add(input.count);
        Ok(())
    })
}

/// Hard-resets [`crate::game::sim::RunState`], the
/// [`crate::game::progression::PrestigeProfile`], and the session-tick counter
/// back to their starter-fixture values, dropping all in-progress mutations.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Frontend alias**: `gameGateway.resetDevtoolsToStarter()` — invoked by the
/// Session panel (`src/lib/components/devtools/SessionPanel.svelte`).
/// **Mutates state**: yes (overwrites everything wholesale).
/// **Emits `game://state-changed`**: yes.
///
/// Always succeeds; the action-response envelope therefore never carries a
/// `reasonCode`.
///
/// # Errors
/// Returns `Err(String)` only if the lock-acquisition / event-emit pipeline
/// fails (no business-rule rejections are possible).
#[cfg(debug_assertions)]
#[tauri::command]
pub fn game_devtools_reset_to_starter(
    app: tauri::AppHandle<impl tauri::Runtime>,
    game_state: tauri::State<'_, GameState>,
    _devtools_state: tauri::State<'_, DevtoolsState>,
    last_emitted: tauri::State<'_, LastEmittedSnapshot>,
) -> Result<serde_json::Value, String> {
    run_devtools_mutation(&app, &game_state, &last_emitted, |run, profile, session_ticks| {
        reset_devtools_session(run, profile, session_ticks);
        Ok(())
    })
}
