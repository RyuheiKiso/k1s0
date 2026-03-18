use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::Message;
use serde::Deserialize;

use crate::infrastructure::cache::FlagCache;
use crate::infrastructure::config::KafkaConfig;

#[derive(Debug, Deserialize)]
struct FlagChangedEvent {
    flag_key: String,
}

/// フィーチャーフラグ変更イベントを購読し、キャッシュを無効化するバックグラウンドタスクを起動する。
///
/// at-least-once セマンティクスのため auto.commit を無効化し、処理成功後に手動コミットする。
pub fn spawn_flag_cache_invalidator(
    kafka: KafkaConfig,
    cache: Arc<FlagCache>,
) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    // at-least-once セマンティクスのため auto.commit を無効化する
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "featureflag-cache-invalidator")
        .set("bootstrap.servers", kafka.brokers.join(","))
        .set("security.protocol", kafka.security_protocol)
        .set("enable.auto.commit", "false")
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
                    // 処理成功後にオフセットを手動コミットする
                    if let Err(e) = consumer.commit_message(&m, CommitMode::Async) {
                        tracing::warn!(error = %e, "failed to commit kafka offset");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "featureflag cache consumer error");
                }
            }
        }
    }))
}
