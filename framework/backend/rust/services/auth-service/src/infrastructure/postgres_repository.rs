//! PostgreSQLリポジトリ実装

use sqlx::{PgPool, Row};
use std::time::SystemTime;

use crate::domain::{
    AuthError, Permission, PermissionRepository, Role, RoleRepository, TokenRepository, User,
    UserRepository, UserStatus,
};

/// PostgreSQLユーザーリポジトリ
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    /// 新しいリポジトリを作成
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UserRepository for PostgresUserRepository {
    async fn get_by_id(&self, user_id: i64) -> Result<Option<User>, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT user_id, login_id, email, display_name, password_hash, status,
                   last_login_at, created_at, updated_at
            FROM fw_m_user
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(row.map(|r| row_to_user(&r)))
    }

    async fn get_by_login_id(&self, login_id: &str) -> Result<Option<User>, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT user_id, login_id, email, display_name, password_hash, status,
                   last_login_at, created_at, updated_at
            FROM fw_m_user
            WHERE login_id = $1
            "#,
        )
        .bind(login_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(row.map(|r| row_to_user(&r)))
    }

    async fn save(&self, user: &User) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            INSERT INTO fw_m_user (user_id, login_id, email, display_name, password_hash, status, last_login_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (user_id)
            DO UPDATE SET
                login_id = EXCLUDED.login_id,
                email = EXCLUDED.email,
                display_name = EXCLUDED.display_name,
                password_hash = EXCLUDED.password_hash,
                status = EXCLUDED.status,
                last_login_at = EXCLUDED.last_login_at,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(user.user_id)
        .bind(&user.login_id)
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(&user.password_hash)
        .bind(user.status.to_i32() as i16)
        .bind(user.last_login_at.map(|t| chrono::DateTime::<chrono::Utc>::from(t)))
        .bind(chrono::DateTime::<chrono::Utc>::from(user.created_at))
        .bind(chrono::DateTime::<chrono::Utc>::from(user.updated_at))
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(())
    }

    async fn update_last_login(&self, user_id: i64) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            UPDATE fw_m_user
            SET last_login_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(())
    }
}

fn row_to_user(row: &sqlx::postgres::PgRow) -> User {
    let status: i16 = row.get("status");
    let last_login_at: Option<chrono::DateTime<chrono::Utc>> = row.get("last_login_at");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

    User {
        user_id: row.get("user_id"),
        login_id: row.get("login_id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        password_hash: row.get("password_hash"),
        status: UserStatus::from_i32(status as i32),
        last_login_at: last_login_at.map(SystemTime::from),
        created_at: SystemTime::from(created_at),
        updated_at: SystemTime::from(updated_at),
    }
}

/// PostgreSQLロールリポジトリ
pub struct PostgresRoleRepository {
    pool: PgPool,
}

impl PostgresRoleRepository {
    /// 新しいリポジトリを作成
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RoleRepository for PostgresRoleRepository {
    async fn get_user_roles(&self, user_id: i64) -> Result<Vec<Role>, AuthError> {
        let rows = sqlx::query(
            r#"
            SELECT r.role_id, r.role_name, r.description
            FROM fw_m_role r
            INNER JOIN fw_m_user_role ur ON r.role_id = ur.role_id
            WHERE ur.user_id = $1
              AND (ur.expires_at IS NULL OR ur.expires_at > CURRENT_TIMESTAMP)
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Role {
                role_id: r.get("role_id"),
                role_name: r.get("role_name"),
                description: r.get::<Option<String>, _>("description").unwrap_or_default(),
            })
            .collect())
    }

    async fn assign_role(&self, user_id: i64, role_id: i64) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            INSERT INTO fw_m_user_role (user_id, role_id)
            VALUES ($1, $2)
            ON CONFLICT (user_id, role_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(())
    }

    async fn revoke_role(&self, user_id: i64, role_id: i64) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            DELETE FROM fw_m_user_role
            WHERE user_id = $1 AND role_id = $2
            "#,
        )
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(())
    }
}

/// PostgreSQLパーミッションリポジトリ
pub struct PostgresPermissionRepository {
    pool: PgPool,
}

