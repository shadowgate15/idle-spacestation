//! Versioned save-data DTO + serialization helpers.
//!
//! **Currently scaffolded; not wired into commands as of 2026-05-08.** This
//! module defines the on-disk schema for [`SaveData`] but no Tauri command
//! invokes it; only the unit tests in [`super`] exercise round-tripping.
//!
//! [`SaveData`] is the authoritative DTO written to `profile-primary.json`
//! and `profile-backup.json` by [`super::SaveManager`]. It bundles a
//! [`RunStateSnapshot`] (a serializable mirror of
//! [`crate::game::sim::state::RunState`]), a [`ProfileState`] (cross-run
//! progression that survives prestige), and a [`SaveSettings`] block
//! (user-facing toggles).
//!
//! # Versioning
//!
//! Every saved file embeds [`SAVE_VERSION`] under the `saveVersion` JSON
//! field. The companion [`super::migration`] module reads this value and
//! dispatches to the per-version deserializer ([`deserialize_v1`] today).
//! When the schema changes incompatibly, bump [`SAVE_VERSION`], add a new
//! `deserialize_vN` helper, and extend [`super::migration::migrate`] to
//! upgrade older payloads in place.
//!
//! # Snapshot vs. Runtime
//!
//! The `*Snapshot` types intentionally mirror the runtime types in
//! [`crate::game::sim`] one-for-one rather than re-using them. This keeps
//! the on-disk schema decoupled from runtime layout changes (and from
//! derives like `BitHash` that have no place in serde output).
//! [`From`]/[`Into`] conversions in this file are the only bridge between
//! the two worlds; keep them total and lossless.

use serde::{Deserialize, Serialize};

use crate::game::sim::{RunState, ServicePauseReason};

/// On-disk format version embedded in every [`SaveData`] payload.
///
/// Bump this when changing [`SaveData`] (or any nested `*Snapshot`) in a way
/// that older saves cannot be deserialized as-is. Old loads then route
/// through [`super::migration::migrate`], which must learn how to upgrade
/// the previous version's payload.
pub const SAVE_VERSION: u32 = 1;

/// Cross-run progression metadata that survives prestige resets.
///
/// Where [`crate::game::sim::state::RunState`] holds *current-run* state,
/// `ProfileState` holds the things a player should keep forever: discovered
/// planets, unlocked doctrines, lifetime tick/prestige counters, and the
/// pool of unspent doctrine fragments. Stored alongside the run snapshot in
/// [`SaveData`] but reconstructed independently at prestige time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileState {
    /// Sorted, deduplicated IDs of every planet the player has discovered
    /// across all runs. Mirrors
    /// [`crate::game::sim::StationState::discovered_planet_ids`].
    pub discovered_planet_ids: Vec<String>,
    /// Sorted, deduplicated IDs of every doctrine the player owns. Mirrors
    /// [`crate::game::sim::StationState::doctrine_ids`].
    pub doctrine_ids: Vec<String>,
    /// Unspent doctrine-fragment currency carried across runs.
    pub doctrine_fragments: u32,
    /// Monotonically increasing total of ticks the player has accrued
    /// across all runs. Updated as `max(self, run_state.tick_count)` so a
    /// freshly reset run does not regress the counter.
    pub lifetime_ticks: u64,
    /// Total number of prestige resets the player has performed.
    pub lifetime_prestiges: u32,
}

impl Default for ProfileState {
    fn default() -> Self {
        Self {
            discovered_planet_ids: Vec::new(),
            doctrine_ids: Vec::new(),
            doctrine_fragments: 0,
            lifetime_ticks: 0,
            lifetime_prestiges: 0,
        }
    }
}

impl ProfileState {
    /// Build a fresh [`ProfileState`] derived entirely from `run_state`.
    ///
    /// Equivalent to [`ProfileState::default`] followed by
    /// [`ProfileState::sync_from_run_state`]. Used both as a fixture helper
    /// and by [`SaveData::fresh`] when constructing a brand-new profile.
    pub fn from_run_state(run_state: &RunState) -> Self {
        let mut profile = Self::default();
        profile.sync_from_run_state(run_state);
        profile
    }

