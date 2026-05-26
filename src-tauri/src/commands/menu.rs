//! Native window menu installation.
//!
//! Owns the single `app.set_menu(...)` call site for the desktop window. The
//! menu is platform-aware:
//!
//! - **macOS**: an app-name submenu comes first (containing the standard
//!   "Quit Idle Spacestation" item with the platform-standard `Cmd+Q`
//!   accelerator), followed by a "File" submenu containing "Close Window"
//!   (`Cmd+W`). macOS places the first submenu under the application's name
//!   in the menu bar regardless of the label, so the app submenu MUST be
//!   first.
//! - **Windows / Linux**: a single "File" submenu containing both "Close
//!   Window" (`Ctrl+W`) and "Quit" (`Ctrl+Q`).
//!
//! In debug builds the existing "Debug" submenu (with the
//! `Toggle Game State Overlay` item) is appended after the platform menus and
//! its click handler is wired up alongside no-op handlers for the predefined
//! Close / Quit items (those are handled natively by Tauri / the OS — see
//! [`tauri::menu::PredefinedMenuItem`]).
//!
//! # Lifecycle
//!
//! Called exactly once from `crate::run`'s `setup` closure. Subsequent calls
//! would replace (not merge with) any previously installed menu, so do not
//! invoke this helper a second time.

use tauri::menu::{MenuBuilder, PredefinedMenuItem, SubmenuBuilder};

#[cfg(debug_assertions)]
use tauri::menu::MenuItem;

/// Menu id for the debug-only "Debug → Toggle Game State Overlay" item.
///
/// Kept colocated with the menu installer (rather than in `commands/devtools/`)
/// because the click handler is registered here via
/// [`tauri::App::on_menu_event`].
#[cfg(debug_assertions)]
const DEVTOOLS_TOGGLE_OVERLAY_MENU_ID: &str = "devtools-toggle-overlay";

/// Installs the native window menu and wires its click handlers.
///
/// **Mutates state**: yes (registers a menu and an `on_menu_event` handler on
/// `app`).
///
/// # Errors
///
/// Returns the underlying [`tauri::Error`] if any submenu builder, menu
/// builder, or [`tauri::App::set_menu`] call fails.
pub(crate) fn install_app_menu<R: tauri::Runtime>(app: &mut tauri::App<R>) -> tauri::Result<()> {
    let close_window = PredefinedMenuItem::close_window(app, Some("Close Window"))?;
    let quit = PredefinedMenuItem::quit(app, Some("Quit Idle Spacestation"))?;

    let mut builder = MenuBuilder::new(app);

    // macOS expects the first submenu to be the app menu (the OS displays it
    // under the application's name regardless of label). Quit lives here per
    // the platform convention; Close Window goes under File.
    #[cfg(target_os = "macos")]
    {
        let app_menu = SubmenuBuilder::new(app, "Idle Spacestation")
            .item(&quit)
            .build()?;
        let file_menu = SubmenuBuilder::new(app, "File")
            .item(&close_window)
            .build()?;
        builder = builder.items(&[&app_menu, &file_menu]);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let file_menu = SubmenuBuilder::new(app, "File")
            .item(&close_window)
            .separator()
            .item(&quit)
            .build()?;
        builder = builder.item(&file_menu);
    }

    #[cfg(debug_assertions)]
    let toggle_overlay = MenuItem::with_id(
        app,
        DEVTOOLS_TOGGLE_OVERLAY_MENU_ID,
        "Toggle Game State Overlay",
        true,
        None::<&str>,
    )?;
    #[cfg(debug_assertions)]
    {
        let debug_menu = SubmenuBuilder::new(app, "Debug")
            .item(&toggle_overlay)
            .build()?;
        builder = builder.item(&debug_menu);
    }

    let menu = builder.build()?;
    app.set_menu(menu)?;

    // Close Window and Quit are PredefinedMenuItems — Tauri / the OS handle
    // their click behavior natively (including the standard CmdOrCtrl+W /
    // CmdOrCtrl+Q accelerators). We only need to register a handler for the
    // debug-only overlay toggle.
    #[cfg(debug_assertions)]
    app.on_menu_event(|app_handle, event| {
        if event.id().0.as_str() == DEVTOOLS_TOGGLE_OVERLAY_MENU_ID {
            let _ = crate::commands::devtools::toggle_devtools_visibility(app_handle);
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(debug_assertions)]
    fn devtools_menu_id_matches_handler_constant() {
        // Guards against drift between the id used when building the menu item
        // and the id matched inside `on_menu_event`.
        assert_eq!(DEVTOOLS_TOGGLE_OVERLAY_MENU_ID, "devtools-toggle-overlay");
    }
}
