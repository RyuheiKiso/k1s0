use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::user::{Pagination, User, UserListResult, UserRoles};
// ドメインエラー型をインポート: anyhow::anyhow! の代わりに型安全なエラーを使用する
use crate::domain::error::AuthError;
use crate::domain::repository::UserRepository;

/// UserPostgresRepository は PostgreSQL ベースのユーザーリポジトリ。
/// auth.users テーブルに対する CRUD 操作を提供する。
pub struct UserPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl UserPostgresRepository {
    // メトリクス不要の簡易コンストラクタ（テストやスクリプト用途で使用予定のため dead_code を許可）
    #[allow(dead_code)]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics: None,
        }
    }

    pub fn with_metrics(pool: PgPool, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
        }
    }

    /// User ドメインモデルから DB 用パラメータへの変換ヘルパー。
    // create/update メソッド経由で使用されるため dead_code を許可
    #[allow(dead_code)]
    fn extract_keycloak_sub(user: &User) -> String {
        user.attributes
            .get("keycloak_sub")
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_default()
    }

    // create/update メソッド経由で使用されるため dead_code を許可
    #[allow(dead_code)]
    fn build_display_name(user: &User) -> String {
        if user.last_name.is_empty() {
            user.first_name.clone()
        } else {
            format!("{} {}", user.first_name, user.last_name)
        }
    }

    // create/update メソッド経由で使用されるため dead_code を許可
    #[allow(dead_code)]
    fn status_from_enabled(enabled: bool) -> &'static str {
        if enabled {
            "active"
        } else {
            "inactive"
        }
    }
}

/// UserRow は auth.users テーブルの行を表す中間構造体。
// UserRepository トレイト実装の find_by_id / list で使用される。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub keycloak_sub: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// DB テーブルの updated_at カラムに対応。sqlx::FromRow で SELECT 時に必要だが、
    /// User ドメインモデルへの変換では使用しないため dead_code を許可する。
    #[allow(dead_code)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        let (first_name, last_name) = split_display_name(&row.display_name);
        let enabled = row.status == "active";

        let mut attributes = std::collections::HashMap::new();
        attributes.insert("keycloak_sub".to_string(), vec![row.keycloak_sub]);

        User {
            id: row.id.to_string(),
            username: row.username,
            email: row.email,
            first_name,
            last_name,
            enabled,
            email_verified: true, // DB 管理ユーザーは検証済みとする
            created_at: row.created_at,
            attributes,
        }
    }
}

/// display_name を first_name と last_name に分割する。
fn split_display_name(display_name: &str) -> (String, String) {
    let parts: Vec<&str> = display_name.splitn(2, ' ').collect();
    match parts.len() {
        0 => (String::new(), String::new()),
        1 => (parts[0].to_string(), String::new()),
        _ => (parts[0].to_string(), parts[1].to_string()),
    }
}

#[async_trait]
impl UserRepository for UserPostgresRepository {
    async fn find_by_id(&self, user_id: &str) -> anyhow::Result<User> {
        // UUID フォーマットが無効な場合はドメインエラー型 ValidationFailed を返す
        // anyhow::anyhow! ではなく AuthError を使用することで型安全なエラー分類が可能になる
        let uuid = Uuid::parse_str(user_id)
            .map_err(|e| AuthError::ValidationFailed(format!("invalid user ID format: {}", e)))?;

        let start = std::time::Instant::now();
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, keycloak_sub, username, email, display_name, status, created_at, updated_at
            FROM auth.users
            WHERE id = $1
            "#,
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("find_by_id", "users", start.elapsed().as_secs_f64());
        }

