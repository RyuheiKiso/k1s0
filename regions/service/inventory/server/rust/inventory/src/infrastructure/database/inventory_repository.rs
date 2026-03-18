use crate::domain::entity::inventory_item::{InventoryFilter, InventoryItem};
use crate::domain::entity::outbox::OutboxEvent;
use crate::domain::error::InventoryError;
use crate::domain::repository::inventory_repository::InventoryRepository;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct InventoryPostgresRepository {
    pool: PgPool,
}

impl InventoryPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventoryRepository for InventoryPostgresRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<InventoryItem>> {
        let row = sqlx::query_as::<_, InventoryItemRow>(
            r#"
            SELECT id, product_id, warehouse_id, qty_available, qty_reserved,
                   version, created_at, updated_at
            FROM inventory_items
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(InventoryItem::from))
    }

    async fn find_by_product_and_warehouse(
        &self,
        product_id: &str,
        warehouse_id: &str,
    ) -> anyhow::Result<Option<InventoryItem>> {
        let row = sqlx::query_as::<_, InventoryItemRow>(
            r#"
            SELECT id, product_id, warehouse_id, qty_available, qty_reserved,
                   version, created_at, updated_at
            FROM inventory_items
            WHERE product_id = $1 AND warehouse_id = $2
            "#,
        )
        .bind(product_id)
        .bind(warehouse_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(InventoryItem::from))
    }

    async fn find_all(&self, filter: &InventoryFilter) -> anyhow::Result<Vec<InventoryItem>> {
        let limit = filter.limit.unwrap_or(50);
        let offset = filter.offset.unwrap_or(0);

        let rows = sqlx::query_as::<_, InventoryItemRow>(
            r#"
            SELECT id, product_id, warehouse_id, qty_available, qty_reserved,
                   version, created_at, updated_at
            FROM inventory_items
            WHERE ($1::text IS NULL OR product_id = $1)
              AND ($2::text IS NULL OR warehouse_id = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(&filter.product_id)
        .bind(&filter.warehouse_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(InventoryItem::from).collect())
    }

    async fn count(&self, filter: &InventoryFilter) -> anyhow::Result<i64> {
        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM inventory_items
            WHERE ($1::text IS NULL OR product_id = $1)
              AND ($2::text IS NULL OR warehouse_id = $2)
            "#,
        )
        .bind(&filter.product_id)
        .bind(&filter.warehouse_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    async fn reserve_stock(
        &self,
        id: Uuid,
        quantity: i32,
        expected_version: i32,
        order_id: &str,
    ) -> anyhow::Result<InventoryItem> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        let row = sqlx::query_as::<_, InventoryItemRow>(
            r#"
            UPDATE inventory_items
            SET qty_available = qty_available - $2,
                qty_reserved = qty_reserved + $2,
                version = version + 1,
                updated_at = $3
            WHERE id = $1 AND version = $4 AND qty_available >= $2
            RETURNING id, product_id, warehouse_id, qty_available, qty_reserved,
                      version, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(quantity)
        .bind(now)
        .bind(expected_version)
        .fetch_optional(&mut *tx)
        .await?;

        let row = match row {
            Some(r) => r,
            None => {
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM inventory_items WHERE id = $1)",
                )
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;

                if exists {
                    return Err(InventoryError::VersionConflict(id.to_string()).into());
                } else {
                    return Err(InventoryError::NotFound(id.to_string()).into());
                }
            }
        };

        let item = InventoryItem::from(row);

        // Outbox イベントを同一トランザクション内に挿入
        let outbox_payload = serde_json::json!({
            "metadata": {
                "event_id": Uuid::new_v4().to_string(),
                "event_type": "inventory.reserved",
                "source": "inventory-server",
                "timestamp": now.timestamp_millis(),
                "trace_id": "",
                "correlation_id": order_id,
                "schema_version": 1
            },
            "order_id": order_id,
            "product_id": item.product_id,
            "quantity": quantity,
            "warehouse_id": item.warehouse_id,
            "reserved_at": now.to_rfc3339(),
        });

        sqlx::query(
            r#"
            INSERT INTO outbox_events (id, aggregate_type, aggregate_id, event_type, payload, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind("inventory")
        .bind(id.to_string())
        .bind("inventory.reserved")
        .bind(&outbox_payload)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(item)
    }

    async fn release_stock(
        &self,
        id: Uuid,
        quantity: i32,
        expected_version: i32,
        order_id: &str,
        reason: &str,
    ) -> anyhow::Result<InventoryItem> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        let row = sqlx::query_as::<_, InventoryItemRow>(
            r#"
            UPDATE inventory_items
            SET qty_available = qty_available + $2,
                qty_reserved = qty_reserved - $2,
                version = version + 1,
                updated_at = $3
            WHERE id = $1 AND version = $4
            RETURNING id, product_id, warehouse_id, qty_available, qty_reserved,
                      version, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(quantity)
        .bind(now)
        .bind(expected_version)
        .fetch_optional(&mut *tx)
        .await?;

        let row = match row {
            Some(r) => r,
            None => {
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM inventory_items WHERE id = $1)",
                )
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;

                if exists {
                    return Err(InventoryError::VersionConflict(id.to_string()).into());
                } else {
                    return Err(InventoryError::NotFound(id.to_string()).into());
                }
            }
        };

        let item = InventoryItem::from(row);

        // Outbox イベントを同一トランザクション内に挿入
        let outbox_payload = serde_json::json!({
            "metadata": {
                "event_id": Uuid::new_v4().to_string(),
                "event_type": "inventory.released",
                "source": "inventory-server",
                "timestamp": now.timestamp_millis(),
                "trace_id": "",
                "correlation_id": order_id,
                "schema_version": 1
            },
            "order_id": order_id,
            "product_id": item.product_id,
            "quantity": quantity,
            "warehouse_id": item.warehouse_id,
            "reason": reason,
            "released_at": now.to_rfc3339(),
        });

        sqlx::query(
            r#"
            INSERT INTO outbox_events (id, aggregate_type, aggregate_id, event_type, payload, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind("inventory")
        .bind(id.to_string())
        .bind("inventory.released")
        .bind(&outbox_payload)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(item)
    }

    async fn update_stock(
        &self,
        id: Uuid,
        qty_available: i32,
        expected_version: i32,
    ) -> anyhow::Result<InventoryItem> {
        let now = Utc::now();

        let row = sqlx::query_as::<_, InventoryItemRow>(
            r#"
            UPDATE inventory_items
            SET qty_available = $2,
                version = version + 1,
                updated_at = $3
            WHERE id = $1 AND version = $4
            RETURNING id, product_id, warehouse_id, qty_available, qty_reserved,
                      version, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(qty_available)
        .bind(now)
        .bind(expected_version)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(InventoryItem::from(r)),
            None => {
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM inventory_items WHERE id = $1)",
                )
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

                if exists {
                    Err(InventoryError::VersionConflict(id.to_string()).into())
                } else {
                    Err(InventoryError::NotFound(id.to_string()).into())
                }
            }
        }
    }

    async fn create(
        &self,
        product_id: &str,
        warehouse_id: &str,
        qty_available: i32,
    ) -> anyhow::Result<InventoryItem> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query_as::<_, InventoryItemRow>(
            r#"
            INSERT INTO inventory_items (id, product_id, warehouse_id, qty_available, qty_reserved, version, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 0, 1, $5, $6)
            RETURNING id, product_id, warehouse_id, qty_available, qty_reserved, version, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(product_id)
        .bind(warehouse_id)
        .bind(qty_available)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(InventoryItem::from(row))
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
        // マルチインスタンス環境での重複処理を防止するため FOR UPDATE SKIP LOCKED を使用する
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

// ── 内部 Row 型 ──

#[derive(sqlx::FromRow)]
struct InventoryItemRow {
    id: Uuid,
    product_id: String,
    warehouse_id: String,
    qty_available: i32,
    qty_reserved: i32,
    version: i32,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<InventoryItemRow> for InventoryItem {
    fn from(row: InventoryItemRow) -> Self {
        Self {
            id: row.id,
            product_id: row.product_id,
            warehouse_id: row.warehouse_id,
            qty_available: row.qty_available,
            qty_reserved: row.qty_reserved,
            version: row.version,
            created_at: row.created_at,
            updated_at: row.updated_at,
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
