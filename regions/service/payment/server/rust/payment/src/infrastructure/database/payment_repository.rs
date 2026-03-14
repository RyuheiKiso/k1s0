use crate::domain::entity::outbox::OutboxEvent;
use crate::domain::entity::payment::{InitiatePayment, Payment, PaymentFilter, PaymentStatus};
use crate::domain::error::PaymentError;
use crate::domain::repository::payment_repository::PaymentRepository;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PaymentPostgresRepository {
    pool: PgPool,
}

impl PaymentPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentRepository for PaymentPostgresRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Payment>> {
        let row = sqlx::query_as::<_, PaymentRow>(
            r#"
            SELECT id, order_id, customer_id, amount, currency, status,
                   payment_method, transaction_id, error_code, error_message,
                   version, created_at, updated_at
            FROM payments
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_all(&self, filter: &PaymentFilter) -> anyhow::Result<Vec<Payment>> {
        let status_str = filter.status.as_ref().map(|s| s.as_str().to_string());
        let limit = filter.limit.unwrap_or(50);
        let offset = filter.offset.unwrap_or(0);

        let rows = sqlx::query_as::<_, PaymentRow>(
            r#"
            SELECT id, order_id, customer_id, amount, currency, status,
                   payment_method, transaction_id, error_code, error_message,
                   version, created_at, updated_at
            FROM payments
            WHERE ($1::text IS NULL OR order_id = $1)
              AND ($2::text IS NULL OR customer_id = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(&filter.order_id)
        .bind(&filter.customer_id)
        .bind(&status_str)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn count(&self, filter: &PaymentFilter) -> anyhow::Result<i64> {
        let status_str = filter.status.as_ref().map(|s| s.as_str().to_string());

        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM payments
            WHERE ($1::text IS NULL OR order_id = $1)
              AND ($2::text IS NULL OR customer_id = $2)
              AND ($3::text IS NULL OR status = $3)
            "#,
        )
        .bind(&filter.order_id)
        .bind(&filter.customer_id)
        .bind(&status_str)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    async fn create(&self, input: &InitiatePayment) -> anyhow::Result<Payment> {
        let mut tx = self.pool.begin().await?;

        let payment_id = Uuid::new_v4();
        let now = Utc::now();

        let payment_row = sqlx::query_as::<_, PaymentRow>(
            r#"
            INSERT INTO payments (id, order_id, customer_id, amount, currency, status,
                                  payment_method, version, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 1, $8, $9)
            RETURNING id, order_id, customer_id, amount, currency, status,
                      payment_method, transaction_id, error_code, error_message,
                      version, created_at, updated_at
            "#,
        )
        .bind(payment_id)
        .bind(&input.order_id)
        .bind(&input.customer_id)
        .bind(input.amount)
        .bind(&input.currency)
        .bind(PaymentStatus::Initiated.as_str())
        .bind(&input.payment_method)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await?;

        // Outbox イベントを同一トランザクション内に挿入
        let outbox_payload = serde_json::json!({
            "metadata": {
                "event_id": Uuid::new_v4().to_string(),
                "event_type": "payment.initiated",
                "source": "payment-server",
                "timestamp": now.timestamp_millis(),
                "trace_id": "",
                "correlation_id": payment_id.to_string(),
                "schema_version": 1
            },
            "payment_id": payment_id.to_string(),
            "order_id": input.order_id,
            "customer_id": input.customer_id,
            "amount": input.amount,
            "currency": input.currency,
            "payment_method": input.payment_method,
            "initiated_at": now.to_rfc3339(),
        });

        sqlx::query(
            r#"
            INSERT INTO outbox_events (id, aggregate_type, aggregate_id, event_type, payload, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind("payment")
        .bind(payment_id.to_string())
        .bind("payment.initiated")
        .bind(&outbox_payload)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        payment_row.try_into()
    }

    async fn complete(
        &self,
        id: Uuid,
        transaction_id: &str,
        expected_version: i32,
    ) -> anyhow::Result<Payment> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        let row = sqlx::query_as::<_, PaymentRow>(
            r#"
            UPDATE payments
            SET status = $2, transaction_id = $3, version = version + 1, updated_at = $4
            WHERE id = $1 AND version = $5
            RETURNING id, order_id, customer_id, amount, currency, status,
                      payment_method, transaction_id, error_code, error_message,
                      version, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(PaymentStatus::Completed.as_str())
        .bind(transaction_id)
        .bind(now)
        .bind(expected_version)
        .fetch_optional(&mut *tx)
        .await?;

        let row = match row {
            Some(r) => r,
            None => {
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM payments WHERE id = $1)",
                )
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;

                if exists {
                    return Err(PaymentError::VersionConflict(id.to_string()).into());
                } else {
                    return Err(PaymentError::NotFound(id.to_string()).into());
                }
            }
        };

        // Outbox イベントを同一トランザクション内に挿入
        let outbox_payload = serde_json::json!({
            "metadata": {
                "event_id": Uuid::new_v4().to_string(),
                "event_type": "payment.completed",
                "source": "payment-server",
                "timestamp": now.timestamp_millis(),
                "trace_id": "",
                "correlation_id": id.to_string(),
                "schema_version": 1
            },
            "payment_id": id.to_string(),
            "order_id": row.order_id,
            "amount": row.amount,
            "currency": row.currency,
            "transaction_id": transaction_id,
            "completed_at": now.to_rfc3339(),
        });

        sqlx::query(
            r#"
            INSERT INTO outbox_events (id, aggregate_type, aggregate_id, event_type, payload, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind("payment")
        .bind(id.to_string())
        .bind("payment.completed")
        .bind(&outbox_payload)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        row.try_into()
    }

    async fn fail(
        &self,
        id: Uuid,
        error_code: &str,
        error_message: &str,
        expected_version: i32,
    ) -> anyhow::Result<Payment> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        let row = sqlx::query_as::<_, PaymentRow>(
            r#"
            UPDATE payments
            SET status = $2, error_code = $3, error_message = $4, version = version + 1, updated_at = $5
            WHERE id = $1 AND version = $6
            RETURNING id, order_id, customer_id, amount, currency, status,
                      payment_method, transaction_id, error_code, error_message,
                      version, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(PaymentStatus::Failed.as_str())
        .bind(error_code)
        .bind(error_message)
        .bind(now)
        .bind(expected_version)
        .fetch_optional(&mut *tx)
        .await?;

        let row = match row {
            Some(r) => r,
            None => {
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM payments WHERE id = $1)",
                )
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;

                if exists {
                    return Err(PaymentError::VersionConflict(id.to_string()).into());
                } else {
                    return Err(PaymentError::NotFound(id.to_string()).into());
                }
            }
        };

        // Outbox イベントを同一トランザクション内に挿入
        let outbox_payload = serde_json::json!({
            "metadata": {
                "event_id": Uuid::new_v4().to_string(),
                "event_type": "payment.failed",
                "source": "payment-server",
                "timestamp": now.timestamp_millis(),
                "trace_id": "",
                "correlation_id": id.to_string(),
                "schema_version": 1
            },
            "payment_id": id.to_string(),
            "order_id": row.order_id,
            "reason": error_message,
            "error_code": error_code,
            "failed_at": now.to_rfc3339(),
        });

        sqlx::query(
            r#"
            INSERT INTO outbox_events (id, aggregate_type, aggregate_id, event_type, payload, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind("payment")
        .bind(id.to_string())
        .bind("payment.failed")
        .bind(&outbox_payload)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        row.try_into()
    }

    async fn refund(
        &self,
        id: Uuid,
        expected_version: i32,
    ) -> anyhow::Result<Payment> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        let row = sqlx::query_as::<_, PaymentRow>(
            r#"
            UPDATE payments
            SET status = $2, version = version + 1, updated_at = $3
            WHERE id = $1 AND version = $4
            RETURNING id, order_id, customer_id, amount, currency, status,
                      payment_method, transaction_id, error_code, error_message,
                      version, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(PaymentStatus::Refunded.as_str())
        .bind(now)
        .bind(expected_version)
        .fetch_optional(&mut *tx)
        .await?;

        let row = match row {
            Some(r) => r,
            None => {
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM payments WHERE id = $1)",
                )
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;

                if exists {
                    return Err(PaymentError::VersionConflict(id.to_string()).into());
                } else {
                    return Err(PaymentError::NotFound(id.to_string()).into());
                }
            }
        };

        // Outbox イベントを同一トランザクション内に挿入
        let outbox_payload = serde_json::json!({
            "metadata": {
                "event_id": Uuid::new_v4().to_string(),
                "event_type": "payment.refunded",
                "source": "payment-server",
                "timestamp": now.timestamp_millis(),
                "trace_id": "",
                "correlation_id": id.to_string(),
                "schema_version": 1
            },
            "payment_id": id.to_string(),
            "order_id": row.order_id,
            "refund_amount": row.amount,
            "currency": row.currency,
            "reason": "refund requested",
            "refunded_at": now.to_rfc3339(),
        });

        sqlx::query(
            r#"
            INSERT INTO outbox_events (id, aggregate_type, aggregate_id, event_type, payload, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind("payment")
        .bind(id.to_string())
        .bind("payment.refunded")
        .bind(&outbox_payload)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        row.try_into()
    }

    async fn insert_outbox_event(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO outbox_events (id, aggregate_type, aggregate_id, event_type, payload, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(event_type)
        .bind(payload)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn fetch_unpublished_events(&self, limit: i64) -> anyhow::Result<Vec<OutboxEvent>> {
        let rows = sqlx::query_as::<_, OutboxEventRow>(
            r#"
            SELECT id, aggregate_type, aggregate_id, event_type, payload,
                   created_at, published_at
            FROM outbox_events
            WHERE published_at IS NULL
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(OutboxEvent::from).collect())
    }

    async fn mark_event_published(&self, event_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE outbox_events
            SET published_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(event_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// -- 内部 Row 型 --

#[derive(sqlx::FromRow)]
struct PaymentRow {
    id: Uuid,
    order_id: String,
    customer_id: String,
    amount: i64,
    currency: String,
    status: String,
    payment_method: Option<String>,
    transaction_id: Option<String>,
    error_code: Option<String>,
    error_message: Option<String>,
    version: i32,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl TryFrom<PaymentRow> for Payment {
    type Error = anyhow::Error;

    fn try_from(row: PaymentRow) -> anyhow::Result<Self> {
        Ok(Self {
            id: row.id,
            order_id: row.order_id,
            customer_id: row.customer_id,
            amount: row.amount,
            currency: row.currency,
            status: row.status.parse::<PaymentStatus>().map_err(|e| anyhow::anyhow!("{}", e))?,
            payment_method: row.payment_method,
            transaction_id: row.transaction_id,
            error_code: row.error_code,
            error_message: row.error_message,
            version: row.version,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct OutboxEventRow {
    id: Uuid,
    aggregate_type: String,
    aggregate_id: String,
    event_type: String,
    payload: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
    published_at: Option<chrono::DateTime<Utc>>,
}

impl From<OutboxEventRow> for OutboxEvent {
    fn from(row: OutboxEventRow) -> Self {
        Self {
            id: row.id,
            aggregate_type: row.aggregate_type,
            aggregate_id: row.aggregate_id,
            event_type: row.event_type,
            payload: row.payload,
            created_at: row.created_at,
            published_at: row.published_at,
        }
    }
}
