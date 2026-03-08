use crate::domain::entity::order::{
    CreateOrder, Order, OrderFilter, OrderItem, OrderStatus,
};
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OrderRepository: Send + Sync {
    /// 注文をIDで取得する。
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Order>>;

    /// 注文の明細一覧を取得する。
    async fn find_items_by_order_id(&self, order_id: Uuid) -> anyhow::Result<Vec<OrderItem>>;

    /// フィルター条件で注文一覧を取得する。
    async fn find_all(&self, filter: &OrderFilter) -> anyhow::Result<Vec<Order>>;

    /// フィルター条件に一致する注文件数を取得する。
    async fn count(&self, filter: &OrderFilter) -> anyhow::Result<i64>;

    /// 注文と明細を作成する。
    async fn create(
        &self,
        input: &CreateOrder,
        created_by: &str,
    ) -> anyhow::Result<(Order, Vec<OrderItem>)>;

    /// 注文ステータスを更新する。
    async fn update_status(&self, id: Uuid, status: &OrderStatus) -> anyhow::Result<Order>;

    /// 注文を削除する（論理削除ではなく物理削除）。
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
}
