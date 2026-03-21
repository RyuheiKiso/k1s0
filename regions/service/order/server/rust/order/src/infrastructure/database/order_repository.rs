use crate::domain::entity::order::{CreateOrder, Order, OrderFilter, OrderItem, OrderStatus};
use crate::domain::entity::outbox::OutboxEvent;
use crate::domain::error::OrderError;
use crate::domain::repository::order_repository::OrderRepository;
use crate::domain::service::order_service::OrderDomainService;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct OrderPostgresRepository {
    pool: PgPool,
}

impl OrderPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrderRepository for OrderPostgresRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Order>> {
        let row = sqlx::query_as::<_, OrderRow>(
            r#"
            SELECT id, customer_id, status, total_amount, currency, notes,
                   created_by, updated_by, version, created_at, updated_at
            FROM orders
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_items_by_order_id(&self, order_id: Uuid) -> anyhow::Result<Vec<OrderItem>> {
        let rows = sqlx::query_as::<_, OrderItemRow>(
            r#"
            SELECT id, order_id, product_id, product_name, quantity, unit_price,
                   subtotal, created_at
            FROM order_items
            WHERE order_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(OrderItem::from).collect())
    }

    async fn find_all(&self, filter: &OrderFilter) -> anyhow::Result<Vec<Order>> {
        let status_str = filter.status.as_ref().map(|s| s.as_str().to_string());
        let limit = filter.limit.unwrap_or(50);
        let offset = filter.offset.unwrap_or(0);

        let rows = sqlx::query_as::<_, OrderRow>(
            r#"
            SELECT id, customer_id, status, total_amount, currency, notes,
                   created_by, updated_by, version, created_at, updated_at
            FROM orders
            WHERE ($1::text IS NULL OR customer_id = $1)
              AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(&filter.customer_id)
        .bind(&status_str)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn count(&self, filter: &OrderFilter) -> anyhow::Result<i64> {
        let status_str = filter.status.as_ref().map(|s| s.as_str().to_string());

        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM orders
            WHERE ($1::text IS NULL OR customer_id = $1)
              AND ($2::text IS NULL OR status = $2)
            "#,
        )
        .bind(&filter.customer_id)
        .bind(&status_str)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    async fn create(
        &self,
        input: &CreateOrder,
        created_by: &str,
    ) -> anyhow::Result<(Order, Vec<OrderItem>)> {
        let mut tx = self.pool.begin().await?;

        let total_amount = OrderDomainService::calculate_total(&input.items);
        let order_id = Uuid::new_v4();
        let now = Utc::now();

        let order_row = sqlx::query_as::<_, OrderRow>(
            r#"
            INSERT INTO orders (id, customer_id, status, total_amount, currency, notes, created_by, version, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 1, $8, $9)
            RETURNING id, customer_id, status, total_amount, currency, notes, created_by, updated_by, version, created_at, updated_at
            "#,
        )
        .bind(order_id)
        .bind(&input.customer_id)
        .bind(OrderStatus::Pending.as_str())
        .bind(total_amount)
        .bind(&input.currency)
        .bind(&input.notes)
        .bind(created_by)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await?;

        let mut items = Vec::with_capacity(input.items.len());
        for item_input in &input.items {
            let subtotal = item_input.quantity as i64 * item_input.unit_price;
            let item_row = sqlx::query_as::<_, OrderItemRow>(
                r#"
                INSERT INTO order_items (id, order_id, product_id, product_name, quantity, unit_price, subtotal, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id, order_id, product_id, product_name, quantity, unit_price, subtotal, created_at
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(order_id)
            .bind(&item_input.product_id)
            .bind(&item_input.product_name)
            .bind(item_input.quantity)
            .bind(item_input.unit_price)
            .bind(subtotal)
            .bind(now)
            .fetch_one(&mut *tx)
            .await?;
            items.push(OrderItem::from(item_row));
        }

        // Outbox イベントを同一トランザクション内に挿入
        let items_json: Vec<serde_json::Value> = items
            .iter()
            .map(|item| {
                serde_json::json!({
                    "product_id": item.product_id,
                    "quantity": item.quantity,
                    "unit_price": item.unit_price,
                })
            })
            .collect();

        let outbox_payload = serde_json::json!({
            "metadata": {
                "event_id": Uuid::new_v4().to_string(),
                "event_type": "order.created",
                "source": "order-server",
                "timestamp": now.timestamp_millis(),
                "trace_id": "",
                "correlation_id": order_id.to_string(),
                "schema_version": 1
            },
            "order_id": order_id.to_string(),
            "customer_id": input.customer_id,
            "items": items_json,
            "total_amount": total_amount,
            "currency": input.currency,
        });

        // idempotency_key に event_id を使用して冪等性を保証する
        let event_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO outbox_events (id, aggregate_type, aggregate_id, event_type, payload, created_at, idempotency_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (idempotency_key) DO NOTHING
            "#,
        )
        .bind(event_id)
        .bind("order")
        .bind(order_id.to_string())
        .bind("order.created")
        .bind(&outbox_payload)
        .bind(now)
        .bind(event_id.to_string())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let order: Order = order_row.try_into()?;
        Ok((order, items))
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: &OrderStatus,
        updated_by: &str,
        expected_version: i32,
    ) -> anyhow::Result<Order> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        let row = sqlx::query_as::<_, OrderRow>(
            r#"
            UPDATE orders
            SET status = $2, updated_by = $3, version = version + 1, updated_at = $4
            WHERE id = $1 AND version = $5
            RETURNING id, customer_id, status, total_amount, currency, notes, created_by, updated_by, version, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(status.as_str())
        .bind(updated_by)
        .bind(now)
        .bind(expected_version)
        .fetch_optional(&mut *tx)
        .await?;

        let row = match row {
            Some(r) => r,
            None => {
                // UPDATE が 0行 → レコード不在 or バージョン不一致を判別
                let exists: bool =
                    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM orders WHERE id = $1)")
                        .bind(id)
                        .fetch_one(&mut *tx)
                        .await?;

                if exists {
                    return Err(OrderError::VersionConflict(id.to_string()).into());
                } else {
                    return Err(OrderError::NotFound(id.to_string()).into());
                }
            }
        };

        // Outbox イベントを同一トランザクション内に挿入
        let event_type = if *status == OrderStatus::Cancelled {
            "order.cancelled"
        } else {
            "order.updated"
        };

        let mut outbox_payload = serde_json::json!({
            "metadata": {
                "event_id": Uuid::new_v4().to_string(),
                "event_type": event_type,
                "source": "order-server",
                "timestamp": now.timestamp_millis(),
                "trace_id": "",
                "correlation_id": id.to_string(),
                "schema_version": 1
            },
            "order_id": id.to_string(),
            "user_id": updated_by,
            "status": status.as_str(),
            "total_amount": row.total_amount,
        });

        if *status == OrderStatus::Cancelled {
            outbox_payload["reason"] = serde_json::json!("status changed to cancelled");
        }

        // idempotency_key に event_id を使用して冪等性を保証する
        let event_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO outbox_events (id, aggregate_type, aggregate_id, event_type, payload, created_at, idempotency_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (idempotency_key) DO NOTHING
            "#,
        )
        .bind(event_id)
        .bind("order")
        .bind(id.to_string())
        .bind(event_type)
        .bind(&outbox_payload)
        .bind(now)
        .bind(event_id.to_string())
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
        // idempotency_key に event_id を使用して冪等性を保証する
        let event_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO outbox_events (id, aggregate_type, aggregate_id, event_type, payload, created_at, idempotency_key)
            VALUES ($1, $2, $3, $4, $5, NOW(), $6)
            ON CONFLICT (idempotency_key) DO NOTHING
            "#,
        )
        .bind(event_id)
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(event_type)
        .bind(payload)
        .bind(event_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 未パブリッシュイベントを FOR UPDATE SKIP LOCKED で取得する（mark は行わない）。
    /// at-least-once 配信のため、publish 成功後に mark_events_published を呼ぶこと。
    async fn fetch_unpublished_events(&self, limit: i64) -> anyhow::Result<Vec<OutboxEvent>> {
        let mut tx = self.pool.begin().await?;

        // 未パブリッシュイベントを FOR UPDATE SKIP LOCKED でロック付き取得する
        let rows = sqlx::query_as::<_, OutboxEventRow>(
            r#"
            SELECT id, aggregate_type, aggregate_id, event_type, payload,
                   created_at, published_at
            FROM outbox_events
            WHERE published_at IS NULL
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(limit)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(rows.into_iter().map(OutboxEvent::from).collect())
    }

    /// 指定した ID のイベントをパブリッシュ済みとしてマークする。
    /// publish 成功後のみ呼び出すことで at-least-once セマンティクスを実現する。
    async fn mark_events_published(&self, ids: &[Uuid]) -> anyhow::Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        sqlx::query(
            r#"
            UPDATE outbox_events
            SET published_at = NOW()
            WHERE id = ANY($1)
            "#,
        )
        .bind(ids)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM order_items WHERE order_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query("DELETE FROM orders WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            anyhow::bail!("Order '{}' not found", id);
        }

        tx.commit().await?;
        Ok(())
    }
}

// ── 内部 Row 型 ──

#[derive(sqlx::FromRow)]
struct OrderRow {
    id: Uuid,
    customer_id: String,
    status: String,
    total_amount: i64,
    currency: String,
    notes: Option<String>,
    created_by: String,
    updated_by: Option<String>,
    version: i32,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl TryFrom<OrderRow> for Order {
    type Error = anyhow::Error;

    fn try_from(row: OrderRow) -> anyhow::Result<Self> {
        Ok(Self {
            id: row.id,
            customer_id: row.customer_id,
            status: row
                .status
                .parse::<OrderStatus>()
                .map_err(|e| anyhow::anyhow!("{}", e))?,
            total_amount: row.total_amount,
            currency: row.currency,
            notes: row.notes,
            created_by: row.created_by,
            updated_by: row.updated_by,
            version: row.version,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct OrderItemRow {
    id: Uuid,
    order_id: Uuid,
    product_id: String,
    product_name: String,
    quantity: i32,
    unit_price: i64,
    subtotal: i64,
    created_at: chrono::DateTime<Utc>,
}

impl From<OrderItemRow> for OrderItem {
    fn from(row: OrderItemRow) -> Self {
        Self {
            id: row.id,
            order_id: row.order_id,
            product_id: row.product_id,
            product_name: row.product_name,
            quantity: row.quantity,
            unit_price: row.unit_price,
            subtotal: row.subtotal,
            created_at: row.created_at,
        }
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
