use crate::domain::entity::order::{
    CreateOrder, Order, OrderFilter, OrderItem, OrderStatus,
};
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
        let row = sqlx::query_as!(
            OrderRow,
            r#"
            SELECT id, customer_id, status, total_amount, currency, notes,
                   created_by, created_at, updated_at
            FROM orders
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_items_by_order_id(&self, order_id: Uuid) -> anyhow::Result<Vec<OrderItem>> {
        let rows = sqlx::query_as!(
            OrderItemRow,
            r#"
            SELECT id, order_id, product_id, product_name, quantity, unit_price,
                   subtotal, created_at
            FROM order_items
            WHERE order_id = $1
            ORDER BY created_at ASC
            "#,
            order_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(OrderItem::from).collect())
    }

    async fn find_all(&self, filter: &OrderFilter) -> anyhow::Result<Vec<Order>> {
        let status_str = filter.status.as_ref().map(|s| s.as_str().to_string());
        let limit = filter.limit.unwrap_or(50);
        let offset = filter.offset.unwrap_or(0);

        let rows = sqlx::query_as!(
            OrderRow,
            r#"
            SELECT id, customer_id, status, total_amount, currency, notes,
                   created_by, created_at, updated_at
            FROM orders
            WHERE ($1::text IS NULL OR customer_id = $1)
              AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            filter.customer_id,
            status_str,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn count(&self, filter: &OrderFilter) -> anyhow::Result<i64> {
        let status_str = filter.status.as_ref().map(|s| s.as_str().to_string());

        let row = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM orders
            WHERE ($1::text IS NULL OR customer_id = $1)
              AND ($2::text IS NULL OR status = $2)
            "#,
            filter.customer_id,
            status_str
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
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

        let order_row = sqlx::query_as!(
            OrderRow,
            r#"
            INSERT INTO orders (id, customer_id, status, total_amount, currency, notes, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, customer_id, status, total_amount, currency, notes, created_by, created_at, updated_at
            "#,
            order_id,
            input.customer_id,
            OrderStatus::Pending.as_str(),
            total_amount,
            input.currency,
            input.notes,
            created_by,
            now,
            now
        )
        .fetch_one(&mut *tx)
        .await?;

        let mut items = Vec::with_capacity(input.items.len());
        for item_input in &input.items {
            let subtotal = item_input.quantity as i64 * item_input.unit_price;
            let item_row = sqlx::query_as!(
                OrderItemRow,
                r#"
                INSERT INTO order_items (id, order_id, product_id, product_name, quantity, unit_price, subtotal, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id, order_id, product_id, product_name, quantity, unit_price, subtotal, created_at
                "#,
                Uuid::new_v4(),
                order_id,
                item_input.product_id,
                item_input.product_name,
                item_input.quantity,
                item_input.unit_price,
                subtotal,
                now
            )
            .fetch_one(&mut *tx)
            .await?;
            items.push(OrderItem::from(item_row));
        }

        tx.commit().await?;

        let order: Order = order_row.try_into()?;
        Ok((order, items))
    }

    async fn update_status(&self, id: Uuid, status: &OrderStatus) -> anyhow::Result<Order> {
        let now = Utc::now();
        let row = sqlx::query_as!(
            OrderRow,
            r#"
            UPDATE orders
            SET status = $2, updated_at = $3
            WHERE id = $1
            RETURNING id, customer_id, status, total_amount, currency, notes, created_by, created_at, updated_at
            "#,
            id,
            status.as_str(),
            now
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Order '{}' not found", id))?;

        row.try_into()
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!("DELETE FROM order_items WHERE order_id = $1", id)
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query!("DELETE FROM orders WHERE id = $1", id)
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

struct OrderRow {
    id: Uuid,
    customer_id: String,
    status: String,
    total_amount: i64,
    currency: String,
    notes: Option<String>,
    created_by: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl TryFrom<OrderRow> for Order {
    type Error = anyhow::Error;

    fn try_from(row: OrderRow) -> anyhow::Result<Self> {
        Ok(Self {
            id: row.id,
            customer_id: row.customer_id,
            status: OrderStatus::from_str(&row.status)?,
            total_amount: row.total_amount,
            currency: row.currency,
            notes: row.notes,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

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
