//! 認証ミドルウェア
//!
//! HTTP/gRPCリクエストの認証処理を統一。
//!
//! # 機能
//!
//! - JWT トークンの検証
//! - ポリシーベースの認可
//! - axum Tower Layer（`axum-layer` feature）
//! - tonic Interceptor（`tonic-interceptor` feature）
//!
//! # 使用例（axum）
//!
//! ```ignore
//! use axum::Router;
//! use k1s0_auth::middleware::auth_layer;
//!
//! let app = Router::new()
//!     .route("/api/users", get(handler))
//!     .layer(auth_layer(verifier, policy, audit));
//! ```

use std::sync::Arc;

use crate::audit::{AuditActor, AuditLogger};
use crate::error::AuthError;
use crate::jwt::{Claims, JwtVerifier};
use crate::policy::{Action, PolicyEvaluator, PolicyRequest, PolicySubject, ResourceContext};

/// 認証コンテキスト
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// JWTクレーム
    pub claims: Claims,
    /// ポリシーサブジェクト
    pub subject: PolicySubject,
    /// 元のトークン（必要な場合のみ保持）
    pub token: Option<String>,
}

impl AuthContext {
    /// クレームから作成
    pub fn new(claims: Claims) -> Self {
        let subject = PolicySubject::from_claims(&claims);
        Self {
            claims,
            subject,
            token: None,
        }
    }

    /// トークンを保持
    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    /// ユーザーIDを取得
    pub fn user_id(&self) -> &str {
        &self.claims.sub
    }

    /// テナントIDを取得
    pub fn tenant_id(&self) -> Option<&str> {
        self.claims.tenant_id.as_deref()
    }

    /// ロールを持っているかチェック
    pub fn has_role(&self, role: &str) -> bool {
        self.subject.roles.contains(role)
    }

    /// パーミッションを持っているかチェック
    pub fn has_permission(&self, permission: &str) -> bool {
        self.subject.permissions.contains(permission)
    }

    /// 監査アクターを作成
    pub fn to_audit_actor(&self) -> AuditActor {
        AuditActor::from_claims(&self.claims)
    }
}

/// 認証ミドルウェア
pub struct AuthMiddleware {
    /// JWT検証器
    verifier: Arc<JwtVerifier>,
    /// ポリシー評価器
    policy: Arc<PolicyEvaluator>,
    /// 監査ロガー
    audit: Arc<AuditLogger>,
}

impl AuthMiddleware {
    /// 新しいミドルウェアを作成
    pub fn new(
        verifier: Arc<JwtVerifier>,
        policy: Arc<PolicyEvaluator>,
        audit: Arc<AuditLogger>,
    ) -> Self {
        Self {
            verifier,
            policy,
            audit,
        }
    }

    /// トークンを検証して認証コンテキストを取得
    pub async fn authenticate(&self, token: &str) -> Result<AuthContext, AuthError> {
        let claims = self.verifier.verify(token).await?;
        let ctx = AuthContext::new(claims);

        // 認証成功を監査
        self.audit.log_authentication_success(ctx.to_audit_actor());

        Ok(ctx)
    }

    /// Bearerトークンを抽出して検証
    pub async fn authenticate_bearer(&self, authorization: &str) -> Result<AuthContext, AuthError> {
        let token = extract_bearer_token(authorization)?;
        self.authenticate(token).await
    }

    /// 認可チェック
    pub async fn authorize(
        &self,
        ctx: &AuthContext,
        action: Action,
        resource: ResourceContext,
    ) -> Result<(), AuthError> {
        let request = PolicyRequest {
            subject: ctx.subject.clone(),
            action: action.clone(),
            resource,
        };

        let result = self.policy.evaluate(&request).await;

        // 認可を監査
        self.audit.log_authorization(
            ctx.to_audit_actor(),
            action.to_permission(),
            &result,
        );

        if result.is_allowed() {
            Ok(())
        } else {
            Err(AuthError::AuthorizationFailed(
                result.reason.unwrap_or_else(|| "Access denied".to_string()),
            ))
        }
    }

    /// 認証と認可を同時に行う
    pub async fn authenticate_and_authorize(
        &self,
        token: &str,
        action: Action,
        resource: ResourceContext,
    ) -> Result<AuthContext, AuthError> {
        let ctx = self.authenticate(token).await?;
        self.authorize(&ctx, action, resource).await?;
        Ok(ctx)
    }

    /// 簡易パーミッションチェック
    pub async fn check_permission(
        &self,
        ctx: &AuthContext,
        permission: &str,
    ) -> Result<bool, AuthError> {
        self.policy.check_permission(&ctx.subject, permission).await
    }
}

