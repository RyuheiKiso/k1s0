use std::collections::HashMap;

use k1s0_session_client::{
    CreateSessionRequest, InMemorySessionClient, RefreshSessionRequest, SessionClient, SessionError,
};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn create_req(user_id: &str, ttl: i64) -> CreateSessionRequest {
    CreateSessionRequest {
        user_id: user_id.to_string(),
        ttl_seconds: ttl,
        metadata: HashMap::new(),
    }
}

fn create_req_with_meta(
    user_id: &str,
    ttl: i64,
    meta: HashMap<String, String>,
) -> CreateSessionRequest {
    CreateSessionRequest {
        user_id: user_id.to_string(),
        ttl_seconds: ttl,
        metadata: meta,
    }
}

// ===========================================================================
// create
// ===========================================================================

// セッション作成が有効なセッションオブジェクトを返すことを確認する。
#[tokio::test]
async fn create_returns_valid_session() {
    let client = InMemorySessionClient::new();
    let session = client.create(create_req("user-1", 3600)).await.unwrap();

    assert_eq!(session.user_id, "user-1");
    assert!(!session.id.is_empty());
    assert!(!session.token.is_empty());
    assert!(!session.revoked);
    assert!(session.expires_at > session.created_at);
}

// 複数回のセッション作成でそれぞれ一意な ID とトークンが生成されることを確認する。
#[tokio::test]
async fn create_generates_unique_ids() {
    let client = InMemorySessionClient::new();
    let s1 = client.create(create_req("user-1", 3600)).await.unwrap();
    let s2 = client.create(create_req("user-1", 3600)).await.unwrap();
    assert_ne!(s1.id, s2.id);
    assert_ne!(s1.token, s2.token);
}

// メタデータ付きでセッションを作成した場合にメタデータが保持されることを確認する。
#[tokio::test]
async fn create_with_metadata() {
    let client = InMemorySessionClient::new();
    let mut meta = HashMap::new();
    meta.insert("device".to_string(), "mobile".to_string());
    meta.insert("ip".to_string(), "1.2.3.4".to_string());

    let session = client
        .create(create_req_with_meta("user-1", 3600, meta.clone()))
        .await
        .unwrap();

    assert_eq!(session.metadata.get("device").unwrap(), "mobile");
    assert_eq!(session.metadata.get("ip").unwrap(), "1.2.3.4");
}

// ===========================================================================
// get
// ===========================================================================

// 作成済みセッションを ID で取得できることを確認する。
#[tokio::test]
async fn get_existing_session() {
    let client = InMemorySessionClient::new();
    let created = client.create(create_req("user-1", 3600)).await.unwrap();

    let fetched = client.get(&created.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.user_id, "user-1");
    assert_eq!(fetched.token, created.token);
}

// 存在しないセッション ID の取得が None を返すことを確認する。
#[tokio::test]
async fn get_nonexistent_returns_none() {
    let client = InMemorySessionClient::new();
    let result = client.get("no-such-id").await.unwrap();
    assert!(result.is_none());
}

// ===========================================================================
// refresh
// ===========================================================================

// セッションのリフレッシュで有効期限が延長されることを確認する。
#[tokio::test]
async fn refresh_extends_expiry() {
    let client = InMemorySessionClient::new();
    let session = client.create(create_req("user-1", 100)).await.unwrap();

    let refreshed = client
        .refresh(RefreshSessionRequest {
            id: session.id.clone(),
            ttl_seconds: 7200,
        })
        .await
        .unwrap();

    assert!(refreshed.expires_at > session.expires_at);
    assert_eq!(refreshed.id, session.id);
}

// 存在しないセッションのリフレッシュが NotFound エラーを返すことを確認する。
#[tokio::test]
async fn refresh_nonexistent_returns_not_found() {
    let client = InMemorySessionClient::new();
    let result = client
        .refresh(RefreshSessionRequest {
            id: "ghost".to_string(),
            ttl_seconds: 3600,
        })
        .await;
    assert!(matches!(result, Err(SessionError::NotFound(_))));
}

// リフレッシュ後に get で取得した場合も新しい有効期限が反映されていることを確認する。
#[tokio::test]
async fn refresh_persists_new_expiry() {
    let client = InMemorySessionClient::new();
    let session = client.create(create_req("user-1", 100)).await.unwrap();

    client
        .refresh(RefreshSessionRequest {
            id: session.id.clone(),
            ttl_seconds: 9999,
        })
        .await
        .unwrap();

    let fetched = client.get(&session.id).await.unwrap().unwrap();
    assert!(fetched.expires_at > session.expires_at);
}

// ===========================================================================
// revoke
// ===========================================================================

// セッションの失効操作で revoked フラグが true になることを確認する。
#[tokio::test]
async fn revoke_marks_session_revoked() {
    let client = InMemorySessionClient::new();
    let session = client.create(create_req("user-1", 3600)).await.unwrap();
    assert!(!session.revoked);

    client.revoke(&session.id).await.unwrap();

    let fetched = client.get(&session.id).await.unwrap().unwrap();
    assert!(fetched.revoked);
}

