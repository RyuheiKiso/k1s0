use std::sync::Arc;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use tokio_stream::StreamExt;
use tracing::{info, warn};

use crate::domain::entity::event_record::EventRecord;
use crate::domain::entity::flow_instance::{FlowInstance, FlowInstanceStatus};
use crate::domain::repository::{
    EventRecordRepository, FlowDefinitionRepository, FlowInstanceRepository,
};
use crate::domain::service::flow_matching::FlowMatchingService;
use crate::infrastructure::config::KafkaConfig;

pub struct EventKafkaConsumer {
    consumer: StreamConsumer,
    event_repo: Arc<dyn EventRecordRepository>,
    flow_def_repo: Arc<dyn FlowDefinitionRepository>,
    flow_inst_repo: Arc<dyn FlowInstanceRepository>,
}

impl EventKafkaConsumer {
    pub fn new(
        config: &KafkaConfig,
        event_repo: Arc<dyn EventRecordRepository>,
        flow_def_repo: Arc<dyn FlowDefinitionRepository>,
        flow_inst_repo: Arc<dyn FlowInstanceRepository>,
    ) -> anyhow::Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", config.brokers.join(","))
            .set("group.id", &config.consumer_group)
            .set("security.protocol", &config.security_protocol)
            .set("auto.offset.reset", "latest")
            .set("enable.auto.commit", "true")
            .create()?;

        Ok(Self {
            consumer,
            event_repo,
            flow_def_repo,
            flow_inst_repo,
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        use rdkafka::consumer::Consumer;

        // Subscribe to topic pattern
        self.consumer
            .subscribe(&["k1s0.*.*.*.v1"])
            .map_err(|e| anyhow::anyhow!("failed to subscribe: {}", e))?;

        info!("Kafka consumer started, subscribing to k1s0.*.*.*.v1");

        let mut stream = self.consumer.stream();
        while let Some(result) = stream.next().await {
            match result {
                Ok(msg) => {
                    if let Err(e) = self.process_message(&msg).await {
                        warn!(error = %e, "failed to process kafka message");
                    }
                }
                Err(e) => {
                    warn!(error = %e, "kafka consumer error");
                }
            }
        }

        Ok(())
    }

    async fn process_message(
        &self,
        msg: &rdkafka::message::BorrowedMessage<'_>,
    ) -> anyhow::Result<()> {
        let topic = msg.topic();

        // Exclude DLQ topics
        if topic.ends_with(".dlq.v1") {
            return Ok(());
        }

        // Extract domain from topic: k1s0.{tier}.{domain}.{event_type}.v1
        let parts: Vec<&str> = topic.split('.').collect();
        let domain = if parts.len() >= 4 {
            format!("{}.{}", parts[1], parts[2])
        } else {
            "unknown".to_string()
        };

        let event_type = msg
            .key()
            .and_then(|k| std::str::from_utf8(k).ok())
            .unwrap_or("unknown")
            .to_string();

        // Extract headers
        let mut correlation_id = String::new();
        let mut trace_id = String::new();
        if let Some(headers) = msg.headers() {
            use rdkafka::message::Headers;
            for header in headers.iter() {
                match header.key {
                    "correlation_id" => {
                        if let Some(v) = header.value {
                            correlation_id =
                                std::str::from_utf8(v).unwrap_or_default().to_string();
                        }
                    }
                    "trace_id" => {
                        if let Some(v) = header.value {
                            trace_id = std::str::from_utf8(v).unwrap_or_default().to_string();
                        }
                    }
                    _ => {}
                }
            }
        }

        if correlation_id.is_empty() {
            correlation_id = uuid::Uuid::new_v4().to_string();
        }

        let source = if parts.len() >= 4 {
            parts[2].to_string()
        } else {
            "unknown".to_string()
        };

        let mut event = EventRecord::new(
            correlation_id.clone(),
            event_type.clone(),
            source,
            domain.clone(),
            trace_id,
            chrono::Utc::now(),
        );

        // Flow matching
        let flow_defs = self
            .flow_def_repo
            .find_by_domain_and_event_type(&domain, &event_type)
            .await?;
        let all_flows = self.flow_def_repo.find_all().await?;

        if let Some((flow_id, step_index)) =
            FlowMatchingService::match_event(&event, &all_flows)
        {
            event.flow_id = Some(flow_id);
            event.flow_step_index = Some(step_index);

            // Update or create flow instance
            if let Some(mut instance) = self
                .flow_inst_repo
                .find_by_correlation_id(&correlation_id)
                .await?
            {
                instance.current_step_index = step_index;

                // Check if flow is complete
                if let Some(flow) = flow_defs.iter().find(|f| f.id == flow_id) {
                    if step_index as usize >= flow.steps.len() - 1 {
                        instance.status = FlowInstanceStatus::Completed;
                        instance.completed_at = Some(chrono::Utc::now());
                        let duration = instance
                            .completed_at
                            .unwrap()
                            .signed_duration_since(instance.started_at);
                        instance.duration_ms = Some(duration.num_milliseconds());
                    }
                }

                self.flow_inst_repo.update(&instance).await?;
            } else if step_index == 0 {
                let instance = FlowInstance::new(flow_id, correlation_id);
                self.flow_inst_repo.create(&instance).await?;
            }
        }

        self.event_repo.create(&event).await?;

        Ok(())
    }
}
