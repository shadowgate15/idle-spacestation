use serde::Serialize;

use crate::game::content::doctrines::{doctrine_by_id, DOCTRINES};
use crate::game::content::planets::{
    planet_by_id_required, survey_threshold, PlanetDefinition, AURORA_PIER_ID, CINDER_FORGE_ID, PLANETS,
    SOLSTICE_ANCHOR_ID,
};
use crate::game::content::services::SURVEY_UPLINK_ID;
use crate::game::content::systems::{
    system_by_id_required, SystemProgression, HABITAT_RING_ID, LOGISTICS_SPINE_ID, REACTOR_CORE_ID,
    SURVEY_ARRAY_ID, SYSTEMS,
};
use crate::game::progression::{
    calculate_station_tier, evaluate_prestige_eligibility, PrestigeIneligibleReason, PrestigeProfile,
    POWER_STABILITY_TICKS_REQUIRED,
};
use crate::game::sim::{
    effective_crew_capacity, effective_data_output_multiplier,
    effective_materials_output_multiplier, effective_service_power_upkeep,
    effective_survey_output_multiplier, RunState, ServicePauseReason,
};
use crate::game::sim::state::{
    HOUSEKEEPING_POWER_PER_SECOND,
    SECONDS_PER_TICK,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResponse {
    pub ok: bool,
    pub snapshot: RawGameSnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLoadResponse {
    pub ok: bool,
    pub status: String,
    pub snapshot: RawGameSnapshot,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawGameSnapshot {
    pub meta: SnapshotMeta,
    pub run: RunSnapshot,
    pub resources: ResourcesSnapshot,
    pub systems: Vec<RawSystemStateSnapshot>,
    pub services: Vec<RawServiceStateSnapshot>,
    pub route_snapshots: RouteSnapshots,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMeta {
    pub source: String,
    pub fixture_name: Option<String>,
    pub tick_count: u64,
}

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesSnapshot {
    pub power: PowerSnapshot,
    pub materials: f32,
    pub data: f32,
    pub crew: CrewSnapshot,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerSnapshot {
    pub generated: f32,
    pub reserved: f32,
    pub available: f32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrewSnapshot {
    pub total: u8,
    pub assigned: u8,
    pub available: u8,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSystemStateSnapshot {
    pub id: String,
    pub level: u8,
}

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteSnapshots {
    pub overview: OverviewRouteSnapshot,
    pub systems: SystemsRouteSnapshot,
    pub services: ServicesRouteSnapshot,
    pub planets: PlanetsRouteSnapshot,
    pub prestige: PrestigeRouteSnapshot,
}

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivePlanetSnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub modifiers: Vec<PlanetModifierSnapshot>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanetModifierSnapshot {
    pub target: String,
    pub label: String,
    pub percent: f32,
    pub effect_text: String,
}

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemsRouteSnapshot {
    pub systems: Vec<SystemRouteEntrySnapshot>,
}

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemCapSnapshot {
    pub key: String,
    pub label: String,
    pub value: f32,
    pub unit: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicesRouteSnapshot {
    pub services: Vec<ServiceRouteEntrySnapshot>,
    pub utilization: ServiceUtilizationSnapshot,
    pub deficit_warnings: Vec<WarningSnapshot>,
}

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceCrewAssignmentSnapshot {
    pub current: u8,
    pub required: u8,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicePowerUsageSnapshot {
    pub upkeep: f32,
    pub output: f32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanetsRouteSnapshot {
    pub active_planet_id: String,
    pub planets: Vec<PlanetRouteEntrySnapshot>,
    pub survey_progress: SurveyProgressSnapshot,
}

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrestigeRouteSnapshot {
    pub eligibility: PrestigeEligibilitySnapshot,
    pub doctrine_fragments: u32,
    pub unlocked_doctrines: Vec<DoctrineSnapshot>,
    pub purchase_options: Vec<DoctrinePurchaseOptionSnapshot>,
    pub reset_consequences: Vec<ResetConsequenceSnapshot>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrestigeEligibilitySnapshot {
    pub eligible: bool,
    pub reason_codes: Vec<String>,
    pub summary: String,
    pub stable_power_seconds: f32,
    pub required_stable_power_seconds: f32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctrineSnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctrinePurchaseOptionSnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost_fragments: u32,
    pub available: bool,
    pub blocked_reason: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetConsequenceSnapshot {
    pub label: String,
    pub outcome: String,
    pub summary: String,
}

pub fn build_snapshot(run_state: &RunState, profile: &PrestigeProfile) -> RawGameSnapshot {
    let station_tier = calculate_station_tier(run_state);
    let stable_power_seconds = stable_power_seconds(run_state.consecutive_stable_power_ticks);
    let doctrine_ids = sorted_unique(
        run_state.station.doctrine_ids.iter().cloned()
            .chain(profile.doctrine_ids.iter().cloned())
    );
    let discovered_planet_ids = sorted_unique(
        run_state.station.discovered_planet_ids.iter().cloned()
            .chain(profile.discovered_planet_ids.iter().cloned())
    );
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
                pause_reason: service.pause_reason.map(|r| r.code().to_string()),
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
                    family: definition.category.family().to_string(),
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
                        .map(|r| r.code().to_string())
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
    let system = system_by_id_required(system_id);
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
    let label = modifier.target.label();

    PlanetModifierSnapshot {
        target: modifier.target.code().to_string(),
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
                planet_by_id_required(next_planet_id);
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
        reason_codes.push(PrestigeIneligibleReason::StationTierBelowFour.code().to_string());
    }
    if run_state
        .station
        .discovered_planet_ids
        .iter()
        .filter(|planet_id| planet_id.as_str() != SOLSTICE_ANCHOR_ID)
        .count()
        < 2
    {
        reason_codes.push(PrestigeIneligibleReason::NeedsTwoNonStarterPlanets.code().to_string());
    }
    if !eligibility.eligible {
        if let Some(PrestigeIneligibleReason::UnstableNetPower) = eligibility.reason {
            reason_codes.push(PrestigeIneligibleReason::UnstableNetPower.code().to_string());
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

fn sorted_unique<I: IntoIterator<Item = String>>(iter: I) -> Vec<String> {
    let mut v: Vec<String> = iter.into_iter().collect();
    v.sort();
    v.dedup();
    v
}

fn stable_power_seconds(stable_power_ticks: u32) -> f32 {
    round2(stable_power_ticks as f32 * SECONDS_PER_TICK)
}

fn system_label(system_id: &str) -> &'static str {
    system_by_id_required(system_id).label
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

fn logistics_active_service_slots(run_state: &RunState) -> u8 {
    match system_by_id_required(LOGISTICS_SPINE_ID)
        .progression
    {
        SystemProgression::LogisticsSpine(levels) => {
            let level = run_state.system_level(LOGISTICS_SPINE_ID).unwrap_or(1).clamp(1, levels.len() as u8);
            levels[(level - 1) as usize].active_service_slots
        }
        _ => unreachable!("logistics-spine progression must use logistics levels"),
    }
}

/// Compare two snapshots with bitwise-precise f32 equality.
/// Uses `f32::to_bits()` for float comparisons (handles NaN consistently by bit pattern).
/// Uses standard `==` for integers, strings, booleans, and enums.
pub fn state_equals(a: &RawGameSnapshot, b: &RawGameSnapshot) -> bool {
    meta_eq(&a.meta, &b.meta)
        && run_eq(&a.run, &b.run)
        && resources_eq(&a.resources, &b.resources)
        && systems_eq(&a.systems, &b.systems)
        && services_eq(&a.services, &b.services)
        && route_snapshots_eq(&a.route_snapshots, &b.route_snapshots)
}

fn meta_eq(a: &SnapshotMeta, b: &SnapshotMeta) -> bool {
    a.source == b.source && a.fixture_name == b.fixture_name && a.tick_count == b.tick_count
}

fn run_eq(a: &RunSnapshot, b: &RunSnapshot) -> bool {
    a.active_planet_id == b.active_planet_id
        && a.discovered_planet_ids == b.discovered_planet_ids
        && a.doctrine_ids == b.doctrine_ids
        && a.doctrine_fragments == b.doctrine_fragments
        && a.survey_progress.to_bits() == b.survey_progress.to_bits()
        && a.station_tier == b.station_tier
        && a.stable_power_seconds.to_bits() == b.stable_power_seconds.to_bits()
}

fn resources_eq(a: &ResourcesSnapshot, b: &ResourcesSnapshot) -> bool {
    power_eq(&a.power, &b.power)
        && a.materials.to_bits() == b.materials.to_bits()
        && a.data.to_bits() == b.data.to_bits()
        && crew_eq(&a.crew, &b.crew)
}

fn power_eq(a: &PowerSnapshot, b: &PowerSnapshot) -> bool {
    a.generated.to_bits() == b.generated.to_bits()
        && a.reserved.to_bits() == b.reserved.to_bits()
        && a.available.to_bits() == b.available.to_bits()
}

fn crew_eq(a: &CrewSnapshot, b: &CrewSnapshot) -> bool {
    a.total == b.total && a.assigned == b.assigned && a.available == b.available
}

fn systems_eq(a: &[RawSystemStateSnapshot], b: &[RawSystemStateSnapshot]) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(av, bv)| av.id == bv.id && av.level == bv.level)
}

fn services_eq(a: &[RawServiceStateSnapshot], b: &[RawServiceStateSnapshot]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b.iter()).all(|(av, bv)| {
            av.id == bv.id
                && av.desired_active == bv.desired_active
                && av.is_active == bv.is_active
                && av.is_paused == bv.is_paused
                && av.pause_reason == bv.pause_reason
                && av.priority == bv.priority
                && av.assigned_crew == bv.assigned_crew
        })
}

fn route_snapshots_eq(a: &RouteSnapshots, b: &RouteSnapshots) -> bool {
    overview_eq(&a.overview, &b.overview)
        && systems_route_eq(&a.systems, &b.systems)
        && services_route_eq(&a.services, &b.services)
        && planets_route_eq(&a.planets, &b.planets)
        && prestige_route_eq(&a.prestige, &b.prestige)
}

fn overview_eq(a: &OverviewRouteSnapshot, b: &OverviewRouteSnapshot) -> bool {
    active_planet_eq(&a.active_planet, &b.active_planet)
        && resource_deltas_eq(&a.resource_deltas, &b.resource_deltas)
        && warnings_eq(&a.deficit_warnings, &b.deficit_warnings)
        && a.station_tier.current == b.station_tier.current
        && a.station_tier.max == b.station_tier.max
        && a.station_tier.label == b.station_tier.label
        && a.service_utilization.active == b.service_utilization.active
        && a.service_utilization.capacity == b.service_utilization.capacity
        && a.service_utilization.available == b.service_utilization.available
        && a.service_utilization.summary == b.service_utilization.summary
        && survey_progress_eq(&a.survey_progress, &b.survey_progress)
        && a.guidance_triggers == b.guidance_triggers
}

fn active_planet_eq(a: &ActivePlanetSnapshot, b: &ActivePlanetSnapshot) -> bool {
    a.id == b.id
        && a.name == b.name
        && a.description == b.description
        && a.modifiers.len() == b.modifiers.len()
        && a.modifiers
            .iter()
            .zip(b.modifiers.iter())
            .all(|(av, bv)| {
                av.target == bv.target
                    && av.label == bv.label
                    && av.percent.to_bits() == bv.percent.to_bits()
                    && av.effect_text == bv.effect_text
            })
}

fn resource_deltas_eq(a: &[ResourceDeltaSnapshot], b: &[ResourceDeltaSnapshot]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b.iter()).all(|(av, bv)| {
            av.id == bv.id
                && av.label == bv.label
                && av.delta_per_second.to_bits() == bv.delta_per_second.to_bits()
                && av.trend == bv.trend
        })
}

fn warnings_eq(a: &[WarningSnapshot], b: &[WarningSnapshot]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .all(|(av, bv)| av.code == bv.code && av.severity == bv.severity && av.title == bv.title && av.body == bv.body)
}

fn survey_progress_eq(a: &SurveyProgressSnapshot, b: &SurveyProgressSnapshot) -> bool {
    a.current.to_bits() == b.current.to_bits()
        && a.next_threshold.map(|x| x.to_bits()) == b.next_threshold.map(|x| x.to_bits())
        && a.next_planet_id == b.next_planet_id
        && a.next_planet_name == b.next_planet_name
        && a.summary == b.summary
}

fn systems_route_eq(a: &SystemsRouteSnapshot, b: &SystemsRouteSnapshot) -> bool {
    a.systems.len() == b.systems.len()
        && a.systems
            .iter()
            .zip(b.systems.iter())
            .all(|(av, bv)| {
                av.id == bv.id
                    && av.name == bv.name
                    && av.description == bv.description
                    && av.level == bv.level
                    && av.max_level == bv.max_level
                    && av.cap_values.len() == bv.cap_values.len()
                    && av.cap_values
                        .iter()
                        .zip(bv.cap_values.iter())
                        .all(|(acv, bcv)| {
                            acv.key == bcv.key
                                && acv.label == bcv.label
                                && acv.value.to_bits() == bcv.value.to_bits()
                                && acv.unit == bcv.unit
                        })
                    && av.upgrade_cost_materials == bv.upgrade_cost_materials
                    && av.can_upgrade == bv.can_upgrade
                    && av.upgrade_blocked_reason == bv.upgrade_blocked_reason
            })
}

fn services_route_eq(a: &ServicesRouteSnapshot, b: &ServicesRouteSnapshot) -> bool {
    a.services.len() == b.services.len()
        && a.services
            .iter()
            .zip(b.services.iter())
            .all(|(av, bv)| {
                av.id == bv.id
                    && av.name == bv.name
                    && av.description == bv.description
                    && av.family == bv.family
                    && av.priority_order == bv.priority_order
                    && av.status == bv.status
                    && av.status_label == bv.status_label
                    && av.desired_active == bv.desired_active
                    && av.crew_assignment.current == bv.crew_assignment.current
                    && av.crew_assignment.required == bv.crew_assignment.required
                    && av.power_usage.upkeep.to_bits() == bv.power_usage.upkeep.to_bits()
                    && av.power_usage.output.to_bits() == bv.power_usage.output.to_bits()
                    && av.disabled_reasons == bv.disabled_reasons
            })
        && a.utilization.active == b.utilization.active
        && a.utilization.capacity == b.utilization.capacity
        && a.utilization.available == b.utilization.available
        && a.utilization.summary == b.utilization.summary
        && warnings_eq(&a.deficit_warnings, &b.deficit_warnings)
}

fn planets_route_eq(a: &PlanetsRouteSnapshot, b: &PlanetsRouteSnapshot) -> bool {
    a.active_planet_id == b.active_planet_id
        && a.planets.len() == b.planets.len()
        && a.planets
            .iter()
            .zip(b.planets.iter())
            .all(|(av, bv)| {
                av.id == bv.id
                    && av.name == bv.name
                    && av.description == bv.description
                    && av.discovered == bv.discovered
                    && av.active == bv.active
                    && av.selectable_for_next_run == bv.selectable_for_next_run
                    && av.selectability_reason == bv.selectability_reason
                    && av.modifiers.len() == bv.modifiers.len()
                    && av.modifiers
                        .iter()
                        .zip(bv.modifiers.iter())
                        .all(|(am, bm)| {
                            am.target == bm.target
                                && am.label == bm.label
                                && am.percent.to_bits() == bm.percent.to_bits()
                                && am.effect_text == bm.effect_text
                        })
                    && av.survey_threshold.map(|x| x.to_bits()) == bv.survey_threshold.map(|x| x.to_bits())
                    && av.survey_progress.to_bits() == bv.survey_progress.to_bits()
            })
        && survey_progress_eq(&a.survey_progress, &b.survey_progress)
}

fn prestige_route_eq(a: &PrestigeRouteSnapshot, b: &PrestigeRouteSnapshot) -> bool {
    a.eligibility.eligible == b.eligibility.eligible
        && a.eligibility.reason_codes == b.eligibility.reason_codes
        && a.eligibility.summary == b.eligibility.summary
        && a.eligibility.stable_power_seconds.to_bits() == b.eligibility.stable_power_seconds.to_bits()
        && a.eligibility.required_stable_power_seconds.to_bits() == b.eligibility.required_stable_power_seconds.to_bits()
        && a.doctrine_fragments == b.doctrine_fragments
        && a.unlocked_doctrines.len() == b.unlocked_doctrines.len()
        && a.unlocked_doctrines
            .iter()
            .zip(b.unlocked_doctrines.iter())
            .all(|(av, bv)| av.id == bv.id && av.name == bv.name && av.description == bv.description)
        && a.purchase_options.len() == b.purchase_options.len()
        && a.purchase_options
            .iter()
            .zip(b.purchase_options.iter())
            .all(|(av, bv)| {
                av.id == bv.id
                    && av.name == bv.name
                    && av.description == bv.description
                    && av.cost_fragments == bv.cost_fragments
                    && av.available == bv.available
                    && av.blocked_reason == bv.blocked_reason
            })
        && a.reset_consequences.len() == b.reset_consequences.len()
        && a.reset_consequences
            .iter()
            .zip(b.reset_consequences.iter())
            .all(|(av, bv)| av.label == bv.label && av.outcome == bv.outcome && av.summary == bv.summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::progression::PrestigeProfile;
    use crate::game::sim::{tick, RunState};

    fn make_test_snapshot() -> RawGameSnapshot {
        let run_state = RunState::starter_fixture();
        let profile = PrestigeProfile::default();
        build_snapshot(&run_state, &profile)
    }

    #[test]
    fn state_equals_clones_are_equal() {
        let snapshot = make_test_snapshot();
        let clone = snapshot.clone();
        assert!(state_equals(&snapshot, &clone), "cloned snapshots should be equal");
    }

    #[test]
    fn state_equals_different_integer_field() {
        let mut snapshot1 = make_test_snapshot();
        let mut snapshot2 = make_test_snapshot();
        snapshot1.meta.tick_count = 100;
        snapshot2.meta.tick_count = 200;
        assert!(!state_equals(&snapshot1, &snapshot2), "snapshots with different tick_count should not be equal");
    }

    #[test]
    fn state_equals_different_f32_field() {
        let mut snapshot1 = make_test_snapshot();
        let mut snapshot2 = make_test_snapshot();
        snapshot1.resources.materials = 100.5;
        snapshot2.resources.materials = 200.5;
        assert!(!state_equals(&snapshot1, &snapshot2), "snapshots with different f32 fields should not be equal");
    }

    #[test]
    fn state_equals_nan_same_bit_pattern() {
        let mut snapshot1 = make_test_snapshot();
        let mut snapshot2 = make_test_snapshot();
        snapshot1.resources.materials = f32::NAN;
        snapshot2.resources.materials = f32::NAN;
        assert!(state_equals(&snapshot1, &snapshot2), "NaN values with same bit pattern should compare equal");
    }

    #[test]
    fn state_equals_nan_different_bit_patterns() {
        let mut snapshot1 = make_test_snapshot();
        let mut snapshot2 = make_test_snapshot();
        snapshot1.resources.materials = f32::NAN; // 0x7FC00000
        snapshot2.resources.materials = f32::from_bits(0x7fc00001); // different NaN pattern
        assert!(!state_equals(&snapshot1, &snapshot2), "NaN values with different bit patterns should not compare equal");
    }

    #[test]
    fn state_equals_empty_vs_nonempty_vec() {
        let mut snapshot1 = make_test_snapshot();
        let mut snapshot2 = make_test_snapshot();
        snapshot1.systems = vec![];
        snapshot2.systems = vec![RawSystemStateSnapshot {
            id: "test-system".to_string(),
            level: 1,
        }];
        assert!(!state_equals(&snapshot1, &snapshot2), "snapshots with different system vec lengths should not be equal");
    }

    #[test]
    fn state_equals_same_len_different_element() {
        let mut snapshot1 = make_test_snapshot();
        let mut snapshot2 = make_test_snapshot();
        if !snapshot1.systems.is_empty() {
            snapshot1.systems[0].level = 1;
            snapshot2.systems[0].level = 2;
            assert!(!state_equals(&snapshot1, &snapshot2), "snapshots with different system levels should not be equal");
        }
    }

    #[test]
    fn replay_preserves_state_equals_bit_for_bit() {
        let mut run = RunState::starter_fixture();
        let profile = PrestigeProfile::default();
        let baseline_snapshots: Vec<_> = (0..100)
            .map(|_| {
                tick(&mut run);
                build_snapshot(&run, &profile)
            })
            .collect();

        // After code changes, this same sequence MUST produce state_equals snapshots.
        // For the test itself, verify build_snapshot is deterministic:
        let mut run2 = RunState::starter_fixture();
        for (i, baseline) in baseline_snapshots.iter().enumerate() {
            tick(&mut run2);
            let new_snapshot = build_snapshot(&run2, &profile);
            assert!(
                state_equals(&new_snapshot, baseline),
                "snapshot diverges at tick {i}"
            );
        }
    }
}
