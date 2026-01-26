//! 認証サービス

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::{
    AuthError, AuthToken, PermissionRepository, Role, RoleRepository, TokenClaims,
    TokenRepository, User, UserRepository, UserStatus,
};

/// 認証サービス
pub struct AuthService<U, R, P, T>
where
    U: UserRepository,
    R: RoleRepository,
    P: PermissionRepository,
    T: TokenRepository,
{
    user_repo: Arc<U>,
    role_repo: Arc<R>,
    permission_repo: Arc<P>,
    token_repo: Arc<T>,
    /// トークン発行者
    issuer: String,
    /// アクセストークン有効期間（秒）
    access_token_ttl: i64,
    /// リフレッシュトークン有効期間（秒）
    refresh_token_ttl: i64,
    /// JWT シークレット
    jwt_secret: String,
}

impl<U, R, P, T> AuthService<U, R, P, T>
where
    U: UserRepository,
    R: RoleRepository,
    P: PermissionRepository,
    T: TokenRepository,
{
    /// 新しいサービスを作成
    pub fn new(
        user_repo: Arc<U>,
        role_repo: Arc<R>,
        permission_repo: Arc<P>,
        token_repo: Arc<T>,
        issuer: impl Into<String>,
        jwt_secret: impl Into<String>,
    ) -> Self {
        Self {
            user_repo,
            role_repo,
            permission_repo,
            token_repo,
            issuer: issuer.into(),
            access_token_ttl: 3600,       // 1時間
            refresh_token_ttl: 604800,    // 7日
            jwt_secret: jwt_secret.into(),
        }
    }

    /// アクセストークン有効期間を設定
    pub fn with_access_token_ttl(mut self, ttl: i64) -> Self {
        self.access_token_ttl = ttl;
        self
    }

    /// リフレッシュトークン有効期間を設定
    pub fn with_refresh_token_ttl(mut self, ttl: i64) -> Self {
        self.refresh_token_ttl = ttl;
        self
    }

    /// 認証（ログイン）
    pub async fn authenticate(
        &self,
        login_id: &str,
        password: &str,
    ) -> Result<AuthToken, AuthError> {
        // ユーザーを検索
        let user = self
            .user_repo
            .get_by_login_id(login_id)
            .await?
            .ok_or_else(|| AuthError::authentication_failed("invalid credentials"))?;

        // ステータスチェック
        match user.status {
            UserStatus::Inactive => return Err(AuthError::account_inactive(user.user_id)),
            UserStatus::Locked => return Err(AuthError::account_locked(user.user_id)),
            UserStatus::Active => {}
        }

        // パスワード検証（簡易実装：本番ではbcrypt等を使用）
        if !self.verify_password(password, &user.password_hash) {
            return Err(AuthError::authentication_failed("invalid credentials"));
        }

        // 最終ログイン更新
        self.user_repo.update_last_login(user.user_id).await?;

        // トークン生成
        self.generate_tokens(&user).await
    }

    /// トークンリフレッシュ
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken, AuthError> {
        // リフレッシュトークンを検証
        let user_id = self
            .token_repo
            .validate_refresh_token(refresh_token)
            .await?
            .ok_or_else(|| AuthError::invalid_token("invalid or expired refresh token"))?;

        // ユーザーを取得
        let user = self
            .user_repo
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| AuthError::user_not_found(user_id))?;

        // 古いトークンを無効化
        self.token_repo.revoke_refresh_token(refresh_token).await?;

        // 新しいトークンを生成
        self.generate_tokens(&user).await
    }

    /// パーミッションチェック
    pub async fn check_permission(
        &self,
        user_id: i64,
        permission_key: &str,
        service_name: Option<&str>,
    ) -> Result<bool, AuthError> {
        // ユーザーの存在確認
        let user = self
            .user_repo
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| AuthError::user_not_found(user_id))?;

        // アクティブでなければ拒否
        if !user.status.is_active() {
            return Ok(false);
        }

        // パーミッションをチェック
        self.permission_repo
            .check_permission(user_id, permission_key, service_name)
            .await
    }

    /// ユーザー情報を取得
    pub async fn get_user(&self, user_id: i64) -> Result<User, AuthError> {
        self.user_repo
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| AuthError::user_not_found(user_id))
    }

    /// ユーザーのロール一覧を取得
    pub async fn list_user_roles(&self, user_id: i64) -> Result<Vec<Role>, AuthError> {
        // ユーザーの存在確認
        self.user_repo
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| AuthError::user_not_found(user_id))?;

        self.role_repo.get_user_roles(user_id).await
    }

    /// トークンを生成
    async fn generate_tokens(&self, user: &User) -> Result<AuthToken, AuthError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // ロールを取得
        let roles = self.role_repo.get_user_roles(user.user_id).await?;
        let role_names: Vec<String> = roles.iter().map(|r| r.role_name.clone()).collect();

        // アクセストークンのクレーム
        let claims = TokenClaims::new(user.user_id, &self.issuer, now + self.access_token_ttl)
            .with_roles(role_names);

        // アクセストークン生成（簡易実装）
        let access_token = self.encode_token(&claims)?;

        // リフレッシュトークン生成
        let refresh_token = self.generate_refresh_token();

        // リフレッシュトークンを保存
        self.token_repo
            .save_refresh_token(user.user_id, &refresh_token, now + self.refresh_token_ttl)
            .await?;

        Ok(AuthToken::new(access_token, refresh_token, self.access_token_ttl))
    }

    /// パスワード検証（簡易実装）
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        // 本番ではbcrypt等を使用
        // この実装は開発用のプレースホルダー
        format!("hash:{}", password) == hash
    }

    /// トークンエンコード（簡易実装）
    fn encode_token(&self, claims: &TokenClaims) -> Result<String, AuthError> {
        // 本番ではjsonwebtoken等を使用
        // この実装は開発用のプレースホルダー
        let payload = format!(
            "{{\"sub\":{},\"iss\":\"{}\",\"exp\":{},\"iat\":{},\"roles\":{:?}}}",
            claims.sub, claims.iss, claims.exp, claims.iat, claims.roles
        );
        Ok(format!(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.{}.{}",
            base64_encode(&payload),
            base64_encode(&self.jwt_secret)
        ))
    }

    /// リフレッシュトークン生成
    fn generate_refresh_token(&self) -> String {
        // 本番ではsecure randomを使用
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("rt_{:x}", timestamp)
    }
}