    /// Refresh this profile from a live [`RunState`].
    ///
    /// Copies discovered planets, doctrines, and fragment count out of the
    /// run state, sorting/deduplicating list fields for deterministic disk
    /// output. `lifetime_ticks` is updated as `max(current, tick_count)` so
    /// it never regresses. `lifetime_prestiges` is **not** touched here —
    /// only the prestige command should bump that counter.
    pub fn sync_from_run_state(&mut self, run_state: &RunState) {
        self.discovered_planet_ids = sorted_unique(run_state.station.discovered_planet_ids.clone());
        self.doctrine_ids = sorted_unique(run_state.station.doctrine_ids.clone());
        self.doctrine_fragments = run_state.station.doctrine_fragments;
        self.lifetime_ticks = self.lifetime_ticks.max(run_state.tick_count);
    }
}

/// User-facing persistence preferences stored alongside the run state.
///
/// Currently exposes a single toggle ([`SaveSettings::autosave_enabled`])
/// but lives in its own struct so future settings (cadence overrides,
/// cloud-sync flags, etc.) can be added without bumping [`SAVE_VERSION`]
/// for unrelated schema work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSettings {
    /// Whether [`super::SaveManager::save_if_autosave_due`] is permitted to
    /// write. Manual save triggers (visibility/close/prestige) ignore this
    /// flag and always fire.
    pub autosave_enabled: bool,
}

impl Default for SaveSettings {
    fn default() -> Self {
        Self {
            autosave_enabled: true,
        }
    }
}

/// Top-level on-disk save payload.
///
/// Serialized as JSON to `profile-primary.json` and `profile-backup.json` by
/// [`super::SaveManager`]. Every field is `pub` so deserialized saves can
/// be split apart by [`SaveData::into_runtime`] and re-bundled at write
/// time by [`SaveData::from_runtime`]. The `saveVersion` JSON field is the
/// schema discriminator read by [`super::migration::extract_save_version`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveData {
    /// On-disk schema version. Always equals [`SAVE_VERSION`] for payloads
    /// produced by this build; older values are upgraded by
    /// [`super::migration::migrate`] before reaching the runtime.
    pub save_version: u32,
    /// Serializable mirror of the live [`crate::game::sim::state::RunState`].
    pub run_state: RunStateSnapshot,
    /// Cross-run progression metadata (see [`ProfileState`]).
    pub profile_state: ProfileState,
    /// User-facing persistence preferences (see [`SaveSettings`]).
    pub settings: SaveSettings,
}

impl SaveData {
    /// Build a write-ready [`SaveData`] from the live runtime triple.
    ///
    /// Always stamps [`save_version`](SaveData::save_version) with the
    /// current [`SAVE_VERSION`] and re-syncs `profile_state` from
    /// `run_state` via [`ProfileState::sync_from_run_state`] so the
    /// persisted profile cannot drift from the run it was captured with.
    pub fn from_runtime(
        run_state: &RunState,
        profile_state: &ProfileState,
        settings: &SaveSettings,
    ) -> Self {
        let mut synced_profile = profile_state.clone();
        synced_profile.sync_from_run_state(run_state);

        Self {
            save_version: SAVE_VERSION,
            run_state: RunStateSnapshot::from(run_state),
            profile_state: synced_profile,
            settings: settings.clone(),
        }
    }

    /// Decompose a loaded [`SaveData`] back into the runtime triple.
    ///
    /// Returns `(RunState, ProfileState, SaveSettings)` in the same order
    /// expected by [`SaveData::from_runtime`] — round-tripping is lossless
    /// modulo the snapshot/runtime conversion in [`RunStateSnapshot`].
    pub fn into_runtime(self) -> (RunState, ProfileState, SaveSettings) {
        (self.run_state.into(), self.profile_state, self.settings)
    }

