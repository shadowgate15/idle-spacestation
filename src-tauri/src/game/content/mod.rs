//! Static data catalogs for all game content.
//!
//! These modules define the read-only tables that describe every playable entity in the
//! space station simulation: systems, services, planets, resources, and doctrines.
//! All data is declared as `&'static` constants and is never mutated at runtime.
//!
//! See [`crate::game::sim::state::RunState`] for the mutable game state that consumes this data.

#![allow(dead_code)]

pub mod doctrines;
pub mod planets;
pub mod resources;
pub mod services;
pub mod systems;

#[allow(unused_imports)]
pub use doctrines::{DoctrineDefinition, DoctrineEffect, DOCTRINES};
#[allow(unused_imports)]
pub use planets::{PlanetDefinition, PlanetModifier, PlanetModifierTarget, PLANETS};
#[allow(unused_imports)]
pub use resources::{CrewPoolSemantics, ResourceMetadata, ResourceModel, RESOURCES};
#[allow(unused_imports)]
pub use services::{ServiceCategory, ServiceDefinition, SERVICES};
#[allow(unused_imports)]
pub use systems::{
    HabitatRingLevel, LogisticsSpineLevel, ReactorCoreLevel, SurveyArrayLevel, SystemDefinition,
    SystemProgression, SYSTEMS,
};
