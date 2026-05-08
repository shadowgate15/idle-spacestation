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

#[cfg(debug_assertions)]
const DEVTOOLS_TOGGLE_OVERLAY_MENU_ID: &str = "devtools-toggle-overlay";
#[cfg(debug_assertions)]
const DEVTOOLS_VISIBILITY_CHANGED_EVENT: &str = "devtools:visibility-changed";

#[cfg(debug_assertions)]
pub(crate) fn read_devtools_visibility(devtools_state: &DevtoolsState) -> bool {
    *devtools_state.lock()
}

#[cfg(debug_assertions)]
pub(crate) fn set_devtools_visibility_state(devtools_state: &DevtoolsState, visible: bool) -> bool {
    let mut guard = devtools_state.lock();
    *guard = visible;
    *guard
}

#[cfg(debug_assertions)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn toggle_devtools_visibility_state(devtools_state: &DevtoolsState) -> bool {
    let mut guard = devtools_state.lock();
    *guard = !*guard;
    *guard
}

#[cfg(debug_assertions)]
pub(crate) fn devtools_visibility_payload(visible: bool) -> DevtoolsVisibilityChangedEvent {
    DevtoolsVisibilityChangedEvent { visible }
}

#[cfg(debug_assertions)]
pub(crate) fn build_devtools_state_response(game_state: &GameState, visible: bool) -> DevtoolsStateResponse {
    let guard = game_state.lock();
    DevtoolsStateResponse {
        visible,
        snapshot: crate::game::snapshot::build_snapshot(&guard.run, &guard.profile),
    }
}

#[cfg(debug_assertions)]
pub(crate) fn current_devtools_state_response(
    game_state: &GameState,
    devtools_state: &DevtoolsState,
) -> DevtoolsStateResponse {
    build_devtools_state_response(game_state, read_devtools_visibility(devtools_state))
}

#[cfg(debug_assertions)]
pub(crate) fn emit_devtools_visibility_changed<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    visible: bool,
) -> Result<(), String> {
    app.emit(DEVTOOLS_VISIBILITY_CHANGED_EVENT, devtools_visibility_payload(visible))
        .map_err(|error| error.to_string())
}

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

#[cfg(debug_assertions)]
pub(crate) fn toggle_devtools_visibility<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<DevtoolsStateResponse, String> {
    let devtools_state = app.state::<DevtoolsState>();
    let visible = !read_devtools_visibility(&devtools_state);
    update_devtools_visibility(app, visible)
}

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

#[cfg(debug_assertions)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn devtools_enabled() -> bool {
    true
}

#[cfg(not(debug_assertions))]
pub(crate) fn devtools_enabled() -> bool {
    false
}

#[cfg(debug_assertions)]
pub(crate) fn devtools_action_success(run_state: &RunState, profile: &PrestigeProfile) -> serde_json::Value {
    serde_json::json!({
        "ok": true,
        "snapshot": crate::game::snapshot::build_snapshot(run_state, profile),
    })
}

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
    let crate::GameRunState { run, profile, session_ticks } = &mut *guard;

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
