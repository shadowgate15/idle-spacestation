use std::convert::TryFrom;

use serde_json::Value;

use super::save::{deserialize_v1, SaveData, SAVE_VERSION};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    InvalidJson,
    InvalidShape,
    MissingSaveVersion,
    UnsupportedVersion(u32),
}

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

pub fn deserialize_with_migration(data: &str) -> Result<SaveData, MigrationError> {
    let version = extract_save_version(data)?;
    migrate(version, data)
}