    /// Construct a brand-new [`SaveData`] for first-launch or recovery.
    ///
    /// Uses [`crate::game::sim::state::RunState::starter_fixture`] for the
    /// run state, derives a matching [`ProfileState`], and applies default
    /// [`SaveSettings`] (autosave on). [`super::recover_save`] returns this
    /// when both primary and backup are unusable.
    pub fn fresh() -> Self {
        let run_state = RunState::starter_fixture();
        let profile_state = ProfileState::from_run_state(&run_state);
        Self::from_runtime(&run_state, &profile_state, &SaveSettings::default())
    }
}

/// Serialize a [`SaveData`] to pretty-printed JSON for on-disk storage.
///
/// Pretty-printing is deliberate: save files are human-inspectable for
/// debugging and have no size pressure. Only fails if a value cannot be
/// represented in JSON (in practice never, given the current field types).
pub fn serialize_save(data: &SaveData) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(data)
}

/// Deserialize a v1-format save payload directly into [`SaveData`].
///
/// Crate-private because callers should always go through
/// [`super::migration::deserialize_with_migration`] (or
/// [`super::migration::migrate`]) so the version dispatch is honored.
/// Calling this on a non-v1 payload may parse successfully but produces a
/// [`SaveData`] whose [`save_version`](SaveData::save_version) is wrong —
/// [`super::migration::migrate`] guards against that.
pub(crate) fn deserialize_v1(data: &str) -> Result<SaveData, serde_json::Error> {
    serde_json::from_str(data)
}

/// Serializable mirror of [`crate::game::sim::state::RunState`].
///
/// Field-for-field copy of the runtime type, kept as a separate struct so
/// the on-disk schema is immune to runtime-only changes (new derives,
/// reordered fields, internal helper additions). Conversion is done by
/// the [`From<&RunState>`] / [`From<RunStateSnapshot>`] impls below;
/// keep them lossless and total when adding new fields to either side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStateSnapshot {
    /// Number of ticks elapsed in the current run.
    pub tick_count: u64,
    /// Snapshot of the station header (active planet, doctrines, tier).
    pub station: StationStateSnapshot,
    /// Snapshot of resource pools (power, materials, data, crew).
    pub resources: ResourceStateSnapshot,
    /// One snapshot per service, in stable order.
    pub services: Vec<ServiceStateSnapshot>,
    /// One snapshot per system, in stable order.
    pub systems: Vec<SystemStateSnapshot>,
    /// Tick counter for "uninterrupted stable power" used by progression checks.
    pub consecutive_stable_power_ticks: u32,
    /// Total `data` produced across the lifetime of this run (used for prestige math).
    pub lifetime_data_produced: u64,
    /// `true` when the simulation has flagged that an autosave should fire on the next opportunity.
    pub autosave_due: bool,
    /// Number of autosaves performed during this run.
    pub autosave_count: u32,
    /// Tick at which the most recent autosave fired, if any.
    pub last_autosave_tick: Option<u64>,
    /// Whether the run currently meets the prestige unlock criteria.
    pub prestige_eligible: bool,
}

impl From<&RunState> for RunStateSnapshot {
    fn from(value: &RunState) -> Self {
        Self {
            tick_count: value.tick_count,
            station: StationStateSnapshot::from(&value.station),
            resources: ResourceStateSnapshot::from(&value.resources),
            services: value.services.iter().map(ServiceStateSnapshot::from).collect(),
            systems: value.systems.iter().map(SystemStateSnapshot::from).collect(),
            consecutive_stable_power_ticks: value.consecutive_stable_power_ticks,
            lifetime_data_produced: value.lifetime_data_produced,
            autosave_due: value.autosave_due,
            autosave_count: value.autosave_count,
            last_autosave_tick: value.last_autosave_tick,
            prestige_eligible: value.prestige_eligible,
        }
    }
}

impl From<RunStateSnapshot> for RunState {
    fn from(value: RunStateSnapshot) -> Self {
        Self {
            tick_count: value.tick_count,
            station: value.station.into(),
            resources: value.resources.into(),
            services: value.services.into_iter().map(Into::into).collect(),
            systems: value.systems.into_iter().map(Into::into).collect(),
            consecutive_stable_power_ticks: value.consecutive_stable_power_ticks,
            lifetime_data_produced: value.lifetime_data_produced,
            autosave_due: value.autosave_due,
            autosave_count: value.autosave_count,
            last_autosave_tick: value.last_autosave_tick,
            prestige_eligible: value.prestige_eligible,
        }
    }
}

