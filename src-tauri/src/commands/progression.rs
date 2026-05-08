use crate::commands::action_response;
use crate::commands::inputs::{ConfirmPrestigeInput, PurchaseDoctrineInput};
use crate::game::content::doctrines::doctrine_by_id;
use crate::game::progression::{execute_prestige, DoctrinePurchaseError, PrestigeExecutionError};
use crate::game::snapshot::ActionResponse;
use crate::runtime::refresh_runtime_state;
use crate::{commit_and_emit, GameState, LastEmittedSnapshot};

#[tauri::command]
pub fn game_purchase_doctrine(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: PurchaseDoctrineInput,
    state: tauri::State<GameState>,
    last_emitted: tauri::State<LastEmittedSnapshot>,
) -> ActionResponse {
    let mut guard = state.lock();

    if doctrine_by_id(&input.doctrine_id).is_none() {
        return action_response(&guard.run, &guard.profile, false, Some("unknown-doctrine"));
    }

    match crate::game::progression::purchase_doctrine(&mut guard.profile, &input.doctrine_id) {
        Ok(()) => {
            guard.run.station.doctrine_ids = guard.profile.doctrine_ids.clone();
            guard.run.station.doctrine_fragments = guard.profile.doctrine_fragments;
            let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
            action_response(&guard.run, &guard.profile, true, None)
        }
        Err(DoctrinePurchaseError::UnknownDoctrine) => {
            action_response(&guard.run, &guard.profile, false, Some("unknown-doctrine"))
        }
        Err(DoctrinePurchaseError::AlreadyUnlocked) => {
            action_response(&guard.run, &guard.profile, false, Some("already-unlocked"))
        }
        Err(DoctrinePurchaseError::InsufficientFragments) => {
            action_response(&guard.run, &guard.profile, false, Some("insufficient-fragments"))
        }
    }
}

#[tauri::command]
pub fn game_execute_prestige(
    app: tauri::AppHandle<impl tauri::Runtime>,
    input: ConfirmPrestigeInput,
    state: tauri::State<GameState>,
    last_emitted: tauri::State<LastEmittedSnapshot>,
) -> ActionResponse {
    let mut guard = state.lock();

    if !input.confirm {
        return action_response(&guard.run, &guard.profile, false, Some("confirmation-required"));
    }

    match execute_prestige(&guard.run, &guard.profile, guard.run.consecutive_stable_power_ticks) {
        Ok((run_state, profile, stable_ticks)) => {
            guard.run = run_state;
            guard.profile = profile;
            guard.session_ticks = stable_ticks;
            refresh_runtime_state(&mut guard.run);
            let _ = commit_and_emit(&app, &guard.run, &guard.profile, &last_emitted);
            action_response(&guard.run, &guard.profile, true, None)
        }
        Err(PrestigeExecutionError::Ineligible(reason)) => {
            action_response(&guard.run, &guard.profile, false, Some(reason.code()))
        }
        Err(PrestigeExecutionError::Save(_)) => {
            action_response(&guard.run, &guard.profile, false, Some("not-implemented"))
        }
    }
}
