use chrono::Utc;

use crate::domain::entity::flow_definition::FlowDefinition;
use crate::domain::entity::flow_instance::{FlowInstance, FlowInstanceStatus};

#[allow(dead_code)]
pub struct TimeoutDetectionService;

/// Represents a detected timeout for a flow instance.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TimeoutResult {
    pub instance: FlowInstance,
    pub flow_name: String,
    pub timed_out_step_index: i32,
    pub timeout_seconds: i32,
    pub elapsed_seconds: i64,
}

impl TimeoutDetectionService {
    /// Check in-progress flow instances against their flow definitions and detect timeouts.
    /// Returns a list of instances that have exceeded the timeout for their current step.
    #[allow(dead_code)]
    pub fn detect_timeouts(
        instances: &[FlowInstance],
        flow_definitions: &[FlowDefinition],
    ) -> Vec<TimeoutResult> {
        let now = Utc::now();
        let mut results = Vec::new();

        for instance in instances {
            if instance.status != FlowInstanceStatus::InProgress {
                continue;
            }

            let flow = match flow_definitions.iter().find(|f| f.id == instance.flow_id) {
                Some(f) => f,
                None => continue,
            };

            if !flow.enabled {
                continue;
            }

            let next_step_index = (instance.current_step_index + 1) as usize;
            if next_step_index >= flow.steps.len() {
                // Already at or past the last step, check overall SLO timeout
                let elapsed = (now - instance.started_at).num_seconds();
                if elapsed > flow.slo.target_completion_seconds as i64 {
                    results.push(TimeoutResult {
                        instance: instance.clone(),
                        flow_name: flow.name.clone(),
                        timed_out_step_index: instance.current_step_index,
                        timeout_seconds: flow.slo.target_completion_seconds,
                        elapsed_seconds: elapsed,
                    });
                }
                continue;
            }

            let next_step = &flow.steps[next_step_index];
            if next_step.timeout_seconds <= 0 {
                continue;
            }

            // Calculate how long the instance has been waiting at the current step
            let elapsed = (now - instance.started_at).num_seconds();
            // Sum of timeouts for all steps up to and including the next step
            let cumulative_timeout: i32 = flow.steps[..=next_step_index]
                .iter()
                .map(|s| s.timeout_seconds)
                .sum();

            if elapsed > cumulative_timeout as i64 {
                results.push(TimeoutResult {
                    instance: instance.clone(),
                    flow_name: flow.name.clone(),
                    timed_out_step_index: next_step_index as i32,
                    timeout_seconds: next_step.timeout_seconds,
                    elapsed_seconds: elapsed,
                });
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::flow_definition::{FlowSlo, FlowStep};
    use chrono::Duration;
    use uuid::Uuid;

    fn make_flow(steps: Vec<(&str, &str, i32)>) -> FlowDefinition {
        let now = Utc::now();
        FlowDefinition {
            id: Uuid::new_v4(),
            tenant_id: "system".to_string(),
            name: "test_flow".to_string(),
            description: String::new(),
            domain: "service.task".to_string(),
            steps: steps
                .into_iter()
                .map(|(et, src, timeout)| FlowStep {
                    event_type: et.to_string(),
                    source: src.to_string(),
                    source_filter: Some(src.to_string()),
                    timeout_seconds: timeout,
                    description: String::new(),
                })
                .collect(),
            slo: FlowSlo {
                target_completion_seconds: 120,
                target_success_rate: 0.995,
                alert_on_violation: true,
            },
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    fn make_instance(flow_id: Uuid, step_index: i32, started_ago_secs: i64) -> FlowInstance {
        FlowInstance {
            id: Uuid::new_v4(),
            tenant_id: "system".to_string(),
            flow_id,
            correlation_id: format!("corr-{}", Uuid::new_v4()),
            status: FlowInstanceStatus::InProgress,
            current_step_index: step_index,
            started_at: Utc::now() - Duration::seconds(started_ago_secs),
            completed_at: None,
            duration_ms: None,
        }
    }

    #[test]
    fn test_no_timeout() {
        let flow = make_flow(vec![
            ("TaskCreated", "task-server", 0),
            ("ActivityCreated", "activity-server", 60),
        ]);
        let instance = make_instance(flow.id, 0, 10); // 10 seconds ago, timeout is 60
        let results = TimeoutDetectionService::detect_timeouts(&[instance], &[flow]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_timeout_detected() {
        let flow = make_flow(vec![
            ("TaskCreated", "task-server", 0),
            ("ActivityCreated", "activity-server", 30),
        ]);
        let instance = make_instance(flow.id, 0, 60); // 60 seconds ago, timeout is 30
        let results = TimeoutDetectionService::detect_timeouts(&[instance], &[flow]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].timed_out_step_index, 1);
    }

    #[test]
    fn test_completed_instance_skipped() {
        let flow = make_flow(vec![
            ("TaskCreated", "task-server", 0),
            ("ActivityCreated", "activity-server", 30),
        ]);
        let mut instance = make_instance(flow.id, 0, 60);
        instance.status = FlowInstanceStatus::Completed;
        let results = TimeoutDetectionService::detect_timeouts(&[instance], &[flow]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_disabled_flow_skipped() {
        let mut flow = make_flow(vec![
            ("TaskCreated", "task-server", 0),
            ("ActivityCreated", "activity-server", 30),
        ]);
        flow.enabled = false;
        let instance = make_instance(flow.id, 0, 60);
        let results = TimeoutDetectionService::detect_timeouts(&[instance], &[flow]);
        assert!(results.is_empty());
    }
}
