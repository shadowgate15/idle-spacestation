//! Corrupted-save recovery: primary → backup → fresh-profile fallback.
//!
//! **Currently scaffolded; not wired into commands as of 2026-05-08.** This
//! module is exercised only by the unit tests in [`super`]; no Tauri command
//! invokes [`recover_save`].
//!
//! # Recovery Order
//!
//! [`recover_save`] is the entry point. Given the (already-read) raw JSON
//! payloads of the primary and backup save files (each wrapped in a
//! [`SaveSource`]), it tries to produce a usable [`SaveData`] in this
//! order:
//!
//! 1. **Primary OK** — primary deserializes cleanly.
//!    [`SaveLoadResult::Ok`].
//! 2. **Restored from backup** — primary failed with some
//!    [`SaveFailureCode`] but backup deserializes cleanly.
//!    [`SaveLoadResult::RestoredFromBackup`] with `reason` set to the
//!    primary failure code.
//! 3. **Fresh profile created** — neither file is usable. Returns
//!    [`SaveData::fresh`] with both failure codes recorded under
//!    [`FreshProfileReason::NoUsableSave`].
//!
//! The function is total (never returns `Err`); every filesystem and parse
//! failure is folded into a [`SaveFailureCode`] inside the result.
//!
//! # Failure Code Mapping
//!
//! Failures fall into two layers:
//!
//! - Filesystem (handled in [`super::SaveManager::read_source`]):
//!   [`SaveFailureCode::Missing`], [`SaveFailureCode::ReadFailure`].
//! - Parsing/migration (mapped from [`MigrationError`] via
//!   [`From<MigrationError> for SaveFailureCode`] at the bottom of this
//!   file): [`SaveFailureCode::InvalidJson`],
//!   [`SaveFailureCode::MissingSaveVersion`],
//!   [`SaveFailureCode::UnsupportedVersion`],
//!   [`SaveFailureCode::InvalidShape`].
//!
//! See also: [`crate::game::sim::state::RunState`] — the runtime state
//! ultimately recovered (after [`SaveData::into_runtime`]).

#![allow(dead_code)]

use super::migration::{deserialize_with_migration, MigrationError};
use super::save::SaveData;

/// Outcome of attempting to *acquire* one save file's raw contents.
///
/// Produced by [`super::SaveManager::read_source`] for each of
/// `profile-primary.json` and `profile-backup.json`, then handed to
/// [`recover_save`]. The recovery layer never touches the filesystem
/// itself — it only reasons about already-read payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveSource {
    /// File was readable and produced this raw JSON payload (still
    /// unparsed — parse errors surface later via
    /// [`deserialize_with_migration`]).
    Available(String),
    /// File could not be read; the variant carries the reason
    /// (typically [`SaveFailureCode::Missing`] or
    /// [`SaveFailureCode::ReadFailure`]).
    Unavailable(SaveFailureCode),
}

/// User-facing failure classification for a single save file.
///
/// Produced both by filesystem reads ([`SaveFailureCode::Missing`],
/// [`SaveFailureCode::ReadFailure`]) and by parse/migration failures
/// (every other variant; see [`From<MigrationError>`]). Stable enough to
/// surface to the UI as recovery-status badges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFailureCode {
    /// File does not exist on disk (`io::ErrorKind::NotFound`).
    Missing,
    /// File exists but could not be read (permissions, I/O error, etc.).
    ReadFailure,
    /// File contents are not valid JSON.
    InvalidJson,
    /// JSON object lacks the required `saveVersion` field.
    MissingSaveVersion,
    /// `saveVersion` is present but no migration path is registered for it.
    UnsupportedVersion,
    /// JSON parsed but the structure does not match the [`SaveData`] schema.
    InvalidShape,
}