impl PostgresPermissionRepository {
    /// 新しいリポジトリを作成
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PermissionRepository for PostgresPermissionRepository {
    async fn check_permission(
        &self,
        user_id: i64,
        permission_key: &str,
        service_name: Option<&str>,
    ) -> Result<bool, AuthError> {
        let query = match service_name {
            Some(svc) => {
                sqlx::query(
                    r#"
                    SELECT EXISTS(
                        SELECT 1
                        FROM fw_m_user_role ur
                        INNER JOIN fw_m_role_permission rp ON ur.role_id = rp.role_id
                        INNER JOIN fw_m_permission p ON rp.permission_id = p.permission_id
                        WHERE ur.user_id = $1
                          AND p.permission_key = $2
                          AND (p.service_name IS NULL OR p.service_name = $3)
                          AND (ur.expires_at IS NULL OR ur.expires_at > CURRENT_TIMESTAMP)
                    ) AS has_permission
                    "#,
                )
                .bind(user_id)
                .bind(permission_key)
                .bind(svc)
            }
            None => {
                sqlx::query(
                    r#"
                    SELECT EXISTS(
                        SELECT 1
                        FROM fw_m_user_role ur
                        INNER JOIN fw_m_role_permission rp ON ur.role_id = rp.role_id
                        INNER JOIN fw_m_permission p ON rp.permission_id = p.permission_id
                        WHERE ur.user_id = $1
                          AND p.permission_key = $2
                          AND (ur.expires_at IS NULL OR ur.expires_at > CURRENT_TIMESTAMP)
                    ) AS has_permission
                    "#,
                )
                .bind(user_id)
                .bind(permission_key)
            }
        };

        let row = query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(row.get("has_permission"))
    }

    async fn get_role_permissions(&self, role_id: i64) -> Result<Vec<Permission>, AuthError> {
        let rows = sqlx::query(
            r#"
            SELECT p.permission_id, p.permission_key, p.service_name
            FROM fw_m_permission p
            INNER JOIN fw_m_role_permission rp ON p.permission_id = rp.permission_id
            WHERE rp.role_id = $1
            "#,
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Permission {
                permission_id: r.get("permission_id"),
                permission_key: r.get("permission_key"),
                service_name: r.get("service_name"),
            })
            .collect())
    }
}

/// PostgreSQLトークンリポジトリ
pub struct PostgresTokenRepository {
    pool: PgPool,
}

impl PostgresTokenRepository {
    /// 新しいリポジトリを作成
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// トークンテーブルが存在しない場合に作成（初期化用）
    pub async fn ensure_table(&self) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS fw_t_refresh_token (
                token_id BIGSERIAL PRIMARY KEY,
                user_id BIGINT NOT NULL REFERENCES fw_m_user(user_id) ON DELETE CASCADE,
                token_hash VARCHAR(255) NOT NULL UNIQUE,
                expires_at TIMESTAMPTZ NOT NULL,
                revoked_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(())
    }
}

impl TokenRepository for PostgresTokenRepository {
    async fn save_refresh_token(
        &self,
        user_id: i64,
        token: &str,
        expires_at: i64,
    ) -> Result<(), AuthError> {
        let expires_at_dt = chrono::DateTime::from_timestamp(expires_at, 0)
            .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());

        sqlx::query(
            r#"
            INSERT INTO fw_t_refresh_token (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(user_id)
        .bind(token)
        .bind(expires_at_dt)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(())
    }

    async fn validate_refresh_token(&self, token: &str) -> Result<Option<i64>, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT user_id
            FROM fw_t_refresh_token
            WHERE token_hash = $1
              AND expires_at > CURRENT_TIMESTAMP
              AND revoked_at IS NULL
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(row.map(|r| r.get("user_id")))
    }

    async fn revoke_refresh_token(&self, token: &str) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            UPDATE fw_t_refresh_token
            SET revoked_at = CURRENT_TIMESTAMP
            WHERE token_hash = $1
            "#,
        )
        .bind(token)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(())
    }

    async fn revoke_all_user_tokens(&self, user_id: i64) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            UPDATE fw_t_refresh_token
            SET revoked_at = CURRENT_TIMESTAMP
            WHERE user_id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::storage(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_postgres_repository_structs() {
        // Struct creation tests - actual database tests require integration setup
    }
}
