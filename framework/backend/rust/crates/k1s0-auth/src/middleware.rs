//! 認証ミドルウェア
//!
//! HTTP/gRPCリクエストの認証処理を統一

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
