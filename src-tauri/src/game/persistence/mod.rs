//! Save-data persistence for the idle-spacestation game state.
//!
//! **Currently scaffolded; not wired into commands as of 2026-05-08.**
//! [`SaveManager`] and [`SaveData`] exist and are exercised by the unit tests
//! at the bottom of this module, but no Tauri command in
//! [`crate::commands`] (production or devtools) calls them, and the tick
//! loop in the crate root (`lib.rs`) does not write saves to disk. Wiring this module
//! into the runtime is a future task; until then it should be treated as
//! a self-contained subsystem with stable public types.
//!
//! # Layout
//!
//! - [`save`] — versioned [`SaveData`] DTO + [`ProfileState`] + [`SaveSettings`].
//! - [`migration`] — version detection and per-version deserialization.
//! - [`recovery`] — primary/backup fallback and fresh-profile creation.
//!
//! # Persistence Model
//!
//! [`SaveManager`] writes two JSON files under a configurable root directory:
//!
//! - `profile-primary.json` — the latest snapshot.
//! - `profile-backup.json` — the previous primary contents (rotated on save).
//!
//! On load, [`SaveManager::load`] reads both files and delegates to
//! [`recover_save`] to pick the freshest usable snapshot or fall back to a
//! fresh profile. Saves are triggered explicitly by callers via the
//! [`SaveTrigger`]-tagged `save_*` helpers, or implicitly by
//! [`SaveManager::save_if_autosave_due`] which checks
//! [`crate::game::sim::state::AUTOSAVE_CADENCE_TICKS`].
//!
//! See also: [`crate::game::sim::state::RunState`] — the runtime state that
//! [`SaveData`] persists (via [`save::RunStateSnapshot`]).

pub mod migration;
pub mod recovery;
pub mod save;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::game::sim::state::AUTOSAVE_CADENCE_TICKS;
use crate::game::sim::{tick, RunState};

pub use migration::{deserialize_with_migration, extract_save_version, migrate, MigrationError};
pub use recovery::{
    recover_save, FreshProfileReason, SaveFailureCode, SaveLoadOutcome, SaveLoadResult, SaveSource,
};
pub use save::{ProfileState, SaveData, SaveSettings, SAVE_VERSION};

/// Reason a save was written.
///
/// Used to tag the resulting [`SaveWriteSummary`] so callers (and future
/// telemetry) can distinguish routine autosaves from user-driven saves.
/// The variants are advisory metadata only — the on-disk format is identical
/// across triggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveTrigger {
    /// Periodic autosave fired by [`SaveManager::save_if_autosave_due`] when
    /// the tick cadence in [`crate::game::sim::state::AUTOSAVE_CADENCE_TICKS`]
    /// elapses or [`crate::game::sim::state::RunState::autosave_due`] is set.
    Autosave,
    /// Save written when the application window becomes hidden (e.g. tab/window
    /// blur on the desktop shell). Triggered via
    /// [`SaveManager::save_for_visibility_change`].
    VisibilityHidden,
    /// Save written immediately before the OS window closes. Triggered via
    /// [`SaveManager::save_for_window_close`].
    WindowClose,
    /// Save written defensively just before a prestige reset, so a crash
    /// during prestige cannot lose pre-reset progress. Triggered via
    /// [`SaveManager::save_before_prestige`].
    BeforePrestige,
}

/// Bookkeeping returned by every successful [`SaveManager`] write.
///
/// Records the trigger, the on-disk format version, and the absolute paths
/// of the primary and backup save files that were touched. Callers can use
/// the paths for diagnostics or to surface "last saved" UI without
/// re-deriving them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveWriteSummary {
    /// Why this save was written. See [`SaveTrigger`].
    pub trigger: SaveTrigger,
    /// On-disk format version that was written. Equals [`SAVE_VERSION`] for
    /// every save produced by the current build.
    pub version: u32,
    /// Absolute path of the primary save file (`profile-primary.json`).
    pub primary_path: PathBuf,
    /// Absolute path of the rotated backup file (`profile-backup.json`).
    pub backup_path: PathBuf,
}

/// Error returned by [`SaveManager`] write operations.
///
/// Reads use the typed [`SaveLoadResult`] / [`SaveFailureCode`] pair instead;
/// this enum only surfaces failures that occur while *writing* (filesystem I/O
/// or serialization).
#[derive(Debug)]
pub enum SaveManagerError {
    /// Filesystem I/O failure while creating the save root, rotating the
    /// backup, or writing the primary file.
    Io(io::Error),
    /// `serde_json` failed to serialize the [`SaveData`] DTO. In practice this
    /// only happens if the in-memory snapshot contains values that cannot be
    /// expressed in JSON (e.g. non-finite floats with a stricter serializer).
    Serialize(serde_json::Error),
}

