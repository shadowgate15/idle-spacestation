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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveTrigger {
    Autosave,
    VisibilityHidden,
    WindowClose,
    BeforePrestige,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveWriteSummary {
    pub trigger: SaveTrigger,
    pub version: u32,
    pub primary_path: PathBuf,
    pub backup_path: PathBuf,
}

#[derive(Debug)]
pub enum SaveManagerError {
    Io(io::Error),
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

#[derive(Debug, Clone)]
pub struct SaveManager {
    root: PathBuf,
}

impl SaveManager {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn load(&self) -> SaveLoadOutcome {
        recover_save(
            self.read_source(&self.primary_path()),
            self.read_source(&self.backup_path()),
        )
    }

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

    pub fn save_for_window_close(
        &self,
        run_state: &RunState,
        profile_state: &ProfileState,
        settings: &SaveSettings,
    ) -> Result<SaveWriteSummary, SaveManagerError> {
        self.save(run_state, profile_state, settings, SaveTrigger::WindowClose)
    }

    pub fn save_before_prestige(
        &self,
        run_state: &RunState,
        profile_state: &ProfileState,
        settings: &SaveSettings,
    ) -> Result<SaveWriteSummary, SaveManagerError> {
        self.save(run_state, profile_state, settings, SaveTrigger::BeforePrestige)
    }

    fn primary_path(&self) -> PathBuf {
        self.root.join("profile-primary.json")
    }

    fn backup_path(&self) -> PathBuf {
        self.root.join("profile-backup.json")
    }

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
