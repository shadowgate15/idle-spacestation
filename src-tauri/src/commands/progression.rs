//! Progression-domain Tauri command handlers.
//!
//! Covers doctrine purchases (spending prestige fragments) and prestige
//! execution (resetting the run for permanent bonuses). Both commands mutate
//! state and emit `game://state-changed` via [`commit_and_emit`] on success.
//!
//! See also: [`crate::commands`], [`crate::game::progression`].

use crate::commands::action_response;
use crate::commands::inputs::{ConfirmPrestigeInput, PurchaseDoctrineInput};
use crate::game::content::doctrines::doctrine_by_id;
use crate::game::progression::{execute_prestige, DoctrinePurchaseError, PrestigeExecutionError};
use crate::game::snapshot::ActionResponse;
use crate::runtime::refresh_runtime_state;
use crate::{commit_and_emit, GameState, LastEmittedSnapshot};

/// Purchase a doctrine using prestige fragments.
///
/// **Frontend alias**: `game_purchase_doctrine` (same as Rust name)
/// **Mutates state**: yes
/// **Emits `game://state-changed`**: yes (via [`commit_and_emit`]) on success
///
/// On success, debits fragments from the persistent
/// [`crate::game::progression::PrestigeProfile`] and mirrors the unlocked
/// doctrine list onto the active run's station so the live snapshot reflects
/// the purchase immediately.
///
/// # Errors
/// Returns an `ActionResponse { ok: false, reason_code: Some(_) }` (mapped to
/// `DoctrinePurchaseRejectionCode` on the frontend) for one of:
/// - `unknown-doctrine`: no doctrine in content matches the requested id.
/// - `already-unlocked`: the doctrine is already owned by the profile.
/// - `insufficient-fragments`: profile fragment balance is below the cost.
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

/// Execute a prestige reset, swapping in a fresh run derived from the prestige
/// profile.
///
/// **Frontend alias**: `game_confirm_prestige`
/// **Mutates state**: yes
/// **Emits `game://state-changed`**: yes (via [`commit_and_emit`]) on success
///
/// Requires the caller to set `confirm: true` on the input as an explicit
/// guard against accidental resets. On success, replaces the current run and
/// profile with the post-prestige state and resets the session-tick counter.
///
/// # Errors
/// Returns an `ActionResponse { ok: false, reason_code: Some(_) }` (mapped to
/// `PrestigeRejectionCode` on the frontend) for one of:
/// - `confirmation-required`: caller did not set `confirm: true`.
/// - `station-tier-below-four`: station tier prerequisite not met.
/// - `needs-two-non-starter-planets`: missing the required discovered planets.
/// - `unstable-net-power`: net power has not been stable long enough.
/// - `not-implemented`: persistence layer rejected the post-prestige save
///   (the persistence subsystem is not yet wired in production).
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
