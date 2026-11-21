use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    fn distance(self, other: Point) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
    fn minus(self, other: Point) -> Point {
        Point { x: self.x - other.x, y: self.y - other.y }
    }
    fn plus_scaled(self, delta: Point, scale: f64) -> Point {
        Point { x: self.x + delta.x * scale, y: self.y + delta.y * scale }
    }
    fn dot(self, other: Point) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub start: Point,
    pub goal: Point,
    pub speed_mps: f64,
    pub radius_m: f64,
    pub deadline_s: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MultiAgentInput {
    pub agent_a: Agent,
    pub agent_b: Agent,
    pub safety_margin_m: f64,
    pub delay_step_s: f64,
    pub max_delay_s: f64,
}

impl Default for MultiAgentInput {
    fn default() -> Self {
        Self {
            agent_a: Agent {
                start: Point { x: -2.0, y: 0.0 },
                goal: Point { x: 2.0, y: 0.0 },
                speed_mps: 1.0,
                radius_m: 0.2,
                deadline_s: 8.0,
            },
            agent_b: Agent {
                start: Point { x: 0.0, y: -2.0 },
                goal: Point { x: 0.0, y: 2.0 },
                speed_mps: 1.0,
                radius_m: 0.2,
                deadline_s: 8.0,
            },
            safety_margin_m: 0.05,
            delay_step_s: 0.25,
            max_delay_s: 3.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentTiming {
    pub delay_s: f64,
    pub arrival_s: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct JointSchedule {
    pub a: AgentTiming,
    pub b: AgentTiming,
    pub minimum_separation_m: f64,
    pub makespan_s: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MultiAgentResult {
    pub required_separation_m: f64,
    pub independent_minimum_separation_m: f64,
    pub independent_collision: bool,
    pub independent_meets_both_deadlines: bool,
    pub joint_schedule_found: bool,
    pub joint_schedule: Option<JointSchedule>,
    pub search_scope: String,
}

fn validate_agent(name: &str, a: &Agent) -> Result<(), String> {
    if [a.start.x, a.start.y, a.goal.x, a.goal.y, a.speed_mps, a.radius_m, a.deadline_s]
        .iter().any(|x| !x.is_finite()) {
        return Err(format!("{name}: all quantities must be finite"));
    }
    if a.speed_mps <= 0.0 || a.radius_m < 0.0 || a.deadline_s <= 0.0 {
        return Err(format!("{name}: speed and deadline must be positive, radius nonnegative"));
    }
    Ok(())
}

fn duration(a: &Agent) -> f64 {
    a.start.distance(a.goal) / a.speed_mps
}

fn location(a: &Agent, delay: f64, time: f64) -> Point {
    let distance = a.start.distance(a.goal);
    if distance == 0.0 {
        return a.goal;
    }
    let progress = ((time - delay) * a.speed_mps / distance).clamp(0.0, 1.0);
    a.start.plus_scaled(a.goal.minus(a.start), progress)
}

/// Continuous-time minimum separation: relative motion is linear between
/// start/finish events. Sampling only fixed time ticks would miss collisions.
fn min_separation(a: &Agent, da: f64, b: &Agent, db: f64) -> f64 {
    let ta = da + duration(a);
    let tb = db + duration(b);
    let mut events = [0.0, da, ta, db, tb];
    events.sort_by(f64::total_cmp);
    let mut minimum = location(a, da, 0.0).distance(location(b, db, 0.0));
    for w in events.windows(2) {
        let (t0, t1) = (w[0], w[1]);
        if t1 <= t0 {
            continue;
        }
        let r0 = location(a, da, t0).minus(location(b, db, t0));
        let r1 = location(a, da, t1).minus(location(b, db, t1));
        let dr = r1.minus(r0);
        let norm_sq = dr.dot(dr);
        let alpha = if norm_sq > 0.0 {
            (-r0.dot(dr) / norm_sq).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let near = r0.plus_scaled(dr, alpha);
        minimum = minimum.min(near.x.hypot(near.y));
    }
    minimum
}

pub fn run(input: MultiAgentInput) -> Result<MultiAgentResult, String> {
    validate_agent("agent_a", &input.agent_a)?;
    validate_agent("agent_b", &input.agent_b)?;
    if [input.safety_margin_m, input.delay_step_s, input.max_delay_s]
        .iter().any(|x| !x.is_finite()) {
        return Err("scheduling parameters must be finite".into());
    }
    if input.safety_margin_m < 0.0 || input.delay_step_s <= 0.0 || input.max_delay_s < 0.0 {
        return Err("margin and maximum delay must be nonnegative, delay step positive".into());
    }
    let max_steps = (input.max_delay_s / input.delay_step_s).floor();
    if !max_steps.is_finite() || max_steps > 100.0 {
        return Err("at most 100 delay steps per agent are supported".into());
    }
    let count = max_steps as usize;
    let a = &input.agent_a;
    let b = &input.agent_b;
    let d_a = duration(a);
    let d_b = duration(b);
    if !d_a.is_finite() || !d_b.is_finite() {
        return Err("path duration exceeds numeric range".into());
    }
    let required = a.radius_m + b.radius_m + input.safety_margin_m;
    let independent_min = min_separation(a, 0.0, b, 0.0);
    let independent_deadlines = d_a <= a.deadline_s && d_b <= b.deadline_s;
    let mut best: Option<JointSchedule> = None;

    for i in 0..=count {
        let delay_a = i as f64 * input.delay_step_s;
        let arrival_a = delay_a + d_a;
        if arrival_a > a.deadline_s {
            continue;
        }
        for j in 0..=count {
            let delay_b = j as f64 * input.delay_step_s;
            let arrival_b = delay_b + d_b;
            if arrival_b > b.deadline_s {
                continue;
            }
            let separation = min_separation(a, delay_a, b, delay_b);
            if separation < required {
                continue;
            }
            let candidate = JointSchedule {
                a: AgentTiming { delay_s: delay_a, arrival_s: arrival_a },
                b: AgentTiming { delay_s: delay_b, arrival_s: arrival_b },
                minimum_separation_m: separation,
                makespan_s: arrival_a.max(arrival_b),
            };
            let better = best.as_ref().map_or(true, |existing| {
                candidate.makespan_s < existing.makespan_s - 1e-9
                    || ((candidate.makespan_s - existing.makespan_s).abs() < 1e-9
                        && delay_a + delay_b
                            < existing.a.delay_s + existing.b.delay_s - 1e-9)
            });
            if better {
                best = Some(candidate);
            }
        }
    }

    Ok(MultiAgentResult {
        required_separation_m: required,
        independent_minimum_separation_m: independent_min,
        independent_collision: independent_min < required,
        independent_meets_both_deadlines: independent_deadlines,
        joint_schedule_found: best.is_some(),
        joint_schedule: best,
        search_scope: "Two constant-speed straight-line agents; finite-grid start delays only. No route changes, acceleration, sensing delays or adversarial disturbances.".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independently_valid_crossing_paths_collide() {
        let result = run(MultiAgentInput::default()).unwrap();
        assert!(result.independent_meets_both_deadlines);
        assert!(result.independent_collision);
        let plan = result.joint_schedule.unwrap();
        assert!(plan.minimum_separation_m >= result.required_separation_m);
        assert!(plan.a.arrival_s <= 8.0 && plan.b.arrival_s <= 8.0);
    }

    #[test]
    fn tight_deadlines_make_search_infeasible() {
        let mut input = MultiAgentInput::default();
        input.agent_a.deadline_s = 4.0;
        input.agent_b.deadline_s = 4.0;
        let result = run(input).unwrap();
        assert!(!result.joint_schedule_found);
    }

    #[test]
    fn overlapping_start_locations_are_detected() {
        let mut input = MultiAgentInput::default();
        input.agent_b.start = input.agent_a.start;
        let result = run(input).unwrap();
        assert!(result.independent_collision);
        assert!(!result.joint_schedule_found);
    }
}
