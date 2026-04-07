// Outbox ポーラー。未送信イベントを定期的に Kafka へ発行する。
// ARCH-HIGH-003 対応: 逐次送信から並行送信（buffer_unordered）へ改善。
// Kafka 送信失敗時は outbox_dead_letter テーブルへ退避し、
// outbox_events.published_at を更新して再ポーリングを防止する。
// BSL-CRIT-001 監査対応: CancellationToken による graceful shutdown を追加する
use crate::infrastructure::kafka::board_producer::BoardKafkaProducer;
use futures::StreamExt as _;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
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

    /// 未送信イベントを取得してKafkaへ並行発行し、結果に応じて published_at 更新または DLQ 退避を行う。
    /// Phase 1: SELECT + SKIP LOCKED でイベントを取得（ロックはコミットまで保持）。
    /// Phase 2: buffer_unordered によって最大 10 件を並行して Kafka へ送信する。
    /// Phase 3: 成功イベントは published_at を更新、失敗イベントは DLQ へ INSERT して再ポーリングを防止する。
    async fn poll_once(&self) -> anyhow::Result<()> {
        // Phase 1: トランザクション開始、未送信イベントを取得してロックを保持する
        let mut tx = self.pool.begin().await?;
        let rows: Vec<(Uuid, String, serde_json::Value)> = sqlx::query_as(
            "SELECT id, event_type, payload FROM board_service.outbox_events WHERE published_at IS NULL ORDER BY created_at LIMIT 100 FOR UPDATE SKIP LOCKED",
        )
        .fetch_all(&mut *tx)
        .await?;

        if rows.is_empty() {
            // ロックを解放してトランザクションを終了する
            tx.commit().await?;
            return Ok(());
        }

        // Phase 2: ロックを保持したまま Kafka へ並行送信する（最大 10 件同時）。
        // tx は DB 接続を保持するが Kafka 送信には使用しないため並行実行が可能。
        let producer = Arc::clone(&self.producer);
        let send_results: Vec<(Uuid, Result<(), String>)> =
            futures::stream::iter(rows.into_iter())
                .map(|(id, event_type, payload)| {
                    let producer = Arc::clone(&producer);
                    async move {
                        // ペイロードをバイト列にシリアライズしてKafkaへ送信する
                        let bytes = match serde_json::to_vec(&payload) {
                            Ok(b) => b,
                            Err(e) => {
                                return (id, Err(format!("serialize error: {e}")));
                            }
                        };
                        // column_id → board_id → id の優先順位でパーティションキーを決定する
                        let partition_key = payload
                            .get("column_id")
                            .and_then(|v| v.as_str())
                            .or_else(|| payload.get("board_id").and_then(|v| v.as_str()))
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| id.to_string());
                        match producer.publish(&event_type, &bytes, &partition_key).await {
                            Ok(()) => (id, Ok(())),
                            Err(e) => (id, Err(format!("kafka publish error: {e}"))),
                        }
                    }
                })
                .buffer_unordered(10)
                .collect()
                .await;

        // Phase 3: 送信結果に応じて DB を更新する
        for (id, result) in send_results {
            match result {
                Ok(()) => {
                    // 送信成功: published_at を現在時刻に更新して次回ポーリング対象から除外する
                    sqlx::query(
                        "UPDATE board_service.outbox_events SET published_at = NOW() WHERE id = $1",
                    )
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                }
                Err(error_message) => {
                    // 送信失敗: DLQ テーブルへ退避し、outbox_events を完了済みとしてマークする。
                    // published_at を設定することで無限リトライを防止する。
                    // 運用担当者は outbox_dead_letter を確認して手動再送またはスキップを判断する。
                    tracing::error!(
                        event_id = %id,
                        error = %error_message,
                        "outbox event failed to publish, moving to dead letter queue"
                    );
                    sqlx::query(
                        "INSERT INTO board_service.outbox_dead_letter (original_id, event_type, payload, error_message) \
                         SELECT id, event_type, payload, $2 FROM board_service.outbox_events WHERE id = $1",
                    )
                    .bind(id)
                    .bind(&error_message)
                    .execute(&mut *tx)
                    .await?;
                    // DLQ 退避後に outbox_events を完了済みとしてマークする
                    sqlx::query(
                        "UPDATE board_service.outbox_events SET published_at = NOW() WHERE id = $1",
                    )
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        // 全処理完了後にトランザクションをコミットしてロックを解放する
        tx.commit().await?;
        Ok(())
    }
}
