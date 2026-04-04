// Outbox ポーラー。未送信イベントを定期的に Kafka へ発行する。
// FOR UPDATE SKIP LOCKED により複数インスタンスが同一レコードを重複処理しないようにする。
// Kafka 送信失敗時はログを記録してスキップし、他のイベントの処理を継続する。
// BSL-CRIT-001 監査対応: CancellationToken による graceful shutdown を追加する
use crate::infrastructure::kafka::activity_producer::ActivityKafkaProducer;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct OutboxPoller {
    pool: PgPool,
    producer: Arc<ActivityKafkaProducer>,
}

impl OutboxPoller {
    pub fn new(pool: PgPool, producer: Arc<ActivityKafkaProducer>) -> Self {
        Self { pool, producer }
    }

    /// ポーリングループ。サービス起動後にバックグラウンドタスクとして実行する。
    /// CancellationToken がキャンセルされるとループを抜けて graceful shutdown する。
    /// tokio::select! の biased モードにより shutdown シグナルを優先的に処理する。
    pub async fn run(self, token: CancellationToken) {
        loop {
            tokio::select! {
                biased;
                _ = token.cancelled() => {
                    tracing::info!("outbox poller received shutdown signal, stopping");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    if let Err(e) = self.poll_once().await {
                        tracing::warn!(error = %e, "outbox poll error");
                    }
                }
            }
        }
    }

    /// 未送信イベントを取得してKafkaへ発行し、published_at を現在時刻に更新する。
    /// SELECT と UPDATE を同一トランザクション内で実行する。
    /// FOR UPDATE SKIP LOCKED により複数インスタンスの重複処理を防止する。
    async fn poll_once(&self) -> anyhow::Result<()> {
        // トランザクションを開始して SELECT と UPDATE を原子的に実行する
        let mut tx = self.pool.begin().await?;

        // published_at IS NULL で未送信レコードのみを対象とし、他インスタンスが処理中のレコードをスキップする
        let rows: Vec<(Uuid, String, serde_json::Value)> = sqlx::query_as(
            "SELECT id, event_type, payload FROM activity_service.outbox_events WHERE published_at IS NULL ORDER BY created_at LIMIT 100 FOR UPDATE SKIP LOCKED",
        )
        .fetch_all(&mut *tx)
        .await?;

        for (id, event_type, payload) in rows {
            // ペイロードをバイト列にシリアライズしてKafkaへ送信する
            let bytes = match serde_json::to_vec(&payload) {
                Ok(b) => b,
                Err(e) => {
                    // シリアライズ失敗時はログを記録して次のイベントへ継続する
                    tracing::error!(error = %e, event_id = %id, "failed to serialize outbox payload, skipping");
                    continue;
                }
            };

            // ペイロードから activity_id を取得してパーティションキーとして使用する。
            // activity_id が取得できない場合はイベント ID の文字列をフォールバックキーとして使用する。
            // 同一アクティビティのイベントを同一パーティションへ送信することで順序保証と負荷分散を両立する。
            let partition_key_str = payload
                .get("activity_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| id.to_string());

            // Kafka 送信失敗時はログを記録して残りのイベント処理を継続する
            if let Err(e) = self.producer.publish(&event_type, &bytes, &partition_key_str).await {
                tracing::error!(error = %e, event_id = %id, event_type = %event_type, "failed to publish outbox event to kafka, skipping");
                continue;
            }

            // 送信成功したイベントのみ published_at を現在時刻に更新する
            sqlx::query("UPDATE activity_service.outbox_events SET published_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }

        // 全処理完了後にトランザクションをコミットする
        tx.commit().await?;
        Ok(())
    }
}