// 存在しないセッションの失効操作が NotFound エラーを返すことを確認する。
#[tokio::test]
async fn revoke_nonexistent_returns_not_found() {
    let client = InMemorySessionClient::new();
    let result = client.revoke("missing").await;
    assert!(matches!(result, Err(SessionError::NotFound(_))));
}

// ===========================================================================
// list_user_sessions
// ===========================================================================

// list_user_sessions が指定ユーザーのセッションのみを返すことを確認する。
#[tokio::test]
async fn list_user_sessions_filters_by_user() {
    let client = InMemorySessionClient::new();
    client.create(create_req("user-1", 3600)).await.unwrap();
    client.create(create_req("user-1", 3600)).await.unwrap();
    client.create(create_req("user-2", 3600)).await.unwrap();

    let sessions = client.list_user_sessions("user-1").await.unwrap();
    assert_eq!(sessions.len(), 2);
    assert!(sessions.iter().all(|s| s.user_id == "user-1"));
}

// 存在しないユーザーのセッション一覧が空リストを返すことを確認する。
#[tokio::test]
async fn list_user_sessions_empty_for_unknown_user() {
    let client = InMemorySessionClient::new();
    let sessions = client.list_user_sessions("nobody").await.unwrap();
    assert!(sessions.is_empty());
}

// ===========================================================================
// revoke_all
// ===========================================================================

// revoke_all が指定ユーザーのすべてのセッションを失効させることを確認する。
#[tokio::test]
async fn revoke_all_revokes_all_user_sessions() {
    let client = InMemorySessionClient::new();
    client.create(create_req("user-1", 3600)).await.unwrap();
    client.create(create_req("user-1", 3600)).await.unwrap();
    client.create(create_req("user-2", 3600)).await.unwrap();

    let count = client.revoke_all("user-1").await.unwrap();
    assert_eq!(count, 2);

    let sessions = client.list_user_sessions("user-1").await.unwrap();
    assert!(sessions.iter().all(|s| s.revoked));

    // user-2 should be unaffected
    let user2 = client.list_user_sessions("user-2").await.unwrap();
    assert!(user2.iter().all(|s| !s.revoked));
}

// 存在しないユーザーへの revoke_all が 0 を返すことを確認する。
#[tokio::test]
async fn revoke_all_returns_zero_for_unknown_user() {
    let client = InMemorySessionClient::new();
    let count = client.revoke_all("nobody").await.unwrap();
    assert_eq!(count, 0);
}

// revoke_all が既に失効済みのセッションをスキップして未失効分のみカウントすることを確認する。
#[tokio::test]
async fn revoke_all_skips_already_revoked() {
    let client = InMemorySessionClient::new();
    let s1 = client.create(create_req("user-1", 3600)).await.unwrap();
    client.create(create_req("user-1", 3600)).await.unwrap();

    // Revoke one first
    client.revoke(&s1.id).await.unwrap();

    // revoke_all should only count the one that was not yet revoked
    let count = client.revoke_all("user-1").await.unwrap();
    assert_eq!(count, 1);
}

// ===========================================================================
// expiry handling
// ===========================================================================

// TTL の差に応じてセッションの有効期限が異なることを確認する。
#[tokio::test]
async fn session_expiry_is_based_on_ttl() {
    let client = InMemorySessionClient::new();
    let short = client.create(create_req("user-1", 10)).await.unwrap();
    let long = client.create(create_req("user-1", 86400)).await.unwrap();

    assert!(long.expires_at > short.expires_at);
}

// セッションの created_at が expires_at より前であることを確認する。
#[tokio::test]
async fn session_created_at_is_before_expires_at() {
    let client = InMemorySessionClient::new();
    let session = client.create(create_req("user-1", 3600)).await.unwrap();
    assert!(session.created_at < session.expires_at);
}

// ===========================================================================
// error variant coverage
// ===========================================================================

// NotFound エラーのメッセージにセッション ID が含まれることを確認する。
#[test]
fn error_display_not_found() {
    let e = SessionError::NotFound("sess-123".to_string());
    assert!(format!("{e}").contains("sess-123"));
}

// Expired エラーの表示文字列が空でないことを確認する。
#[test]
fn error_display_expired() {
    let e = SessionError::Expired;
    assert!(!format!("{e}").is_empty());
}

// Revoked エラーの表示文字列が空でないことを確認する。
#[test]
fn error_display_revoked() {
    let e = SessionError::Revoked;
    assert!(!format!("{e}").is_empty());
}

// Connection エラーのメッセージに原因文字列が含まれることを確認する。
#[test]
fn error_display_connection() {
    let e = SessionError::Connection("refused".to_string());
    assert!(format!("{e}").contains("refused"));
}

// Internal エラーのメッセージに原因文字列が含まれることを確認する。
#[test]
fn error_display_internal() {
    let e = SessionError::Internal("panic".to_string());
    assert!(format!("{e}").contains("panic"));
}

// ===========================================================================
// Default trait
// ===========================================================================

// Default トレイトで生成したクライアントが空のセッションストアを持つことを確認する。
#[tokio::test]
async fn default_creates_empty_client() {
    let client = InMemorySessionClient::default();
    let sessions = client.list_user_sessions("any").await.unwrap();
    assert!(sessions.is_empty());
}
