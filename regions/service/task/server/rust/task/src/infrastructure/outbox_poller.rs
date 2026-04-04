// Outbox ポーラー。未送信イベントを定期的に Kafka へ発行する。
// FOR UPDATE SKIP LOCKED により複数インスタンスが同一レコードを重複処理しないようにする。
// Kafka 送信失敗時はログを記録してスキップし、他のイベントの処理を継続する。
// M-011 監査対応: CancellationToken による graceful shutdown を追加する
// 以前は無限ループで停止できなかった。tokio_util::sync::CancellationToken を使用する
use crate::infrastructure::kafka::task_producer::TaskKafkaProducer;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct OutboxPoller {
    pool: PgPool,
    producer: Arc<TaskKafkaProducer>,
}

impl OutboxPoller {
    pub fn new(pool: PgPool, producer: Arc<TaskKafkaProducer>) -> Self {
        Self { pool, producer }
    }

    /// ポーリングループ。サービス起動後にバックグラウンドタスクとして実行する。
    /// CancellationToken がキャンセルされた場合はループを終了して graceful shutdown する。
    pub async fn run(self, token: CancellationToken) {
        loop {
            // CancellationToken がキャンセルされた場合はループを終了する
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

    /// 未送信イベントを取得してKafkaへ発行し、published フラグを更新する。
    /// SELECT と UPDATE を同一トランザクション内で実行する。
    /// FOR UPDATE SKIP LOCKED により複数インスタンスの重複処理を防止する。
    async fn poll_once(&self) -> anyhow::Result<()> {
        // トランザクションを開始して SELECT と UPDATE を原子的に実行する
        let mut tx = self.pool.begin().await?;

        // FOR UPDATE SKIP LOCKED で他インスタンスが処理中のレコードをスキップする
        let rows: Vec<(Uuid, String, serde_json::Value)> = sqlx::query_as(
            "SELECT id, event_type, payload FROM task_service.outbox_events WHERE published_at IS NULL ORDER BY created_at LIMIT 100 FOR UPDATE SKIP LOCKED",
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

            // ペイロードから task_id を取得してパーティションキーとして使用する。
            // task_id が取得できない場合はイベント ID の文字列をフォールバックキーとして使用する。
            let task_id_str = payload
                .get("task_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| id.to_string());

            // Kafka 送信失敗時はログを記録して残りのイベント処理を継続する
            if let Err(e) = self.producer.publish(&event_type, &bytes, &task_id_str).await {
                tracing::error!(error = %e, event_id = %id, event_type = %event_type, "failed to publish outbox event to kafka, skipping");
                continue;
            }

            // 送信成功したイベントのみ published_at を現在時刻に更新する
            sqlx::query("UPDATE task_service.outbox_events SET published_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }

        // 全処理完了後にトランザクションをコミットする
        tx.commit().await?;
        Ok(())
    }
}
