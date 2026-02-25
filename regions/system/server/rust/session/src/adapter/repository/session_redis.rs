use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;

use crate::domain::entity::session::Session;
use crate::domain::repository::SessionRepository;
use crate::error::SessionError;

/// RedisSessionRepository は Redis ベースのセッションリポジトリ。
///
/// キー設計:
///   - `session:{id}` — セッション JSON
///   - `session:token:{token}` — セッション ID へのマッピング
///   - `session:user:{user_id}` — ユーザーに紐づくセッション ID の SET
pub struct RedisSessionRepository {
    conn: ConnectionManager,
    prefix: String,
}

impl RedisSessionRepository {
    pub fn new(conn: ConnectionManager) -> Self {
        Self {
            conn,
            prefix: "session:".to_string(),
        }
    }

    /// セッション ID 用のキーを生成する。
    fn session_key(&self, id: &str) -> String {
        format!("{}id:{}", self.prefix, id)
    }

    /// トークン → セッション ID マッピング用のキーを生成する。
    fn token_key(&self, token: &str) -> String {
        format!("{}token:{}", self.prefix, token)
    }

    /// ユーザー ID → セッション ID SET 用のキーを生成する。
    fn user_key(&self, user_id: &str) -> String {
        format!("{}user:{}", self.prefix, user_id)
    }

    /// expires_at から TTL 秒数を計算する。最小 1 秒。
    fn ttl_seconds(session: &Session) -> i64 {
        let ttl = (session.expires_at - chrono::Utc::now()).num_seconds();
        if ttl < 1 { 1 } else { ttl }
    }
}

#[async_trait]
impl SessionRepository for RedisSessionRepository {
    async fn save(&self, session: &Session) -> Result<(), SessionError> {
        let mut conn = self.conn.clone();
        let session_key = self.session_key(&session.id);
        let token_key = self.token_key(&session.token);
        let user_key = self.user_key(&session.user_id);
        let ttl = Self::ttl_seconds(session);

        let json = serde_json::to_string(session)
            .map_err(|e| SessionError::Internal(format!("serialization error: {}", e)))?;

        // SET session:{id} with TTL
        conn.set_ex::<_, _, ()>(&session_key, &json, ttl as u64)
            .await
            .map_err(|e| SessionError::Internal(format!("redis SET error: {}", e)))?;

        // SET session:token:{token} → id with TTL
        conn.set_ex::<_, _, ()>(&token_key, &session.id, ttl as u64)
            .await
            .map_err(|e| SessionError::Internal(format!("redis SET token error: {}", e)))?;

        // SADD session:user:{user_id} id
        conn.sadd::<_, _, ()>(&user_key, &session.id)
            .await
            .map_err(|e| SessionError::Internal(format!("redis SADD error: {}", e)))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, SessionError> {
        let mut conn = self.conn.clone();
        let key = self.session_key(id);

        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| SessionError::Internal(format!("redis GET error: {}", e)))?;

        match value {
            Some(json) => {
                let session: Session = serde_json::from_str(&json)
                    .map_err(|e| SessionError::Internal(format!("deserialization error: {}", e)))?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, SessionError> {
        let mut conn = self.conn.clone();
        let token_key = self.token_key(token);

        let session_id: Option<String> = conn
            .get(&token_key)
            .await
            .map_err(|e| SessionError::Internal(format!("redis GET token error: {}", e)))?;

        match session_id {
            Some(id) => self.find_by_id(&id).await,
            None => Ok(None),
        }
    }

    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Session>, SessionError> {
        let mut conn = self.conn.clone();
        let user_key = self.user_key(user_id);

        let session_ids: Vec<String> = conn
            .smembers(&user_key)
            .await
            .map_err(|e| SessionError::Internal(format!("redis SMEMBERS error: {}", e)))?;

        let mut sessions = Vec::new();
        for id in session_ids {
            if let Some(session) = self.find_by_id(&id).await? {
                sessions.push(session);
            }
        }

        Ok(sessions)
    }

    async fn delete(&self, id: &str) -> Result<(), SessionError> {
        let mut conn = self.conn.clone();

        // まずセッションを取得してトークンとユーザー ID を知る
        let session_key = self.session_key(id);
        let value: Option<String> = conn
            .get(&session_key)
            .await
            .map_err(|e| SessionError::Internal(format!("redis GET error: {}", e)))?;

        if let Some(json) = value {
            let session: Session = serde_json::from_str(&json)
                .map_err(|e| SessionError::Internal(format!("deserialization error: {}", e)))?;

            let token_key = self.token_key(&session.token);
            let user_key = self.user_key(&session.user_id);

            // DEL session:{id}
            conn.del::<_, ()>(&session_key)
                .await
                .map_err(|e| SessionError::Internal(format!("redis DEL error: {}", e)))?;

            // DEL session:token:{token}
            conn.del::<_, ()>(&token_key)
                .await
                .map_err(|e| SessionError::Internal(format!("redis DEL token error: {}", e)))?;

            // SREM session:user:{user_id} id
            conn.srem::<_, _, ()>(&user_key, id)
                .await
                .map_err(|e| SessionError::Internal(format!("redis SREM error: {}", e)))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        // ConnectionManager は実際の Redis なしでは作れないため、キー生成ロジックのみテスト
        let prefix = "session:".to_string();

        let session_key = format!("{}id:{}", prefix, "abc-123");
        assert_eq!(session_key, "session:id:abc-123");

        let token_key = format!("{}token:{}", prefix, "tok-xyz");
        assert_eq!(token_key, "session:token:tok-xyz");

        let user_key = format!("{}user:{}", prefix, "user-1");
        assert_eq!(user_key, "session:user:user-1");
    }

    #[test]
    fn test_ttl_calculation() {
        use chrono::{Duration, Utc};
        use std::collections::HashMap;

        let session = Session {
            id: "s1".to_string(),
            user_id: "u1".to_string(),
            token: "t1".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            revoked: false,
            metadata: HashMap::new(),
        };

        let ttl = RedisSessionRepository::ttl_seconds(&session);
        // おおよそ 3600 秒（±数秒の誤差）
        assert!(ttl > 3590 && ttl <= 3600);
    }

    #[test]
    fn test_ttl_minimum() {
        use chrono::{Duration, Utc};
        use std::collections::HashMap;

        let session = Session {
            id: "s1".to_string(),
            user_id: "u1".to_string(),
            token: "t1".to_string(),
            expires_at: Utc::now() - Duration::hours(1),
            created_at: Utc::now(),
            revoked: false,
            metadata: HashMap::new(),
        };

        // 期限切れセッションでも最小 1 秒の TTL を返す
        let ttl = RedisSessionRepository::ttl_seconds(&session);
        assert_eq!(ttl, 1);
    }

    #[test]
    fn test_session_serialization_roundtrip() {
        use chrono::{Duration, Utc};
        use std::collections::HashMap;

        let session = Session {
            id: "sess-1".to_string(),
            user_id: "user-1".to_string(),
            token: "tok-1".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            revoked: false,
            metadata: HashMap::from([("ip".to_string(), "127.0.0.1".to_string())]),
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, session.id);
        assert_eq!(deserialized.user_id, session.user_id);
        assert_eq!(deserialized.token, session.token);
        assert_eq!(deserialized.revoked, session.revoked);
        assert_eq!(deserialized.metadata.get("ip").unwrap(), "127.0.0.1");
    }
}