/// Serializable mirror of [`crate::game::sim::StationState`].
///
/// Captures the station header: which planet is active, every planet
/// discovered, owned doctrines, fragment currency, survey progress for
/// the active planet, and the station tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationStateSnapshot {
    /// ID of the planet the station is currently orbiting/working.
    pub active_planet_id: String,
    /// Sorted, deduplicated IDs of every planet ever discovered.
    pub discovered_planet_ids: Vec<String>,
    /// Sorted, deduplicated IDs of every doctrine the player owns.
    pub doctrine_ids: Vec<String>,
    /// Unspent doctrine-fragment currency.
    pub doctrine_fragments: u32,
    /// Survey progress (0.0..) toward unlocking the next planet.
    pub survey_progress: f32,
    /// Current station tier (1-based); gates which services/systems unlock.
    pub station_tier: u8,
}

impl From<&crate::game::sim::StationState> for StationStateSnapshot {
    fn from(value: &crate::game::sim::StationState) -> Self {
        Self {
            active_planet_id: value.active_planet_id.clone(),
            discovered_planet_ids: value.discovered_planet_ids.clone(),
            doctrine_ids: value.doctrine_ids.clone(),
            doctrine_fragments: value.doctrine_fragments,
            survey_progress: value.survey_progress,
            station_tier: value.station_tier,
        }
    }
}

impl From<StationStateSnapshot> for crate::game::sim::StationState {
    fn from(value: StationStateSnapshot) -> Self {
        Self {
            active_planet_id: value.active_planet_id,
            discovered_planet_ids: value.discovered_planet_ids,
            doctrine_ids: value.doctrine_ids,
            doctrine_fragments: value.doctrine_fragments,
            survey_progress: value.survey_progress,
            station_tier: value.station_tier,
        }
    }
}

/// Serializable mirror of [`crate::game::sim::ResourceState`].
///
/// Captures the per-tick power ledger (generated/reserved/available),
/// material/data stockpiles, and crew accounting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceStateSnapshot {
    /// Total power produced this tick (sum of generator outputs).
    pub power_generated: f32,
    /// Power claimed by active services (reservation against generated).
    pub power_reserved: f32,
    /// `power_generated - power_reserved`; what is free for new activations.
    pub power_available: f32,
    /// Current material stockpile.
    pub materials: f32,
    /// Current data stockpile.
    pub data: f32,
    /// Total crew available to the station.
    pub crew_total: u8,
    /// Crew currently assigned to services.
    pub crew_assigned: u8,
    /// `crew_total - crew_assigned`.
    pub crew_available: u8,
}

impl From<&crate::game::sim::ResourceState> for ResourceStateSnapshot {
    fn from(value: &crate::game::sim::ResourceState) -> Self {
        Self {
            power_generated: value.power_generated,
            power_reserved: value.power_reserved,
            power_available: value.power_available,
            materials: value.materials,
            data: value.data,
            crew_total: value.crew_total,
            crew_assigned: value.crew_assigned,
            crew_available: value.crew_available,
        }
    }
}

impl From<ResourceStateSnapshot> for crate::game::sim::ResourceState {
    fn from(value: ResourceStateSnapshot) -> Self {
        Self {
            power_generated: value.power_generated,
            power_reserved: value.power_reserved,
            power_available: value.power_available,
            materials: value.materials,
            data: value.data,
            crew_total: value.crew_total,
            crew_assigned: value.crew_assigned,
            crew_available: value.crew_available,
        }
    }
}

