#![allow(dead_code)]

use super::migration::{deserialize_with_migration, MigrationError};
use super::save::SaveData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveSource {
    Available(String),
    Unavailable(SaveFailureCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFailureCode {
    Missing,
    ReadFailure,
    InvalidJson,
    MissingSaveVersion,
    UnsupportedVersion,
    InvalidShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FreshProfileReason {
    NoUsableSave {
        primary: SaveFailureCode,
        backup: SaveFailureCode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveLoadResult {
    Ok,
    RestoredFromBackup { reason: SaveFailureCode },
    FreshProfileCreated { reason: FreshProfileReason },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SaveLoadOutcome {
    pub data: SaveData,
    pub result: SaveLoadResult,
}

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
