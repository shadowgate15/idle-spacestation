//! Save-format version detection and per-version deserialization.
//!
//! **Currently scaffolded; not wired into commands as of 2026-05-08.** Only
//! [`super::SaveManager`] (itself unwired) and the unit tests in [`super`]
//! call into this module.
//!
//! # Migration Strategy
//!
//! Every save embeds a top-level `saveVersion` (camelCase) integer field.
//! Loading proceeds in two phases:
//!
//! 1. [`extract_save_version`] parses the JSON shallowly and reads
//!    `saveVersion`. Failures here are classified as [`MigrationError`]
//!    variants ([`MigrationError::InvalidJson`],
//!    [`MigrationError::InvalidShape`],
//!    [`MigrationError::MissingSaveVersion`]).
//! 2. [`migrate`] dispatches on that version. Today only
//!    [`super::save::SAVE_VERSION`] (= `1`) is supported and is parsed via
//!    [`super::save::deserialize_v1`]. Anything else returns
//!    [`MigrationError::UnsupportedVersion`].
//!
//! # Adding a New Version
//!
//! When [`super::save::SAVE_VERSION`] is bumped:
//!
//! 1. Define the new on-disk schema in [`super::save`] and add a
//!    `deserialize_vN` helper that yields the latest [`SaveData`].
//! 2. Extend [`migrate`] with a match arm for the *previous* version that
//!    parses the old schema (kept around as a separate type), upgrades it
//!    to the new [`SaveData`], and returns it.
//! 3. Keep the old `deserialize_vN` helpers around — players can have
//!    arbitrarily old saves on disk.
//!
//! See also: [`super::recover_save`] which folds [`MigrationError`] into
//! [`super::SaveFailureCode`] for the user-facing recovery flow, and
//! [`crate::game::sim::state::RunState`] which is what the migrated
//! [`SaveData`] ultimately becomes.

use std::convert::TryFrom;

use serde_json::Value;

use super::save::{deserialize_v1, SaveData, SAVE_VERSION};

/// Reasons a save payload could not be parsed or upgraded.
///
/// These are the *parsing*-side failure modes. The recovery layer
/// ([`super::recover_save`]) maps each variant to a [`super::SaveFailureCode`]
/// so the higher-level outcome enum stays free of `serde_json` types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// The payload is not syntactically valid JSON.
    InvalidJson,
    /// The payload is valid JSON but does not match the expected shape
    /// (e.g. top level is not an object, or fields have the wrong types).
    InvalidShape,
    /// The payload is a JSON object but lacks the `saveVersion` field
    /// required by [`extract_save_version`].
    MissingSaveVersion,
    /// The `saveVersion` is well-formed but no migration path is registered
    /// for it. The raw value is included for diagnostics.
    UnsupportedVersion(u32),
}

/// Read the `saveVersion` discriminator from a raw save payload.
///
/// Performs a shallow `serde_json::Value` parse so the dispatcher can
/// decide which schema to deserialize without committing to one. Errors:
///
/// - [`MigrationError::InvalidJson`] — the payload is not valid JSON.
/// - [`MigrationError::InvalidShape`] — the top-level value is not a JSON
///   object, or `saveVersion` is present but not a non-negative integer
///   that fits in `u32`.
/// - [`MigrationError::MissingSaveVersion`] — the object has no
///   `saveVersion` key at all.
pub fn extract_save_version(data: &str) -> Result<u32, MigrationError> {
    let value: Value = serde_json::from_str(data).map_err(|_| MigrationError::InvalidJson)?;
    let object = value.as_object().ok_or(MigrationError::InvalidShape)?;
    let version = object
        .get("saveVersion")
        .ok_or(MigrationError::MissingSaveVersion)?
        .as_u64()
        .ok_or(MigrationError::InvalidShape)?;

    u32::try_from(version).map_err(|_| MigrationError::InvalidShape)
}

/// Deserialize `data` as a save of the given `version`, upgrading to the
/// current [`SaveData`] schema.
///
/// Today only [`SAVE_VERSION`] (= `1`) is supported; older payloads return
/// [`MigrationError::UnsupportedVersion`] until a migration path exists.
///
/// As a defense-in-depth check, the parsed payload's
/// [`SaveData::save_version`] field is compared against [`SAVE_VERSION`];
/// a mismatch (e.g. someone passed `version = 1` but the payload's body
/// claims a different version) returns
/// [`MigrationError::UnsupportedVersion`] rather than silently accepting
/// the inconsistent save.
pub fn migrate(version: u32, data: &str) -> Result<SaveData, MigrationError> {
    match version {
        SAVE_VERSION => {
            let save = deserialize_v1(data).map_err(|_| MigrationError::InvalidShape)?;
            if save.save_version != SAVE_VERSION {
                return Err(MigrationError::UnsupportedVersion(save.save_version));
            }

            Ok(save)
        }
        other => Err(MigrationError::UnsupportedVersion(other)),
    }
}

/// One-shot helper that combines [`extract_save_version`] and [`migrate`].
///
/// Most callers (including [`super::SaveManager`] and
/// [`super::recover_save`]) want both steps; this helper avoids forcing
/// them to thread the version through manually. Errors propagate from the
/// underlying helpers unchanged.
pub fn deserialize_with_migration(data: &str) -> Result<SaveData, MigrationError> {
    let version = extract_save_version(data)?;
    migrate(version, data)
}