/// 簡易Base64エンコード
fn base64_encode(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::{
        InMemoryPermissionRepository, InMemoryRoleRepository, InMemoryTokenRepository,
        InMemoryUserRepository,
    };

    fn create_test_service() -> AuthService<
        InMemoryUserRepository,
        InMemoryRoleRepository,
        InMemoryPermissionRepository,
        InMemoryTokenRepository,
    > {
        AuthService::new(
            Arc::new(InMemoryUserRepository::new()),
            Arc::new(InMemoryRoleRepository::new()),
            Arc::new(InMemoryPermissionRepository::new()),
            Arc::new(InMemoryTokenRepository::new()),
            "k1s0-test",
            "secret123",
        )
    }

    #[tokio::test]
    async fn test_authenticate_success() {
        let service = create_test_service();

        // テストユーザーを作成
        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password123");
        service.user_repo.save(&user).await.unwrap();

        // 認証
        let result = service.authenticate("testuser", "password123").await;
        assert!(result.is_ok());

        let token = result.unwrap();
        assert_eq!(token.token_type, "Bearer");
        assert!(token.access_token.starts_with("eyJ"));
    }

    #[tokio::test]
    async fn test_authenticate_invalid_password() {
        let service = create_test_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password123");
        service.user_repo.save(&user).await.unwrap();

        let result = service.authenticate("testuser", "wrongpassword").await;
        assert!(matches!(result, Err(AuthError::AuthenticationFailed { .. })));
    }

    #[tokio::test]
    async fn test_authenticate_user_not_found() {
        let service = create_test_service();

        let result = service.authenticate("nonexistent", "password").await;
        assert!(matches!(result, Err(AuthError::AuthenticationFailed { .. })));
    }

    #[tokio::test]
    async fn test_authenticate_account_locked() {
        let service = create_test_service();

        let user = User::new(1, "lockeduser", "locked@example.com", "Locked User", "hash:password123")
            .with_status(UserStatus::Locked);
        service.user_repo.save(&user).await.unwrap();

        let result = service.authenticate("lockeduser", "password123").await;
        assert!(matches!(result, Err(AuthError::AccountLocked { .. })));
    }

    #[tokio::test]
    async fn test_get_user() {
        let service = create_test_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password");
        service.user_repo.save(&user).await.unwrap();

        let result = service.get_user(1).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().login_id, "testuser");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let service = create_test_service();

        let result = service.get_user(999).await;
        assert!(matches!(result, Err(AuthError::UserNotFound { .. })));
    }

    #[tokio::test]
    async fn test_check_permission() {
        let service = create_test_service();

        let user = User::new(1, "admin", "admin@example.com", "Admin", "hash:password");
        service.user_repo.save(&user).await.unwrap();

        // パーミッションを付与
        service
            .permission_repo
            .add_permission(1, "user:read", None);

        let result = service.check_permission(1, "user:read", None).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_refresh_token() {
        let service = create_test_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password123");
        service.user_repo.save(&user).await.unwrap();

        // 認証してトークンを取得
        let token = service.authenticate("testuser", "password123").await.unwrap();

        // トークンをリフレッシュ
        let result = service.refresh_token(&token.refresh_token).await;
        assert!(result.is_ok());

        let new_token = result.unwrap();
        assert_ne!(new_token.refresh_token, token.refresh_token);
    }
}
