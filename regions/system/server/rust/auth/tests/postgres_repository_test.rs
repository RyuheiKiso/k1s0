use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use k1s0_auth_server::adapter::repository::audit_log_postgres::AuditLogPostgresRepository;
use k1s0_auth_server::adapter::repository::user_postgres::UserPostgresRepository;
use k1s0_auth_server::domain::entity::audit_log::{AuditLog, AuditLogSearchParams};
use k1s0_auth_server::domain::entity::user::User;
use k1s0_auth_server::domain::repository::{AuditLogRepository, UserRepository};

// -------------------------------------------------------
// UserPostgresRepository テスト（モック使用）
// -------------------------------------------------------

#[cfg(test)]
mod user_postgres_tests {
    use super::*;
    use k1s0_auth_server::adapter::repository::user_postgres::UserRow;

    #[test]
    fn test_user_row_to_user() {
        let row = UserRow {
            id: Uuid::new_v4(),
            keycloak_sub: "kc-sub-12345".to_string(),
            username: "taro.yamada".to_string(),
            email: "taro@example.com".to_string(),
            display_name: "Taro Yamada".to_string(),
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user: User = row.clone().into();
        assert_eq!(user.id, row.id.to_string());
        assert_eq!(user.username, "taro.yamada");
        assert_eq!(user.email, "taro@example.com");
        assert_eq!(user.first_name, "Taro");
        assert_eq!(user.last_name, "Yamada");
        assert!(user.enabled);
        assert!(user.email_verified);
    }

    #[test]
    fn test_user_row_to_user_inactive_status() {
        let row = UserRow {
            id: Uuid::new_v4(),
            keycloak_sub: "kc-sub-99999".to_string(),
            username: "suspended.user".to_string(),
            email: "suspended@example.com".to_string(),
            display_name: "Suspended User".to_string(),
            status: "suspended".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user: User = row.into();
        assert!(!user.enabled);
    }

    #[test]
    fn test_user_row_to_user_single_name() {
        let row = UserRow {
            id: Uuid::new_v4(),
            keycloak_sub: "kc-sub-single".to_string(),
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
            display_name: "Admin".to_string(),
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user: User = row.into();
        assert_eq!(user.first_name, "Admin");
        assert_eq!(user.last_name, "");
    }

    #[test]
    fn test_user_row_to_user_keycloak_sub_in_attributes() {
        let row = UserRow {
            id: Uuid::new_v4(),
            keycloak_sub: "kc-sub-attr-test".to_string(),
            username: "test.user".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user: User = row.into();
        assert_eq!(
            user.attributes.get("keycloak_sub").unwrap(),
            &vec!["kc-sub-attr-test".to_string()]
        );
    }
}

// -------------------------------------------------------
// AuditLogPostgresRepository テスト（既存のモックテスト補完）
// -------------------------------------------------------

#[cfg(test)]
mod audit_log_postgres_tests {
    use super::*;
    use k1s0_auth_server::adapter::repository::audit_log_postgres::AuditLogRow;

    #[test]
    fn test_audit_log_row_conversion_with_metadata() {
        let row = AuditLogRow {
            id: Uuid::new_v4(),
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: "user-1".to_string(),
            ip_address: "192.168.1.100".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            resource: "/api/v1/auth/token".to_string(),
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            metadata: serde_json::json!({"client_id": "react-spa", "session_id": "sess-123"}),
            recorded_at: Utc::now(),
        };

        let log: AuditLog = row.into();
        assert_eq!(log.event_type, "LOGIN_SUCCESS");
        assert_eq!(log.metadata.get("client_id").unwrap(), "react-spa");
        assert_eq!(log.metadata.get("session_id").unwrap(), "sess-123");
    }

    #[test]
    fn test_audit_log_row_conversion_with_null_metadata() {
        let row = AuditLogRow {
            id: Uuid::new_v4(),
            event_type: "TOKEN_VALIDATE".to_string(),
            user_id: "user-2".to_string(),
            ip_address: "10.0.0.1".to_string(),
            user_agent: "".to_string(),
            resource: "/api/v1/auth/token/validate".to_string(),
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            metadata: serde_json::Value::Null,
            recorded_at: Utc::now(),
        };

        let log: AuditLog = row.into();
        assert!(log.metadata.is_empty());
    }
}

// -------------------------------------------------------
// sqlx::test を使った実データベーステスト
// DATABASE_URL が設定されている場合のみ実行
// -------------------------------------------------------

#[cfg(test)]
mod database_integration_tests {
    use super::*;

    // sqlx::test は DATABASE_URL 環境変数が設定されている場合に
    // テスト用データベースを自動作成・マイグレーション実行する。
    // CI/CD やローカルの Docker 環境で実行可能。

    #[cfg(feature = "db-tests")]
    mod with_db {
        use super::*;
        use sqlx::PgPool;

        #[sqlx::test(
            migrations = "../../database/auth-db/migrations"
        )]
        async fn test_user_create_and_find_by_id(pool: PgPool) {
            let repo = UserPostgresRepository::new(pool);

            let user = User {
                id: Uuid::new_v4().to_string(),
                username: "test.user.create".to_string(),
                email: "create@example.com".to_string(),
                first_name: "Test".to_string(),
                last_name: "User".to_string(),
                enabled: true,
                email_verified: true,
                created_at: Utc::now(),
                attributes: HashMap::from([(
                    "keycloak_sub".to_string(),
                    vec!["kc-sub-test-create".to_string()],
                )]),
            };

            let created = repo.create(&user).await.unwrap();
            assert_eq!(created.username, "test.user.create");

            let found = repo.find_by_id(&created.id).await.unwrap();
            assert_eq!(found.username, "test.user.create");
            assert_eq!(found.email, "create@example.com");
        }

        #[sqlx::test(
            migrations = "../../database/auth-db/migrations"
        )]
        async fn test_user_find_by_keycloak_sub(pool: PgPool) {
            let repo = UserPostgresRepository::new(pool);

            let user = User {
                id: Uuid::new_v4().to_string(),
                username: "test.kcsub".to_string(),
                email: "kcsub@example.com".to_string(),
                first_name: "KC".to_string(),
                last_name: "Sub".to_string(),
                enabled: true,
                email_verified: true,
                created_at: Utc::now(),
                attributes: HashMap::from([(
                    "keycloak_sub".to_string(),
                    vec!["unique-kc-sub-value".to_string()],
                )]),
            };

            repo.create(&user).await.unwrap();

            let found = repo
                .find_by_keycloak_sub("unique-kc-sub-value")
                .await
                .unwrap();
            assert!(found.is_some());
            assert_eq!(found.unwrap().username, "test.kcsub");
        }

        #[sqlx::test(
            migrations = "../../database/auth-db/migrations"
        )]
        async fn test_user_list_with_pagination(pool: PgPool) {
            let repo = UserPostgresRepository::new(pool.clone());

            for i in 0..15 {
                let user = User {
                    id: Uuid::new_v4().to_string(),
                    username: format!("list.user.{}", i),
                    email: format!("list{}@example.com", i),
                    first_name: "List".to_string(),
                    last_name: format!("User{}", i),
                    enabled: true,
                    email_verified: true,
                    created_at: Utc::now(),
                    attributes: HashMap::from([(
                        "keycloak_sub".to_string(),
                        vec![format!("kc-sub-list-{}", i)],
                    )]),
                };
                repo.create(&user).await.unwrap();
            }

            let result = repo.list(1, 10, None, None).await.unwrap();
            assert_eq!(result.users.len(), 10);
            assert_eq!(result.pagination.total_count, 15);
            assert!(result.pagination.has_next);

            let result = repo.list(2, 10, None, None).await.unwrap();
            assert_eq!(result.users.len(), 5);
            assert!(!result.pagination.has_next);
        }

        #[sqlx::test(
            migrations = "../../database/auth-db/migrations"
        )]
        async fn test_user_list_with_search(pool: PgPool) {
            let repo = UserPostgresRepository::new(pool);

            let user1 = User {
                id: Uuid::new_v4().to_string(),
                username: "search.alice".to_string(),
                email: "alice@example.com".to_string(),
                first_name: "Alice".to_string(),
                last_name: "Smith".to_string(),
                enabled: true,
                email_verified: true,
                created_at: Utc::now(),
                attributes: HashMap::from([(
                    "keycloak_sub".to_string(),
                    vec!["kc-alice".to_string()],
                )]),
            };
            repo.create(&user1).await.unwrap();

            let user2 = User {
                id: Uuid::new_v4().to_string(),
                username: "search.bob".to_string(),
                email: "bob@example.com".to_string(),
                first_name: "Bob".to_string(),
                last_name: "Jones".to_string(),
                enabled: true,
                email_verified: true,
                created_at: Utc::now(),
                attributes: HashMap::from([(
                    "keycloak_sub".to_string(),
                    vec!["kc-bob".to_string()],
                )]),
            };
            repo.create(&user2).await.unwrap();

            let result = repo
                .list(1, 20, Some("alice".to_string()), None)
                .await
                .unwrap();
            assert_eq!(result.users.len(), 1);
            assert_eq!(result.users[0].username, "search.alice");
        }

        #[sqlx::test(
            migrations = "../../database/auth-db/migrations"
        )]
        async fn test_user_update(pool: PgPool) {
            let repo = UserPostgresRepository::new(pool);

            let user = User {
                id: Uuid::new_v4().to_string(),
                username: "update.user".to_string(),
                email: "update@example.com".to_string(),
                first_name: "Update".to_string(),
                last_name: "User".to_string(),
                enabled: true,
                email_verified: true,
                created_at: Utc::now(),
                attributes: HashMap::from([(
                    "keycloak_sub".to_string(),
                    vec!["kc-update".to_string()],
                )]),
            };

            let created = repo.create(&user).await.unwrap();

            let mut updated_user = created.clone();
            updated_user.first_name = "Updated".to_string();
            updated_user.last_name = "Name".to_string();
            updated_user.enabled = false;

            let updated = repo.update(&updated_user).await.unwrap();
            assert_eq!(updated.first_name, "Updated");
            assert_eq!(updated.last_name, "Name");
            assert!(!updated.enabled);
        }

        #[sqlx::test(
            migrations = "../../database/auth-db/migrations"
        )]
        async fn test_audit_log_create_and_search(pool: PgPool) {
            let repo = AuditLogPostgresRepository::new(pool);

            let log = AuditLog {
                id: Uuid::new_v4(),
                event_type: "LOGIN_SUCCESS".to_string(),
                user_id: "user-1".to_string(),
                ip_address: "192.168.1.100".to_string(),
                user_agent: "Mozilla/5.0".to_string(),
                resource: "/api/v1/auth/token".to_string(),
                action: "POST".to_string(),
                result: "SUCCESS".to_string(),
                metadata: HashMap::from([("client_id".to_string(), "react-spa".to_string())]),
                recorded_at: Utc::now(),
            };

            repo.create(&log).await.unwrap();

            let params = AuditLogSearchParams {
                user_id: Some("user-1".to_string()),
                page: 1,
                page_size: 20,
                ..Default::default()
            };

            let (logs, total) = repo.search(&params).await.unwrap();
            assert_eq!(total, 1);
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].event_type, "LOGIN_SUCCESS");
        }
    }
}
