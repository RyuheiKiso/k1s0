use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use serde::Deserialize;

use crate::infrastructure::cache::FlagCache;
use crate::infrastructure::config::KafkaConfig;

#[derive(Debug, Deserialize)]
struct FlagChangedEvent {
    flag_key: String,
}

pub fn spawn_flag_cache_invalidator(
    kafka: KafkaConfig,
    cache: Arc<FlagCache>,
) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "featureflag-cache-invalidator")
        .set("bootstrap.servers", kafka.brokers.join(","))
        .set("security.protocol", kafka.security_protocol)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "latest")
        .create()
        .context("failed to create featureflag kafka consumer")?;

    consumer
        .subscribe(&[&kafka.topic])
        .context("failed to subscribe to featureflag topic")?;

    Ok(tokio::spawn(async move {
        let mut stream = consumer.stream();
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(m) => {
                    if let Some(payload) = m.payload() {
                        match serde_json::from_slice::<FlagChangedEvent>(payload) {
                            Ok(event) => {
                                cache.invalidate(&event.flag_key).await;
                            }
                            Err(e) => {
                                tracing::warn!(error = %e, "failed to decode FLAG_CHANGED event");
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "featureflag cache consumer error");
                }
            }
        }
    }))
}