/// Why [`recover_save`] fell back to a fresh profile.
///
/// Currently exhaustive with a single variant, but defined as an enum so
/// future fallback reasons (e.g. user-requested reset) can be added
/// without breaking the [`SaveLoadResult`] match arms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FreshProfileReason {
    /// Both primary and backup were unusable. Both failure codes are
    /// recorded so the UI can explain *why* recovery dropped the player
    /// into a fresh profile rather than restoring something.
    NoUsableSave {
        /// Failure classification for `profile-primary.json`.
        primary: SaveFailureCode,
        /// Failure classification for `profile-backup.json`.
        backup: SaveFailureCode,
    },
}

/// High-level classification of the [`recover_save`] result.
///
/// Distinct from [`SaveData`] so callers can distinguish "loaded normally"
/// from "fell back to backup" from "started a fresh profile" without
/// inspecting the data itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveLoadResult {
    /// Primary save loaded successfully.
    Ok,
    /// Primary failed; backup loaded successfully. The `reason` field
    /// carries the [`SaveFailureCode`] that caused the primary to be rejected.
    RestoredFromBackup { reason: SaveFailureCode },
    /// Both primary and backup failed; a fresh [`SaveData::fresh`] was
    /// substituted. The `reason` field records why neither was usable
    /// (see [`FreshProfileReason`]).
    FreshProfileCreated { reason: FreshProfileReason },
}

/// Pair of `(data, result)` produced by [`recover_save`].
///
/// `data` is always populated — even in the fresh-profile case it carries
/// [`SaveData::fresh`] so the caller can proceed without a separate
/// "no save" code path. `result` describes how `data` was obtained.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveLoadOutcome {
    /// The [`SaveData`] the caller should use, regardless of recovery path.
    pub data: SaveData,
    /// Classification of how `data` was obtained.
    pub result: SaveLoadResult,
}

/// Run the recovery cascade over a primary/backup pair.
///
/// Always returns a [`SaveLoadOutcome`] — never panics, never returns
/// `Err`. Filesystem and parse failures are folded into
/// [`SaveFailureCode`] inside the result. See the module docs for the
/// exact recovery order.
pub fn recover_save(primary: SaveSource, backup: SaveSource) -> SaveLoadOutcome {
    match load_from_source(primary) {
        Ok(data) => SaveLoadOutcome {
            data,
            result: SaveLoadResult::Ok,
        },
        Err(primary_error) => match load_from_source(backup) {
            Ok(data) => SaveLoadOutcome {
                data,
                result: SaveLoadResult::RestoredFromBackup {
                    reason: primary_error,
                },
            },
            Err(backup_error) => SaveLoadOutcome {
                data: SaveData::fresh(),
                result: SaveLoadResult::FreshProfileCreated {
                    reason: FreshProfileReason::NoUsableSave {
                        primary: primary_error,
                        backup: backup_error,
                    },
                },
            },
        },
    }
}

/// Try to deserialize one [`SaveSource`] into a [`SaveData`].
///
/// [`SaveSource::Available`] is parsed via
/// [`deserialize_with_migration`] (any [`MigrationError`] is mapped to
/// [`SaveFailureCode`] via the [`From`] impl below).
/// [`SaveSource::Unavailable`] short-circuits with the carried failure
/// code so the recovery layer's match arms stay simple.
fn load_from_source(source: SaveSource) -> Result<SaveData, SaveFailureCode> {
    match source {
        SaveSource::Available(raw) => deserialize_with_migration(&raw).map_err(SaveFailureCode::from),
        SaveSource::Unavailable(code) => Err(code),
    }
}

impl From<MigrationError> for SaveFailureCode {
    fn from(value: MigrationError) -> Self {
        match value {
            MigrationError::InvalidJson => Self::InvalidJson,
            MigrationError::InvalidShape => Self::InvalidShape,
            MigrationError::MissingSaveVersion => Self::MissingSaveVersion,
            MigrationError::UnsupportedVersion(_) => Self::UnsupportedVersion,
        }
    }
}