        // 対象ユーザーが存在しない場合はドメインエラー型 NotFound を返す
        // anyhow::anyhow! ではなく AuthError を使用することで HTTP 404 に正確にマッピングされる
        let row = row?.ok_or_else(|| AuthError::NotFound(user_id.to_string()))?;
        Ok(row.into())
    }

    async fn list(
        &self,
        page: i32,
        page_size: i32,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<UserListResult> {
        // ページ番号・ページサイズの最小値をガードする
        let page = if page < 1 { 1 } else { page };
        let page_size = if page_size < 1 { 20 } else { page_size };
        let offset = (page - 1) * page_size;

        // COUNT クエリ: QueryBuilder で WHERE 句を動的に組み立てる
        // format! によるプレースホルダー番号の手動管理を排除し、保守性を向上させる
        let mut count_qb =
            sqlx::QueryBuilder::new("SELECT COUNT(*) FROM auth.users WHERE 1=1");

        if let Some(ref s) = search {
            // ILIKE による部分一致検索: username / email / display_name を対象とする
            count_qb
                .push(" AND (username ILIKE ")
                .push_bind(format!("%{}%", s))
                .push(" OR email ILIKE ")
                .push_bind(format!("%{}%", s))
                .push(" OR display_name ILIKE ")
                .push_bind(format!("%{}%", s))
                .push(")");
        }

        if let Some(en) = enabled {
            // enabled フラグを DB の status 文字列に変換して絞り込む
            let status = if en { "active" } else { "inactive" };
            count_qb.push(" AND status = ").push_bind(status.to_string());
        }

        let start = std::time::Instant::now();
        let total_count: i64 = count_qb
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("list_count", "users", start.elapsed().as_secs_f64());
        }

        // DATA クエリ: QueryBuilder で WHERE 句・ORDER BY・LIMIT・OFFSET を動的に組み立てる
        let mut data_qb = sqlx::QueryBuilder::new(
            "SELECT id, keycloak_sub, username, email, display_name, status, created_at, updated_at FROM auth.users WHERE 1=1",
        );

        if let Some(ref s) = search {
            // COUNT クエリと同一条件で絞り込む
            data_qb
                .push(" AND (username ILIKE ")
                .push_bind(format!("%{}%", s))
                .push(" OR email ILIKE ")
                .push_bind(format!("%{}%", s))
                .push(" OR display_name ILIKE ")
                .push_bind(format!("%{}%", s))
                .push(")");
        }

        if let Some(en) = enabled {
            let status = if en { "active" } else { "inactive" };
            data_qb.push(" AND status = ").push_bind(status.to_string());
        }

        // ページネーション用の ORDER BY / LIMIT / OFFSET を追加する
        data_qb
            .push(" ORDER BY created_at DESC LIMIT ")
            .push_bind(page_size as i64)
            .push(" OFFSET ")
            .push_bind(offset as i64);

        let start = std::time::Instant::now();
        let rows: Vec<UserRow> = data_qb
            .build_query_as::<UserRow>()
            .fetch_all(&self.pool)
            .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("list", "users", start.elapsed().as_secs_f64());
        }

        let users: Vec<User> = rows.into_iter().map(|r| r.into()).collect();
        let has_next = (page as i64 * page_size as i64) < total_count;

        Ok(UserListResult {
            users,
            pagination: Pagination {
                total_count,
                page,
                page_size,
                has_next,
            },
        })
    }

    async fn get_roles(&self, user_id: &str) -> anyhow::Result<UserRoles> {
        // ロール情報は Keycloak が管理するため、DB からは取得しない。
        // このメソッドは UserRepository トレイトの互換性のために存在する。
        // anyhow::bail! ではなく AuthError::Internal を使用してドメインエラー型で伝播させる
        Err(AuthError::Internal(format!(
            "UserPostgresRepository does not support get_roles; use KeycloakClient instead: {}",
            user_id
        ))
        .into())
    }
}

