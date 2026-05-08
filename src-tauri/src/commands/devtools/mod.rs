//! Devtools command module — overlay visibility and live state-mutation helpers.
//!
//! **Debug builds only** — every public item in this module (and its submodules
//! `apply`, `handlers`, `inputs`) is gated behind `#[cfg(debug_assertions)]`
//! and therefore stripped from release builds. The lone exception is the
//! release-build `devtools_enabled` stub, which always returns `false` so that
//! callers can ask "are devtools available?" without conditional compilation.
//!
//! # Responsibilities
//! - Track the overlay's visibility flag in [`crate::DevtoolsState`] and emit the
//!   `devtools:visibility-changed` event whenever it flips.
//! - Wrap mutating helpers in `run_devtools_mutation`, which preserves the
//!   `GameState` → `LastEmittedSnapshot` lock order required by
//!   [`crate::commit_and_emit`] and emits `game://state-changed` on success.
//! - Install the native "Debug → Toggle Game State Overlay" menu via
//!   `install_debug_menu`.
//!
//! # Frontend coupling
//! The Svelte devtools overlay (`src/lib/components/DevtoolsOverlay.svelte` plus
//! the six panels under `src/lib/components/devtools/`) calls these commands
//! through `gameGateway` (`src/lib/game/api/gateway.ts`). While an overlay input
//! is focused, `+layout.svelte` calls `gameState.deferUntilBlur(true)` so that
//! inbound `game://state-changed` events are buffered and don't clobber the
//! user's in-flight edit.
//!
//! See also: [`crate::commands`] for production (release-safe) command handlers.

pub(crate) mod apply;
pub(crate) mod handlers;
pub(crate) mod inputs;

use crate::commands::devtools::inputs::{DevtoolsStateResponse, DevtoolsVisibilityChangedEvent};
use crate::game::progression::PrestigeProfile;
use crate::game::sim::RunState;
use crate::{DevtoolsState, GameState, LastEmittedSnapshot};
use tauri::Emitter;

#[cfg(debug_assertions)]
use tauri::menu::{MenuBuilder, MenuItem, SubmenuBuilder};
#[cfg(debug_assertions)]
use tauri::Manager;

/// Native menu id for the "Debug → Toggle Game State Overlay" item installed by
/// [`install_debug_menu`]. Must match the string compared in `on_menu_event`.
#[cfg(debug_assertions)]
const DEVTOOLS_TOGGLE_OVERLAY_MENU_ID: &str = "devtools-toggle-overlay";
/// Tauri event name fired when the devtools overlay visibility flips.
///
/// Frontend listens via `gameGateway.subscribeToDevtoolsVisibilityChanges()`.
#[cfg(debug_assertions)]
const DEVTOOLS_VISIBILITY_CHANGED_EVENT: &str = "devtools:visibility-changed";

/// Returns the current overlay visibility flag held in [`crate::DevtoolsState`].
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: no.
#[cfg(debug_assertions)]
pub(crate) fn read_devtools_visibility(devtools_state: &DevtoolsState) -> bool {
    *devtools_state.lock()
}

/// Overwrites the overlay visibility flag and returns the new value.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes (only [`crate::DevtoolsState`], not the game state).
/// **Emits event**: no — callers must invoke [`emit_devtools_visibility_changed`]
/// (or [`update_devtools_visibility`]) to notify the frontend.
#[cfg(debug_assertions)]
pub(crate) fn set_devtools_visibility_state(devtools_state: &DevtoolsState, visible: bool) -> bool {
    let mut guard = devtools_state.lock();
    *guard = visible;
    *guard
}

/// Flips the overlay visibility flag in place and returns the new value.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes ([`crate::DevtoolsState`] only).
/// **Emits event**: no.
///
/// The `#[cfg_attr(not(test), allow(dead_code))]` attribute suppresses an unused
/// warning in non-test debug builds: production callers always go through
/// [`toggle_devtools_visibility`], which both flips and emits.
#[cfg(debug_assertions)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn toggle_devtools_visibility_state(devtools_state: &DevtoolsState) -> bool {
    let mut guard = devtools_state.lock();
    *guard = !*guard;
    *guard
}

/// Builds the serializable payload for the `devtools:visibility-changed` event.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
#[cfg(debug_assertions)]
pub(crate) fn devtools_visibility_payload(visible: bool) -> DevtoolsVisibilityChangedEvent {
    DevtoolsVisibilityChangedEvent { visible }
}

/// Snapshots the current game state and pairs it with the supplied visibility
/// flag, producing the response shape consumed by `game_devtools_get_state` and
/// `game_devtools_set_visibility`.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: no — acquires the [`GameState`] mutex read-only.
#[cfg(debug_assertions)]
pub(crate) fn build_devtools_state_response(
    game_state: &GameState,
    visible: bool,
) -> DevtoolsStateResponse {
    let guard = game_state.lock();
    DevtoolsStateResponse {
        visible,
        snapshot: crate::game::snapshot::build_snapshot(&guard.run, &guard.profile),
    }
}