/// Serializable mirror of [`crate::game::sim::ServiceState`].
///
/// Captures one service's persistent state: identity, the player-requested
/// active flag, the actual current activation, pause status with optional
/// reason, scheduling priority, and crew assignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStateSnapshot {
    /// ID of the service definition (see [`crate::game::content::services`]).
    pub service_id: String,
    /// Player-requested activation. The simulation may keep `is_active`
    /// `false` even when this is `true` (e.g. crew or power deficit).
    pub desired_active: bool,
    /// Whether the service is currently running this tick.
    pub is_active: bool,
    /// Whether the service is paused (a temporary, automatic suspension).
    pub is_paused: bool,
    /// Optional explanation for `is_paused`. See [`ServicePauseReasonSnapshot`].
    pub pause_reason: Option<ServicePauseReasonSnapshot>,
    /// Scheduling priority (lower runs first when contending for resources).
    pub priority: u8,
    /// Number of crew currently assigned to this service.
    pub assigned_crew: u8,
}

impl From<&crate::game::sim::ServiceState> for ServiceStateSnapshot {
    fn from(value: &crate::game::sim::ServiceState) -> Self {
        Self {
            service_id: value.service_id.clone(),
            desired_active: value.desired_active,
            is_active: value.is_active,
            is_paused: value.is_paused,
            pause_reason: value.pause_reason.map(Into::into),
            priority: value.priority,
            assigned_crew: value.assigned_crew,
        }
    }
}

impl From<ServiceStateSnapshot> for crate::game::sim::ServiceState {
    fn from(value: ServiceStateSnapshot) -> Self {
        Self {
            service_id: value.service_id,
            desired_active: value.desired_active,
            is_active: value.is_active,
            is_paused: value.is_paused,
            pause_reason: value.pause_reason.map(Into::into),
            priority: value.priority,
            assigned_crew: value.assigned_crew,
        }
    }
}

/// Serializable mirror of [`crate::game::sim::ServicePauseReason`].
///
/// Distinct from the runtime enum so the on-disk representation does not
/// inherit runtime-only derives (e.g. `BitHash`). Conversion is total via
/// the `From` impls below; keep it in sync when adding new pause reasons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServicePauseReasonSnapshot {
    /// Paused because no service slot is available at the current tier.
    Capacity,
    /// Paused because no crew could be assigned.
    Crew,
    /// Paused because an input resource (materials/data/power) was in deficit.
    Deficit,
    /// Paused because power capacity was exceeded.
    PowerCap,
}

impl From<ServicePauseReason> for ServicePauseReasonSnapshot {
    fn from(value: ServicePauseReason) -> Self {
        match value {
            ServicePauseReason::Capacity => Self::Capacity,
            ServicePauseReason::Crew => Self::Crew,
            ServicePauseReason::Deficit => Self::Deficit,
            ServicePauseReason::PowerCap => Self::PowerCap,
        }
    }
}

impl From<ServicePauseReasonSnapshot> for ServicePauseReason {
    fn from(value: ServicePauseReasonSnapshot) -> Self {
        match value {
            ServicePauseReasonSnapshot::Capacity => Self::Capacity,
            ServicePauseReasonSnapshot::Crew => Self::Crew,
            ServicePauseReasonSnapshot::Deficit => Self::Deficit,
            ServicePauseReasonSnapshot::PowerCap => Self::PowerCap,
        }
    }
}

/// Serializable mirror of [`crate::game::sim::SystemState`].
///
/// Captures one system's identity and current upgrade level. Static
/// definition data (cost curves, unlocks, etc.) is not persisted — it is
/// resolved from [`crate::game::content::systems`] at load time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStateSnapshot {
    /// ID of the system definition (see [`crate::game::content::systems`]).
    pub system_id: String,
    /// Current upgrade level (0-based).
    pub level: u8,
}

impl From<&crate::game::sim::SystemState> for SystemStateSnapshot {
    fn from(value: &crate::game::sim::SystemState) -> Self {
        Self {
            system_id: value.system_id.clone(),
            level: value.level,
        }
    }
}

impl From<SystemStateSnapshot> for crate::game::sim::SystemState {
    fn from(value: SystemStateSnapshot) -> Self {
        Self {
            system_id: value.system_id,
            level: value.level,
        }
    }
}

/// Sort and deduplicate a `Vec<String>` in place, returning ownership.
///
/// Used by [`ProfileState::sync_from_run_state`] so list fields produce
/// byte-stable JSON regardless of the order the runtime accumulated them
/// in. Insertion order in the runtime is not load-bearing for these fields.
fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}
