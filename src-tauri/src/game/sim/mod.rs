#![allow(dead_code)]

//! In-place simulation state, tick processing, and deficit recovery.
//!
//! This module owns the mutable run model consumed by the snapshot layer and
//! advances it through the deterministic tick loop used by the Tauri backend.
//! The loop is intentionally split from command handling so both tests and the
//! daemon thread can mutate `RunState` through the same rules.

pub mod deficit;
pub mod state;
pub mod tick;

#[allow(unused_imports)]
pub use state::{RunState, ServicePauseReason, ServiceState, SimState, StationState, SystemState};
#[allow(unused_imports)]
pub use tick::tick;
#[allow(unused_imports)]
pub(crate) use tick::{
    effective_crew_capacity, effective_data_output_multiplier,
    effective_materials_output_multiplier, effective_service_power_upkeep,
    effective_survey_output_multiplier, habitat_level, logistics_level, planet_modifier_total,
    reactor_level, survey_array_level,
};

#[allow(unused_imports)]
pub use self::state::ResourceState;