/// Convenience wrapper that reads the current visibility flag and pairs it with
/// a fresh game-state snapshot.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: no.
#[cfg(debug_assertions)]
pub(crate) fn current_devtools_state_response(
    game_state: &GameState,
    devtools_state: &DevtoolsState,
) -> DevtoolsStateResponse {
    build_devtools_state_response(game_state, read_devtools_visibility(devtools_state))
}

/// Emits the `devtools:visibility-changed` Tauri event with the supplied flag.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: no.
/// **Emits event**: yes (`devtools:visibility-changed`).
///
/// # Errors
/// Returns `Err(String)` containing the formatted Tauri error if the runtime
/// rejects the emit (window destroyed, etc.).
#[cfg(debug_assertions)]
pub(crate) fn emit_devtools_visibility_changed<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    visible: bool,
) -> Result<(), String> {
    app.emit(
        DEVTOOLS_VISIBILITY_CHANGED_EVENT,
        devtools_visibility_payload(visible),
    )
    .map_err(|error| error.to_string())
}

/// Sets the overlay visibility, builds the response snapshot, then emits the
/// visibility-changed event so the frontend store stays in sync.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes ([`crate::DevtoolsState`]).
/// **Emits event**: yes (`devtools:visibility-changed`).
///
/// # Errors
/// Returns `Err(String)` from [`emit_devtools_visibility_changed`] when the
/// runtime cannot deliver the event.
#[cfg(debug_assertions)]
pub(crate) fn update_devtools_visibility<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    visible: bool,
) -> Result<DevtoolsStateResponse, String> {
    let devtools_state = app.state::<DevtoolsState>();
    let visible = set_devtools_visibility_state(&devtools_state, visible);
    let game_state = app.state::<GameState>();
    let response = build_devtools_state_response(&game_state, visible);
    emit_devtools_visibility_changed(app, visible)?;
    Ok(response)
}

/// Flips the overlay visibility flag and pushes the new state to the frontend.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes ([`crate::DevtoolsState`]).
/// **Emits event**: yes (`devtools:visibility-changed`).
///
/// Invoked by the native "Debug → Toggle Game State Overlay" menu item.
///
/// # Errors
/// Propagates from [`update_devtools_visibility`].
#[cfg(debug_assertions)]
pub(crate) fn toggle_devtools_visibility<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<DevtoolsStateResponse, String> {
    let devtools_state = app.state::<DevtoolsState>();
    let visible = !read_devtools_visibility(&devtools_state);
    update_devtools_visibility(app, visible)
}

/// Installs the native "Debug → Toggle Game State Overlay" menu and wires its
/// click handler to [`toggle_devtools_visibility`].
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes (registers a Tauri menu and event handler on `app`).
///
/// Called once from `lib.rs::run()`'s `setup` closure.
///
/// # Errors
/// Returns `tauri::Error` if menu/submenu construction or `app.set_menu` fails.
#[cfg(debug_assertions)]
pub(crate) fn install_debug_menu<R: tauri::Runtime>(app: &mut tauri::App<R>) -> tauri::Result<()> {
    let toggle_overlay = MenuItem::with_id(
        app,
        DEVTOOLS_TOGGLE_OVERLAY_MENU_ID,
        "Toggle Game State Overlay",
        true,
        None::<&str>,
    )?;
    let debug_menu = SubmenuBuilder::new(app, "Debug")
        .item(&toggle_overlay)
        .build()?;
    let menu = MenuBuilder::new(app).items(&[&debug_menu]).build()?;

    app.set_menu(menu)?;
    app.on_menu_event(|app_handle, event| {
        if event.id().0.as_str() == DEVTOOLS_TOGGLE_OVERLAY_MENU_ID {
            let _ = toggle_devtools_visibility(app_handle);
        }
    });

    Ok(())
}

/// Returns `true` in debug builds — devtools commands are wired up.
///
/// **Debug builds only** variant: stripped from release via
/// `#[cfg(debug_assertions)]`. The release variant immediately below always
/// returns `false`, so callers can use a single non-cfg call site.
///
/// `#[cfg_attr(not(test), allow(dead_code))]` silences an unused warning when
/// the binary doesn't actually call the helper outside tests.
#[cfg(debug_assertions)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn devtools_enabled() -> bool {
    true
}

/// Returns `false` in release builds — devtools commands are stripped.
///
/// Release counterpart to the `#[cfg(debug_assertions)]` variant above.
#[cfg(not(debug_assertions))]
pub(crate) fn devtools_enabled() -> bool {
    false
}