impl From<io::Error> for SaveManagerError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for SaveManagerError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialize(value)
    }
}

/// Filesystem-backed save controller for one save-slot directory.
///
/// **Currently scaffolded; not wired into commands as of 2026-05-08.** No
/// Tauri command constructs a [`SaveManager`] in production; the type is
/// covered only by the unit tests in this module.
///
/// A `SaveManager` owns a `root` directory and reads/writes two files inside
/// it: `profile-primary.json` (latest snapshot) and `profile-backup.json`
/// (previous primary, rotated on every successful save). All disk I/O is
/// synchronous and routed through `std::fs`; callers are expected to invoke
/// the save methods from a thread where blocking is acceptable (typically
/// the simulation tick thread or a Tauri command handler).
///
/// The [`SaveManager`] does not hold any in-memory state beyond `root`, so it
/// is cheap to clone and safe to share across threads.
#[derive(Debug, Clone)]
pub struct SaveManager {
    /// Directory that contains `profile-primary.json` and
    /// `profile-backup.json`. Created lazily on the first successful save.
    root: PathBuf,
}

impl SaveManager {
    /// Construct a [`SaveManager`] rooted at `root`.
    ///
    /// The directory is **not** created or validated here; it is created on
    /// demand by [`SaveManager::save`] (via `fs::create_dir_all`). Callers
    /// that need eager validation should `fs::create_dir_all` themselves.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Read both save files and return the recovered [`SaveLoadOutcome`].
    ///
    /// Reads `profile-primary.json` first, then falls back to
    /// `profile-backup.json` if the primary is missing or corrupted. If both
    /// are unusable, [`SaveData::fresh`] is returned so the caller can
    /// continue with a clean profile. Detailed classification (primary OK,
    /// restored from backup, fresh profile) lives in
    /// [`SaveLoadOutcome::result`].
    ///
    /// This method never fails — every filesystem or parse error is folded
    /// into a [`SaveFailureCode`] inside the returned outcome.
    pub fn load(&self) -> SaveLoadOutcome {
        recover_save(
            self.read_source(&self.primary_path()),
            self.read_source(&self.backup_path()),
        )
    }

    /// Serialize the supplied state and write it to disk under the given
    /// [`SaveTrigger`].
    ///
    /// Order of operations:
    /// 1. Build a [`SaveData`] DTO via [`SaveData::from_runtime`] (which
    ///    syncs the profile from `run_state`).
    /// 2. Serialize the DTO with [`save::serialize_save`].
    /// 3. `fs::create_dir_all(root)` if necessary.
    /// 4. Rotate the backup: copy the existing primary (if any) into
    ///    `profile-backup.json`. If no primary exists, the new serialized
    ///    payload is written to the backup path so a corrupted primary on
    ///    the next run can still recover.
    /// 5. Overwrite `profile-primary.json` with the new payload.
    ///
    /// Returns a [`SaveWriteSummary`] describing the write or a
    /// [`SaveManagerError`] if filesystem I/O or serialization failed. On
    /// error, the on-disk state may be partially updated (e.g. backup
    /// rotated but primary unwritten); callers should treat any error as
    /// "save did not complete".
    pub fn save(
        &self,
        run_state: &RunState,
        profile_state: &ProfileState,
        settings: &SaveSettings,
        trigger: SaveTrigger,
    ) -> Result<SaveWriteSummary, SaveManagerError> {
        let data = SaveData::from_runtime(run_state, profile_state, settings);
        let serialized = save::serialize_save(&data)?;

        fs::create_dir_all(&self.root)?;

        let primary_path = self.primary_path();
        let backup_path = self.backup_path();

        match fs::read_to_string(&primary_path) {
            Ok(existing_primary) => fs::write(&backup_path, existing_primary)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => fs::write(&backup_path, &serialized)?,
            Err(error) => return Err(error.into()),
        }

        fs::write(&primary_path, serialized)?;

        Ok(SaveWriteSummary {
            trigger,
            version: SAVE_VERSION,
            primary_path,
            backup_path,
        })
    }

