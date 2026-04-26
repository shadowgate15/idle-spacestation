#![allow(dead_code)]

use crate::game::content::planets::SOLSTICE_ANCHOR_ID;
use crate::game::content::services::SERVICES;
use crate::game::content::systems::{
    system_by_id, SystemProgression, HABITAT_RING_ID, LOGISTICS_SPINE_ID, REACTOR_CORE_ID,
    SURVEY_ARRAY_ID,
};
use crate::game::persistence::{SaveManager, SaveManagerError, SaveSettings};
use crate::game::sim::state::{
    RunState, ServiceState, StationState, SystemState, HOUSEKEEPING_POWER_PER_SECOND,
};

pub const POWER_STABILITY_TICKS_REQUIRED: u32 = 1_200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrestigeProfile {
    pub discovered_planet_ids: Vec<String>,
    pub doctrine_ids: Vec<String>,
    pub doctrine_fragments: u32,
    pub lifetime_ticks: u64,
    pub lifetime_prestiges: u32,
    pub lifetime_data_produced: u64,
    pub fastest_prestige_ticks: Option<u64>,
    pub settings: SaveSettings,
}

impl Default for PrestigeProfile {
    fn default() -> Self {
        Self {
            discovered_planet_ids: vec![SOLSTICE_ANCHOR_ID.to_string()],
            doctrine_ids: Vec::new(),
            doctrine_fragments: 0,
            lifetime_ticks: 0,
            lifetime_prestiges: 0,
            lifetime_data_produced: 0,
            fastest_prestige_ticks: None,
            settings: SaveSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrestigeIneligibleReason {
    StationTierBelowFour,
    NeedsTwoNonStarterPlanets,
    UnstableNetPower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrestigeEligibility {
    pub eligible: bool,
    pub reason: Option<PrestigeIneligibleReason>,
}

#[derive(Debug)]
pub enum PrestigeExecutionError {
    Ineligible(PrestigeIneligibleReason),
    Save(SaveManagerError),
}

impl From<SaveManagerError> for PrestigeExecutionError {
    fn from(value: SaveManagerError) -> Self {
        Self::Save(value)
    }
}

pub fn update_stable_power_ticks(current_ticks: u32, net_power: f32) -> u32 {
    if net_power >= 0.0 {
        current_ticks.saturating_add(1)
    } else {
        0
    }
}

pub fn calculate_station_tier(run_state: &RunState) -> u8 {
    let total_levels = run_state
        .systems
        .iter()
        .map(|system| system.level as i16)
        .sum::<i16>();
    (total_levels - 3).clamp(1, 4) as u8
}

pub fn evaluate_prestige_eligibility(
    run_state: &RunState,
    consecutive_stable_power_ticks: u32,
) -> PrestigeEligibility {
    if calculate_station_tier(run_state) < 4 {
        return PrestigeEligibility {
            eligible: false,
            reason: Some(PrestigeIneligibleReason::StationTierBelowFour),
        };
    }

    let non_starter_planet_count = run_state
        .station
        .discovered_planet_ids
        .iter()
        .filter(|planet_id| planet_id.as_str() != SOLSTICE_ANCHOR_ID)
        .count();
    if non_starter_planet_count < 2 {
        return PrestigeEligibility {
            eligible: false,
            reason: Some(PrestigeIneligibleReason::NeedsTwoNonStarterPlanets),
        };
    }

    if consecutive_stable_power_ticks < POWER_STABILITY_TICKS_REQUIRED {
        return PrestigeEligibility {
            eligible: false,
            reason: Some(PrestigeIneligibleReason::UnstableNetPower),
        };
    }

    PrestigeEligibility {
        eligible: true,
        reason: None,
    }
}

pub fn doctrine_fragment_reward(discovered_planet_count: usize, lifetime_data_produced: u64) -> u32 {
    let discovered_component = discovered_planet_count.saturating_sub(1) as u32;
    let data_component = (lifetime_data_produced / 1_500) as u32;
    (1 + discovered_component + data_component).min(6)
}

pub fn execute_prestige(
    run_state: &RunState,
    profile: &PrestigeProfile,
    consecutive_stable_power_ticks: u32,
) -> Result<(RunState, PrestigeProfile, u32), PrestigeExecutionError> {
    let eligibility = evaluate_prestige_eligibility(run_state, consecutive_stable_power_ticks);
    if let Some(reason) = eligibility.reason {
        return Err(PrestigeExecutionError::Ineligible(reason));
    }

    let mut next_profile = profile.clone();
    next_profile.discovered_planet_ids = sorted_unique(run_state.station.discovered_planet_ids.clone());
    next_profile.doctrine_ids = sorted_unique(run_state.station.doctrine_ids.clone());
    next_profile.lifetime_ticks = next_profile.lifetime_ticks.saturating_add(run_state.tick_count);
    next_profile.lifetime_prestiges = next_profile.lifetime_prestiges.saturating_add(1);
    next_profile.lifetime_data_produced = next_profile
        .lifetime_data_produced
        .saturating_add(run_state.resources.data.max(0.0).floor() as u64);
    next_profile.fastest_prestige_ticks = match next_profile.fastest_prestige_ticks {
        Some(existing) => Some(existing.min(run_state.tick_count)),
        None => Some(run_state.tick_count),
    };

    let fragments_awarded = doctrine_fragment_reward(
        next_profile.discovered_planet_ids.len(),
        next_profile.lifetime_data_produced,
    );
    next_profile.doctrine_fragments = run_state
        .station
        .doctrine_fragments
        .saturating_add(fragments_awarded);

    let reset_run_state = reset_run_state(&next_profile);
    Ok((reset_run_state, next_profile, fragments_awarded))
}

pub fn execute_prestige_with_persistence(
    save_manager: &SaveManager,
    run_state: &RunState,
    profile: &PrestigeProfile,
    consecutive_stable_power_ticks: u32,
) -> Result<(RunState, PrestigeProfile, u32), PrestigeExecutionError> {
    save_manager.save_before_prestige(run_state, &crate::game::persistence::ProfileState {
        discovered_planet_ids: profile.discovered_planet_ids.clone(),
        doctrine_ids: profile.doctrine_ids.clone(),
        doctrine_fragments: profile.doctrine_fragments,
        lifetime_ticks: profile.lifetime_ticks,
        lifetime_prestiges: profile.lifetime_prestiges,
    }, &profile.settings)?;

    execute_prestige(run_state, profile, consecutive_stable_power_ticks)
}

fn reset_run_state(profile: &PrestigeProfile) -> RunState {
    let reactor_level = level_one_reactor_power_output();
    let crew_total = level_one_habitat_crew_capacity();

    RunState {
        tick_count: 0,
        station: StationState {
            active_planet_id: SOLSTICE_ANCHOR_ID.to_string(),
            discovered_planet_ids: profile.discovered_planet_ids.clone(),
            doctrine_ids: profile.doctrine_ids.clone(),
            doctrine_fragments: profile.doctrine_fragments,
            survey_progress: 0.0,
            station_tier: 1,
        },
        resources: crate::game::sim::ResourceState {
            power_generated: reactor_level,
            power_reserved: HOUSEKEEPING_POWER_PER_SECOND,
            power_available: reactor_level - HOUSEKEEPING_POWER_PER_SECOND,
            materials: 0.0,
            data: 0.0,
            crew_total,
            crew_assigned: 0,
            crew_available: crew_total,
        },
        services: SERVICES
            .iter()
            .enumerate()
            .map(|(index, service)| ServiceState {
                service_id: service.id.to_string(),
                desired_active: false,
                is_active: false,
                is_paused: false,
                pause_reason: None,
                priority: (index + 1) as u8,
                assigned_crew: 0,
            })
            .collect(),
        systems: vec![
            SystemState::new(REACTOR_CORE_ID, 1),
            SystemState::new(HABITAT_RING_ID, 1),
            SystemState::new(LOGISTICS_SPINE_ID, 1),
            SystemState::new(SURVEY_ARRAY_ID, 1),
        ],
        autosave_due: false,
        autosave_count: 0,
        last_autosave_tick: None,
        prestige_eligible: false,
    }
}

fn level_one_reactor_power_output() -> f32 {
    match system_by_id(REACTOR_CORE_ID)
        .expect("reactor-core system must exist")
        .progression
    {
        SystemProgression::ReactorCore(levels) => levels[0].power_output,
        _ => unreachable!("reactor-core progression must use reactor levels"),
    }
}

fn level_one_habitat_crew_capacity() -> u8 {
    match system_by_id(HABITAT_RING_ID)
        .expect("habitat-ring system must exist")
        .progression
    {
        SystemProgression::HabitatRing(levels) => levels[0].crew_capacity,
        _ => unreachable!("habitat-ring progression must use habitat levels"),
    }
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::content::doctrines::EFFICIENT_SHIFTS_ID;
    use crate::game::content::planets::{AURORA_PIER_ID, CINDER_FORGE_ID};

    fn eligible_fixture() -> RunState {
        let mut run_state = RunState::starter_fixture();
        run_state.tick_count = 1_440;
        run_state.resources.data = 3_200.0;
        run_state.resources.materials = 275.0;
        run_state.resources.crew_assigned = 5;
        run_state.resources.crew_available = 1;
        run_state.resources.power_generated = 20.0;
        run_state.resources.power_reserved = 18.0;
        run_state.resources.power_available = 2.0;
        run_state.station.discovered_planet_ids = vec![
            AURORA_PIER_ID.to_string(),
            CINDER_FORGE_ID.to_string(),
            SOLSTICE_ANCHOR_ID.to_string(),
        ];
        run_state.station.doctrine_ids = vec![EFFICIENT_SHIFTS_ID.to_string()];
        run_state.station.doctrine_fragments = 2;
        run_state.station.survey_progress = 1_550.0;
        run_state.services[0].desired_active = true;
        run_state.services[0].is_active = true;
        run_state.services[0].assigned_crew = 2;
        run_state.services[1].desired_active = true;
        run_state.services[1].is_active = true;
        run_state.services[1].assigned_crew = 1;
        run_state.services[2].desired_active = true;
        run_state.services[2].is_active = true;
        run_state.services[2].assigned_crew = 1;
        run_state.services[3].desired_active = true;
        run_state.services[3].is_active = true;
        run_state.services[3].assigned_crew = 1;
        run_state.systems = vec![
            SystemState::new(REACTOR_CORE_ID, 2),
            SystemState::new(HABITAT_RING_ID, 2),
            SystemState::new(LOGISTICS_SPINE_ID, 2),
            SystemState::new(SURVEY_ARRAY_ID, 1),
        ];
        run_state.station.station_tier = calculate_station_tier(&run_state);
        run_state.prestige_eligible = true;
        run_state
    }

    #[test]
    fn progression_prestige_gate_uses_exact_contract() {
        let eligible = eligible_fixture();
        let mut below_tier = eligible_fixture();
        below_tier.systems = vec![
            SystemState::new(REACTOR_CORE_ID, 1),
            SystemState::new(HABITAT_RING_ID, 2),
            SystemState::new(LOGISTICS_SPINE_ID, 2),
            SystemState::new(SURVEY_ARRAY_ID, 1),
        ];

        let mut only_starter = eligible_fixture();
        only_starter.station.discovered_planet_ids = vec![SOLSTICE_ANCHOR_ID.to_string()];

        let unstable_power = eligible_fixture();

        let eligible_result = evaluate_prestige_eligibility(&eligible, POWER_STABILITY_TICKS_REQUIRED);
        let tier_result = evaluate_prestige_eligibility(&below_tier, POWER_STABILITY_TICKS_REQUIRED);
        let planet_result = evaluate_prestige_eligibility(&only_starter, POWER_STABILITY_TICKS_REQUIRED);
        let power_result = evaluate_prestige_eligibility(&unstable_power, POWER_STABILITY_TICKS_REQUIRED - 1);

        println!(
            "prestige-gate\nfixture=eligible eligible={} reason={:?}\nfixture=below-tier eligible={} reason={:?}\nfixture=starter-only eligible={} reason={:?}\nfixture=unstable-power eligible={} reason={:?}",
            eligible_result.eligible,
            eligible_result.reason,
            tier_result.eligible,
            tier_result.reason,
            planet_result.eligible,
            planet_result.reason,
            power_result.eligible,
            power_result.reason,
        );

        assert_eq!(eligible_result, PrestigeEligibility { eligible: true, reason: None });
        assert_eq!(tier_result.reason, Some(PrestigeIneligibleReason::StationTierBelowFour));
        assert_eq!(planet_result.reason, Some(PrestigeIneligibleReason::NeedsTwoNonStarterPlanets));
        assert_eq!(power_result.reason, Some(PrestigeIneligibleReason::UnstableNetPower));
    }

    #[test]
    fn progression_prestige_reward_formula_matches_plan_and_caps_at_six() {
        assert_eq!(doctrine_fragment_reward(1, 0), 1);
        assert_eq!(doctrine_fragment_reward(3, 1_499), 3);
        assert_eq!(doctrine_fragment_reward(3, 3_000), 5);
        assert_eq!(doctrine_fragment_reward(3, 15_000), 6);
    }

    #[test]
    fn progression_prestige_reset_persists_only_profile_fields() {
        let run_state = eligible_fixture();
        let profile = PrestigeProfile {
            discovered_planet_ids: vec![SOLSTICE_ANCHOR_ID.to_string()],
            doctrine_ids: Vec::new(),
            doctrine_fragments: 0,
            lifetime_ticks: 900,
            lifetime_prestiges: 2,
            lifetime_data_produced: 1_600,
            fastest_prestige_ticks: Some(2_000),
            settings: SaveSettings {
                autosave_enabled: false,
            },
        };

        let (reset_run_state, next_profile, fragments_awarded) =
            execute_prestige(&run_state, &profile, POWER_STABILITY_TICKS_REQUIRED)
                .expect("eligible fixture should prestige");

        println!(
            "prestige-reset-table\nfield\tbefore\tafter\tstatus\ndiscovered_planets\t{:?}\t{:?}\tpersisted\ndoctrines\t{:?}\t{:?}\tpersisted\ndoctrine_fragments\t{}\t{}\tpersisted+reward\nlifetime_ticks\t{}\t{}\tpersisted\nlifetime_prestiges\t{}\t{}\tpersisted\nlifetime_data_produced\t{}\t{}\tpersisted\nfastest_prestige_ticks\t{:?}\t{:?}\tpersisted\nsettings.autosave_enabled\t{}\t{}\tpersisted\nrun.tick_count\t{}\t{}\treset\nrun.materials\t{}\t{}\treset\nrun.data\t{}\t{}\treset\nrun.survey_progress\t{}\t{}\treset\nrun.active_services\t{}\t{}\treset\nrun.crew_assigned\t{}\t{}\treset\nrun.autosave_count\t{}\t{}\treset",
            run_state.station.discovered_planet_ids,
            next_profile.discovered_planet_ids,
            run_state.station.doctrine_ids,
            next_profile.doctrine_ids,
            run_state.station.doctrine_fragments,
            next_profile.doctrine_fragments,
            profile.lifetime_ticks,
            next_profile.lifetime_ticks,
            profile.lifetime_prestiges,
            next_profile.lifetime_prestiges,
            profile.lifetime_data_produced,
            next_profile.lifetime_data_produced,
            profile.fastest_prestige_ticks,
            next_profile.fastest_prestige_ticks,
            profile.settings.autosave_enabled,
            next_profile.settings.autosave_enabled,
            run_state.tick_count,
            reset_run_state.tick_count,
            run_state.resources.materials,
            reset_run_state.resources.materials,
            run_state.resources.data,
            reset_run_state.resources.data,
            run_state.station.survey_progress,
            reset_run_state.station.survey_progress,
            run_state.services.iter().filter(|service| service.is_active).count(),
            reset_run_state.services.iter().filter(|service| service.is_active).count(),
            run_state.resources.crew_assigned,
            reset_run_state.resources.crew_assigned,
            run_state.autosave_count,
            reset_run_state.autosave_count,
        );

        assert_eq!(fragments_awarded, 6);
        assert_eq!(
            next_profile.discovered_planet_ids,
            vec![
                AURORA_PIER_ID.to_string(),
                CINDER_FORGE_ID.to_string(),
                SOLSTICE_ANCHOR_ID.to_string(),
            ]
        );
        assert_eq!(next_profile.doctrine_ids, vec![EFFICIENT_SHIFTS_ID.to_string()]);
        assert_eq!(next_profile.doctrine_fragments, 8);
        assert_eq!(next_profile.lifetime_ticks, 2_340);
        assert_eq!(next_profile.lifetime_prestiges, 3);
        assert_eq!(next_profile.lifetime_data_produced, 4_800);
        assert_eq!(next_profile.fastest_prestige_ticks, Some(1_440));
        assert!(!next_profile.settings.autosave_enabled);

        assert_eq!(reset_run_state.tick_count, 0);
        assert_eq!(reset_run_state.station.active_planet_id, SOLSTICE_ANCHOR_ID);
        assert_eq!(reset_run_state.station.survey_progress, 0.0);
        assert_eq!(reset_run_state.station.station_tier, 1);
        assert_eq!(reset_run_state.resources.materials, 0.0);
        assert_eq!(reset_run_state.resources.data, 0.0);
        assert_eq!(reset_run_state.resources.crew_assigned, 0);
        assert_eq!(reset_run_state.resources.crew_available, reset_run_state.resources.crew_total);
        assert_eq!(reset_run_state.autosave_count, 0);
        assert_eq!(reset_run_state.last_autosave_tick, None);
        assert!(reset_run_state.services.iter().all(|service| {
            !service.desired_active && !service.is_active && !service.is_paused && service.assigned_crew == 0
        }));
        assert!(reset_run_state.systems.iter().all(|system| system.level == 1));
    }
}
