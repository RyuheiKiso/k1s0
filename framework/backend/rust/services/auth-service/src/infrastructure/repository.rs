//! リポジトリ実装

use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::time::SystemTime;

use crate::domain::{
    AuthError, Permission, PermissionRepository, Role, RoleRepository, TokenRepository, User,
    UserRepository,
};

/// インメモリユーザーリポジトリ
pub struct InMemoryUserRepository {
    users: RwLock<HashMap<i64, User>>,
    login_index: RwLock<HashMap<String, i64>>,
}

impl InMemoryUserRepository {
    /// 新しいリポジトリを作成
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            login_index: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl UserRepository for InMemoryUserRepository {
    async fn get_by_id(&self, user_id: i64) -> Result<Option<User>, AuthError> {
        let users = self.users.read().unwrap();
        Ok(users.get(&user_id).cloned())
    }

    async fn get_by_login_id(&self, login_id: &str) -> Result<Option<User>, AuthError> {
        let login_index = self.login_index.read().unwrap();
        let users = self.users.read().unwrap();

        if let Some(&user_id) = login_index.get(login_id) {
            Ok(users.get(&user_id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn save(&self, user: &User) -> Result<(), AuthError> {
        let mut users = self.users.write().unwrap();
        let mut login_index = self.login_index.write().unwrap();

        login_index.insert(user.login_id.clone(), user.user_id);
        users.insert(user.user_id, user.clone());
        Ok(())
    }

    async fn update_last_login(&self, user_id: i64) -> Result<(), AuthError> {
        let mut users = self.users.write().unwrap();

        if let Some(user) = users.get_mut(&user_id) {
            user.last_login_at = Some(SystemTime::now());
            user.updated_at = SystemTime::now();
            Ok(())
        } else {
            Err(AuthError::user_not_found(user_id))
        }
    }
}

/// インメモリロールリポジトリ
pub struct InMemoryRoleRepository {
    roles: RwLock<HashMap<i64, Role>>,
    user_roles: RwLock<HashMap<i64, HashSet<i64>>>,
}

impl InMemoryRoleRepository {
    /// 新しいリポジトリを作成
    pub fn new() -> Self {
        Self {
            roles: RwLock::new(HashMap::new()),
            user_roles: RwLock::new(HashMap::new()),
        }
    }

    /// ロールを追加
    pub fn add_role(&self, role: Role) {
        let mut roles = self.roles.write().unwrap();
        roles.insert(role.role_id, role);
    }
}

impl Default for InMemoryRoleRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl RoleRepository for InMemoryRoleRepository {
    async fn get_user_roles(&self, user_id: i64) -> Result<Vec<Role>, AuthError> {
        let user_roles = self.user_roles.read().unwrap();
        let roles = self.roles.read().unwrap();

        let role_ids = user_roles.get(&user_id);
        let result = role_ids
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| roles.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default();

        Ok(result)
    }

    async fn assign_role(&self, user_id: i64, role_id: i64) -> Result<(), AuthError> {
        let mut user_roles = self.user_roles.write().unwrap();
        user_roles
            .entry(user_id)
            .or_insert_with(HashSet::new)
            .insert(role_id);
        Ok(())
    }

    async fn revoke_role(&self, user_id: i64, role_id: i64) -> Result<(), AuthError> {
        let mut user_roles = self.user_roles.write().unwrap();
        if let Some(roles) = user_roles.get_mut(&user_id) {
            roles.remove(&role_id);
        }
        Ok(())
    }
}

/// インメモリパーミッションリポジトリ
pub struct InMemoryPermissionRepository {
    permissions: RwLock<HashMap<i64, Permission>>,
    user_permissions: RwLock<HashMap<i64, HashSet<String>>>,
    role_permissions: RwLock<HashMap<i64, HashSet<i64>>>,
}

impl InMemoryPermissionRepository {
    /// 新しいリポジトリを作成
    pub fn new() -> Self {
        Self {
            permissions: RwLock::new(HashMap::new()),
            user_permissions: RwLock::new(HashMap::new()),
            role_permissions: RwLock::new(HashMap::new()),
        }
    }

    /// パーミッションを追加
    pub fn add_permission(&self, user_id: i64, permission_key: &str, service_name: Option<&str>) {
        let mut user_permissions = self.user_permissions.write().unwrap();
        let key = match service_name {
            Some(svc) => format!("{}:{}", svc, permission_key),
            None => permission_key.to_string(),
        };
        user_permissions
            .entry(user_id)
            .or_insert_with(HashSet::new)
            .insert(key);
    }
}

impl Default for InMemoryPermissionRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionRepository for InMemoryPermissionRepository {
    async fn check_permission(
        &self,
        user_id: i64,
        permission_key: &str,
        service_name: Option<&str>,
    ) -> Result<bool, AuthError> {
        let user_permissions = self.user_permissions.read().unwrap();

        if let Some(perms) = user_permissions.get(&user_id) {
            // 完全マッチをチェック
            let key = match service_name {
                Some(svc) => format!("{}:{}", svc, permission_key),
                None => permission_key.to_string(),
            };

            if perms.contains(&key) {
                return Ok(true);
            }

            // サービス名なしでもチェック
            if perms.contains(permission_key) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn get_role_permissions(&self, role_id: i64) -> Result<Vec<Permission>, AuthError> {
        let role_permissions = self.role_permissions.read().unwrap();
        let permissions = self.permissions.read().unwrap();

        let perm_ids = role_permissions.get(&role_id);
        let result = perm_ids
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| permissions.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default();

        Ok(result)
    }
}

/// インメモリトークンリポジトリ
pub struct InMemoryTokenRepository {
    /// トークン -> (user_id, expires_at)
    tokens: RwLock<HashMap<String, (i64, i64)>>,
}

impl InMemoryTokenRepository {
    /// 新しいリポジトリを作成
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
        }
    }

    /// 現在のUnixタイムスタンプを取得
    fn now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
}

impl Default for InMemoryTokenRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenRepository for InMemoryTokenRepository {
    async fn save_refresh_token(
        &self,
        user_id: i64,
        token: &str,
        expires_at: i64,
    ) -> Result<(), AuthError> {
        let mut tokens = self.tokens.write().unwrap();
        tokens.insert(token.to_string(), (user_id, expires_at));
        Ok(())
    }

    async fn validate_refresh_token(&self, token: &str) -> Result<Option<i64>, AuthError> {
        let tokens = self.tokens.read().unwrap();

        if let Some(&(user_id, expires_at)) = tokens.get(token) {
            if expires_at > Self::now() {
                return Ok(Some(user_id));
            }
        }

        Ok(None)
    }

    async fn revoke_refresh_token(&self, token: &str) -> Result<(), AuthError> {
        let mut tokens = self.tokens.write().unwrap();
        tokens.remove(token);
        Ok(())
    }

    async fn revoke_all_user_tokens(&self, user_id: i64) -> Result<(), AuthError> {
        let mut tokens = self.tokens.write().unwrap();
        tokens.retain(|_, (uid, _)| *uid != user_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // InMemoryUserRepository Tests
    // ========================================

    #[tokio::test]
    async fn test_user_repository_new() {
        let repo = InMemoryUserRepository::new();
        assert!(repo.get_by_id(1).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_user_repository_default() {
        let repo = InMemoryUserRepository::default();
        assert!(repo.get_by_id(1).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_user_repository_save_and_get_by_id() {
        let repo = InMemoryUserRepository::new();
        let user = User::new(1, "john", "john@example.com", "John", "hash:pass");

        repo.save(&user).await.unwrap();

        let found = repo.get_by_id(1).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().login_id, "john");
    }

    #[tokio::test]
    async fn test_user_repository_save_and_get_by_login_id() {
        let repo = InMemoryUserRepository::new();
        let user = User::new(1, "john", "john@example.com", "John", "hash:pass");

        repo.save(&user).await.unwrap();

        let found = repo.get_by_login_id("john").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().user_id, 1);
    }

    #[tokio::test]
    async fn test_user_repository_get_by_id_not_found() {
        let repo = InMemoryUserRepository::new();
        let not_found = repo.get_by_id(999).await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_user_repository_get_by_login_id_not_found() {
        let repo = InMemoryUserRepository::new();
        let not_found = repo.get_by_login_id("nonexistent").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_user_repository_save_overwrites_existing() {
        let repo = InMemoryUserRepository::new();
        let user1 = User::new(1, "john", "john@example.com", "John", "hash1");
        let user2 = User::new(1, "john_updated", "john2@example.com", "John Updated", "hash2");

        repo.save(&user1).await.unwrap();
        repo.save(&user2).await.unwrap();

        let found = repo.get_by_id(1).await.unwrap().unwrap();
        assert_eq!(found.login_id, "john_updated");
        assert_eq!(found.email, "john2@example.com");
    }

    #[tokio::test]
    async fn test_user_repository_update_last_login() {
        let repo = InMemoryUserRepository::new();
        let user = User::new(1, "john", "john@example.com", "John", "hash:pass");

        repo.save(&user).await.unwrap();

        // last_login_at is None initially
        let before = repo.get_by_id(1).await.unwrap().unwrap();
        assert!(before.last_login_at.is_none());

        // Update last login
        repo.update_last_login(1).await.unwrap();

        let after = repo.get_by_id(1).await.unwrap().unwrap();
        assert!(after.last_login_at.is_some());
    }

    #[tokio::test]
    async fn test_user_repository_update_last_login_not_found() {
        let repo = InMemoryUserRepository::new();
        let result = repo.update_last_login(999).await;
        assert!(matches!(result, Err(AuthError::UserNotFound { .. })));
    }

    #[tokio::test]
    async fn test_user_repository_multiple_users() {
        let repo = InMemoryUserRepository::new();

        for i in 1..=10 {
            let user = User::new(i, format!("user{}", i), format!("user{}@example.com", i), format!("User {}", i), "hash");
            repo.save(&user).await.unwrap();
        }

        for i in 1..=10 {
            let found = repo.get_by_id(i).await.unwrap();
            assert!(found.is_some());
            assert_eq!(found.unwrap().login_id, format!("user{}", i));
        }
    }

    // ========================================
    // InMemoryRoleRepository Tests
    // ========================================

    #[tokio::test]
    async fn test_role_repository_new() {
        let repo = InMemoryRoleRepository::new();
        let roles = repo.get_user_roles(1).await.unwrap();
        assert!(roles.is_empty());
    }

    #[tokio::test]
    async fn test_role_repository_default() {
        let repo = InMemoryRoleRepository::default();
        let roles = repo.get_user_roles(1).await.unwrap();
        assert!(roles.is_empty());
    }

    #[tokio::test]
    async fn test_role_repository_add_and_get_roles() {
        let repo = InMemoryRoleRepository::new();

        let admin_role = Role::new(1, "admin", "Administrator");
        let user_role = Role::new(2, "user", "Normal user");

        repo.add_role(admin_role);
        repo.add_role(user_role);

        repo.assign_role(100, 1).await.unwrap();
        repo.assign_role(100, 2).await.unwrap();

        let roles = repo.get_user_roles(100).await.unwrap();
        assert_eq!(roles.len(), 2);
    }

    #[tokio::test]
    async fn test_role_repository_assign_same_role_twice() {
        let repo = InMemoryRoleRepository::new();

        let role = Role::new(1, "admin", "Administrator");
        repo.add_role(role);

        repo.assign_role(100, 1).await.unwrap();
        repo.assign_role(100, 1).await.unwrap(); // Assign again

        let roles = repo.get_user_roles(100).await.unwrap();
        assert_eq!(roles.len(), 1); // Should still be 1 (HashSet behavior)
    }

    #[tokio::test]
    async fn test_role_repository_revoke_role() {
        let repo = InMemoryRoleRepository::new();

        let role = Role::new(1, "admin", "Administrator");
        repo.add_role(role);

        repo.assign_role(100, 1).await.unwrap();
        let roles = repo.get_user_roles(100).await.unwrap();
        assert_eq!(roles.len(), 1);

        repo.revoke_role(100, 1).await.unwrap();
        let roles = repo.get_user_roles(100).await.unwrap();
        assert!(roles.is_empty());
    }

    #[tokio::test]
    async fn test_role_repository_revoke_nonexistent_role() {
        let repo = InMemoryRoleRepository::new();

        // Should not error even if role is not assigned
        repo.revoke_role(100, 999).await.unwrap();
    }

    #[tokio::test]
    async fn test_role_repository_multiple_users() {
        let repo = InMemoryRoleRepository::new();

        let admin_role = Role::new(1, "admin", "Administrator");
        let user_role = Role::new(2, "user", "Normal user");

        repo.add_role(admin_role);
        repo.add_role(user_role);

        repo.assign_role(1, 1).await.unwrap(); // User 1 gets admin
        repo.assign_role(2, 2).await.unwrap(); // User 2 gets user

        let user1_roles = repo.get_user_roles(1).await.unwrap();
        let user2_roles = repo.get_user_roles(2).await.unwrap();

        assert_eq!(user1_roles.len(), 1);
        assert_eq!(user1_roles[0].role_name, "admin");
        assert_eq!(user2_roles.len(), 1);
        assert_eq!(user2_roles[0].role_name, "user");
    }

    #[tokio::test]
    async fn test_role_repository_assign_nonexistent_role() {
        let repo = InMemoryRoleRepository::new();

        // Assign a role that doesn't exist in the roles map
        repo.assign_role(100, 999).await.unwrap();

        // get_user_roles should return empty since the role doesn't exist
        let roles = repo.get_user_roles(100).await.unwrap();
        assert!(roles.is_empty());
    }

    // ========================================
    // InMemoryPermissionRepository Tests
    // ========================================

    #[tokio::test]
    async fn test_permission_repository_new() {
        let repo = InMemoryPermissionRepository::new();
        let has_perm = repo.check_permission(1, "any", None).await.unwrap();
        assert!(!has_perm);
    }

    #[tokio::test]
    async fn test_permission_repository_default() {
        let repo = InMemoryPermissionRepository::default();
        let has_perm = repo.check_permission(1, "any", None).await.unwrap();
        assert!(!has_perm);
    }

    #[tokio::test]
    async fn test_permission_repository_add_and_check() {
        let repo = InMemoryPermissionRepository::new();

        repo.add_permission(1, "user:read", None);

        assert!(repo.check_permission(1, "user:read", None).await.unwrap());
        assert!(!repo.check_permission(1, "user:write", None).await.unwrap());
    }

    #[tokio::test]
    async fn test_permission_repository_with_service_name() {
        let repo = InMemoryPermissionRepository::new();

        repo.add_permission(1, "user:write", Some("user-service"));

        assert!(repo
            .check_permission(1, "user:write", Some("user-service"))
            .await
            .unwrap());

        // Without service name - permission is scoped to service, so should not match
        // (This depends on implementation: if scoped permissions require exact match)
        // Testing actual behavior: scoped permissions require the service name
        assert!(!repo.check_permission(1, "user:write", None).await.unwrap());
    }

    #[tokio::test]
    async fn test_permission_repository_different_users() {
        let repo = InMemoryPermissionRepository::new();

        repo.add_permission(1, "admin:all", None);
        repo.add_permission(2, "user:read", None);

        assert!(repo.check_permission(1, "admin:all", None).await.unwrap());
        assert!(!repo.check_permission(1, "user:read", None).await.unwrap());
        assert!(!repo.check_permission(2, "admin:all", None).await.unwrap());
        assert!(repo.check_permission(2, "user:read", None).await.unwrap());
    }

    #[tokio::test]
    async fn test_permission_repository_nonexistent_permission() {
        let repo = InMemoryPermissionRepository::new();
        assert!(!repo.check_permission(1, "admin:all", None).await.unwrap());
    }

    #[tokio::test]
    async fn test_permission_repository_get_role_permissions_empty() {
        let repo = InMemoryPermissionRepository::new();
        let perms = repo.get_role_permissions(1).await.unwrap();
        assert!(perms.is_empty());
    }

    // ========================================
    // InMemoryTokenRepository Tests
    // ========================================

    #[tokio::test]
    async fn test_token_repository_new() {
        let repo = InMemoryTokenRepository::new();
        let user_id = repo.validate_refresh_token("nonexistent").await.unwrap();
        assert!(user_id.is_none());
    }

    #[tokio::test]
    async fn test_token_repository_default() {
        let repo = InMemoryTokenRepository::default();
        let user_id = repo.validate_refresh_token("nonexistent").await.unwrap();
        assert!(user_id.is_none());
    }

    #[tokio::test]
    async fn test_token_repository_save_and_validate() {
        let repo = InMemoryTokenRepository::new();
        let future = InMemoryTokenRepository::now() + 3600;

        repo.save_refresh_token(1, "token123", future).await.unwrap();

        let user_id = repo.validate_refresh_token("token123").await.unwrap();
        assert_eq!(user_id, Some(1));
    }

    #[tokio::test]
    async fn test_token_repository_revoke() {
        let repo = InMemoryTokenRepository::new();
        let future = InMemoryTokenRepository::now() + 3600;

        repo.save_refresh_token(1, "token123", future).await.unwrap();
        repo.revoke_refresh_token("token123").await.unwrap();

        let user_id = repo.validate_refresh_token("token123").await.unwrap();
        assert_eq!(user_id, None);
    }

    #[tokio::test]
    async fn test_token_repository_expired() {
        let repo = InMemoryTokenRepository::new();
        let past = InMemoryTokenRepository::now() - 1;

        repo.save_refresh_token(1, "expired_token", past)
            .await
            .unwrap();

        let user_id = repo.validate_refresh_token("expired_token").await.unwrap();
        assert_eq!(user_id, None);
    }

    #[tokio::test]
    async fn test_revoke_all_user_tokens() {
        let repo = InMemoryTokenRepository::new();
        let future = InMemoryTokenRepository::now() + 3600;

        repo.save_refresh_token(1, "token1", future).await.unwrap();
        repo.save_refresh_token(1, "token2", future).await.unwrap();
        repo.save_refresh_token(2, "token3", future).await.unwrap();

        repo.revoke_all_user_tokens(1).await.unwrap();

        assert!(repo.validate_refresh_token("token1").await.unwrap().is_none());
        assert!(repo.validate_refresh_token("token2").await.unwrap().is_none());
        assert!(repo.validate_refresh_token("token3").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_token_repository_overwrite_same_token() {
        let repo = InMemoryTokenRepository::new();
        let future = InMemoryTokenRepository::now() + 3600;

        repo.save_refresh_token(1, "token", future).await.unwrap();
        repo.save_refresh_token(2, "token", future).await.unwrap(); // Same token, different user

        let user_id = repo.validate_refresh_token("token").await.unwrap();
        assert_eq!(user_id, Some(2)); // Should be the latest user
    }

    #[tokio::test]
    async fn test_token_repository_multiple_tokens_same_user() {
        let repo = InMemoryTokenRepository::new();
        let future = InMemoryTokenRepository::now() + 3600;

        repo.save_refresh_token(1, "token_a", future).await.unwrap();
        repo.save_refresh_token(1, "token_b", future).await.unwrap();
        repo.save_refresh_token(1, "token_c", future).await.unwrap();

        assert_eq!(repo.validate_refresh_token("token_a").await.unwrap(), Some(1));
        assert_eq!(repo.validate_refresh_token("token_b").await.unwrap(), Some(1));
        assert_eq!(repo.validate_refresh_token("token_c").await.unwrap(), Some(1));
    }

    #[tokio::test]
    async fn test_token_repository_revoke_nonexistent_token() {
        let repo = InMemoryTokenRepository::new();
        // Should not panic
        repo.revoke_refresh_token("nonexistent").await.unwrap();
    }

    #[tokio::test]
    async fn test_token_repository_validate_after_expiry_boundary() {
        let repo = InMemoryTokenRepository::new();
        let exactly_now = InMemoryTokenRepository::now();

        repo.save_refresh_token(1, "boundary_token", exactly_now).await.unwrap();

        // Token expires exactly at `now`, so it should be invalid (exp <= now)
        let user_id = repo.validate_refresh_token("boundary_token").await.unwrap();
        assert_eq!(user_id, None);
    }

    // ========================================
    // Thread Safety Tests (basic)
    // ========================================

    #[tokio::test]
    async fn test_user_repository_concurrent_access() {
        let repo = std::sync::Arc::new(InMemoryUserRepository::new());

        let mut handles = vec![];
        for i in 0..10 {
            let repo_clone = std::sync::Arc::clone(&repo);
            handles.push(tokio::spawn(async move {
                let user = User::new(i, format!("user{}", i), format!("user{}@test.com", i), "Name", "hash");
                repo_clone.save(&user).await.unwrap();
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all users were saved
        for i in 0..10 {
            assert!(repo.get_by_id(i).await.unwrap().is_some());
        }
    }

    #[tokio::test]
    async fn test_token_repository_concurrent_access() {
        let repo = std::sync::Arc::new(InMemoryTokenRepository::new());
        let future = InMemoryTokenRepository::now() + 3600;

        let mut handles = vec![];
        for i in 0..10 {
            let repo_clone = std::sync::Arc::clone(&repo);
            handles.push(tokio::spawn(async move {
                repo_clone.save_refresh_token(i, &format!("token{}", i), future).await.unwrap();
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all tokens were saved
        for i in 0..10 {
            assert!(repo.validate_refresh_token(&format!("token{}", i)).await.unwrap().is_some());
        }
    }
}