    /// Save only when an autosave is due.
    ///
    /// An autosave fires when **all** of the following are true:
    ///
    /// - [`SaveSettings::autosave_enabled`] is `true`.
    /// - `run_state.tick_count > 0` (skip the initial state).
    /// - Either [`crate::game::sim::state::RunState::autosave_due`] is set,
    ///   or `tick_count` is a multiple of
    ///   [`crate::game::sim::state::AUTOSAVE_CADENCE_TICKS`] (defaults to
    ///   60 ticks ≈ 15 s of wall-clock time at the 250 ms tick cadence).
    ///
    /// Returns `Ok(Some(summary))` when a save was written, `Ok(None)` when
    /// no autosave was due (no I/O performed), or `Err(_)` if a save was
    /// attempted and failed.
    pub fn save_if_autosave_due(
        &self,
        run_state: &RunState,
        profile_state: &ProfileState,
        settings: &SaveSettings,
    ) -> Result<Option<SaveWriteSummary>, SaveManagerError> {
        if settings.autosave_enabled
            && run_state.tick_count > 0
            && (run_state.autosave_due || run_state.tick_count % AUTOSAVE_CADENCE_TICKS == 0)
        {
            return self
                .save(run_state, profile_state, settings, SaveTrigger::Autosave)
                .map(Some);
        }

        Ok(None)
    }

    /// Save unconditionally with [`SaveTrigger::VisibilityHidden`].
    ///
    /// Intended to be called from a window-visibility hook on the desktop
    /// shell so progress is durable when the user backgrounds the app.
    /// Ignores [`SaveSettings::autosave_enabled`] — visibility saves are
    /// considered user-driven and always fire.
    pub fn save_for_visibility_change(
        &self,
        run_state: &RunState,
        profile_state: &ProfileState,
        settings: &SaveSettings,
    ) -> Result<SaveWriteSummary, SaveManagerError> {
        self.save(
            run_state,
            profile_state,
            settings,
            SaveTrigger::VisibilityHidden,
        )
    }

    /// Save unconditionally with [`SaveTrigger::WindowClose`].
    ///
    /// Intended to be called from the Tauri close handler so the latest
    /// state survives a window close. Ignores
    /// [`SaveSettings::autosave_enabled`].
    pub fn save_for_window_close(
        &self,
        run_state: &RunState,
        profile_state: &ProfileState,
        settings: &SaveSettings,
    ) -> Result<SaveWriteSummary, SaveManagerError> {
        self.save(run_state, profile_state, settings, SaveTrigger::WindowClose)
    }

    /// Save unconditionally with [`SaveTrigger::BeforePrestige`].
    ///
    /// Should be called by the prestige command immediately before the run
    /// state is reset so that a crash mid-prestige cannot lose pre-reset
    /// progress. Ignores [`SaveSettings::autosave_enabled`].
    pub fn save_before_prestige(
        &self,
        run_state: &RunState,
        profile_state: &ProfileState,
        settings: &SaveSettings,
    ) -> Result<SaveWriteSummary, SaveManagerError> {
        self.save(run_state, profile_state, settings, SaveTrigger::BeforePrestige)
    }

    /// Absolute path of the primary save file inside `root`.
    fn primary_path(&self) -> PathBuf {
        self.root.join("profile-primary.json")
    }

    /// Absolute path of the rotated backup file inside `root`.
    fn backup_path(&self) -> PathBuf {
        self.root.join("profile-backup.json")
    }

    /// Read a save file from disk and classify the result as a
    /// [`SaveSource`].
    ///
    /// `NotFound` is mapped to [`SaveFailureCode::Missing`] and any other
    /// I/O error to [`SaveFailureCode::ReadFailure`]. Successful reads
    /// return [`SaveSource::Available`] with the raw JSON payload — parsing
    /// happens later in [`recover_save`] via [`deserialize_with_migration`].
    fn read_source(&self, path: &Path) -> SaveSource {
        match fs::read_to_string(path) {
            Ok(raw) => SaveSource::Available(raw),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                SaveSource::Unavailable(SaveFailureCode::Missing)
            }
            Err(_) => SaveSource::Unavailable(SaveFailureCode::ReadFailure),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn advance_ticks(state: &mut RunState, ticks: usize) {
        for _ in 0..ticks {
            tick(state);
        }
    }

    fn unique_temp_dir() -> PathBuf {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let unique_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "idle-spacestation-persistence-{}-{}-{}",
            std::process::id(),
            timestamp,
            unique_id
        ));

