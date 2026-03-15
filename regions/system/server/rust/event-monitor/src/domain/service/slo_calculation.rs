use crate::domain::entity::flow_definition::FlowDefinition;
use crate::domain::entity::flow_instance::{FlowInstance, FlowInstanceStatus};
use crate::domain::entity::flow_kpi::{BurnRateWindow, SloStatus};

pub struct SloCalculationService;

impl SloCalculationService {
    pub fn calculate(flow: &FlowDefinition, instances: &[FlowInstance]) -> SloStatus {
        let total = instances.len() as f64;
        if total == 0.0 {
            return SloStatus {
                target_completion_seconds: flow.slo.target_completion_seconds,
                target_success_rate: flow.slo.target_success_rate,
                current_success_rate: 1.0,
                is_violated: false,
                burn_rate: 0.0,
                estimated_budget_exhaustion_hours: f64::INFINITY,
            };
        }

        let completed = instances
            .iter()
            .filter(|i| i.status == FlowInstanceStatus::Completed)
            .count() as f64;
        let current_success_rate = completed / total;
        let is_violated = current_success_rate < flow.slo.target_success_rate;

        let error_budget = 1.0 - flow.slo.target_success_rate;
        let actual_error_rate = 1.0 - current_success_rate;
        let burn_rate = if error_budget > 0.0 {
            actual_error_rate / error_budget
        } else {
            0.0
        };

        let estimated_budget_exhaustion_hours = if burn_rate > 0.0 {
            // Assume 30-day budget window
            (30.0 * 24.0) / burn_rate
        } else {
            f64::INFINITY
        };

        SloStatus {
            target_completion_seconds: flow.slo.target_completion_seconds,
            target_success_rate: flow.slo.target_success_rate,
            current_success_rate,
            is_violated,
            burn_rate,
            estimated_budget_exhaustion_hours,
        }
    }

    /// Calculate SLO status from a pre-computed FlowKpi (used with cached KPI data).
    pub fn calculate_from_kpi(
        flow: &FlowDefinition,
        kpi: &crate::domain::entity::flow_kpi::FlowKpi,
    ) -> SloStatus {
        let total = kpi.total_started as f64;
        if total == 0.0 {
            return SloStatus {
                target_completion_seconds: flow.slo.target_completion_seconds,
                target_success_rate: flow.slo.target_success_rate,
                current_success_rate: 1.0,
                is_violated: false,
                burn_rate: 0.0,
                estimated_budget_exhaustion_hours: f64::INFINITY,
            };
        }

        let current_success_rate = kpi.completion_rate;
        let is_violated = current_success_rate < flow.slo.target_success_rate;

        let error_budget = 1.0 - flow.slo.target_success_rate;
        let actual_error_rate = 1.0 - current_success_rate;
        let burn_rate = if error_budget > 0.0 {
            actual_error_rate / error_budget
        } else {
            0.0
        };

        let estimated_budget_exhaustion_hours = if burn_rate > 0.0 {
            (30.0 * 24.0) / burn_rate
        } else {
            f64::INFINITY
        };

        SloStatus {
            target_completion_seconds: flow.slo.target_completion_seconds,
            target_success_rate: flow.slo.target_success_rate,
            current_success_rate,
            is_violated,
            burn_rate,
            estimated_budget_exhaustion_hours,
        }
    }

    pub fn calculate_burn_rate(
        flow: &FlowDefinition,
        instances_by_window: &[(&str, &[FlowInstance])],
    ) -> Vec<BurnRateWindow> {
        let error_budget = 1.0 - flow.slo.target_success_rate;

        instances_by_window
            .iter()
            .map(|(window, instances)| {
                let total = instances.len() as f64;
                if total == 0.0 {
                    return BurnRateWindow {
                        window: window.to_string(),
                        burn_rate: 0.0,
                        error_budget_remaining: 1.0,
                    };
                }

                let completed = instances
                    .iter()
                    .filter(|i| i.status == FlowInstanceStatus::Completed)
                    .count() as f64;
                let actual_error_rate = 1.0 - (completed / total);
                let burn_rate = if error_budget > 0.0 {
                    actual_error_rate / error_budget
                } else {
                    0.0
                };
                let error_budget_remaining = if error_budget > 0.0 {
                    (1.0 - (actual_error_rate / error_budget)).max(0.0)
                } else {
                    1.0
                };

                BurnRateWindow {
                    window: window.to_string(),
                    burn_rate,
                    error_budget_remaining,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::flow_definition::{FlowSlo, FlowStep};
    use chrono::Utc;
    use uuid::Uuid;

    fn make_flow() -> FlowDefinition {
        let now = Utc::now();
        FlowDefinition {
            id: Uuid::new_v4(),
            name: "test_flow".to_string(),
            description: String::new(),
            domain: "service.order".to_string(),
            steps: vec![FlowStep {
                event_type: "OrderCreated".to_string(),
                source: "order-service".to_string(),
                source_filter: Some("order-service".to_string()),
                timeout_seconds: 30,
                description: String::new(),
            }],
            slo: FlowSlo {
                target_completion_seconds: 120,
                target_success_rate: 0.99,
                alert_on_violation: true,
            },
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    fn make_instance(status: FlowInstanceStatus) -> FlowInstance {
        FlowInstance {
            id: Uuid::new_v4(),
            flow_id: Uuid::new_v4(),
            correlation_id: "corr".to_string(),
            status,
            current_step_index: 0,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
        }
    }

    #[test]
    fn test_calculate_no_instances() {
        let flow = make_flow();
        let status = SloCalculationService::calculate(&flow, &[]);
        assert_eq!(status.current_success_rate, 1.0);
        assert!(!status.is_violated);
    }

    #[test]
    fn test_calculate_all_completed() {
        let flow = make_flow();
        let instances: Vec<_> = (0..100)
            .map(|_| make_instance(FlowInstanceStatus::Completed))
            .collect();
        let status = SloCalculationService::calculate(&flow, &instances);
        assert_eq!(status.current_success_rate, 1.0);
        assert!(!status.is_violated);
    }

    #[test]
    fn test_calculate_slo_violated() {
        let flow = make_flow();
        let mut instances: Vec<_> = (0..90)
            .map(|_| make_instance(FlowInstanceStatus::Completed))
            .collect();
        instances.extend((0..10).map(|_| make_instance(FlowInstanceStatus::Failed)));
        let status = SloCalculationService::calculate(&flow, &instances);
        assert!((status.current_success_rate - 0.9).abs() < 0.01);
        assert!(status.is_violated);
        assert!(status.burn_rate > 1.0);
    }

    #[test]
    fn test_burn_rate_windows() {
        let flow = make_flow();
        let good: Vec<_> = (0..99)
            .map(|_| make_instance(FlowInstanceStatus::Completed))
            .collect();
        let bad: Vec<_> = vec![make_instance(FlowInstanceStatus::Failed)];
        let mut all = good.clone();
        all.extend(bad);

        let windows =
            SloCalculationService::calculate_burn_rate(&flow, &[("1h", &all), ("24h", &good)]);
        assert_eq!(windows.len(), 2);
        assert!(windows[0].burn_rate > 0.0);
        assert_eq!(windows[1].burn_rate, 0.0);
    }
}
