use serde::{Deserialize, Serialize};

use crate::game::snapshot::RawGameSnapshot;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsVisibilityInput {
    pub(crate) visible: bool,
}

#[derive(Serialize)]
pub struct DevtoolsStateResponse {
    pub(crate) visible: bool,
    pub(crate) snapshot: RawGameSnapshot,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsApplyResourcesInput {
    pub(crate) materials: f32,
    pub(crate) data: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsApplyCrewInput {
    pub(crate) crew_total: u8,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsApplySystemEntry {
    pub(crate) id: String,
    pub(crate) level: u8,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsApplySystemsInput {
    pub(crate) systems: Vec<DevtoolsApplySystemEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsServiceEntry {
    pub(crate) id: String,
    pub(crate) desired_active: bool,
    pub(crate) assigned_crew: u8,
    pub(crate) priority: u8,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsApplyServicesInput {
    pub(crate) services: Vec<DevtoolsServiceEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsApplyProgressionInput {
    pub(crate) doctrine_fragments: u32,
    pub(crate) unlocked_doctrines: Vec<String>,
    pub(crate) discovered_planets: Vec<String>,
    pub(crate) active_planet: String,
    pub(crate) survey_progress: std::collections::HashMap<String, f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsAdvanceTicksInput {
    pub(crate) count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DevtoolsVisibilityChangedEvent {
    pub(crate) visible: bool,
}
