// Outbox ポーラー。未送信イベントを定期的に Kafka へ発行する。
// FOR UPDATE SKIP LOCKED により複数インスタンスが同一レコードを重複処理しないようにする。
// Kafka 送信失敗時はログを記録してスキップし、他のイベントの処理を継続する。
use crate::infrastructure::kafka::board_producer::BoardKafkaProducer;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub struct OutboxPoller {
    pool: PgPool,
    producer: Arc<BoardKafkaProducer>,
}

impl OutboxPoller {
    pub fn new(pool: PgPool, producer: Arc<BoardKafkaProducer>) -> Self {
        Self { pool, producer }
    }

    /// ポーリングループ。サービス起動後にバックグラウンドタスクとして実行する。
    pub async fn run(self) {
        loop {
            if let Err(e) = self.poll_once().await {
                tracing::warn!(error = %e, "outbox poll error");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
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
            "SELECT id, event_type, payload FROM board.outbox_events WHERE published = false ORDER BY created_at LIMIT 100 FOR UPDATE SKIP LOCKED",
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

            // Kafka 送信失敗時はログを記録して残りのイベント処理を継続する
            if let Err(e) = self.producer.publish(&event_type, &bytes).await {
                tracing::error!(error = %e, event_id = %id, event_type = %event_type, "failed to publish outbox event to kafka, skipping");
                continue;
            }

            // 送信成功したイベントのみ published = true に更新する
            sqlx::query("UPDATE board.outbox_events SET published = true WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }

        // 全処理完了後にトランザクションをコミットする
        tx.commit().await?;
        Ok(())
    }
}
