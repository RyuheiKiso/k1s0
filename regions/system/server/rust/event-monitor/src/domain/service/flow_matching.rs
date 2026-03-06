use uuid::Uuid;

use crate::domain::entity::event_record::EventRecord;
use crate::domain::entity::flow_definition::FlowDefinition;

pub struct FlowMatchingService;

impl FlowMatchingService {
    pub fn match_event(
        event: &EventRecord,
        flow_definitions: &[FlowDefinition],
    ) -> Option<(Uuid, i32)> {
        for flow in flow_definitions {
            if !flow.enabled {
                continue;
            }
            // Fix 1: domain filter - flow.domain must match event.domain
            if flow.domain != event.domain {
                continue;
            }
            for (index, step) in flow.steps.iter().enumerate() {
                // Fix 5: source_filter - only check source when source_filter is Some
                let source_matches = match &step.source_filter {
                    Some(filter) => filter == &event.source,
                    None => true, // skip source check when no filter specified
                };
                if step.event_type == event.event_type && source_matches {
                    return Some((flow.id, index as i32));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::flow_definition::{FlowSlo, FlowStep};
    use chrono::Utc;

    fn make_flow(name: &str, steps: Vec<(&str, &str)>, enabled: bool) -> FlowDefinition {
        let now = Utc::now();
        FlowDefinition {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: String::new(),
            domain: "service.order".to_string(),
            steps: steps
                .into_iter()
                .map(|(et, src)| FlowStep {
                    event_type: et.to_string(),
                    source: src.to_string(),
                    source_filter: Some(src.to_string()),
                    timeout_seconds: 30,
                    description: String::new(),
                })
                .collect(),
            slo: FlowSlo {
                target_completion_seconds: 120,
                target_success_rate: 0.995,
                alert_on_violation: true,
            },
            enabled,
            created_at: now,
            updated_at: now,
        }
    }

    fn make_event(event_type: &str, source: &str) -> EventRecord {
        EventRecord::new(
            "corr-1".to_string(),
            event_type.to_string(),
            source.to_string(),
            "service.order".to_string(),
            "trace-1".to_string(),
            Utc::now(),
        )
    }

    #[test]
    fn test_match_event_found() {
        let flow = make_flow(
            "order_fulfillment",
            vec![
                ("OrderCreated", "order-service"),
                ("PaymentProcessed", "payment-service"),
            ],
            true,
        );
        let event = make_event("PaymentProcessed", "payment-service");
        let result = FlowMatchingService::match_event(&event, &[flow.clone()]);
        assert!(result.is_some());
        let (flow_id, step_index) = result.unwrap();
        assert_eq!(flow_id, flow.id);
        assert_eq!(step_index, 1);
    }

    #[test]
    fn test_match_event_not_found() {
        let flow = make_flow(
            "order_fulfillment",
            vec![("OrderCreated", "order-service")],
            true,
        );
        let event = make_event("UnknownEvent", "unknown-service");
        let result = FlowMatchingService::match_event(&event, &[flow]);
        assert!(result.is_none());
    }

    #[test]
    fn test_match_event_disabled_flow_skipped() {
        let flow = make_flow(
            "disabled_flow",
            vec![("OrderCreated", "order-service")],
            false,
        );
        let event = make_event("OrderCreated", "order-service");
        let result = FlowMatchingService::match_event(&event, &[flow]);
        assert!(result.is_none());
    }

    #[test]
    fn test_match_event_first_matching_flow() {
        let flow1 = make_flow(
            "flow1",
            vec![("OrderCreated", "order-service")],
            true,
        );
        let flow2 = make_flow(
            "flow2",
            vec![("OrderCreated", "order-service")],
            true,
        );
        let event = make_event("OrderCreated", "order-service");
        let result = FlowMatchingService::match_event(&event, &[flow1.clone(), flow2]);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, flow1.id);
    }
}
