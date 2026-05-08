//! IPC payload types for the devtools commands and overlay.
//!
//! **Debug builds only** — the entire `commands::devtools` module is stripped
//! from release via `#[cfg(debug_assertions)]`, so these DTOs are not part of
//! the production IPC surface. Field renaming uses
//! `#[serde(rename_all = "camelCase")]` to match the JavaScript naming
//! conventions used by `gameGateway` and the Svelte panel state machines under
//! `src/lib/components/devtools/`.

use serde::{Deserialize, Serialize};

use crate::game::snapshot::RawGameSnapshot;

/// Payload for `game_devtools_set_visibility`: a single boolean indicating the
/// desired overlay visibility.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsVisibilityInput {
    pub(crate) visible: bool,
}

/// Combined response for `game_devtools_get_state` / `game_devtools_set_visibility`:
/// the current overlay flag plus a fresh snapshot the frontend can apply.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
#[derive(Serialize)]
pub struct DevtoolsStateResponse {
    pub(crate) visible: bool,
    pub(crate) snapshot: RawGameSnapshot,
}

/// Payload for `game_devtools_apply_resources`: target values for the two
/// stockpile resources tracked in [`crate::game::sim::RunState::resources`].
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// Both fields must be `>= 0.0`; negative values are rejected with
/// `invalid_range`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsApplyResourcesInput {
    pub(crate) materials: f32,
    pub(crate) data: f32,
}

/// Payload for `game_devtools_apply_crew`: the desired crew headcount.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// Must be at least 1, must not drop below currently-assigned crew, and must
/// not exceed the habitat ring capacity reported by
/// [`crate::runtime::habitat_crew_capacity`].
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsApplyCrewInput {
    pub(crate) crew_total: u8,
}

/// One row in the systems-panel apply payload: target level for a specific
/// system identifier.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsApplySystemEntry {
    pub(crate) id: String,
    pub(crate) level: u8,
}

/// Payload for `game_devtools_apply_systems`: a batch of per-system level
/// overrides. Duplicate ids are rejected with `constraint_violation`; unknown
/// ids or out-of-range levels with `unknown_id` / `invalid_range`.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsApplySystemsInput {
    pub(crate) systems: Vec<DevtoolsApplySystemEntry>,
}

/// One row in the services-panel apply payload: desired activation, crew
/// assignment, and execution priority for a service.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsServiceEntry {
    pub(crate) id: String,
    pub(crate) desired_active: bool,
    pub(crate) assigned_crew: u8,
    pub(crate) priority: u8,
}

/// Payload for `game_devtools_apply_services`: a batch of per-service
/// configuration rows. Validation enforces unique ids, unique priorities, crew
/// counts within the service's `crew_required`, and priorities within
/// `1..=run_state.services.len()`.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsApplyServicesInput {
    pub(crate) services: Vec<DevtoolsServiceEntry>,
}

/// Payload for `game_devtools_apply_progression`: the full progression-panel
/// state — doctrine fragments, unlocked doctrine ids, discovered planets, the
/// active planet, and per-planet survey progress (`0.0..=1.0`).
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// The starter planet (`solstice-anchor`) must remain in `discovered_planets`,
/// and `active_planet` must appear in that list.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsApplyProgressionInput {
    pub(crate) doctrine_fragments: u32,
    pub(crate) unlocked_doctrines: Vec<String>,
    pub(crate) discovered_planets: Vec<String>,
    pub(crate) active_planet: String,
    pub(crate) survey_progress: std::collections::HashMap<String, f32>,
}

/// Payload for `game_devtools_advance_ticks`: number of simulation ticks to
/// fast-forward, clamped to `1..=240` (one minute at the 4 Hz tick cadence).
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsAdvanceTicksInput {
    pub(crate) count: u32,
}

/// Payload for the `devtools:visibility-changed` Tauri event.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// Mirrors [`DevtoolsVisibilityInput`] on the wire; the frontend uses it to
/// reconcile overlay state when the visibility flips outside its own UI (e.g.
/// from the native Debug menu).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DevtoolsVisibilityChangedEvent {
    pub(crate) visible: bool,
}
