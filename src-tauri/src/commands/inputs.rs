use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleServiceInput {
    pub(crate) service_id: String,
    pub(crate) active: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeSystemInput {
    pub(crate) system_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseDoctrineInput {
    pub(crate) doctrine_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPrestigeInput {
    pub(crate) confirm: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignServiceCrewInput {
    pub(crate) service_id: String,
    pub(crate) assigned_crew: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReprioritizeServiceInput {
    pub(crate) service_id: String,
    pub(crate) direction: ServicePriorityDirection,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ServicePriorityDirection {
    Up,
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_service_input_accepts_camel_case_payload() {
        let input: ToggleServiceInput = serde_json::from_str(
            r#"{"serviceId":"solar-harvester","active":true}"#,
        )
        .expect("camelCase payload should deserialize");

        assert_eq!(input.service_id, "solar-harvester");
        assert!(input.active);
    }

    #[test]
    fn upgrade_system_input_accepts_camel_case_payload() {
        let input: UpgradeSystemInput = serde_json::from_str(r#"{"systemId":"reactor-core"}"#)
            .expect("camelCase payload should deserialize");

        assert_eq!(input.system_id, "reactor-core");
    }
}
