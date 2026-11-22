//! Deterministic event-time model of a producer/consumer deployment.
//! Each version can be individually coherent while a change invalidates
//! requests already in flight. This is a model of execution, not Git itself.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducerChange {
    pub at_ms: u64,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerChange {
    pub at_ms: u64,
    pub accepted_versions: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub sent_at_ms: u64,
    /// Absolute deadline measured in the same clock as sent_at_ms.
    pub deadline_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DeploymentInput {
    pub initial_producer_version: u32,
    pub initial_consumer_accepts: Vec<u32>,
    pub producer_changes: Vec<ProducerChange>,
    pub consumer_changes: Vec<ConsumerChange>,
    pub requests: Vec<Request>,
    /// Fixed transport delay. Proxy for the physical communication path.
    pub network_latency_ms: u64,
    /// Single-worker processing time for each accepted request.
    pub processing_time_ms: u64,
}

impl Default for DeploymentInput {
    fn default() -> Self {
        Self {
            initial_producer_version: 1,
            initial_consumer_accepts: vec![1],
            producer_changes: vec![ProducerChange {
                at_ms: 5,
                version: 2,
            }],
            consumer_changes: vec![ConsumerChange {
                at_ms: 5,
                accepted_versions: vec![2],
            }],
            requests: vec![
                Request {
                    id: "before".into(),
                    sent_at_ms: 0,
                    deadline_at_ms: 10,
                },
                Request {
                    id: "in_flight".into(),
                    sent_at_ms: 3,
                    deadline_at_ms: 15,
                },
                Request {
                    id: "after".into(),
                    sent_at_ms: 6,
                    deadline_at_ms: 20,
                },
            ],
            network_latency_ms: 4,
            processing_time_ms: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Completed,
    SchemaRejected,
    DeadlineMissed,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestTrace {
    pub id: String,
    pub producer_version: u32,
    pub sent_at_ms: u64,
    pub received_at_ms: u64,
    pub started_at_ms: u64,
    pub finished_at_ms: u64,
    pub deadline_at_ms: u64,
    pub compatible_when_sent: bool,
    pub accepted_versions_when_started: Vec<u32>,
    pub status: RequestStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeploymentResult {
    pub initial_configuration_compatible: bool,
    pub final_configuration_compatible: bool,
    pub completed: usize,
    pub schema_rejected: usize,
    pub deadline_missed: usize,
    pub all_operations_successful: bool,
    pub trace: Vec<RequestTrace>,
    pub interpretation: String,
}

fn producer_version_at(input: &DeploymentInput, time: u64) -> u32 {
    let mut version = input.initial_producer_version;
    for change in &input.producer_changes {
        if change.at_ms > time {
            break;
        }
        version = change.version;
    }
    version
}

fn accepted_at(input: &DeploymentInput, time: u64) -> &[u32] {
    let mut accepted = input.initial_consumer_accepts.as_slice();
    for change in &input.consumer_changes {
        if change.at_ms > time {
            break;
        }
        accepted = &change.accepted_versions;
    }
    accepted
}

pub fn run(input: DeploymentInput) -> Result<DeploymentResult, String> {
    if input.requests.len() > 10_000 {
        return Err("at most 10,000 requests are supported".into());
    }
    if input.producer_changes.len() > 1_000 || input.consumer_changes.len() > 1_000 {
        return Err("at most 1,000 changes of each kind are supported".into());
    }
    if input
        .producer_changes
        .windows(2)
        .any(|pair| pair[0].at_ms >= pair[1].at_ms)
        || input
            .consumer_changes
            .windows(2)
            .any(|pair| pair[0].at_ms >= pair[1].at_ms)
    {
        return Err("change times must be strictly increasing within each sequence".into());
    }

    let mut ids = HashSet::new();
    let mut arrivals = Vec::with_capacity(input.requests.len());
    for (index, request) in input.requests.iter().enumerate() {
        if request.id.is_empty() || !ids.insert(&request.id) {
            return Err("request IDs must be nonempty and unique".into());
        }
        if request.deadline_at_ms < request.sent_at_ms {
            return Err(format!(
                "request {} has a deadline before it was sent",
                request.id
            ));
        }
        let received_at = request
            .sent_at_ms
            .checked_add(input.network_latency_ms)
            .ok_or("transport timestamp overflow")?;
        arrivals.push((index, received_at));
    }
    // Arrival-time order with input-order tie breaking is the FIFO service rule.
    arrivals.sort_by_key(|(index, received_at)| (*received_at, *index));

    let initially_compatible = input
        .initial_consumer_accepts
        .contains(&input.initial_producer_version);
    let finally_compatible =
        accepted_at(&input, u64::MAX).contains(&producer_version_at(&input, u64::MAX));

    let mut worker_available_at: u64 = 0;
    let mut trace = Vec::with_capacity(arrivals.len());
    let mut completed = 0;
    let mut schema_rejected = 0;
    let mut deadline_missed = 0;

    for (index, received_at) in arrivals {
        let request = &input.requests[index];
        let version = producer_version_at(&input, request.sent_at_ms);
        let compatible_when_sent = accepted_at(&input, request.sent_at_ms).contains(&version);
        let started_at = received_at.max(worker_available_at);
        let accepted = accepted_at(&input, started_at);
        let compatible_at_start = accepted.contains(&version);
        let accepted_versions_when_started = accepted.to_vec();

        // Rejections consume no processing slot in this minimal model.
        // Accepted operations continue under the version checked at start,
        // even if a consumer change occurs before completion.
        let finished_at = if compatible_at_start {
            let finished = started_at
                .checked_add(input.processing_time_ms)
                .ok_or("processing timestamp overflow")?;
            worker_available_at = finished;
            finished
        } else {
            started_at
        };

        let status = if !compatible_at_start {
            schema_rejected += 1;
            RequestStatus::SchemaRejected
        } else if finished_at > request.deadline_at_ms {
            deadline_missed += 1;
            RequestStatus::DeadlineMissed
        } else {
            completed += 1;
            RequestStatus::Completed
        };
        trace.push(RequestTrace {
            id: request.id.clone(),
            producer_version: version,
            sent_at_ms: request.sent_at_ms,
            received_at_ms: received_at,
            started_at_ms: started_at,
            finished_at_ms: finished_at,
            deadline_at_ms: request.deadline_at_ms,
            compatible_when_sent,
            accepted_versions_when_started,
            status,
        });
    }
    Ok(DeploymentResult {
        initial_configuration_compatible: initially_compatible,
        final_configuration_compatible: finally_compatible,
        completed,
        schema_rejected,
        deadline_missed,
        all_operations_successful: schema_rejected == 0 && deadline_missed == 0,
        trace,
        interpretation: "Version compatibility is checked at service start, not at source commit or request send. Changes at time t take effect at t. Execution has fixed transport delay and one FIFO consumer worker. A successful state before and after a deployment does not guarantee the requests crossing it succeed.".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locally_valid_versions_can_fail_during_rollout() {
        let result = run(DeploymentInput::default()).unwrap();
        assert!(result.initial_configuration_compatible);
        assert!(result.final_configuration_compatible);
        assert_eq!(result.completed, 2);
        assert_eq!(result.schema_rejected, 1);
        assert!(!result.all_operations_successful);
        let crossing = result.trace.iter().find(|t| t.id == "in_flight").unwrap();
        assert!(crossing.compatible_when_sent);
        assert_eq!(crossing.status, RequestStatus::SchemaRejected);
    }

    #[test]
    fn compatibility_overlap_preserves_in_flight_requests() {
        let mut input = DeploymentInput::default();
        input.consumer_changes[0].accepted_versions = vec![1, 2];
        let result = run(input).unwrap();
        assert_eq!(result.completed, 3);
        assert_eq!(result.schema_rejected, 0);
        assert!(result.all_operations_successful);
    }

    #[test]
    fn adequate_versions_do_not_guarantee_deadline_feasibility() {
        let mut input = DeploymentInput::default();
        input.producer_changes.clear();
        input.consumer_changes.clear();
        input.processing_time_ms = 8;
        let result = run(input).unwrap();
        assert_eq!(result.schema_rejected, 0);
        assert!(result.deadline_missed > 0);
        assert!(!result.all_operations_successful);
    }

    #[test]
    fn changes_must_have_a_deterministic_order() {
        let mut input = DeploymentInput::default();
        input.producer_changes.push(ProducerChange {
            at_ms: 5,
            version: 3,
        });
        assert!(run(input).is_err());
    }

    #[test]
    fn catches_timestamp_overflow() {
        let mut input = DeploymentInput::default();
        input.requests[0].sent_at_ms = u64::MAX;
        input.requests[0].deadline_at_ms = u64::MAX;
        assert!(run(input).is_err());
    }
}