/// Builds the success envelope returned by every devtools mutation: an `ok:true`
/// flag plus a fresh `RawGameSnapshot` so the frontend can apply the post-write
/// state immediately (no need to wait for the next tick's emit).
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: no.
#[cfg(debug_assertions)]
pub(crate) fn devtools_action_success(
    run_state: &RunState,
    profile: &PrestigeProfile,
) -> serde_json::Value {
    serde_json::json!({
        "ok": true,
        "snapshot": crate::game::snapshot::build_snapshot(run_state, profile),
    })
}

/// Builds the failure envelope returned when a devtools mutation rejects its
/// input: `ok:false`, a `reasonCode` mapped to one of the typed rejection codes
/// in `gateway.ts`, plus the unchanged snapshot.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: no.
#[cfg(debug_assertions)]
pub(crate) fn devtools_action_failure(
    run_state: &RunState,
    profile: &PrestigeProfile,
    reason_code: &str,
) -> serde_json::Value {
    serde_json::json!({
        "ok": false,
        "reasonCode": reason_code,
        "snapshot": crate::game::snapshot::build_snapshot(run_state, profile),
    })
}

/// Lock → mutate → refresh → emit → respond pipeline for devtools mutators.
/// On `Err(reason_code)` no event is emitted (state unchanged); on `Ok` the
/// helper preserves the documented `GameState` → `LastEmittedSnapshot` lock
/// order required by `commit_and_emit`.
///
/// **Debug builds only**: stripped from release via `#[cfg(debug_assertions)]`.
/// **Mutates state**: yes (acquires [`GameState`] mutably, runs `mutate`,
/// refreshes runtime projections, then triggers [`crate::commit_and_emit`]).
/// **Emits event**: yes — `game://state-changed` is fired via
/// [`crate::commit_and_emit`] when the mutation succeeds and the diff cache
/// reports a change. On `Err(reason_code)` the helper short-circuits to a
/// failure envelope and emits **nothing** (state is unchanged).
///
/// # Lock-order contract
/// [`crate::commit_and_emit`] acquires [`LastEmittedSnapshot`] *after* this
/// function has dropped the [`GameState`] guard (the guard goes out of scope at
/// the end of the function body before `commit_and_emit` runs). Reordering
/// these acquisitions will deadlock — see the project AGENTS.md anti-patterns.
///
/// # Frontend interaction
/// Successful mutations push a snapshot via `game://state-changed`. While a
/// devtools input is focused the layout calls
/// `gameState.deferUntilBlur(true)` so the inbound snapshot is buffered until
/// blur, preventing the user's in-flight edit from being clobbered.
///
/// # Errors
/// Returns `Err(String)` only when something below the helper itself fails
/// (the mutation closure's `Err(&'static str)` is folded into an `Ok(failure
/// envelope)` instead, mirroring the action-response shape used elsewhere).
#[cfg(debug_assertions)]
pub(crate) fn run_devtools_mutation<R, F>(
    app: &tauri::AppHandle<R>,
    game_state: &GameState,
    last_emitted: &LastEmittedSnapshot,
    mutate: F,
) -> Result<serde_json::Value, String>
where
    R: tauri::Runtime,
    F: FnOnce(&mut RunState, &mut PrestigeProfile, &mut u32) -> Result<(), &'static str>,
{
    let mut guard = game_state.lock();
    let crate::GameRunState {
        run,
        profile,
        session_ticks,
    } = &mut *guard;

    match mutate(run, profile, session_ticks) {
        Ok(()) => {
            crate::runtime::refresh_runtime_state(run);
            let _ = crate::commit_and_emit(app, run, profile, last_emitted);
            Ok(devtools_action_success(run, profile))
        }
        Err(reason_code) => Ok(devtools_action_failure(run, profile, reason_code)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn devtools_visibility_toggle_flips_separate_mutex_state() {
        let devtools_state = DevtoolsState(Mutex::new(false));

        assert!(!read_devtools_visibility(&devtools_state));
        assert!(toggle_devtools_visibility_state(&devtools_state));
        assert!(read_devtools_visibility(&devtools_state));
        assert!(!toggle_devtools_visibility_state(&devtools_state));
        assert!(!read_devtools_visibility(&devtools_state));
        assert!(set_devtools_visibility_state(&devtools_state, true));
        assert!(read_devtools_visibility(&devtools_state));
    }

    #[test]
    fn devtools_visibility_event_payload_matches_frontend_contract() {
        let payload = serde_json::to_value(devtools_visibility_payload(true))
            .expect("event payload should serialize");

        assert_eq!(payload, serde_json::json!({ "visible": true }));
    }

    #[test]
    fn devtools_enabled_helper_matches_build_gating() {
        assert_eq!(devtools_enabled(), cfg!(debug_assertions));
    }
}
