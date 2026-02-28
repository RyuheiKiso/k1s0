# system-server 実装ガイド

> **仕様**: 構造定義・ディレクトリ構成・設定テーブルは [implementation.md](./implementation.md) を参照。

---

## サービス固有 DI（main.rs）

共通起動シーケンスの後に、認証サーバー固有の依存注入を行う。

```rust
    // --- DI ---
    let jwks_verifier = Arc::new(
        JwksVerifier::new(
            cfg.auth.oidc.jwks_uri.clone(),
            cfg.auth.oidc.jwks_cache_ttl(),
        ),
    );
    let keycloak_client = Arc::new(KeycloakClient::new(&cfg.auth.oidc));
    let audit_repo = Arc::new(persistence::AuditLogRepositoryImpl::new(pool.clone()));
    let auth_domain_svc = Arc::new(AuthDomainService::new());

    let validate_token_uc = ValidateTokenUseCase::new(
        jwks_verifier.clone(), cfg.auth.jwt.clone(),
    );
    let get_user_uc = GetUserUseCase::new(keycloak_client.clone());
    let list_users_uc = ListUsersUseCase::new(keycloak_client.clone());
    let get_user_roles_uc = GetUserRolesUseCase::new(keycloak_client.clone());
    let check_permission_uc = CheckPermissionUseCase::new(auth_domain_svc.clone());
    let record_audit_log_uc = RecordAuditLogUseCase::new(
        audit_repo.clone(), Arc::new(producer),
    );
    let search_audit_log_uc = SearchAuditLogUseCase::new(audit_repo.clone());

    let app_state = AppState {
        validate_token_uc: Arc::new(validate_token_uc),
        get_user_uc: Arc::new(get_user_uc),
        list_users_uc: Arc::new(list_users_uc),
        get_user_roles_uc: Arc::new(get_user_roles_uc),
        record_audit_log_uc: Arc::new(record_audit_log_uc),
        search_audit_log_uc: Arc::new(search_audit_log_uc),
        keycloak_client: keycloak_client.clone(),
        pool: pool.clone(),
    };
```

---

## ユースケース実装

### ValidateTokenUseCase

issuer・audience の追加検証を含むトークン検証の実装例。

```rust
// src/usecase/validate_token.rs
use std::sync::Arc;

use tracing::instrument;

use crate::infrastructure::auth::JwksVerifier;
use crate::infrastructure::config::JwtConfig;

pub struct ValidateTokenUseCase {
    verifier: Arc<JwksVerifier>,
    jwt_config: JwtConfig,
}

impl ValidateTokenUseCase {
    pub fn new(verifier: Arc<JwksVerifier>, jwt_config: JwtConfig) -> Self {
        Self { verifier, jwt_config }
    }

    #[instrument(skip(self, token), fields(service = "auth-server"))]
    pub async fn execute(&self, token: &str) -> Result<TokenClaims, AuthError> {
        let claims = self.verifier.verify_token(token).await?;

        // issuer・audience の追加検証
        if claims.iss != self.jwt_config.issuer {
            return Err(AuthError::InvalidIssuer);
        }
        if claims.aud != self.jwt_config.audience {
            return Err(AuthError::InvalidAudience);
        }

        Ok(claims)
    }
}
```

---

## REST ハンドラー実装

ルーティング定義とハンドラー実装の全体像。各ハンドラーはユースケースを呼び出し、`ErrorResponse` でエラーを変換する。

