#![allow(dead_code)]

use crate::game::content::planets::{AURORA_PIER_ID, CINDER_FORGE_ID};
use crate::game::content::services::SURVEY_UPLINK_ID;
use crate::game::content::systems::{system_by_id, SystemProgression, SURVEY_ARRAY_ID};
use crate::game::progression::doctrines::survey_progress_doctrine_multiplier;
use crate::game::sim::state::{
    AURORA_PIER_SURVEY_THRESHOLD, CINDER_FORGE_SURVEY_THRESHOLD, RunState,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SurveyOutcome {
    pub progress_gained: f32,
    pub discovered_planet_ids: Vec<String>,
}

pub fn accumulate_survey_progress(run_state: &mut RunState, elapsed_seconds: f32) -> SurveyOutcome {
    if elapsed_seconds <= 0.0 {
        return SurveyOutcome {
            progress_gained: 0.0,
            discovered_planet_ids: Vec::new(),
        };
    }

    let survey_uplink_active = run_state
        .service_state(SURVEY_UPLINK_ID)
        .map(|service| service.is_active || service.desired_active)
        .unwrap_or(false);

    if !survey_uplink_active {
        return SurveyOutcome {
            progress_gained: 0.0,
            discovered_planet_ids: Vec::new(),
        };
    }

    let progress_gained = elapsed_seconds
        * survey_array_multiplier(run_state)
        * survey_progress_doctrine_multiplier(&run_state.station.doctrine_ids, SURVEY_UPLINK_ID);

    run_state.station.survey_progress += progress_gained;

    let mut discovered_planet_ids = Vec::new();
    unlock_planet_if_ready(
        &mut run_state.station.discovered_planet_ids,
        run_state.station.survey_progress,
        CINDER_FORGE_ID,
        CINDER_FORGE_SURVEY_THRESHOLD,
        &mut discovered_planet_ids,
    );
    unlock_planet_if_ready(
        &mut run_state.station.discovered_planet_ids,
        run_state.station.survey_progress,
        AURORA_PIER_ID,
        AURORA_PIER_SURVEY_THRESHOLD,
        &mut discovered_planet_ids,
    );

    SurveyOutcome {
        progress_gained,
        discovered_planet_ids,
    }
}

fn survey_array_multiplier(run_state: &RunState) -> f32 {
    match system_by_id(SURVEY_ARRAY_ID)
        .expect("survey-array system must exist")
        .progression
    {
        SystemProgression::SurveyArray(levels) => {
            let level = run_state.system_level(SURVEY_ARRAY_ID).unwrap_or(1).clamp(1, levels.len() as u8);
            levels[(level - 1) as usize].survey_multiplier
        }
        _ => unreachable!("survey-array progression must be survey levels"),
    }
}

fn unlock_planet_if_ready(
    discovered_planet_ids: &mut Vec<String>,
    survey_progress: f32,
    planet_id: &str,
    threshold: f32,
    newly_discovered: &mut Vec<String>,
) {
    if survey_progress + f32::EPSILON < threshold {
        return;
    }

    if !discovered_planet_ids.iter().any(|candidate| candidate == planet_id) {
        discovered_planet_ids.push(planet_id.to_string());
        discovered_planet_ids.sort();
        newly_discovered.push(planet_id.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::content::doctrines::DEEP_SURVEY_PROTOCOLS_ID;
    use crate::game::content::systems::SURVEY_ARRAY_ID;
    use crate::game::sim::state::SystemState;

    #[test]
    fn progression_survey_accumulates_correctly_and_discovers_planets_at_exact_thresholds() {
        let mut run_state = RunState::starter_fixture();
        run_state.service_state_mut(SURVEY_UPLINK_ID).unwrap().desired_active = true;
        run_state.service_state_mut(SURVEY_UPLINK_ID).unwrap().is_active = true;

        let pre_threshold = accumulate_survey_progress(&mut run_state, 599.0);
        assert_eq!(pre_threshold.progress_gained, 599.0);
        assert!(pre_threshold.discovered_planet_ids.is_empty());
        assert_eq!(run_state.station.survey_progress, 599.0);

        let cinder_forge = accumulate_survey_progress(&mut run_state, 1.0);
        assert_eq!(cinder_forge.progress_gained, 1.0);
        assert_eq!(cinder_forge.discovered_planet_ids, vec![CINDER_FORGE_ID.to_string()]);
        assert_eq!(run_state.station.survey_progress, 600.0);

        let aurora_pier = accumulate_survey_progress(&mut run_state, 800.0);
        assert_eq!(aurora_pier.progress_gained, 800.0);
        assert_eq!(aurora_pier.discovered_planet_ids, vec![AURORA_PIER_ID.to_string()]);
        assert_eq!(run_state.station.survey_progress, 1400.0);
        assert_eq!(
            run_state.station.discovered_planet_ids,
            vec![
                AURORA_PIER_ID.to_string(),
                CINDER_FORGE_ID.to_string(),
                "solstice-anchor".to_string(),
            ]
        );
    }

    #[test]
    fn progression_survey_uses_survey_array_and_doctrine_multipliers() {
        let mut run_state = RunState::starter_fixture();
        run_state.service_state_mut(SURVEY_UPLINK_ID).unwrap().desired_active = true;
        run_state.service_state_mut(SURVEY_UPLINK_ID).unwrap().is_active = true;
        run_state.station.doctrine_ids = vec![DEEP_SURVEY_PROTOCOLS_ID.to_string()];
        run_state.systems = vec![
            SystemState::new("reactor-core", 1),
            SystemState::new("habitat-ring", 1),
            SystemState::new("logistics-spine", 1),
            SystemState::new(SURVEY_ARRAY_ID, 3),
        ];

        let outcome = accumulate_survey_progress(&mut run_state, 10.0);

        assert!((outcome.progress_gained - 15.6).abs() < 0.000_001);
        assert!((run_state.station.survey_progress - 15.6).abs() < 0.000_001);
    }
}
