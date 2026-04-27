use serde::Serialize;

use crate::game::content::doctrines::{doctrine_by_id, DOCTRINES};
use crate::game::content::planets::{
    planet_by_id, PlanetDefinition, PlanetModifierTarget, AURORA_PIER_ID, CINDER_FORGE_ID, PLANETS,
    SOLSTICE_ANCHOR_ID,
};
use crate::game::content::services::{service_by_id, ServiceCategory, SURVEY_UPLINK_ID};
use crate::game::content::systems::{
    system_by_id, SystemProgression, HABITAT_RING_ID, LOGISTICS_SPINE_ID, REACTOR_CORE_ID,
    SURVEY_ARRAY_ID, SYSTEMS,
};
use crate::game::progression::{
    calculate_station_tier, evaluate_prestige_eligibility, PrestigeIneligibleReason, PrestigeProfile,
    POWER_STABILITY_TICKS_REQUIRED,
};
use crate::game::sim::{RunState, ServicePauseReason};
use crate::game::sim::state::{
    AURORA_PIER_SURVEY_THRESHOLD, CINDER_FORGE_SURVEY_THRESHOLD, HOUSEKEEPING_POWER_PER_SECOND,
    SECONDS_PER_TICK,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResponse {
    pub ok: bool,
    pub snapshot: RawGameSnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLoadResponse {
    pub ok: bool,
    pub status: String,
    pub snapshot: RawGameSnapshot,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawGameSnapshot {
    pub meta: SnapshotMeta,
    pub run: RunSnapshot,
    pub resources: ResourcesSnapshot,
    pub systems: Vec<RawSystemStateSnapshot>,
    pub services: Vec<RawServiceStateSnapshot>,
    pub route_snapshots: RouteSnapshots,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMeta {
    pub source: String,
    pub fixture_name: Option<String>,
    pub tick_count: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunSnapshot {
    pub active_planet_id: String,
    pub discovered_planet_ids: Vec<String>,
    pub doctrine_ids: Vec<String>,
    pub doctrine_fragments: u32,
    pub survey_progress: f32,
    pub station_tier: u8,
    pub stable_power_seconds: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesSnapshot {
    pub power: PowerSnapshot,
    pub materials: f32,
    pub data: f32,
    pub crew: CrewSnapshot,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerSnapshot {
    pub generated: f32,
    pub reserved: f32,
    pub available: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrewSnapshot {
    pub total: u8,
    pub assigned: u8,
    pub available: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSystemStateSnapshot {
    pub id: String,
    pub level: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawServiceStateSnapshot {
    pub id: String,
    pub desired_active: bool,
    pub is_active: bool,
    pub is_paused: bool,
    pub pause_reason: Option<String>,
    pub priority: u8,
    pub assigned_crew: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteSnapshots {
    pub overview: OverviewRouteSnapshot,
    pub systems: SystemsRouteSnapshot,
    pub services: ServicesRouteSnapshot,
    pub planets: PlanetsRouteSnapshot,
    pub prestige: PrestigeRouteSnapshot,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewRouteSnapshot {
    pub active_planet: ActivePlanetSnapshot,
    pub resource_deltas: Vec<ResourceDeltaSnapshot>,
    pub deficit_warnings: Vec<WarningSnapshot>,
    pub station_tier: StationTierSnapshot,
    pub service_utilization: ServiceUtilizationSnapshot,
    pub survey_progress: SurveyProgressSnapshot,
    pub guidance_triggers: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivePlanetSnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub modifiers: Vec<PlanetModifierSnapshot>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanetModifierSnapshot {
    pub target: String,
    pub label: String,
    pub percent: f32,
    pub effect_text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDeltaSnapshot {
    pub id: String,
    pub label: String,
    pub delta_per_second: f32,
    pub trend: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WarningSnapshot {
    pub code: String,
    pub severity: String,
    pub title: String,
    pub body: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StationTierSnapshot {
    pub current: u8,
    pub max: u8,
    pub label: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceUtilizationSnapshot {
    pub active: usize,
    pub capacity: usize,
    pub available: usize,
    pub summary: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurveyProgressSnapshot {
    pub current: f32,
    pub next_threshold: Option<f32>,
    pub next_planet_id: Option<String>,
    pub next_planet_name: Option<String>,
    pub summary: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemsRouteSnapshot {
    pub systems: Vec<SystemRouteEntrySnapshot>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemRouteEntrySnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub level: u8,
    pub max_level: u8,
    pub cap_values: Vec<SystemCapSnapshot>,
    pub upgrade_cost_materials: Option<u32>,
    pub can_upgrade: bool,
    pub upgrade_blocked_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemCapSnapshot {
    pub key: String,
    pub label: String,
    pub value: f32,
    pub unit: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicesRouteSnapshot {
    pub services: Vec<ServiceRouteEntrySnapshot>,
    pub utilization: ServiceUtilizationSnapshot,
    pub deficit_warnings: Vec<WarningSnapshot>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRouteEntrySnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub family: String,
    pub priority_order: u8,
    pub status: String,
    pub status_label: String,
    pub desired_active: bool,
    pub crew_assignment: ServiceCrewAssignmentSnapshot,
    pub power_usage: ServicePowerUsageSnapshot,
    pub disabled_reasons: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceCrewAssignmentSnapshot {
    pub current: u8,
    pub required: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicePowerUsageSnapshot {
    pub upkeep: f32,
    pub output: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanetsRouteSnapshot {
    pub active_planet_id: String,
    pub planets: Vec<PlanetRouteEntrySnapshot>,
    pub survey_progress: SurveyProgressSnapshot,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanetRouteEntrySnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub discovered: bool,
    pub active: bool,
    pub selectable_for_next_run: bool,
    pub selectability_reason: Option<String>,
    pub modifiers: Vec<PlanetModifierSnapshot>,
    pub survey_threshold: Option<f32>,
    pub survey_progress: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrestigeRouteSnapshot {
    pub eligibility: PrestigeEligibilitySnapshot,
    pub doctrine_fragments: u32,
    pub unlocked_doctrines: Vec<DoctrineSnapshot>,
    pub purchase_options: Vec<DoctrinePurchaseOptionSnapshot>,
    pub reset_consequences: Vec<ResetConsequenceSnapshot>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrestigeEligibilitySnapshot {
    pub eligible: bool,
    pub reason_codes: Vec<String>,
    pub summary: String,
    pub stable_power_seconds: f32,
    pub required_stable_power_seconds: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctrineSnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctrinePurchaseOptionSnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost_fragments: u32,
    pub available: bool,
    pub blocked_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetConsequenceSnapshot {
    pub label: String,
    pub outcome: String,
    pub summary: String,
}

pub fn build_snapshot(run_state: &RunState, profile: &PrestigeProfile) -> RawGameSnapshot {
    let station_tier = calculate_station_tier(run_state);
    let stable_power_seconds = stable_power_seconds(run_state.consecutive_stable_power_ticks);
    let doctrine_ids = sorted_unique(merged_ids(&run_state.station.doctrine_ids, &profile.doctrine_ids));
    let discovered_planet_ids =
        sorted_unique(merged_ids(&run_state.station.discovered_planet_ids, &profile.discovered_planet_ids));
    let deficit_warnings = build_deficit_warnings(run_state);
    let service_utilization = build_service_utilization(run_state);
    let survey_progress = build_survey_progress(run_state);
    let prestige_reason_codes = build_prestige_reason_codes(run_state);

    RawGameSnapshot {
        meta: SnapshotMeta {
            source: "tauri".to_string(),
            fixture_name: None,
            tick_count: run_state.tick_count,
        },
        run: RunSnapshot {
            active_planet_id: run_state.station.active_planet_id.clone(),
            discovered_planet_ids: discovered_planet_ids.clone(),
            doctrine_ids: doctrine_ids.clone(),
            doctrine_fragments: run_state.station.doctrine_fragments,
            survey_progress: run_state.station.survey_progress,
            station_tier,
            stable_power_seconds,
        },
        resources: ResourcesSnapshot {
            power: PowerSnapshot {
                generated: run_state.resources.power_generated,
                reserved: run_state.resources.power_reserved,
                available: run_state.resources.power_available,
            },
            materials: run_state.resources.materials,
            data: run_state.resources.data,
            crew: CrewSnapshot {
                total: run_state.resources.crew_total,
                assigned: run_state.resources.crew_assigned,
                available: run_state.resources.crew_available,
            },
        },
        systems: run_state
            .systems
            .iter()
            .map(|system| RawSystemStateSnapshot {
                id: system.system_id.clone(),
                level: system.level,
            })
            .collect(),
        services: run_state
            .services
            .iter()
            .map(|service| RawServiceStateSnapshot {
                id: service.service_id.clone(),
                desired_active: service.desired_active,
                is_active: service.is_active,
                is_paused: service.is_paused,
                pause_reason: service.pause_reason.map(service_pause_reason_code),
                priority: service.priority,
                assigned_crew: service.assigned_crew,
            })
            .collect(),
        route_snapshots: RouteSnapshots {
            overview: build_overview_route(
                run_state,
                station_tier,
                deficit_warnings.clone(),
                service_utilization.clone(),
                survey_progress.clone(),
                prestige_reason_codes.is_empty(),
            ),
            systems: build_systems_route(run_state),
            services: build_services_route(run_state, service_utilization, deficit_warnings),
            planets: build_planets_route(run_state, survey_progress),
            prestige: build_prestige_route(
                run_state,
                doctrine_ids,
                stable_power_seconds,
                prestige_reason_codes,
            ),
        },
    }
}

fn build_overview_route(
    run_state: &RunState,
    station_tier: u8,
    deficit_warnings: Vec<WarningSnapshot>,
    service_utilization: ServiceUtilizationSnapshot,
    survey_progress: SurveyProgressSnapshot,
    prestige_eligible: bool,
) -> OverviewRouteSnapshot {
    let active_planet = run_state
        .active_planet_definition();
    let mut guidance_triggers = vec!["review-station-status".to_string()];

    if !deficit_warnings.is_empty() {
        guidance_triggers.push("clear-power-deficit".to_string());
    }
    if station_tier < 4 {
        guidance_triggers.push("upgrade-reactor-core".to_string());
    }
    if service_utilization.active >= service_utilization.capacity {
        guidance_triggers.push("upgrade-logistics-spine".to_string());
    }
    if !run_state
        .services
        .iter()
        .any(|service| service.service_id == SURVEY_UPLINK_ID && service.is_active)
    {
        guidance_triggers.push("activate-survey-uplink".to_string());
    }
    if run_state
        .station
        .discovered_planet_ids
        .iter()
        .filter(|planet_id| planet_id.as_str() != SOLSTICE_ANCHOR_ID)
        .count()
        < 2
    {
        guidance_triggers.push("discover-second-planet".to_string());
    }
    if run_state.station.doctrine_fragments > 0 {
        guidance_triggers.push("spend-doctrine-fragment".to_string());
    }
    if prestige_eligible {
        guidance_triggers.push("prestige-now".to_string());
    }

    guidance_triggers.sort();
    guidance_triggers.dedup();

    OverviewRouteSnapshot {
        active_planet: ActivePlanetSnapshot {
            id: active_planet.id.to_string(),
            name: active_planet.label.to_string(),
            description: active_planet.description.to_string(),
            modifiers: active_planet
                .modifiers
                .iter()
                .map(build_planet_modifier)
                .collect(),
        },
        resource_deltas: build_resource_deltas(run_state),
        deficit_warnings,
        station_tier: StationTierSnapshot {
            current: station_tier,
            max: 4,
            label: format!("Tier {station_tier}"),
        },
        service_utilization,
        survey_progress,
        guidance_triggers,
    }
}

fn build_systems_route(run_state: &RunState) -> SystemsRouteSnapshot {
    SystemsRouteSnapshot {
        systems: SYSTEMS
            .iter()
            .map(|system| build_system_entry(run_state, system.id))
            .collect(),
    }
}

fn build_services_route(
    run_state: &RunState,
    utilization: ServiceUtilizationSnapshot,
    deficit_warnings: Vec<WarningSnapshot>,
) -> ServicesRouteSnapshot {
    let mut services: Vec<_> = run_state.services.iter().collect();
    services.sort_by_key(|service| service.priority);

    ServicesRouteSnapshot {
        services: services
            .into_iter()
            .map(|service| {
                let definition = service.definition();
                let status = if service.is_active {
                    "active"
                } else if service.is_paused || service.desired_active {
                    "paused"
                } else {
                    "disabled"
                };

                ServiceRouteEntrySnapshot {
                    id: definition.id.to_string(),
                    name: definition.label.to_string(),
                    description: service_description(definition.id).to_string(),
                    family: service_family(definition.category).to_string(),
                    priority_order: service.priority,
                    status: status.to_string(),
                    status_label: title_case(status),
                    desired_active: service.desired_active,
                    crew_assignment: ServiceCrewAssignmentSnapshot {
                        current: service.assigned_crew,
                        required: definition.crew_required,
                    },
                    power_usage: ServicePowerUsageSnapshot {
                        upkeep: round2(effective_service_power_upkeep(run_state, definition.id)),
                        output: round2(definition.power_output),
                    },
                    disabled_reasons: service
                        .pause_reason
                        .map(service_pause_reason_code)
                        .into_iter()
                        .collect(),
                }
            })
            .collect(),
        utilization,
        deficit_warnings,
    }
}

fn build_planets_route(
    run_state: &RunState,
    survey_progress: SurveyProgressSnapshot,
) -> PlanetsRouteSnapshot {
    PlanetsRouteSnapshot {
        active_planet_id: run_state.station.active_planet_id.clone(),
        planets: PLANETS.iter().map(|planet| build_planet_entry(run_state, planet)).collect(),
        survey_progress,
    }
}

fn build_prestige_route(
    run_state: &RunState,
    doctrine_ids: Vec<String>,
    stable_seconds: f32,
    reason_codes: Vec<String>,
) -> PrestigeRouteSnapshot {
    let doctrine_fragments = run_state.station.doctrine_fragments;

    PrestigeRouteSnapshot {
        eligibility: PrestigeEligibilitySnapshot {
            eligible: reason_codes.is_empty(),
            summary: if reason_codes.is_empty() {
                "Prestige is available. Doctrine fragments and discovered planets will persist into the next run."
                    .to_string()
            } else {
                format!("Prestige blocked: {}.", reason_codes.join(", "))
            },
            reason_codes,
            stable_power_seconds: stable_seconds,
            required_stable_power_seconds: stable_power_seconds(POWER_STABILITY_TICKS_REQUIRED),
        },
        doctrine_fragments,
        unlocked_doctrines: doctrine_ids
            .iter()
            .filter_map(|doctrine_id| doctrine_by_id(doctrine_id))
            .map(|doctrine| DoctrineSnapshot {
                id: doctrine.id.to_string(),
                name: doctrine.label.to_string(),
                description: doctrine.description.to_string(),
            })
            .collect(),
        purchase_options: DOCTRINES
            .iter()
            .map(|doctrine| DoctrinePurchaseOptionSnapshot {
                id: doctrine.id.to_string(),
                name: doctrine.label.to_string(),
                description: doctrine.description.to_string(),
                cost_fragments: 1,
                available: !doctrine_ids.iter().any(|owned| owned == doctrine.id) && doctrine_fragments > 0,
                blocked_reason: if doctrine_ids.iter().any(|owned| owned == doctrine.id) {
                    Some("already-unlocked".to_string())
                } else if doctrine_fragments == 0 {
                    Some("insufficient-fragments".to_string())
                } else {
                    None
                },
            })
            .collect(),
        reset_consequences: vec![
            ResetConsequenceSnapshot {
                label: "Discovered planets".to_string(),
                outcome: "retain".to_string(),
                summary: "Unlocked planets remain selectable for future runs.".to_string(),
            },
            ResetConsequenceSnapshot {
                label: "Unlocked doctrines".to_string(),
                outcome: "retain".to_string(),
                summary: "Doctrine unlocks and spent fragments persist.".to_string(),
            },
            ResetConsequenceSnapshot {
                label: "Doctrine fragments".to_string(),
                outcome: "retain".to_string(),
                summary: "Current fragment balance carries into the next run.".to_string(),
            },
            ResetConsequenceSnapshot {
                label: "Lifetime stats".to_string(),
                outcome: "retain".to_string(),
                summary: "Lifetime ticks, prestiges, and best pace remain in the profile.".to_string(),
            },
            ResetConsequenceSnapshot {
                label: "Materials and Data".to_string(),
                outcome: "reset".to_string(),
                summary: "Run stockpiles return to fresh-profile values.".to_string(),
            },
            ResetConsequenceSnapshot {
                label: "Services and assignments".to_string(),
                outcome: "reset".to_string(),
                summary: "All services return to inactive with no Crew assigned.".to_string(),
            },
            ResetConsequenceSnapshot {
                label: "System levels and survey progress".to_string(),
                outcome: "reset".to_string(),
                summary: "System upgrades and current survey progress are cleared.".to_string(),
            },
        ],
    }
}

fn build_system_entry(run_state: &RunState, system_id: &str) -> SystemRouteEntrySnapshot {
    let system = system_by_id(system_id).expect("system must exist in catalog");
    let level = run_state.system_level(system_id).unwrap_or(1);

    match system.progression {
        SystemProgression::ReactorCore(levels) => {
            let current = levels[(level.saturating_sub(1)) as usize];
            let cap_values = vec![
                SystemCapSnapshot {
                    key: "power-output".to_string(),
                    label: "Power output".to_string(),
                    value: current.power_output,
                    unit: "power".to_string(),
                },
                SystemCapSnapshot {
                    key: "service-power-cap".to_string(),
                    label: "Service power cap".to_string(),
                    value: current.service_power_cap as f32,
                    unit: "power".to_string(),
                },
            ];

            build_system_route_entry(
                run_state,
                system_id,
                level,
                levels.len() as u8,
                current.upgrade_cost_materials,
                cap_values,
            )
        }
        SystemProgression::HabitatRing(levels) => {
            let current = levels[(level.saturating_sub(1)) as usize];
            let cap_values = vec![
                SystemCapSnapshot {
                    key: "crew-capacity".to_string(),
                    label: "Crew capacity".to_string(),
                    value: effective_crew_capacity(run_state, current.crew_capacity) as f32,
                    unit: "crew".to_string(),
                },
                SystemCapSnapshot {
                    key: "crew-recovery".to_string(),
                    label: "Crew recovery ceiling".to_string(),
                    value: current.recovery_ceiling_per_minute,
                    unit: "crew/min".to_string(),
                },
            ];

            build_system_route_entry(
                run_state,
                system_id,
                level,
                levels.len() as u8,
                current.upgrade_cost_materials,
                cap_values,
            )
        }
        SystemProgression::LogisticsSpine(levels) => {
            let current = levels[(level.saturating_sub(1)) as usize];
            let cap_values = vec![
                SystemCapSnapshot {
                    key: "active-service-slots".to_string(),
                    label: "Active service slots".to_string(),
                    value: current.active_service_slots as f32,
                    unit: "slots".to_string(),
                },
                SystemCapSnapshot {
                    key: "materials-capacity".to_string(),
                    label: "Materials capacity".to_string(),
                    value: current.materials_capacity as f32,
                    unit: "materials".to_string(),
                },
            ];

            build_system_route_entry(
                run_state,
                system_id,
                level,
                levels.len() as u8,
                current.upgrade_cost_materials,
                cap_values,
            )
        }
        SystemProgression::SurveyArray(levels) => {
            let current = levels[(level.saturating_sub(1)) as usize];
            let cap_values = vec![
                SystemCapSnapshot {
                    key: "data-multiplier".to_string(),
                    label: "Data multiplier".to_string(),
                    value: current.data_multiplier,
                    unit: "x".to_string(),
                },
                SystemCapSnapshot {
                    key: "survey-multiplier".to_string(),
                    label: "Survey multiplier".to_string(),
                    value: current.survey_multiplier,
                    unit: "x".to_string(),
                },
            ];

            build_system_route_entry(
                run_state,
                system_id,
                level,
                levels.len() as u8,
                current.upgrade_cost_materials,
                cap_values,
            )
        }
    }
}

fn build_system_route_entry(
    run_state: &RunState,
    system_id: &str,
    level: u8,
    max_level: u8,
    upgrade_cost_materials: Option<u32>,
    cap_values: Vec<SystemCapSnapshot>,
) -> SystemRouteEntrySnapshot {
    let can_upgrade = upgrade_cost_materials
        .map(|cost| run_state.resources.materials >= cost as f32)
        .unwrap_or(false);
    let upgrade_blocked_reason = match upgrade_cost_materials {
        None => Some("Max level reached.".to_string()),
        Some(cost) if can_upgrade => None,
        Some(cost) => Some(format!("Needs {cost} Materials.")),
    };

    SystemRouteEntrySnapshot {
        id: system_id.to_string(),
        name: system_label(system_id).to_string(),
        description: system_description(system_id).to_string(),
        level,
        max_level,
        cap_values,
        upgrade_cost_materials,
        can_upgrade,
        upgrade_blocked_reason,
    }
}

fn build_planet_entry(run_state: &RunState, planet: &PlanetDefinition) -> PlanetRouteEntrySnapshot {
    let discovered = run_state
        .station
        .discovered_planet_ids
        .iter()
        .any(|planet_id| planet_id == planet.id);
    let active = run_state.station.active_planet_id == planet.id;
    let survey_threshold = survey_threshold(planet.id);

    PlanetRouteEntrySnapshot {
        id: planet.id.to_string(),
        name: planet.label.to_string(),
        description: planet.description.to_string(),
        discovered,
        active,
        selectable_for_next_run: discovered && !active,
        selectability_reason: if !discovered {
            Some("Survey progress has not reached this world yet.".to_string())
        } else if active {
            Some("Current run already operates on this planet.".to_string())
        } else {
            None
        },
        modifiers: planet.modifiers.iter().map(build_planet_modifier).collect(),
        survey_threshold,
        survey_progress: survey_threshold
            .map(|threshold| run_state.station.survey_progress.min(threshold))
            .unwrap_or(run_state.station.survey_progress),
    }
}

fn build_planet_modifier(modifier: &crate::game::content::planets::PlanetModifier) -> PlanetModifierSnapshot {
    let label = planet_modifier_label(modifier.target);

    PlanetModifierSnapshot {
        target: planet_modifier_target_code(modifier.target).to_string(),
        label: label.to_string(),
        percent: modifier.percent,
        effect_text: format!("{:+.0}% {label}", modifier.percent * 100.0),
    }
}

fn build_resource_deltas(run_state: &RunState) -> Vec<ResourceDeltaSnapshot> {
    let active_services: Vec<_> = run_state.services.iter().filter(|service| service.is_active).collect();
    let materials_delta_per_second: f32 = active_services
        .iter()
        .map(|service| {
            let definition = service.definition();
            let materials_output = definition.materials_output * effective_materials_output_multiplier(run_state);
            let materials_input = definition.materials_upkeep + (-definition.materials_input).max(0.0);
            materials_output - materials_input
        })
        .sum();
    let data_delta_per_second: f32 = active_services
        .iter()
        .map(|service| service.definition().data_output * effective_data_output_multiplier(run_state))
        .sum();
    let survey_delta_per_second: f32 = active_services
        .iter()
        .map(|service| {
            service.definition().survey_points
                * effective_survey_output_multiplier(run_state, &service.service_id)
        })
        .sum();
    let power_delta_per_second: f32 = active_services
        .iter()
        .map(|service| service.definition().power_output)
        .sum::<f32>()
        - active_services
            .iter()
            .map(|service| effective_service_power_upkeep(run_state, &service.service_id))
            .sum::<f32>()
        - HOUSEKEEPING_POWER_PER_SECOND;

    vec![
        make_resource_delta("power", "Power", power_delta_per_second),
        make_resource_delta("materials", "Materials", materials_delta_per_second),
        make_resource_delta("data", "Data", data_delta_per_second),
        make_resource_delta("crew", "Survey progress", survey_delta_per_second),
    ]
}

fn build_deficit_warnings(run_state: &RunState) -> Vec<WarningSnapshot> {
    let mut warnings = Vec::new();
    let deficit_services: Vec<_> = run_state
        .services
        .iter()
        .filter(|service| matches!(service.pause_reason, Some(ServicePauseReason::Deficit)))
        .collect();

    if run_state.resources.power_available < 0.0 {
        warnings.push(WarningSnapshot {
            code: "power-deficit".to_string(),
            severity: "critical".to_string(),
            title: "Power deficit in progress".to_string(),
            body: format!(
                "Reserve is {:.1} below zero. Lower-priority services are being shed.",
                run_state.resources.power_available.abs()
            ),
        });
    }

    if !deficit_services.is_empty() {
        warnings.push(WarningSnapshot {
            code: "services-paused-by-deficit".to_string(),
            severity: "warning".to_string(),
            title: "Services paused by deficit handling".to_string(),
            body: deficit_services
                .iter()
                .map(|service| service.definition().label)
                .collect::<Vec<_>>()
                .join(", "),
        });
    }

    warnings
}

fn build_service_utilization(run_state: &RunState) -> ServiceUtilizationSnapshot {
    let active = run_state.services.iter().filter(|service| service.is_active).count();
    let capacity = logistics_active_service_slots(run_state) as usize;

    ServiceUtilizationSnapshot {
        active,
        capacity,
        available: capacity.saturating_sub(active),
        summary: format!("{active} of {capacity} active service slots in use"),
    }
}

fn build_survey_progress(run_state: &RunState) -> SurveyProgressSnapshot {
    let next_planet = [CINDER_FORGE_ID, AURORA_PIER_ID]
        .into_iter()
        .find(|planet_id| {
            !run_state
                .station
                .discovered_planet_ids
                .iter()
                .any(|discovered| discovered == planet_id)
        });

    match next_planet {
        None => SurveyProgressSnapshot {
            current: run_state.station.survey_progress,
            next_threshold: None,
            next_planet_id: None,
            next_planet_name: None,
            summary: "All survey targets discovered.".to_string(),
        },
        Some(next_planet_id) => {
            let next_planet_definition =
                planet_by_id(next_planet_id).expect("next survey target must exist in catalog");
            let next_threshold = survey_threshold(next_planet_id);

            SurveyProgressSnapshot {
                current: run_state.station.survey_progress,
                next_threshold,
                next_planet_id: Some(next_planet_id.to_string()),
                next_planet_name: Some(next_planet_definition.label.to_string()),
                summary: format!(
                    "{} unlocks at {} survey progress.",
                    next_planet_definition.label,
                    round2(next_threshold.unwrap_or(0.0))
                ),
            }
        }
    }
}

fn build_prestige_reason_codes(run_state: &RunState) -> Vec<String> {
    let eligibility = evaluate_prestige_eligibility(run_state, run_state.consecutive_stable_power_ticks);
    let station_tier = calculate_station_tier(run_state);
    let mut reason_codes = Vec::new();

    if station_tier < 4 {
        reason_codes.push(prestige_reason_code(PrestigeIneligibleReason::StationTierBelowFour));
    }
    if run_state
        .station
        .discovered_planet_ids
        .iter()
        .filter(|planet_id| planet_id.as_str() != SOLSTICE_ANCHOR_ID)
        .count()
        < 2
    {
        reason_codes.push(prestige_reason_code(
            PrestigeIneligibleReason::NeedsTwoNonStarterPlanets,
        ));
    }
    if !eligibility.eligible {
        if let Some(PrestigeIneligibleReason::UnstableNetPower) = eligibility.reason {
            reason_codes.push(prestige_reason_code(PrestigeIneligibleReason::UnstableNetPower));
        }
    }

    reason_codes
}

fn make_resource_delta(id: &str, label: &str, delta_per_second: f32) -> ResourceDeltaSnapshot {
    ResourceDeltaSnapshot {
        id: id.to_string(),
        label: label.to_string(),
        delta_per_second: round2(delta_per_second),
        trend: if delta_per_second > 0.0 {
            "positive"
        } else if delta_per_second < 0.0 {
            "negative"
        } else {
            "neutral"
        }
        .to_string(),
    }
}

fn merged_ids(left: &[String], right: &[String]) -> Vec<String> {
    left.iter().chain(right.iter()).cloned().collect()
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn stable_power_seconds(stable_power_ticks: u32) -> f32 {
    round2(stable_power_ticks as f32 * SECONDS_PER_TICK)
}

fn survey_threshold(planet_id: &str) -> Option<f32> {
    match planet_id {
        SOLSTICE_ANCHOR_ID => None,
        CINDER_FORGE_ID => Some(CINDER_FORGE_SURVEY_THRESHOLD),
        AURORA_PIER_ID => Some(AURORA_PIER_SURVEY_THRESHOLD),
        _ => None,
    }
}

fn system_label(system_id: &str) -> &'static str {
    system_by_id(system_id)
        .expect("system must exist in catalog")
        .label
}

fn system_description(system_id: &str) -> &'static str {
    match system_id {
        REACTOR_CORE_ID => "Defines baseline power throughput and service power cap.",
        HABITAT_RING_ID => "Defines crew capacity and recovery ceiling.",
        LOGISTICS_SPINE_ID => "Defines active service slots and materials stockpile cap.",
        SURVEY_ARRAY_ID => "Defines data and survey multipliers for discovery progress.",
        _ => "",
    }
}

fn service_description(service_id: &str) -> &'static str {
    match service_id {
        "solar-harvester" => "Primary renewable power source for early station operations.",
        "ore-reclaimer" => "Consumes station capacity to turn scrap flow into materials.",
        "survey-uplink" => "Builds survey progress and trickles research data.",
        "maintenance-bay" => "Reduces global service power upkeep pressure.",
        "command-relay" => "Stabilizes priority handling and increases survey speed.",
        "fabrication-loop" => "Converts materials into research data.",
        _ => "",
    }
}

fn service_family(category: ServiceCategory) -> &'static str {
    match category {
        ServiceCategory::Production => "production",
        ServiceCategory::Support => "support",
        ServiceCategory::Conversion => "conversion",
    }
}

fn service_pause_reason_code(reason: ServicePauseReason) -> String {
    match reason {
        ServicePauseReason::Capacity => "capacity",
        ServicePauseReason::Crew => "crew",
        ServicePauseReason::Deficit => "deficit",
        ServicePauseReason::PowerCap => "power-cap",
    }
    .to_string()
}

fn prestige_reason_code(reason: PrestigeIneligibleReason) -> String {
    match reason {
        PrestigeIneligibleReason::StationTierBelowFour => "station-tier-below-four",
        PrestigeIneligibleReason::NeedsTwoNonStarterPlanets => "needs-two-non-starter-planets",
        PrestigeIneligibleReason::UnstableNetPower => "unstable-net-power",
    }
    .to_string()
}

fn planet_modifier_target_code(target: PlanetModifierTarget) -> &'static str {
    match target {
        PlanetModifierTarget::CrewEfficiency => "crew-efficiency",
        PlanetModifierTarget::DataOutput => "data-output",
        PlanetModifierTarget::MaterialsOutput => "materials-output",
        PlanetModifierTarget::ServicePowerUpkeep => "service-power-upkeep",
        PlanetModifierTarget::CrewCapacity => "crew-capacity",
    }
}

fn planet_modifier_label(target: PlanetModifierTarget) -> &'static str {
    match target {
        PlanetModifierTarget::CrewEfficiency => "Crew efficiency",
        PlanetModifierTarget::DataOutput => "Data output",
        PlanetModifierTarget::MaterialsOutput => "Materials output",
        PlanetModifierTarget::ServicePowerUpkeep => "Service power upkeep",
        PlanetModifierTarget::CrewCapacity => "Crew capacity",
    }
}

fn title_case(value: &str) -> String {
    value
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn round2(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}

fn active_service_power_modifier(run_state: &RunState) -> f32 {
    run_state
        .services
        .iter()
        .filter(|service| service.is_active)
        .map(|service| service.definition().global_service_power_modifier)
        .sum()
}

fn effective_service_power_upkeep(run_state: &RunState, service_id: &str) -> f32 {
    let definition = service_by_id(service_id).expect("service must exist in catalog");
    let modifier = planet_modifier_total(run_state, PlanetModifierTarget::ServicePowerUpkeep)
        + active_service_power_modifier(run_state);

    (definition.power_upkeep * (1.0 + modifier)).max(0.0)
}

fn effective_materials_output_multiplier(run_state: &RunState) -> f32 {
    1.0 + planet_modifier_total(run_state, PlanetModifierTarget::MaterialsOutput)
}

fn effective_data_output_multiplier(run_state: &RunState) -> f32 {
    survey_array_level(run_state).0 * (1.0 + planet_modifier_total(run_state, PlanetModifierTarget::DataOutput))
}

fn effective_survey_output_multiplier(run_state: &RunState, service_id: &str) -> f32 {
    let doctrine_multiplier = run_state
        .station
        .doctrine_ids
        .iter()
        .filter_map(|doctrine_id| doctrine_by_id(doctrine_id))
        .filter_map(|doctrine| match doctrine.effect {
            crate::game::content::doctrines::DoctrineEffect::SurveyProgressMultiplier {
                source_service_id,
                multiplier,
            } if source_service_id == service_id => Some(multiplier),
            _ => None,
        })
        .fold(1.0, |acc, multiplier| acc * multiplier);
    let service_multiplier = 1.0
        + run_state
            .services
            .iter()
            .filter(|service| service.is_active)
            .map(|service| service.definition().survey_speed_modifier)
            .sum::<f32>();

    survey_array_level(run_state).1 * service_multiplier * doctrine_multiplier
}

fn planet_modifier_total(run_state: &RunState, target: PlanetModifierTarget) -> f32 {
    run_state
        .active_planet_definition()
        .modifiers
        .iter()
        .filter(|modifier| modifier.target == target)
        .map(|modifier| modifier.percent)
        .sum()
}

fn effective_crew_capacity(run_state: &RunState, base_capacity: u8) -> u8 {
    ((base_capacity as f32) * (1.0 + planet_modifier_total(run_state, PlanetModifierTarget::CrewCapacity)))
        .floor()
        .max(1.0) as u8
}

fn logistics_active_service_slots(run_state: &RunState) -> u8 {
    match system_by_id(LOGISTICS_SPINE_ID)
        .expect("logistics-spine system must exist")
        .progression
    {
        SystemProgression::LogisticsSpine(levels) => {
            let level = run_state.system_level(LOGISTICS_SPINE_ID).unwrap_or(1).clamp(1, levels.len() as u8);
            levels[(level - 1) as usize].active_service_slots
        }
        _ => unreachable!("logistics-spine progression must use logistics levels"),
    }
}

fn survey_array_level(run_state: &RunState) -> (f32, f32) {
    match system_by_id(SURVEY_ARRAY_ID)
        .expect("survey-array system must exist")
        .progression
    {
        SystemProgression::SurveyArray(levels) => {
            let level = run_state.system_level(SURVEY_ARRAY_ID).unwrap_or(1).clamp(1, levels.len() as u8);
            let current = levels[(level - 1) as usize];
            (current.data_multiplier, current.survey_multiplier)
        }
        _ => unreachable!("survey-array progression must use survey levels"),
    }
}
