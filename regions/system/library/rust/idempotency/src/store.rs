use async_trait::async_trait;

use crate::{IdempotencyError, IdempotencyRecord, IdempotencyStatus};

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait IdempotencyStore: Send + Sync {
    /// レコードを取得する（期限切れは None を返す）
    async fn get(&self, key: &str) -> Result<Option<IdempotencyRecord>, IdempotencyError>;
    /// 新規レコードを挿入する（重複キーは Err(Duplicate)）
    async fn insert(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError>;
    /// レコードのステータスと結果を更新する
    async fn update(
        &self,
        key: &str,
        status: IdempotencyStatus,
        response_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError>;
    /// レコードを削除する
    async fn delete(&self, key: &str) -> Result<bool, IdempotencyError>;
}
