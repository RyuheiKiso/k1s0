use crate::domain::entity::flow_instance::{FlowInstance, FlowInstanceStatus};
use crate::domain::entity::flow_kpi::FlowKpi;

pub struct KpiAggregationService;

impl KpiAggregationService {
    pub fn aggregate(instances: &[FlowInstance]) -> FlowKpi {
        if instances.is_empty() {
            return FlowKpi::default();
        }

        let total_started = instances.len() as i64;
        let total_completed = instances
            .iter()
            .filter(|i| i.status == FlowInstanceStatus::Completed)
            .count() as i64;
        let total_failed = instances
            .iter()
            .filter(|i| {
                i.status == FlowInstanceStatus::Failed
                    || i.status == FlowInstanceStatus::Timeout
            })
            .count() as i64;
        let total_in_progress = instances
            .iter()
            .filter(|i| i.status == FlowInstanceStatus::InProgress)
            .count() as i64;

        let completion_rate = if total_started > 0 {
            total_completed as f64 / total_started as f64
        } else {
            0.0
        };

        let mut durations: Vec<f64> = instances
            .iter()
            .filter_map(|i| i.duration_ms.map(|d| d as f64 / 1000.0))
            .collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let avg_duration_seconds = if durations.is_empty() {
            0.0
        } else {
            durations.iter().sum::<f64>() / durations.len() as f64
        };

        let p50 = percentile(&durations, 0.50);
        let p95 = percentile(&durations, 0.95);
        let p99 = percentile(&durations, 0.99);

        FlowKpi {
            total_started,
            total_completed,
            total_failed,
            total_in_progress,
            completion_rate,
            avg_duration_seconds,
            p50_duration_seconds: p50,
            p95_duration_seconds: p95,
            p99_duration_seconds: p99,
            bottleneck_step: None,
        }
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let index = p * (sorted.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let frac = index - lower as f64;
        sorted[lower] * (1.0 - frac) + sorted[upper] * frac
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_instance(status: FlowInstanceStatus, duration_ms: Option<i64>) -> FlowInstance {
        FlowInstance {
            id: Uuid::new_v4(),
            flow_id: Uuid::new_v4(),
            correlation_id: "corr".to_string(),
            status,
            current_step_index: 0,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms,
        }
    }

    #[test]
    fn test_aggregate_empty() {
        let kpi = KpiAggregationService::aggregate(&[]);
        assert_eq!(kpi.total_started, 0);
        assert_eq!(kpi.completion_rate, 0.0);
    }

    #[test]
    fn test_aggregate_mixed() {
        let instances = vec![
            make_instance(FlowInstanceStatus::Completed, Some(1000)),
            make_instance(FlowInstanceStatus::Completed, Some(2000)),
            make_instance(FlowInstanceStatus::Failed, Some(500)),
            make_instance(FlowInstanceStatus::InProgress, None),
        ];
        let kpi = KpiAggregationService::aggregate(&instances);
        assert_eq!(kpi.total_started, 4);
        assert_eq!(kpi.total_completed, 2);
        assert_eq!(kpi.total_failed, 1);
        assert_eq!(kpi.total_in_progress, 1);
        assert!((kpi.completion_rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_aggregate_percentiles() {
        let instances: Vec<FlowInstance> = (1..=100)
            .map(|i| make_instance(FlowInstanceStatus::Completed, Some(i * 1000)))
            .collect();
        let kpi = KpiAggregationService::aggregate(&instances);
        assert!(kpi.p50_duration_seconds > 49.0 && kpi.p50_duration_seconds < 51.0);
        assert!(kpi.p95_duration_seconds > 94.0 && kpi.p95_duration_seconds < 96.0);
        assert!(kpi.p99_duration_seconds > 98.0 && kpi.p99_duration_seconds < 100.0);
    }
}
