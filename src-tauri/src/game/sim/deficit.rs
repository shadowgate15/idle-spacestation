#![allow(dead_code)]

use crate::game::sim::state::{catalog_service_order, RunState, ServicePauseReason};

#[derive(Debug, Clone, Copy, Default)]
pub struct PendingServiceDelta {
    pub power_upkeep: f32,
    pub power_output: f32,
    pub materials_delta: f32,
    pub data_delta: f32,
    pub survey_delta: f32,
}

impl PendingServiceDelta {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

pub fn resolve_power_deficit(state: &mut RunState, pending: &mut [PendingServiceDelta]) {
    if state.resources.power_available >= 0.0 {
        return;
    }

    let refund_ratio = state.hardened_relays_refund_ratio();
    let mut indices: Vec<usize> = state
        .services
        .iter()
        .enumerate()
        .filter(|(_, service)| service.is_active)
        .map(|(index, _)| index)
        .collect();

    indices.sort_by_key(|&index| {
        (
            std::cmp::Reverse(state.services[index].priority),
            std::cmp::Reverse(catalog_service_order(&state.services[index].service_id)),
        )
    });

    for index in indices {
        if state.resources.power_available >= 0.0 {
            break;
        }

        let service = &mut state.services[index];
        let service_pending = &mut pending[index];

        state.resources.power_available -= service_pending.power_output;
        state.resources.power_reserved -= service_pending.power_upkeep;
        state.resources.power_available += service_pending.power_upkeep;
        state.resources.power_available += service_pending.power_upkeep * refund_ratio;

        service.is_active = false;
        service.is_paused = true;
        service.pause_reason = Some(ServicePauseReason::Deficit);
        service.assigned_crew = 0;

        service_pending.clear();
    }

    let crew_assigned: u16 = state
        .services
        .iter()
        .filter(|service| service.is_active)
        .map(|service| service.assigned_crew as u16)
        .sum();

    state.resources.crew_assigned = crew_assigned.min(state.resources.crew_total as u16) as u8;
    state.resources.crew_available = state
        .resources
        .crew_total
        .saturating_sub(state.resources.crew_assigned);
}
