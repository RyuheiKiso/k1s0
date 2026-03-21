use crate::domain::entity::inventory_item::{InventoryFilter, InventoryItem};
use crate::domain::entity::inventory_reservation::InventoryReservation;
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
        .bind("inventory")
        .bind(id.to_string())
        .bind("inventory.reserved")
        .bind(&outbox_payload)
        .bind(now)
        .bind(event_id.to_string())
        .execute(&mut *tx)
        .await?;

        // 在庫予約レコードを挿入する（冪等性保証: ON CONFLICT DO NOTHING で二重予約を防止）
        let reservation_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO inventory_reservations
                (id, order_id, inventory_item_id, product_id, warehouse_id, quantity, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, 'reserved', $7, $8)
            ON CONFLICT (order_id, inventory_item_id) DO NOTHING
            "#,
        )
        .bind(reservation_id)
        .bind(order_id)
        .bind(item.id)
        .bind(&item.product_id)
        .bind(&item.warehouse_id)
        .bind(quantity)
        .bind(now)
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
        .bind("inventory")
        .bind(id.to_string())
        .bind("inventory.released")
        .bind(&outbox_payload)
        .bind(now)
        .bind(event_id.to_string())
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

    /// 未パブリッシュイベントを取得し、processing_at をセットしてクレームする。
    /// at-least-once 配信のため、publish 成功後に mark_events_published を呼ぶこと。
    async fn fetch_unpublished_events(&self, limit: i64) -> anyhow::Result<Vec<OutboxEvent>> {
        let mut tx = self.pool.begin().await?;

        // FOR UPDATE SKIP LOCKED で排他取得し、同一トランザクション内で processing_at をセット。
        // これにより他のポーラーが同じイベントを取得することを防ぐ。
        // タイムアウト条件: processing_at が 5 分以上経過したイベントも再取得し、
        // ポーラークラッシュによるスタックイベントをリカバリする（H-001）。
        let rows = sqlx::query_as::<_, OutboxEventRow>(
            r#"
            UPDATE outbox_events
            SET processing_at = NOW()
            WHERE id IN (
                SELECT id FROM outbox_events
                WHERE published_at IS NULL
                  AND (processing_at IS NULL OR processing_at < NOW() - INTERVAL '5 minutes')
                ORDER BY created_at ASC
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, aggregate_type, aggregate_id, event_type, payload, created_at, published_at
            "#,
        )
        .bind(limit)
        .fetch_all(&mut *tx)
        .await?;

        // クレーム UPDATE をコミットし、ロックを解放する（processing_at が保護を継続）
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

    /// 注文IDに紐づく予約中（status='reserved'）の在庫予約レコードを取得する。
    /// 補償トランザクション実行前に解放対象を確認するために使用する。
    async fn find_reservations_by_order_id(
        &self,
        order_id: &str,
    ) -> anyhow::Result<Vec<InventoryReservation>> {
        let rows = sqlx::query_as::<_, InventoryReservation>(
            r#"
            SELECT id, order_id, inventory_item_id, product_id, warehouse_id,
                   quantity, status, created_at, updated_at
            FROM inventory_reservations
            WHERE order_id = $1 AND status = 'reserved'
            "#,
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// order_id に紐づく全在庫予約を解放する（Saga 補償トランザクション）。
    /// FOR UPDATE で予約レコードをロックしてから、inventory_items の在庫数量を復元し、
    /// outbox_events に inventory.released イベントを挿入した後、予約ステータスを released に更新する。
    /// fetch + update + outbox_events INSERT を単一トランザクション内で実行することで原子性を保証する。
    async fn compensate_order_reservations(
        &self,
        order_id: &str,
        reason: &str,
    ) -> anyhow::Result<Vec<InventoryItem>> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        // 対象予約レコードを FOR UPDATE でロックして取得する（他トランザクションとの競合を防止）
        let reservations = sqlx::query_as::<_, InventoryReservation>(
            r#"
            SELECT id, order_id, inventory_item_id, product_id, warehouse_id,
                   quantity, status, created_at, updated_at
            FROM inventory_reservations
            WHERE order_id = $1 AND status = 'reserved'
            FOR UPDATE
            "#,
        )
        .bind(order_id)
        .fetch_all(&mut *tx)
        .await?;

        // 予約が存在しない場合は冪等に成功を返す
        if reservations.is_empty() {
            tx.commit().await?;
            return Ok(vec![]);
        }

        let mut released_items = Vec::with_capacity(reservations.len());

        for reservation in &reservations {
            // inventory_items の qty_available を増加、qty_reserved を減少して在庫を復元する
            let row = sqlx::query_as::<_, InventoryItemRow>(
                r#"
                UPDATE inventory_items
                SET qty_available = qty_available + $2,
                    qty_reserved = qty_reserved - $2,
                    version = version + 1,
                    updated_at = $3
                WHERE id = $1
                RETURNING id, product_id, warehouse_id, qty_available, qty_reserved,
                          version, created_at, updated_at
                "#,
            )
            .bind(reservation.inventory_item_id)
            .bind(reservation.quantity)
            .bind(now)
            .fetch_optional(&mut *tx)
            .await?;

            let item = match row {
                Some(r) => InventoryItem::from(r),
                None => {
                    return Err(InventoryError::NotFound(
                        reservation.inventory_item_id.to_string(),
                    )
                    .into());
                }
            };

            // Outbox イベントを同一トランザクション内に挿入して、Kafka への配信を保証する
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
                "product_id": reservation.product_id,
                "quantity": reservation.quantity,
                "warehouse_id": reservation.warehouse_id,
                "reason": reason,
                "released_at": now.to_rfc3339(),
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
            .bind("inventory")
            .bind(reservation.inventory_item_id.to_string())
            .bind("inventory.released")
            .bind(&outbox_payload)
            .bind(now)
            .bind(event_id.to_string())
            .execute(&mut *tx)
            .await?;

            released_items.push(item);
        }

        // 全予約レコードのステータスを 'released' に一括更新する
        let reservation_ids: Vec<Uuid> = reservations.iter().map(|r| r.id).collect();
        sqlx::query(
            r#"
            UPDATE inventory_reservations
            SET status = 'released', updated_at = $2
            WHERE id = ANY($1)
            "#,
        )
        .bind(&reservation_ids)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(released_items)
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
    // クレーム方式のポーラー制御用タイムスタンプ（他のポーラーが重複取得しないよう保護する）
    #[allow(dead_code)]
    processing_at: Option<chrono::DateTime<Utc>>,
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
