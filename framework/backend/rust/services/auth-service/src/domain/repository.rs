//! 認証リポジトリトレイト

use super::entity::{Permission, Role, User};
use super::error::AuthError;

/// ユーザーリポジトリトレイト
pub trait UserRepository: Send + Sync + 'static {
    /// ユーザーIDで取得
    fn get_by_id(
        &self,
        user_id: i64,
    ) -> impl std::future::Future<Output = Result<Option<User>, AuthError>> + Send;

    /// ログインIDで取得
    fn get_by_login_id(
        &self,
        login_id: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, AuthError>> + Send;

    /// ユーザーを保存
    fn save(&self, user: &User) -> impl std::future::Future<Output = Result<(), AuthError>> + Send;

    /// 最終ログイン日時を更新
    fn update_last_login(
        &self,
        user_id: i64,
    ) -> impl std::future::Future<Output = Result<(), AuthError>> + Send;
}

/// ロールリポジトリトレイト
pub trait RoleRepository: Send + Sync + 'static {
    /// ユーザーのロール一覧を取得
    fn get_user_roles(
        &self,
        user_id: i64,
    ) -> impl std::future::Future<Output = Result<Vec<Role>, AuthError>> + Send;

    /// ユーザーにロールを付与
    fn assign_role(
        &self,
        user_id: i64,
        role_id: i64,
    ) -> impl std::future::Future<Output = Result<(), AuthError>> + Send;

    /// ユーザーからロールを削除
    fn revoke_role(
        &self,
        user_id: i64,
        role_id: i64,
    ) -> impl std::future::Future<Output = Result<(), AuthError>> + Send;
}

/// パーミッションリポジトリトレイト
pub trait PermissionRepository: Send + Sync + 'static {
    /// ユーザーのパーミッションをチェック
    fn check_permission(
        &self,
        user_id: i64,
        permission_key: &str,
        service_name: Option<&str>,
    ) -> impl std::future::Future<Output = Result<bool, AuthError>> + Send;

    /// ロールのパーミッション一覧を取得
    fn get_role_permissions(
        &self,
        role_id: i64,
    ) -> impl std::future::Future<Output = Result<Vec<Permission>, AuthError>> + Send;
}

/// トークンリポジトリトレイト（リフレッシュトークン管理用）
pub trait TokenRepository: Send + Sync + 'static {
    /// リフレッシュトークンを保存
    fn save_refresh_token(
        &self,
        user_id: i64,
        token: &str,
        expires_at: i64,
    ) -> impl std::future::Future<Output = Result<(), AuthError>> + Send;

    /// リフレッシュトークンを検証
    fn validate_refresh_token(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<Option<i64>, AuthError>> + Send;

    /// リフレッシュトークンを無効化
    fn revoke_refresh_token(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<(), AuthError>> + Send;

    /// ユーザーのすべてのリフレッシュトークンを無効化
    fn revoke_all_user_tokens(
        &self,
        user_id: i64,
    ) -> impl std::future::Future<Output = Result<(), AuthError>> + Send;
}
