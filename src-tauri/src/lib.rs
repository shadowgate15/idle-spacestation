mod game;

use std::sync::Mutex;

use crate::game::progression::PrestigeProfile;
use crate::game::sim::RunState;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

struct GameState(Mutex<(RunState, PrestigeProfile, u32)>);

#[tauri::command]
fn game_get_snapshot(state: tauri::State<GameState>) -> serde_json::Value {
    let guard = state.0.lock().expect("game state mutex poisoned");
    serde_json::json!({ "tick": guard.0.tick_count })
}

#[tauri::command]
fn game_toggle_service(service_id: String, active: bool, state: tauri::State<GameState>) -> bool {
    let mut guard = state.0.lock().expect("game state mutex poisoned");
    if let Some(service) = guard
        .0
        .services
        .iter_mut()
        .find(|service| service.service_id == service_id)
    {
        service.desired_active = active;
    }
    true
}

#[tauri::command]
fn game_upgrade_system(system_id: String, state: tauri::State<GameState>) -> bool {
    let guard = state.0.lock().expect("game state mutex poisoned");
    guard.0.systems.iter().any(|system| system.system_id == system_id)
}

#[tauri::command]
fn game_select_planet(planet_id: String, state: tauri::State<GameState>) -> bool {
    let mut guard = state.0.lock().expect("game state mutex poisoned");
    guard.0.station.active_planet_id = planet_id;
    true
}

#[tauri::command]
fn game_purchase_doctrine(_doctrine_id: String, _state: tauri::State<GameState>) -> bool {
    false
}

#[tauri::command]
fn game_execute_prestige(_state: tauri::State<GameState>) -> bool {
    false
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .manage(GameState(Mutex::new((
            RunState::starter_fixture(),
            PrestigeProfile::default(),
            0u32,
        ))))
        .invoke_handler(tauri::generate_handler![
            greet,
            game_get_snapshot,
            game_toggle_service,
            game_upgrade_system,
            game_select_planet,
            game_purchase_doctrine,
            game_execute_prestige,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
