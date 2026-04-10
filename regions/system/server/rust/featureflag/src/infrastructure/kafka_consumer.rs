use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::Message;
use serde::Deserialize;

use crate::infrastructure::cache::FlagCache;
use crate::infrastructure::config::KafkaConfig;

/// フラグ変更イベントの Kafka メッセージ構造体。
/// STATIC-CRITICAL-001 監査対応: `tenant_id` を含めてキャッシュキーを正確に特定する。
/// `tenant_id` が存在しない場合はシステムテナント UUID にフォールバックする（ADR-0028 Phase 1）。
#[derive(Debug, Deserialize)]
struct FlagChangedEvent {
    flag_key: String,
    #[serde(default)]
    tenant_id: Option<String>,
}

/// システムテナントUUID: Kafka イベントに `tenant_id` が含まれない場合のフォールバック値。
const SYSTEM_TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";

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
                                // tenant_id が含まれない場合はシステムテナント UUID にフォールバックする（ADR-0028 Phase 1）
                                // HIGH-005 対応: &str 型で直接使用する（Uuid::parse_str 不要）
                                let tenant_id =
                                    event.tenant_id.as_deref().unwrap_or(SYSTEM_TENANT_ID);
                                cache.invalidate(tenant_id, &event.flag_key).await;
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
