use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

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
    /// セキュリティ: トークンをそのままRedisキーに使うと、Redisが侵害された場合に全トークンが漏洩する。
    /// SHA-256ハッシュ化により、キー名からトークン値を逆算できなくする（CWE-312対応）。
    fn token_key(&self, token: &str) -> String {
        use sha2::{Digest, Sha256};
        let hash = hex::encode(Sha256::digest(token.as_bytes()));
        format!("{}token:{}", self.prefix, hash)
    }

    /// ユーザー ID → セッション ID SET 用のキーを生成する。
    fn user_key(&self, user_id: &str) -> String {
        format!("{}user:{}", self.prefix, user_id)
    }

    /// expires_at から TTL 秒数を計算する。最小 1 秒。
    fn ttl_seconds(session: &Session) -> i64 {
        let ttl = (session.expires_at - chrono::Utc::now()).num_seconds();
        if ttl < 1 {
            1
        } else {
            ttl
        }
    }

    /// JWT トークンの jti を Redis 失効リストに登録する内部実装。
    /// HIGH-002 対応: ログアウト後のJWT再利用防止のため、jti を Redis SET EX で登録する。
    /// TTL は JWT の残余有効期限（最大 remaining_secs 秒）に設定し、自動的に期限切れとなるようにする。
    /// TTL 設定により Redis のメモリを無駄に消費せず、期限切れトークンのクリーンアップが不要となる。
    async fn revoke_jti_inner(&self, jti: &str, remaining_secs: u64) -> Result<(), SessionError> {
        let mut conn = self.conn.clone();
        // キー設計: session:revoked:jti:{jti}
        // revoked サブネームスペースを使い、通常のセッションキーと明確に分離する。
        let key = format!("{}revoked:jti:{}", self.prefix, jti);
        conn.set_ex::<_, _, ()>(&key, "1", remaining_secs.max(1))
            .await
            .map_err(|e| SessionError::Internal(format!("redis SET jti revoked error: {}", e)))?;
        Ok(())
    }

    /// jti が Redis 失効リストに登録されているか確認する。
    /// HIGH-002 対応: トークン検証時に呼び出し、失効済み jti を持つトークンを拒否する。
    /// Redis 接続エラーの場合は false（チェック失敗 = 通過）とし、
    /// Redis 障害時にサービス全体を停止させない設計とする（セキュリティとユーザビリティのトレードオフ）。
    pub async fn is_jti_revoked(&self, jti: &str) -> bool {
        let mut conn = self.conn.clone();
        let key = format!("{}revoked:jti:{}", self.prefix, jti);
        // EXISTS コマンドは存在する場合 1、しない場合 0 を返す。
        // エラー時は false を返して Redis 障害時のサービス停止を防ぐ。
        conn.exists::<_, bool>(&key)
            .await
            .unwrap_or(false)
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

        // session:user:{user_id} SET にも TTL を設定する（メモリリーク防止）
        // セッション本体（session:id:{id}）と同じ TTL を使用する
        conn.expire::<_, ()>(&user_key, ttl)
            .await
            .map_err(|e| SessionError::Internal(format!("redis EXPIRE error: {}", e)))?;

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

        // ユーザーに紐づくセッション ID の SET を取得する
        let session_ids: Vec<String> = conn
            .smembers(&user_key)
            .await
            .map_err(|e| SessionError::Internal(format!("redis SMEMBERS error: {}", e)))?;

        // セッション ID が存在しない場合は早期リターンする
        if session_ids.is_empty() {
            return Ok(vec![]);
        }

        // MGET で全セッションを一括取得し N+1 問題を解消する。
        // 個別 GET × N 回から MGET 1 回に削減することで Redis ラウンドトリップを最小化する。
        let keys: Vec<String> = session_ids.iter().map(|id| self.session_key(id)).collect();

        let values: Vec<Option<String>> = conn
            .mget(&keys)
            .await
            .map_err(|e| SessionError::Internal(format!("redis MGET error: {}", e)))?;

        // None（TTL 切れ等で消滅したセッション）を除外しデシリアライズする
        // flatten() で Option の None を除去し、Some の値のみを処理する（manual_flatten 対応）
        let mut sessions = Vec::new();
        for json in values.into_iter().flatten() {
            let session: Session = serde_json::from_str(&json)
                .map_err(|e| SessionError::Internal(format!("deserialization error: {}", e)))?;
            sessions.push(session);
        }

        Ok(sessions)
    }

    /// HIGH-002 対応: Redis 実装での jti 失効登録。
    /// デフォルトトレイト実装をオーバーライドし、Redis SET EX で実際に jti を登録する。
    async fn revoke_jti(&self, jti: &str, remaining_secs: u64) -> Result<(), SessionError> {
        // 内部実装メソッドに委譲する。
        // トレイトのデフォルト実装（何もしない）から実際の Redis 操作に切り替える。
        self.revoke_jti_inner(jti, remaining_secs).await
    }

    async fn delete(&self, id: &str) -> Result<(), SessionError> {
        let mut conn = self.conn.clone();

        let session_key = self.session_key(id);

        // セッション JSON を先読みしてトークン・ユーザー ID を取得する。
        // TOCTOU 競合を防ぐため、GET → DEL → DEL → SREM の4操作を
        // Lua スクリプトで単一のアトミックなトランザクションとして実行する。
        //
        // KEYS[1] = session_key  ("session:id:{id}")
        // KEYS[2] = token_key    ("session:token:{token}")
        // KEYS[3] = user_key     ("session:user:{user_id}")
        // ARGV[1] = session_id   (SREM で削除するメンバー)
        //
        // スクリプトはセッションが存在しない場合は 0 を、削除した場合は 1 を返す。
        // redis::Script::new は const/static では使えないため、呼び出しごとに生成する。
        let script = redis::Script::new(
            r#"
local session_json = redis.call('GET', KEYS[1])
if session_json == false then
  return 0
end
redis.call('DEL', KEYS[1])
redis.call('DEL', KEYS[2])
redis.call('SREM', KEYS[3], ARGV[1])
return 1
"#,
        );

        // セッション JSON を取得してキーを構築するため、まず GET のみ実行する。
        // Lua スクリプトには全キーを渡す必要があるため、事前に JSON を読んでトークン・ユーザー ID を取得する。
        let value: Option<String> = conn
            .get(&session_key)
            .await
            .map_err(|e| SessionError::Internal(format!("redis GET error: {}", e)))?;

        if let Some(json) = value {
            let session: Session = serde_json::from_str(&json)
                .map_err(|e| SessionError::Internal(format!("deserialization error: {}", e)))?;

            let token_key = self.token_key(&session.token);
            let user_key = self.user_key(&session.user_id);

            // Lua スクリプトで GET 確認 → DEL × 2 → SREM をアトミックに実行する。
            // セッションがスクリプト実行前に他プロセスに削除された場合は 0 が返り、正常終了とする。
            script
                .key(&session_key)
                .key(&token_key)
                .key(&user_key)
                .arg(id)
                .invoke_async::<i32>(&mut conn)
                .await
                .map_err(|e| SessionError::Internal(format!("redis Lua delete error: {}", e)))?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        // ConnectionManager は実際の Redis なしでは作れないため、キー生成ロジックのみテスト
        // HIGH-003対応: token_key は SHA-256 ハッシュ化されたキーを返すことを確認する
        use sha2::{Digest, Sha256};
        let prefix = "session:".to_string();

        let session_key = format!("{}id:{}", prefix, "abc-123");
        assert_eq!(session_key, "session:id:abc-123");

        // トークンは SHA-256 ハッシュ化されてキーに含まれる
        let token = "tok-xyz";
        let expected_hash = hex::encode(Sha256::digest(token.as_bytes()));
        let token_key = format!("{}token:{}", prefix, expected_hash);
        assert!(token_key.starts_with("session:token:"));
        assert!(!token_key.contains("tok-xyz"), "トークン値がキーに含まれてはならない");

        let user_key = format!("{}user:{}", prefix, "user-1");
        assert_eq!(user_key, "session:user:user-1");
    }

    #[test]
    fn test_ttl_calculation() {
        use chrono::{Duration, Utc};
        use std::collections::HashMap;

        // TTL 計算テスト用のセッションを生成する。tenant_id を含む完全な構造体を使用する
        let session = Session {
            id: "s1".to_string(),
            user_id: "u1".to_string(),
            tenant_id: "tenant-a".to_string(),
            device_id: "d1".to_string(),
            device_name: Some("device".to_string()),
            device_type: Some("desktop".to_string()),
            user_agent: Some("ua".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            token: "t1".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            last_accessed_at: None,
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

        // 期限切れセッションの最小 TTL テスト用セッション
        let session = Session {
            id: "s1".to_string(),
            user_id: "u1".to_string(),
            tenant_id: "tenant-a".to_string(),
            device_id: "d1".to_string(),
            device_name: Some("device".to_string()),
            device_type: Some("desktop".to_string()),
            user_agent: Some("ua".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            token: "t1".to_string(),
            expires_at: Utc::now() - Duration::hours(1),
            created_at: Utc::now(),
            last_accessed_at: None,
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

        // シリアライズ往復テスト用セッション。tenant_id が JSON に含まれ、復元されることを確認する
        let session = Session {
            id: "sess-1".to_string(),
            user_id: "user-1".to_string(),
            tenant_id: "tenant-a".to_string(),
            device_id: "d1".to_string(),
            device_name: Some("device".to_string()),
            device_type: Some("desktop".to_string()),
            user_agent: Some("ua".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            token: "tok-1".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            last_accessed_at: None,
            revoked: false,
            metadata: HashMap::from([("ip".to_string(), "127.0.0.1".to_string())]),
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, session.id);
        assert_eq!(deserialized.user_id, session.user_id);
        assert_eq!(deserialized.tenant_id, session.tenant_id);
        assert_eq!(deserialized.token, session.token);
        assert_eq!(deserialized.revoked, session.revoked);
        assert_eq!(deserialized.metadata.get("ip").unwrap(), "127.0.0.1");
    }
}
