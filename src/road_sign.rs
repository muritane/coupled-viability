use serde::{Deserialize, Serialize};

/// Simplified longitudinal approach to a fixed sign and irreversible decision line.
/// This is a deterministic feasibility proxy, not a calibrated perception model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RoadSignInput {
    pub approach_distance_m: f64,
    pub speed_mps: f64,
    pub sign_height_m: f64,
    pub sign_width_m: f64,
    pub visibility_m: f64,
    pub visual_quality: f64,
    pub minimum_angular_width_deg: f64,
    pub recognition_latency_s: f64,
    pub reaction_latency_s: f64,
    pub deceleration_mps2: f64,
    pub maneuver_margin_m: f64,
    pub uncertainty_margin_s: f64,
}

impl Default for RoadSignInput {
    fn default() -> Self {
        Self {
            approach_distance_m: 120.0,
            speed_mps: 20.0,
            sign_height_m: 2.0,
            sign_width_m: 1.5,
            visibility_m: 110.0,
            visual_quality: 0.8,
            minimum_angular_width_deg: 0.8,
            recognition_latency_s: 0.35,
            reaction_latency_s: 0.8,
            deceleration_mps2: 5.0,
            maneuver_margin_m: 10.0,
            uncertainty_margin_s: 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RoadSignResult {
    pub recognition_range_m: f64,
    pub initial_observation_distance_m: f64,
    pub detection_time_s: f64,
    pub remaining_distance_at_detection_m: f64,
    pub necessary_distance_m: f64,
    pub stopping_distance_m: f64,
    pub available_slack_m: f64,
    pub recognition_possible: bool,
    pub viable_response: bool,
    pub explanation: String,
}

pub fn run(input: RoadSignInput) -> Result<RoadSignResult, String> {
    let vals = [
        input.approach_distance_m,
        input.speed_mps,
        input.sign_height_m,
        input.sign_width_m,
        input.visibility_m,
        input.visual_quality,
        input.minimum_angular_width_deg,
        input.recognition_latency_s,
        input.reaction_latency_s,
        input.deceleration_mps2,
        input.maneuver_margin_m,
        input.uncertainty_margin_s,
    ];
    if vals.iter().any(|x| !x.is_finite()) {
        return Err("all road-sign parameters must be finite".into());
    }
    if input.approach_distance_m < 0.0
        || input.sign_height_m <= 0.0
        || input.sign_width_m <= 0.0
        || input.speed_mps <= 0.0
        || input.visibility_m < 0.0
        || input.minimum_angular_width_deg <= 0.0
        || input.deceleration_mps2 <= 0.0
        || input.recognition_latency_s < 0.0
        || input.reaction_latency_s < 0.0
        || input.maneuver_margin_m < 0.0
        || input.uncertainty_margin_s < 0.0
        || !(0.0..=1.0).contains(&input.visual_quality)
    {
        return Err("invalid physical bounds; visual_quality must be in [0,1]".into());
    }
    // Proxy for effective legibility: poorer quality requires a larger angular
    // width. No claim is made that visual quality is a universal linear model.
    let effective_width = input.sign_width_m * input.visual_quality;
    let angle_rad = input.minimum_angular_width_deg.to_radians();
    if angle_rad >= std::f64::consts::PI {
        return Err("angular threshold must be below 180 degrees".into());
    }
    let range = if effective_width > 0.0 {
        effective_width / (2.0 * (angle_rad / 2.0).tan())
    } else {
        0.0
    };
    if !range.is_finite() {
        return Err("recognition geometry exceeds numeric range".into());
    }
    let first_observation = range
        .min(input.visibility_m)
        .min(input.approach_distance_m);
    let detection_time = (input.approach_distance_m - first_observation) / input.speed_mps;
    let stopping = input.speed_mps.powi(2) / (2.0 * input.deceleration_mps2);
    let necessary = input.speed_mps
        * (input.recognition_latency_s + input.reaction_latency_s + input.uncertainty_margin_s)
        + stopping
        + input.maneuver_margin_m;
    if !stopping.is_finite() || !necessary.is_finite() {
        return Err("required distance exceeds numeric range".into());
    }
    // Zero visibility/quality means no sensing event, not a detection at zero range.
    let recognized = first_observation > 0.0 && input.visual_quality > 0.0;
    let slack = first_observation - necessary;
    let viable = recognized && slack >= 0.0;

    let explanation = if !recognized {
        "The sign is unobservable under the specified simplified conditions."
    } else if viable {
        "Recognition and braking/maneuver margins fit before the decision line in this simplified straight-road model."
    } else {
        "The sign becomes legible too late for the specified latency and braking/maneuver distance."
    };
    Ok(RoadSignResult {
        recognition_range_m: range,
        initial_observation_distance_m: first_observation,
        detection_time_s: detection_time,
        remaining_distance_at_detection_m: first_observation,
        necessary_distance_m: necessary,
        stopping_distance_m: stopping,
        available_slack_m: slack,
        recognition_possible: recognized,
        viable_response: viable,
        explanation: explanation.into(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RoadSignSweepInput {
    pub baseline: RoadSignInput,
    pub improved: RoadSignInput,
    pub speeds_mps: Vec<f64>,
    pub visibility_factors: Vec<f64>,
    pub quality_factors: Vec<f64>,
}

impl Default for RoadSignSweepInput {
    fn default() -> Self {
        let baseline = RoadSignInput::default();
        let improved = RoadSignInput {
            approach_distance_m: 160.0,
            sign_width_m: 2.0,
            visibility_m: 150.0,
            visual_quality: 0.95,
            recognition_latency_s: 0.25,
            ..RoadSignInput::default()
        };
        Self {
            baseline,
            improved,
            speeds_mps: vec![10.0, 15.0, 20.0, 25.0, 30.0],
            visibility_factors: vec![0.5, 0.75, 1.0],
            quality_factors: vec![0.6, 0.8, 1.0],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SweepResult {
    pub scenario_count: usize,
    pub baseline_viable_count: usize,
    pub improved_viable_count: usize,
    pub baseline_viability_fraction: f64,
    pub improved_viability_fraction: f64,
    pub gained_scenarios: usize,
    pub lost_scenarios: usize,
    pub interpretation: String,
}

pub fn sweep(input: RoadSignSweepInput) -> Result<SweepResult, String> {
    if input.speeds_mps.is_empty()
        || input.visibility_factors.is_empty()
        || input.quality_factors.is_empty()
    {
        return Err("sweep input arrays must be nonempty".into());
    }
    let count = input.speeds_mps.len()
        .checked_mul(input.visibility_factors.len())
        .and_then(|x| x.checked_mul(input.quality_factors.len()))
        .ok_or("scenario count overflow")?;
    if count > 10_000 {
        return Err("at most 10,000 scenarios are supported".into());
    }
    if input.visibility_factors.iter().any(|x| !x.is_finite() || *x < 0.0)
        || input.quality_factors.iter().any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
    {
        return Err("visibility factors must be nonnegative, quality factors in [0,1]".into());
    }
    let mut baseline_count = 0;
    let mut improved_count = 0;
    let mut gained = 0;
    let mut lost = 0;
    for speed in &input.speeds_mps {
        for visibility_factor in &input.visibility_factors {
            for quality_factor in &input.quality_factors {
                let mut baseline = input.baseline.clone();
                baseline.speed_mps = *speed;
                baseline.visibility_m *= visibility_factor;
                baseline.visual_quality *= quality_factor;
                let mut improved = input.improved.clone();
                improved.speed_mps = *speed;
                improved.visibility_m *= visibility_factor;
                improved.visual_quality *= quality_factor;
                let a = run(baseline)?.viable_response;
                let b = run(improved)?.viable_response;
                baseline_count += usize::from(a);
                improved_count += usize::from(b);
                gained += usize::from(!a && b);
                lost += usize::from(a && !b);
            }
        }
    }
    Ok(SweepResult {
        scenario_count: count,
        baseline_viable_count: baseline_count,
        improved_viable_count: improved_count,
        baseline_viability_fraction: baseline_count as f64 / count as f64,
        improved_viability_fraction: improved_count as f64 / count as f64,
        gained_scenarios: gained,
        lost_scenarios: lost,
        interpretation: "Fractions are over a user-defined deterministic grid; they are not probabilities without a justified scenario distribution.".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sign_is_feasible() {
        let r = run(RoadSignInput::default()).unwrap();
        assert!(r.viable_response);
    }

    #[test]
    fn zero_visibility_blocks_detection() {
        let r = run(RoadSignInput { visibility_m: 0.0, ..Default::default() }).unwrap();
        assert!(!r.recognition_possible);
        assert!(!r.viable_response);
    }

    #[test]
    fn late_appearance_fails_even_when_recognizable() {
        let r = run(RoadSignInput { visibility_m: 10.0, ..Default::default() }).unwrap();
        assert!(r.recognition_possible);
        assert!(!r.viable_response);
    }

    #[test]
    fn improved_design_increases_grid_viability() {
        let r = sweep(RoadSignSweepInput::default()).unwrap();
        assert_eq!(r.scenario_count, 45);
        assert!(r.improved_viable_count >= r.baseline_viable_count);
        assert_eq!(r.lost_scenarios, 0);
    }
}