/// UserPostgresRepository に追加の CRUD メソッド。
/// UserRepository トレイトにない DB 固有の操作。
impl UserPostgresRepository {
    /// keycloak_sub でユーザーを検索する。
    // 将来のユーザー同期・認証連携機能で使用予定のため dead_code を許可
    #[allow(dead_code)]
    pub async fn find_by_keycloak_sub(&self, sub: &str) -> anyhow::Result<Option<User>> {
        let start = std::time::Instant::now();
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, keycloak_sub, username, email, display_name, status, created_at, updated_at
            FROM auth.users
            WHERE keycloak_sub = $1
            "#,
        )
        .bind(sub)
        .fetch_optional(&self.pool)
        .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "find_by_keycloak_sub",
                "users",
                start.elapsed().as_secs_f64(),
            );
        }

        Ok(row.map(|r| r.into()))
    }

    /// ユーザーを作成する。
    // 将来のユーザー管理 API で使用予定のため dead_code を許可
    #[allow(dead_code)]
    pub async fn create(&self, user: &User) -> anyhow::Result<User> {
        let keycloak_sub = Self::extract_keycloak_sub(user);
        let display_name = Self::build_display_name(user);
        let status = Self::status_from_enabled(user.enabled);

        let start = std::time::Instant::now();
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO auth.users (keycloak_sub, username, email, display_name, status)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, keycloak_sub, username, email, display_name, status, created_at, updated_at
            "#,
        )
        .bind(&keycloak_sub)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&display_name)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("create", "users", start.elapsed().as_secs_f64());
        }

        Ok(row.into())
    }

    /// ユーザーを更新する。
    // 将来のユーザー管理 API で使用予定のため dead_code を許可
    #[allow(dead_code)]
    pub async fn update(&self, user: &User) -> anyhow::Result<User> {
        // UUID フォーマットが無効な場合はドメインエラー型 ValidationFailed を返す
        let uuid = Uuid::parse_str(&user.id)
            .map_err(|e| AuthError::ValidationFailed(format!("invalid user ID format: {}", e)))?;
        let display_name = Self::build_display_name(user);
        let status = Self::status_from_enabled(user.enabled);

        let start = std::time::Instant::now();
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            UPDATE auth.users
            SET username = $2, email = $3, display_name = $4, status = $5
            WHERE id = $1
            RETURNING id, keycloak_sub, username, email, display_name, status, created_at, updated_at
            "#,
        )
        .bind(uuid)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&display_name)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("update", "users", start.elapsed().as_secs_f64());
        }

        Ok(row.into())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::user_repository::MockUserRepository;
    use std::collections::HashMap;

    #[test]
    fn test_user_row_to_user_conversion() {
        let row = UserRow {
            id: Uuid::new_v4(),
            keycloak_sub: "kc-sub-123".to_string(),
            username: "test.user".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            status: "active".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let user: User = row.clone().into();
        assert_eq!(user.id, row.id.to_string());
        assert_eq!(user.username, "test.user");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.first_name, "Test");
        assert_eq!(user.last_name, "User");
        assert!(user.enabled);
        assert!(user.email_verified);
        assert_eq!(
            user.attributes.get("keycloak_sub").unwrap(),
            &vec!["kc-sub-123".to_string()]
        );
    }

    #[test]
    fn test_user_row_inactive_status() {
        let row = UserRow {
            id: Uuid::new_v4(),
            keycloak_sub: "kc-inactive".to_string(),
            username: "inactive".to_string(),
            email: "inactive@example.com".to_string(),
            display_name: "Inactive User".to_string(),
            status: "inactive".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let user: User = row.into();
        assert!(!user.enabled);
    }

    #[test]
    fn test_user_row_suspended_status() {
        let row = UserRow {
            id: Uuid::new_v4(),
            keycloak_sub: "kc-suspended".to_string(),
            username: "suspended".to_string(),
            email: "suspended@example.com".to_string(),
            display_name: "Suspended User".to_string(),
            status: "suspended".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let user: User = row.into();
        assert!(!user.enabled);
    }

    #[test]
    fn test_split_display_name_two_parts() {
        let (first, last) = split_display_name("Taro Yamada");
        assert_eq!(first, "Taro");
        assert_eq!(last, "Yamada");
    }

    #[test]
    fn test_split_display_name_single_part() {
        let (first, last) = split_display_name("Admin");
        assert_eq!(first, "Admin");
        assert_eq!(last, "");
    }

    #[test]
    fn test_split_display_name_three_parts() {
        let (first, last) = split_display_name("John Paul Smith");
        assert_eq!(first, "John");
        assert_eq!(last, "Paul Smith");
    }

    #[test]
    fn test_split_display_name_empty() {
        let (first, last) = split_display_name("");
        assert_eq!(first, "");
        assert_eq!(last, "");
    }

    #[test]
    fn test_extract_keycloak_sub() {
        let user = User {
            id: "id".to_string(),
            username: "u".to_string(),
            email: "e".to_string(),
            first_name: "f".to_string(),
            last_name: "l".to_string(),
            enabled: true,
            email_verified: true,
            created_at: chrono::Utc::now(),
            attributes: HashMap::from([(
                "keycloak_sub".to_string(),
                vec!["sub-value".to_string()],
            )]),
        };
        assert_eq!(
            UserPostgresRepository::extract_keycloak_sub(&user),
            "sub-value"
        );
    }

    #[test]
    fn test_extract_keycloak_sub_missing() {
        let user = User {
            id: "id".to_string(),
            username: "u".to_string(),
            email: "e".to_string(),
            first_name: "f".to_string(),
            last_name: "l".to_string(),
            enabled: true,
            email_verified: true,
            created_at: chrono::Utc::now(),
            attributes: HashMap::new(),
        };
        assert_eq!(UserPostgresRepository::extract_keycloak_sub(&user), "");
    }

    #[test]
    fn test_build_display_name() {
        let user = User {
            id: "id".to_string(),
            username: "u".to_string(),
            email: "e".to_string(),
            first_name: "Taro".to_string(),
            last_name: "Yamada".to_string(),
            enabled: true,
            email_verified: true,
            created_at: chrono::Utc::now(),
            attributes: HashMap::new(),
        };
        assert_eq!(
            UserPostgresRepository::build_display_name(&user),
            "Taro Yamada"
        );
    }

    #[test]
    fn test_build_display_name_no_last() {
        let user = User {
            id: "id".to_string(),
            username: "u".to_string(),
            email: "e".to_string(),
            first_name: "Admin".to_string(),
            last_name: "".to_string(),
            enabled: true,
            email_verified: true,
            created_at: chrono::Utc::now(),
            attributes: HashMap::new(),
        };
        assert_eq!(UserPostgresRepository::build_display_name(&user), "Admin");
    }

    #[test]
    fn test_status_from_enabled() {
        assert_eq!(UserPostgresRepository::status_from_enabled(true), "active");
        assert_eq!(
            UserPostgresRepository::status_from_enabled(false),
            "inactive"
        );
    }

    #[tokio::test]
    async fn test_mock_find_by_id() {
        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .withf(|id| id == "user-1")
            .returning(|_| {
                Ok(User {
                    id: "user-1".to_string(),
                    username: "test.user".to_string(),
                    email: "test@example.com".to_string(),
                    first_name: "Test".to_string(),
                    last_name: "User".to_string(),
                    enabled: true,
                    email_verified: true,
                    created_at: chrono::Utc::now(),
                    attributes: HashMap::new(),
                })
            });

        let user = mock.find_by_id("user-1").await.unwrap();
        assert_eq!(user.id, "user-1");
    }

    #[tokio::test]
    async fn test_mock_list_users() {
        let mut mock = MockUserRepository::new();
        mock.expect_list()
            .withf(|page, ps, search, enabled| {
                *page == 1 && *ps == 20 && search.is_none() && enabled.is_none()
            })
            .returning(|page, page_size, _, _| {
                Ok(UserListResult {
                    users: vec![],
                    pagination: Pagination {
                        total_count: 0,
                        page,
                        page_size,
                        has_next: false,
                    },
                })
            });

        let result = mock.list(1, 20, None, None).await.unwrap();
        assert!(result.users.is_empty());
        assert_eq!(result.pagination.total_count, 0);
    }
}