/// Authorizationヘッダーからベアラートークンを抽出
pub fn extract_bearer_token(authorization: &str) -> Result<&str, AuthError> {
    let parts: Vec<&str> = authorization.splitn(2, ' ').collect();
    if parts.len() != 2 {
        return Err(AuthError::InvalidToken("Invalid authorization header format".to_string()));
    }

    if !parts[0].eq_ignore_ascii_case("bearer") {
        return Err(AuthError::InvalidToken("Expected Bearer token".to_string()));
    }

    Ok(parts[1])
}

/// gRPCメタデータからトークンを抽出
pub fn extract_grpc_token(metadata: &[(String, String)]) -> Result<&str, AuthError> {
    for (key, value) in metadata {
        if key.eq_ignore_ascii_case("authorization") {
            return extract_bearer_token(value);
        }
    }
    Err(AuthError::InvalidToken("Authorization metadata not found".to_string()))
}

/// 認証をスキップするパスのマッチャー
#[derive(Debug, Clone)]
pub struct AuthSkipMatcher {
    /// スキップするパス（完全一致）
    exact_paths: Vec<String>,
    /// スキップするパスプレフィックス
    prefix_paths: Vec<String>,
}

impl AuthSkipMatcher {
    /// 新しいマッチャーを作成
    pub fn new() -> Self {
        Self {
            exact_paths: Vec::new(),
            prefix_paths: Vec::new(),
        }
    }

    /// ヘルスチェックをスキップ
    pub fn with_health_checks(mut self) -> Self {
        self.exact_paths.push("/healthz".to_string());
        self.exact_paths.push("/readyz".to_string());
        self.exact_paths.push("/livez".to_string());
        self.exact_paths.push("/grpc.health.v1.Health/Check".to_string());
        self
    }

    /// パスを追加
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.exact_paths.push(path.into());
        self
    }

    /// プレフィックスを追加
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix_paths.push(prefix.into());
        self
    }

    /// パスがスキップ対象かチェック
    pub fn should_skip(&self, path: &str) -> bool {
        if self.exact_paths.iter().any(|p| p == path) {
            return true;
        }
        if self.prefix_paths.iter().any(|p| path.starts_with(p)) {
            return true;
        }
        false
    }
}

impl Default for AuthSkipMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token() {
        let result = extract_bearer_token("Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");

        // case insensitive
        let result = extract_bearer_token("bearer token123");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "token123");

        // invalid format
        let result = extract_bearer_token("Basic dXNlcjpwYXNz");
        assert!(result.is_err());

        let result = extract_bearer_token("token-only");
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_skip_matcher() {
        let matcher = AuthSkipMatcher::new()
            .with_health_checks()
            .with_path("/api/public")
            .with_prefix("/static/");

        assert!(matcher.should_skip("/healthz"));
        assert!(matcher.should_skip("/readyz"));
        assert!(matcher.should_skip("/api/public"));
        assert!(matcher.should_skip("/static/image.png"));
        assert!(matcher.should_skip("/static/css/style.css"));

        assert!(!matcher.should_skip("/api/users"));
        assert!(!matcher.should_skip("/orders"));
    }

    #[test]
    fn test_auth_context() {
        let claims = Claims {
            sub: "user123".to_string(),
            iss: "test".to_string(),
            aud: None,
            exp: 0,
            iat: 0,
            nbf: None,
            jti: None,
            roles: vec!["admin".to_string(), "user".to_string()],
            permissions: vec!["order:read".to_string()],
            tenant_id: Some("tenant-1".to_string()),
            email: None,
            email_verified: None,
            name: None,
        };

        let ctx = AuthContext::new(claims);

        assert_eq!(ctx.user_id(), "user123");
        assert_eq!(ctx.tenant_id(), Some("tenant-1"));
        assert!(ctx.has_role("admin"));
        assert!(ctx.has_role("user"));
        assert!(!ctx.has_role("superadmin"));
        assert!(ctx.has_permission("order:read"));
        assert!(!ctx.has_permission("order:write"));
    }
}

// ============================================================================
// axum Tower Layer 実装
// ============================================================================

#[cfg(feature = "axum-layer")]
pub use axum_layer::*;