```rust
// src/adapter/handler/rest_handler.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::adapter::handler::error::ErrorResponse;
use crate::adapter::middleware;

pub fn router(state: AppState) -> Router {
    Router::new()
        // ヘルスチェック
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        // 公開エンドポイント
        .route("/api/v1/auth/token/validate", post(validate_token))
        // 認可付きエンドポイント
        .route(
            "/api/v1/auth/token/introspect",
            post(introspect_token).layer(middleware::require_permission("read", "auth_config")),
        )
        .route(
            "/api/v1/users",
            get(list_users).layer(middleware::require_permission("read", "users")),
        )
        .route(
            "/api/v1/users/:id",
            get(get_user).layer(middleware::require_permission("read", "users")),
        )
        .route(
            "/api/v1/users/:id/roles",
            get(get_user_roles).layer(middleware::require_permission("read", "users")),
        )
        .route(
            "/api/v1/audit/logs",
            post(record_audit_log)
                .layer(middleware::require_permission("write", "audit_logs"))
                .get(search_audit_logs)
                .layer(middleware::require_permission("read", "audit_logs")),
        )
        .with_state(state)
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn readyz(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .is_ok();

    let kc_ok = state.keycloak_client.healthy().await.is_ok();

    if db_ok && kc_ok {
        Ok(Json(serde_json::json!({
            "status": "ready",
            "checks": {"database": "ok", "keycloak": "ok"}
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[derive(Deserialize)]
struct ValidateTokenRequest {
    token: String,
}

async fn validate_token(
    State(state): State<AppState>,
    Json(req): Json<ValidateTokenRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let claims = state
        .validate_token_uc
        .execute(&req.token)
        .await
        .map_err(|_| {
            ErrorResponse::unauthorized(
                "SYS_AUTH_TOKEN_INVALID",
                "トークンの検証に失敗しました",
            )
        })?;

    Ok(Json(serde_json::json!({
        "valid": true,
        "claims": claims,
    })))
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let user = state.get_user_uc.execute(&id).await.map_err(|_| {
        ErrorResponse::not_found(
            "SYS_AUTH_USER_NOT_FOUND",
            "指定されたユーザーが見つかりません",
        )
    })?;

    Ok(Json(serde_json::to_value(user).unwrap()))
}

// ... 他のハンドラーも同様のパターンで実装
```

---

## gRPC ハンドラー実装

tonic を使った gRPC サービス実装。各 RPC メソッドはユースケースに委譲する。

```rust
// src/adapter/handler/grpc_handler.rs
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub mod proto {
    tonic::include_proto!("k1s0.system.auth.v1");
}

use proto::auth_service_server::{AuthService, AuthServiceServer};
use proto::*;

pub struct AuthServiceImpl {
    validate_token_uc: Arc<ValidateTokenUseCase>,
    get_user_uc: Arc<GetUserUseCase>,
    list_users_uc: Arc<ListUsersUseCase>,
    get_user_roles_uc: Arc<GetUserRolesUseCase>,
    check_permission_uc: Arc<CheckPermissionUseCase>,
}

impl AuthServiceImpl {
    pub fn new(
        validate_token_uc: Arc<ValidateTokenUseCase>,
        get_user_uc: Arc<GetUserUseCase>,
        list_users_uc: Arc<ListUsersUseCase>,
        get_user_roles_uc: Arc<GetUserRolesUseCase>,
        check_permission_uc: Arc<CheckPermissionUseCase>,
    ) -> Self {
        Self {
            validate_token_uc,
            get_user_uc,
            list_users_uc,
            get_user_roles_uc,
            check_permission_uc,
        }
    }
}

pub fn auth_service_server(svc: AuthServiceImpl) -> AuthServiceServer<AuthServiceImpl> {
    AuthServiceServer::new(svc)
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        match self.validate_token_uc.execute(&req.token).await {
            Ok(claims) => Ok(Response::new(ValidateTokenResponse {
                valid: true,
                claims: Some(claims.into()),
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(ValidateTokenResponse {
                valid: false,
                claims: None,
                error_message: e.to_string(),
            })),
        }
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .get_user_uc
            .execute(&req.user_id)
            .await
            .map_err(|_| Status::not_found("user not found"))?;

        Ok(Response::new(GetUserResponse {
            user: Some(user.into()),
        }))
    }

    async fn check_permission(
        &self,
        request: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        let req = request.into_inner();

        let (allowed, reason) = self
            .check_permission_uc
            .execute(&req.user_id, &req.permission, &req.resource, &req.roles)
            .await;

        Ok(Response::new(CheckPermissionResponse {
            allowed,
            reason,
        }))
    }

    // ListUsers, GetUserRoles も同様のパターンで実装
}
```

---

## Keycloak クライアント実装

