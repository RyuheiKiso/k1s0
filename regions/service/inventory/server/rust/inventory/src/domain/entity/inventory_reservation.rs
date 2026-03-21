// 在庫予約エンティティ。注文と在庫アイテムの紐付けを管理する。
// Saga 補償トランザクションで order_id から予約を逆引きするために使用する。
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 在庫予約エンティティ
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InventoryReservation {
    pub id: Uuid,
    pub order_id: String,
    pub inventory_item_id: Uuid,
    pub product_id: String,
    pub warehouse_id: String,
    pub quantity: i32,
    /// 予約ステータス: 'reserved'（予約中）/ 'released'（解放済み）/ 'confirmed'（確定済み）
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