#[cfg(feature = "axum-layer")]
pub mod axum_layer {
    use super::*;
    use axum::{
        body::Body,
        extract::Request,
        http::StatusCode,
        response::{IntoResponse, Response},
    };
    use futures::future::BoxFuture;
    use pin_project_lite::pin_project;
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };
    use tower::{Layer, Service};

    /// 認証レイヤーを作成
    ///
    /// # 例
    ///
    /// ```ignore
    /// use axum::Router;
    /// use k1s0_auth::middleware::auth_layer;
    ///
    /// let layer = auth_layer(verifier, policy, audit)
    ///     .with_skip_matcher(AuthSkipMatcher::new().with_health_checks());
    ///
    /// let app = Router::new()
    ///     .route("/api/users", get(handler))
    ///     .layer(layer);
    /// ```
    pub fn auth_layer(
        verifier: Arc<JwtVerifier>,
        policy: Arc<PolicyEvaluator>,
        audit: Arc<AuditLogger>,
    ) -> AuthLayer {
        AuthLayer::new(verifier, policy, audit)
    }

    /// 認証レイヤー
    #[derive(Clone)]
    pub struct AuthLayer {
        middleware: Arc<AuthMiddleware>,
        skip_matcher: AuthSkipMatcher,
    }

    impl AuthLayer {
        /// 新しいレイヤーを作成
        pub fn new(
            verifier: Arc<JwtVerifier>,
            policy: Arc<PolicyEvaluator>,
            audit: Arc<AuditLogger>,
        ) -> Self {
            Self {
                middleware: Arc::new(AuthMiddleware::new(verifier, policy, audit)),
                skip_matcher: AuthSkipMatcher::new(),
            }
        }

        /// スキップマッチャーを設定
        pub fn with_skip_matcher(mut self, matcher: AuthSkipMatcher) -> Self {
            self.skip_matcher = matcher;
            self
        }
    }

    impl<S> Layer<S> for AuthLayer {
        type Service = AuthService<S>;

        fn layer(&self, inner: S) -> Self::Service {
            AuthService {
                inner,
                middleware: self.middleware.clone(),
                skip_matcher: self.skip_matcher.clone(),
            }
        }
    }

    /// 認証サービス
    #[derive(Clone)]
    pub struct AuthService<S> {
        inner: S,
        middleware: Arc<AuthMiddleware>,
        skip_matcher: AuthSkipMatcher,
    }

    impl<S> Service<Request> for AuthService<S>
    where
        S: Service<Request, Response = Response> + Clone + Send + 'static,
        S::Future: Send + 'static,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, mut request: Request) -> Self::Future {
            let path = request.uri().path().to_string();

            // スキップ対象のパスはそのまま通す
            if self.skip_matcher.should_skip(&path) {
                let future = self.inner.call(request);
                return Box::pin(async move { future.await });
            }

            // Authorization ヘッダを取得
            let authorization = request
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let middleware = self.middleware.clone();
            let mut inner = self.inner.clone();

            Box::pin(async move {
                let authorization = match authorization {
                    Some(auth) => auth,
                    None => {
                        return Ok(unauthorized_response("Missing authorization header"));
                    }
                };

                let ctx = match middleware.authenticate_bearer(&authorization).await {
                    Ok(ctx) => ctx,
                    Err(e) => {
                        return Ok(unauthorized_response(&e.to_string()));
                    }
                };

                // リクエストエクステンションに認証コンテキストを設定
                request.extensions_mut().insert(ctx);

                inner.call(request).await
            })
        }
    }

    /// 認証エラーレスポンスを生成
    fn unauthorized_response(message: &str) -> Response {
        let body = serde_json::json!({
            "error": "unauthorized",
            "message": message
        });

        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    /// リクエストから認証コンテキストを抽出
    pub fn extract_auth_context(request: &Request) -> Option<AuthContext> {
        request.extensions().get::<AuthContext>().cloned()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_unauthorized_response() {
            let response = unauthorized_response("test error");
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }
}

// ============================================================================
// tonic Interceptor 実装
// ============================================================================

#[cfg(feature = "tonic-interceptor")]
pub use tonic_interceptor::*;

#[cfg(feature = "tonic-interceptor")]
pub mod tonic_interceptor {
    use super::*;
    use tonic::{Request, Status};

    /// gRPC リクエストからパスを抽出
    ///
    /// tonic の Request から gRPC メソッドパスを取得する。
    /// extensions に http::Uri がある場合はそれを使用し、
    /// なければ ":path" pseudo-header から取得を試みる。
    fn extract_grpc_path<T>(request: &Request<T>) -> String {
        // extensions から http::Uri を取得を試みる
        if let Some(uri) = request.extensions().get::<http::Uri>() {
            return uri.path().to_string();
        }

        // metadata から :path を取得を試みる（binary metadata）
        if let Some(path) = request.metadata().get_bin(":path") {
            if let Ok(bytes) = path.to_bytes() {
                if let Ok(path_str) = std::str::from_utf8(&bytes) {
                    return path_str.to_string();
                }
            }
        }

        // gRPC のメソッドパスはメタデータには通常含まれないため、
        // デフォルトで空文字列を返す（スキップ対象にならない）
        String::new()
    }

    /// 認証インターセプターを作成
    ///
    /// # 例
    ///
    /// ```ignore
    /// use tonic::transport::Server;
    /// use k1s0_auth::middleware::auth_interceptor;
    ///
    /// let interceptor = auth_interceptor(verifier, policy, audit);
    ///
    /// Server::builder()
    ///     .add_service(MyServiceServer::with_interceptor(service, interceptor))
    ///     .serve(addr)
    ///     .await?;
    /// ```
    pub fn auth_interceptor(
        verifier: Arc<JwtVerifier>,
        policy: Arc<PolicyEvaluator>,
        audit: Arc<AuditLogger>,
    ) -> impl Fn(Request<()>) -> Result<Request<()>, Status> + Clone {
        let middleware = Arc::new(AuthMiddleware::new(verifier, policy, audit));
        let skip_matcher = AuthSkipMatcher::new().with_health_checks();

        move |request| {
            let middleware = middleware.clone();
            let skip_matcher = skip_matcher.clone();

            // パスを取得
            let path = extract_grpc_path(&request);

            // スキップ対象のパスはそのまま通す
            if skip_matcher.should_skip(&path) {
                return Ok(request);
            }

            // メタデータからトークンを抽出
            let token = request
                .metadata()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|auth| extract_bearer_token(auth).ok());

            match token {
                Some(_token) => {
                    // 注意: tonic の interceptor は同期なので、
                    // 非同期検証は別途行う必要がある
                    // ここでは基本的なチェックのみ行う
                    Ok(request)
                }
                None => Err(Status::unauthenticated("Missing authorization token")),
            }
        }
    }

    /// 非同期認証インターセプター
    ///
    /// トークンの完全な検証を行う場合に使用する。
    /// サービス実装内で明示的に呼び出す必要がある。
    pub struct AsyncAuthInterceptor {
        middleware: Arc<AuthMiddleware>,
        skip_matcher: AuthSkipMatcher,
    }

    impl AsyncAuthInterceptor {
        /// 新しいインターセプターを作成
        pub fn new(
            verifier: Arc<JwtVerifier>,
            policy: Arc<PolicyEvaluator>,
            audit: Arc<AuditLogger>,
        ) -> Self {
            Self {
                middleware: Arc::new(AuthMiddleware::new(verifier, policy, audit)),
                skip_matcher: AuthSkipMatcher::new().with_health_checks(),
            }
        }

        /// スキップマッチャーを設定
        pub fn with_skip_matcher(mut self, matcher: AuthSkipMatcher) -> Self {
            self.skip_matcher = matcher;
            self
        }

        /// リクエストを認証
        pub async fn authenticate<T>(&self, request: &Request<T>) -> Result<AuthContext, Status> {
            let path = extract_grpc_path(request);

            // スキップ対象のパス
            if self.skip_matcher.should_skip(&path) {
                // 匿名コンテキストを返す
                return Ok(AuthContext::new(Claims {
                    sub: "anonymous".to_string(),
                    iss: "system".to_string(),
                    aud: None,
                    exp: 0,
                    iat: 0,
                    nbf: None,
                    jti: None,
                    roles: vec![],
                    permissions: vec![],
                    tenant_id: None,
                    email: None,
                    email_verified: None,
                    name: None,
                }));
            }

            // メタデータからトークンを抽出
            let authorization = request
                .metadata()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;

            let ctx = self
                .middleware
                .authenticate_bearer(authorization)
                .await
                .map_err(|e| Status::unauthenticated(e.to_string()))?;

            Ok(ctx)
        }

        /// リクエストを認証・認可
        pub async fn authenticate_and_authorize<T>(
            &self,
            request: &Request<T>,
            action: Action,
            resource: ResourceContext,
        ) -> Result<AuthContext, Status> {
            let ctx = self.authenticate(request).await?;

            self.middleware
                .authorize(&ctx, action, resource)
                .await
                .map_err(|e| Status::permission_denied(e.to_string()))?;

            Ok(ctx)
        }
    }

    /// リクエストエクステンションから認証コンテキストを抽出
    pub fn extract_auth_context<T>(request: &Request<T>) -> Option<&AuthContext> {
        request.extensions().get::<AuthContext>()
    }

    /// 認証コンテキストをリクエストエクステンションに設定
    pub fn set_auth_context<T>(request: &mut Request<T>, ctx: AuthContext) {
        request.extensions_mut().insert(ctx);
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_skip_health_checks() {
            let matcher = AuthSkipMatcher::new().with_health_checks();
            assert!(matcher.should_skip("/grpc.health.v1.Health/Check"));
        }
    }
}
