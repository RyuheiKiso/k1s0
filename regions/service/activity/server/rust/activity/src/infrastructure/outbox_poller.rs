use crate::infrastructure::kafka::activity_producer::ActivityKafkaProducer;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub struct OutboxPoller {
    pool: PgPool,
    producer: Arc<ActivityKafkaProducer>,
}

impl OutboxPoller {
    pub fn new(pool: PgPool, producer: Arc<ActivityKafkaProducer>) -> Self {
        Self { pool, producer }
    }

    pub async fn run(self) {
        loop {
            if let Err(e) = self.poll_once().await {
                tracing::warn!(error = %e, "outbox poll error");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn poll_once(&self) -> anyhow::Result<()> {
        let rows: Vec<(Uuid, String, serde_json::Value)> = sqlx::query_as(
            "SELECT id, event_type, payload FROM activity.outbox_events WHERE published = false ORDER BY created_at LIMIT 100",
        )
        .fetch_all(&self.pool)
        .await?;

        for (id, event_type, payload) in rows {
            let bytes = serde_json::to_vec(&payload)?;
            self.producer.publish(&event_type, &bytes).await?;
            sqlx::query("UPDATE activity.outbox_events SET published = true WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }
}
