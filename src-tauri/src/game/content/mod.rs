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
