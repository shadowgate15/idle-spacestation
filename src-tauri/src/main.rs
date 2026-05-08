//! Binary entrypoint for the `idle-spacestation` Tauri desktop app.
//!
//! This crate is intentionally a thin shim: all runtime wiring (Tauri builder,
//! game-state mutexes, command registration, the 250 ms tick loop) lives in the
//! companion library crate [`idle_spacestation_lib`]. Keeping `main.rs` empty of
//! logic lets unit and integration tests exercise the library directly without
//! booting a full Tauri runtime.
//!
//! # Windows subsystem guard
//!
//! The crate-level `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
//! attribute selects the `"windows"` subsystem (instead of the default `"console"`)
//! when debug assertions are disabled, so a release executable launched on Windows
//! does not spawn an attached console window. Debug builds keep the console visible
//! so `eprintln!` and panic output remain readable during development.
//!
//! **Do not remove that attribute.** Removing it regresses the release UX on
//! Windows by attaching a stray terminal window to the GUI app.
//!
//! See also: [`idle_spacestation_lib::run`].

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Process entrypoint that delegates to [`idle_spacestation_lib::run`].
///
/// All initialization — Tauri plugin wiring, managed state, the background tick
/// loop, and command registration — happens inside the library crate so it can
/// be re-used from the mobile entrypoint and exercised from tests.
fn main() {
    idle_spacestation_lib::run()
}