        fs::create_dir_all(&dir).expect("temp save dir should be created");
        dir
    }

    fn fixture_profile(run_state: &RunState) -> ProfileState {
        ProfileState::from_run_state(run_state)
    }

    #[test]
    fn persistence_save_reload_parity_matches_control_after_520_ticks() {
        let save_dir = unique_temp_dir();
        let manager = SaveManager::new(&save_dir);
        let settings = SaveSettings::default();
        let mut control = RunState::starter_fixture();
        let mut persisted = RunState::starter_fixture();
        let profile = fixture_profile(&persisted);

        advance_ticks(&mut control, 480);
        advance_ticks(&mut persisted, 480);

        let autosave = manager
            .save_if_autosave_due(&persisted, &profile, &settings)
            .expect("autosave should succeed");

        assert!(autosave.is_some());

        let outcome = manager.load();
        assert_eq!(outcome.result, SaveLoadResult::Ok);

        let (mut reloaded, _, _) = outcome.data.into_runtime();

        advance_ticks(&mut control, 40);
        advance_ticks(&mut reloaded, 40);

        println!(
            "parity: control_hash={} reloaded_hash={} control_materials={} reloaded_materials={} control_data={} reloaded_data={}",
            control.state_hash(),
            reloaded.state_hash(),
            control.resources.materials,
            reloaded.resources.materials,
            control.resources.data,
            reloaded.resources.data
        );

        assert_eq!(control.state_hash(), reloaded.state_hash());
        assert_eq!(control.resources.materials, reloaded.resources.materials);
        assert_eq!(control.resources.data, reloaded.resources.data);

        fs::remove_dir_all(save_dir).expect("temp save dir should be removed");
    }

    #[test]
    fn persistence_corrupted_primary_recovers_from_backup() {
        let save_dir = unique_temp_dir();
        let manager = SaveManager::new(&save_dir);
        let settings = SaveSettings::default();
        let mut run_state = RunState::starter_fixture();
        advance_ticks(&mut run_state, 120);
        let profile = fixture_profile(&run_state);

        manager
            .save(&run_state, &profile, &settings, SaveTrigger::WindowClose)
            .expect("save should succeed");

        fs::write(save_dir.join("profile-primary.json"), "{ definitely-not-json }")
            .expect("corrupted primary should be written");

        let outcome = manager.load();
        let (recovered, _, _) = outcome.data.clone().into_runtime();

        println!(
            "recovery-backup: result={:?} recovered_hash={} expected_hash={}",
            outcome.result,
            recovered.state_hash(),
            run_state.state_hash()
        );

        assert_eq!(
            outcome.result,
            SaveLoadResult::RestoredFromBackup {
                reason: SaveFailureCode::InvalidJson,
            }
        );
        assert_eq!(recovered.state_hash(), run_state.state_hash());

        fs::remove_dir_all(save_dir).expect("temp save dir should be removed");
    }

    #[test]
    fn persistence_corrupted_primary_and_backup_creates_fresh_profile() {
        let save_dir = unique_temp_dir();
        let manager = SaveManager::new(&save_dir);
        let settings = SaveSettings::default();
        let run_state = RunState::starter_fixture();
        let profile = fixture_profile(&run_state);

        manager
            .save(&run_state, &profile, &settings, SaveTrigger::VisibilityHidden)
            .expect("save should succeed");

        fs::write(save_dir.join("profile-primary.json"), "{ definitely-not-json }")
            .expect("corrupted primary should be written");
        fs::write(save_dir.join("profile-backup.json"), "{ definitely-not-json }")
            .expect("corrupted backup should be written");

        let outcome = manager.load();
        let (fresh_run, fresh_profile, fresh_settings) = outcome.data.clone().into_runtime();
        let expected_fresh = SaveData::fresh();

        println!(
            "recovery-fresh: result={:?} fresh_hash={} autosave_enabled={}",
            outcome.result,
            fresh_run.state_hash(),
            fresh_settings.autosave_enabled
        );

        assert_eq!(
            outcome.result,
            SaveLoadResult::FreshProfileCreated {
                reason: FreshProfileReason::NoUsableSave {
                    primary: SaveFailureCode::InvalidJson,
                    backup: SaveFailureCode::InvalidJson,
                },
            }
        );
        assert_eq!(outcome.data, expected_fresh);
        assert_eq!(fresh_run.state_hash(), RunState::starter_fixture().state_hash());
        assert_eq!(fresh_profile, ProfileState::from_run_state(&RunState::starter_fixture()));
        assert!(fresh_settings.autosave_enabled);

        fs::remove_dir_all(save_dir).expect("temp save dir should be removed");
    }
}