Admin API トークンのキャッシュ付き取得と、ユーザー情報取得の実装。

```rust
// src/adapter/gateway/keycloak_client.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::domain::entity::User;
use crate::infrastructure::config::OidcConfig;

pub struct KeycloakClient {
    base_url: String,
    realm: String,
    client_id: String,
    client_secret: String,
    http_client: reqwest::Client,
    admin_token: Arc<RwLock<Option<CachedToken>>>,
}

struct CachedToken {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl KeycloakClient {
    pub fn new(cfg: &OidcConfig) -> Self {
        Self {
            base_url: extract_base_url(&cfg.discovery_url),
            realm: "k1s0".to_string(),
            client_id: cfg.client_id.clone(),
            client_secret: cfg.client_secret.clone(),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            admin_token: Arc::new(RwLock::new(None)),
        }
    }

    #[instrument(skip(self), fields(service = "auth-server"))]
    pub async fn get_user(&self, user_id: &str) -> anyhow::Result<User> {
        let token = self.get_admin_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/{}",
            self.base_url, self.realm, user_id
        );

        let resp = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("user not found");
        }

        let user: User = resp.error_for_status()?.json().await?;
        Ok(user)
    }

    pub async fn healthy(&self) -> anyhow::Result<()> {
        let url = format!("{}/realms/{}", self.base_url, self.realm);
        let resp = self.http_client.get(&url).send().await?;
        resp.error_for_status()?;
        Ok(())
    }

    async fn get_admin_token(&self) -> anyhow::Result<String> {
        // Client Credentials Grant でトークンを取得（キャッシュ付き）
        let cache = self.admin_token.read().await;
        if let Some(ref cached) = *cache {
            if chrono::Utc::now() < cached.expires_at {
                return Ok(cached.token.clone());
            }
        }
        drop(cache);

        let mut cache = self.admin_token.write().await;
        // POST /realms/k1s0/protocol/openid-connect/token
        // ...（実装省略）
        Ok(cache.as_ref().unwrap().token.clone())
    }
}
```

---

## 設定読み込み実装

`Config::load()` で YAML ファイルを読み込み、`validate()` で必須フィールドを検証する。

```rust
// src/infrastructure/config/mod.rs（実装部分）
impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.app.name.is_empty() {
            anyhow::bail!("app.name is required");
        }
        if self.server.port == 0 {
            anyhow::bail!("server.port must be > 0");
        }
        if self.auth.jwt.issuer.is_empty() {
            anyhow::bail!("auth.jwt.issuer is required");
        }
        if self.auth.oidc.jwks_uri.is_empty() {
            anyhow::bail!("auth.oidc.jwks_uri is required");
        }
        Ok(())
    }
}
```

---

## テスト構成（auth-server）

### JWKS TTL キャッシュテスト

`regions/system/server/rust/auth/tests/jwks_test.rs` に wiremock を使用した JWKS 取得テストを実装する。

| テスト名 | 内容 |
|---------|------|
| `test_jwks_fetch_from_mock_endpoint` | モックエンドポイントから JWKS を取得できる |
| `test_jwks_fetch_failure_returns_error` | JWKS 取得失敗時にエラーを返す |
| `test_jwks_cache_invalidation` | `invalidate_cache()` でキャッシュを無効化し再フェッチする |
| `test_jwks_empty_keys_response` | 空キーレスポンスでエラーを返す |
| `test_jwks_ttl_zero_always_refetches` | TTL=0 の場合は毎回 JWKS エンドポイントへフェッチする |
| `test_jwks_long_ttl_reuses_cached_keys` | TTL が十分長い場合は 2 回目以降キャッシュを再利用する（フェッチ 1 回のみ） |
| `test_jwks_concurrent_requests_no_panic` | 並行リクエスト中にパニックせず全リクエストがエラーを返す |

TTL 動作の詳細:

- `cache_ttl = Duration::ZERO`: キャッシュは常に期限切れ → 毎回フェッチ
- `cache_ttl = Duration::from_secs(3600)`: キャッシュが有効 → 再フェッチなし
- `invalidate_cache()` 呼び出し後は次のリクエストで強制フェッチ
