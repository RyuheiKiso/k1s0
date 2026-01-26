//! 認証ドメインエンティティ

use std::time::SystemTime;

/// ユーザーステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserStatus {
    /// 有効
    Active,
    /// 無効
    Inactive,
    /// ロック中
    Locked,
}

impl UserStatus {
    /// 数値から変換
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => Self::Active,
            2 => Self::Locked,
            _ => Self::Inactive,
        }
    }

    /// 数値に変換
    pub fn to_i32(self) -> i32 {
        match self {
            Self::Active => 1,
            Self::Inactive => 0,
            Self::Locked => 2,
        }
    }

    /// アクティブかどうか
    pub fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }
}

/// ユーザー
#[derive(Debug, Clone)]
pub struct User {
    /// ユーザーID
    pub user_id: i64,
    /// ログインID
    pub login_id: String,
    /// メールアドレス
    pub email: String,
    /// 表示名
    pub display_name: String,
    /// パスワードハッシュ
    pub password_hash: String,
    /// ステータス
    pub status: UserStatus,
    /// 最終ログイン日時
    pub last_login_at: Option<SystemTime>,
    /// 作成日時
    pub created_at: SystemTime,
    /// 更新日時
    pub updated_at: SystemTime,
}

impl User {
    /// 新しいユーザーを作成
    pub fn new(
        user_id: i64,
        login_id: impl Into<String>,
        email: impl Into<String>,
        display_name: impl Into<String>,
        password_hash: impl Into<String>,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            user_id,
            login_id: login_id.into(),
            email: email.into(),
            display_name: display_name.into(),
            password_hash: password_hash.into(),
            status: UserStatus::Active,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// ステータスを設定
    pub fn with_status(mut self, status: UserStatus) -> Self {
        self.status = status;
        self
    }

    /// ログイン可能かどうか
    pub fn can_login(&self) -> bool {
        self.status.is_active()
    }
}

/// ロール
#[derive(Debug, Clone)]
pub struct Role {
    /// ロールID
    pub role_id: i64,
    /// ロール名
    pub role_name: String,
    /// 説明
    pub description: String,
}

impl Role {
    /// 新しいロールを作成
    pub fn new(role_id: i64, role_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            role_id,
            role_name: role_name.into(),
            description: description.into(),
        }
    }
}

/// パーミッション
#[derive(Debug, Clone)]
pub struct Permission {
    /// パーミッションID
    pub permission_id: i64,
    /// パーミッションキー（例: "user:read", "order:write"）
    pub permission_key: String,
    /// サービス名スコープ（オプション）
    pub service_name: Option<String>,
}

impl Permission {
    /// 新しいパーミッションを作成
    pub fn new(permission_id: i64, permission_key: impl Into<String>) -> Self {
        Self {
            permission_id,
            permission_key: permission_key.into(),
            service_name: None,
        }
    }

    /// サービス名スコープを設定
    pub fn with_service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = Some(service_name.into());
        self
    }
}

/// 認証トークン
#[derive(Debug, Clone)]
pub struct AuthToken {
    /// アクセストークン
    pub access_token: String,
    /// リフレッシュトークン
    pub refresh_token: String,
    /// 有効期限（秒）
    pub expires_in: i64,
    /// トークンタイプ
    pub token_type: String,
}

impl AuthToken {
    /// 新しいトークンを作成
    pub fn new(
        access_token: impl Into<String>,
        refresh_token: impl Into<String>,
        expires_in: i64,
    ) -> Self {
        Self {
            access_token: access_token.into(),
            refresh_token: refresh_token.into(),
            expires_in,
            token_type: "Bearer".to_string(),
        }
    }
}

/// トークンクレーム
#[derive(Debug, Clone)]
pub struct TokenClaims {
    /// サブジェクト（ユーザーID）
    pub sub: i64,
    /// 発行者
    pub iss: String,
    /// 有効期限（Unix timestamp）
    pub exp: i64,
    /// 発行日時（Unix timestamp）
    pub iat: i64,
    /// ロール
    pub roles: Vec<String>,
}

impl TokenClaims {
    /// 新しいクレームを作成
    pub fn new(user_id: i64, issuer: impl Into<String>, expires_at: i64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            sub: user_id,
            iss: issuer.into(),
            exp: expires_at,
            iat: now,
            roles: Vec::new(),
        }
    }

    /// ロールを追加
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    /// 有効期限が切れているかどうか
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.exp < now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_status() {
        assert!(UserStatus::Active.is_active());
        assert!(!UserStatus::Inactive.is_active());
        assert!(!UserStatus::Locked.is_active());

        assert_eq!(UserStatus::from_i32(1), UserStatus::Active);
        assert_eq!(UserStatus::from_i32(0), UserStatus::Inactive);
        assert_eq!(UserStatus::from_i32(2), UserStatus::Locked);

        assert_eq!(UserStatus::Active.to_i32(), 1);
        assert_eq!(UserStatus::Inactive.to_i32(), 0);
        assert_eq!(UserStatus::Locked.to_i32(), 2);
    }

    #[test]
    fn test_user_new() {
        let user = User::new(1, "john", "john@example.com", "John Doe", "hash123");
        assert_eq!(user.user_id, 1);
        assert_eq!(user.login_id, "john");
        assert_eq!(user.email, "john@example.com");
        assert!(user.can_login());
    }

    #[test]
    fn test_user_cannot_login_when_inactive() {
        let user = User::new(1, "john", "john@example.com", "John Doe", "hash123")
            .with_status(UserStatus::Inactive);
        assert!(!user.can_login());
    }

    #[test]
    fn test_role_new() {
        let role = Role::new(1, "admin", "Administrator role");
        assert_eq!(role.role_id, 1);
        assert_eq!(role.role_name, "admin");
    }

    #[test]
    fn test_permission_new() {
        let perm = Permission::new(1, "user:read")
            .with_service_name("user-service");
        assert_eq!(perm.permission_key, "user:read");
        assert_eq!(perm.service_name, Some("user-service".to_string()));
    }

    #[test]
    fn test_auth_token_new() {
        let token = AuthToken::new("access123", "refresh123", 3600);
        assert_eq!(token.access_token, "access123");
        assert_eq!(token.token_type, "Bearer");
    }

    #[test]
    fn test_token_claims_expired() {
        let claims = TokenClaims::new(1, "k1s0", 0); // 過去の期限
        assert!(claims.is_expired());
    }

    #[test]
    fn test_token_claims_not_expired() {
        let future = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 3600;
        let claims = TokenClaims::new(1, "k1s0", future);
        assert!(!claims.is_expired());
    }
}
