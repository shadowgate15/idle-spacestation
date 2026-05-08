//! Game-domain modules for content, simulation, progression, persistence, and snapshots.
//!
//! The Tauri backend builds frontend-facing state from this module tree while keeping
//! simulation data, static definitions, and serde DTO projection separated.

pub mod bit_eq;
pub mod content;
pub mod persistence;
pub mod progression;
pub mod sim;
pub mod snapshot;
