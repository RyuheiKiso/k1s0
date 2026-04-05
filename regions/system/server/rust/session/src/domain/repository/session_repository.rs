use async_trait::async_trait;

use crate::domain::entity::session::Session;
use crate::error::SessionError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, session: &Session) -> Result<(), SessionError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, SessionError>;
    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, SessionError>;
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Session>, SessionError>;
    #[allow(dead_code)]
    async fn delete(&self, id: &str) -> Result<(), SessionError>;

    /// HIGH-002 対応: JWT の jti を失効リストに登録する。
    /// remaining_secs は JWT の残余有効期限（TTL）。
    /// デフォルト実装は何もしない（InMemory/テスト用）。
    /// Redis 実装では SET EX でキーを登録し、有効期限後に自動削除する。
    async fn revoke_jti(&self, jti: &str, remaining_secs: u64) -> Result<(), SessionError> {
        // デフォルト実装: InMemory やモックでは jti 失効は不要のためスキップする
        let _ = (jti, remaining_secs);
        Ok(())
    }
}
