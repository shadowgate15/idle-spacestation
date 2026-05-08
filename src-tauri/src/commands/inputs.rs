//! Strongly-typed input DTOs deserialized from Tauri command payloads.
//!
//! Every frontend → Rust command call from `src/lib/game/api/gateway.ts`
//! wraps the user payload in a `{ input: payload }` envelope; Tauri then
//! deserializes that `input` field into one of the structs in this module.
//! All field names use `serde(rename_all = "camelCase")` to match the
//! TypeScript-side payload shape exactly.
//!
//! See also: [`crate::commands`].

use serde::Deserialize;

/// Payload for [`crate::commands::game_toggle_service`] (frontend alias
/// `game_set_service_activation`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleServiceInput {
    /// Identifier of the service whose activation flag is being changed.
    pub(crate) service_id: String,
    /// Desired activation state: `true` to activate, `false` to deactivate.
    pub(crate) active: bool,
}

/// Payload for [`crate::commands::game_upgrade_system`].
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeSystemInput {
    /// Identifier of the system to upgrade by one level.
    pub(crate) system_id: String,
}

/// Payload for [`crate::commands::game_purchase_doctrine`].
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseDoctrineInput {
    /// Identifier of the doctrine to purchase with prestige fragments.
    pub(crate) doctrine_id: String,
}

/// Payload for [`crate::commands::game_execute_prestige`] (frontend alias
/// `game_confirm_prestige`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPrestigeInput {
    /// Explicit confirmation flag; the command rejects the request unless this
    /// is `true`, guarding against accidental prestige resets.
    pub(crate) confirm: bool,
}

/// Payload for [`crate::commands::game_assign_service_crew`].
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignServiceCrewInput {
    /// Identifier of the service receiving the crew assignment.
    pub(crate) service_id: String,
    /// Total crew count to assign (signed because the frontend may send 0
    /// or negative values; negatives are rejected as `invalid-assignment`).
    pub(crate) assigned_crew: i32,
}

/// Payload for [`crate::commands::game_reprioritize_service`].
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReprioritizeServiceInput {
    /// Identifier of the service whose priority is being shifted.
    pub(crate) service_id: String,
    /// Direction to shift priority — see [`ServicePriorityDirection`].
    pub(crate) direction: ServicePriorityDirection,
}

/// Direction modifier for [`ReprioritizeServiceInput`].
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ServicePriorityDirection {
    /// Increase priority (move the service one slot earlier in the order).
    Up,
    /// Decrease priority (move the service one slot later in the order).
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
