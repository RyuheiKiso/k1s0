//! Database integration tests using testcontainers.
//! These tests spin up a real PostgreSQL container and run migrations.
//! Requires Docker to be available.

#[cfg(test)]
mod testcontainers_db_tests {
    use sqlx::PgPool;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;
    use uuid::Uuid;

    use k1s0_auth_server::adapter::repository::audit_log_postgres::AuditLogPostgresRepository;
    use k1s0_auth_server::domain::entity::audit_log::{AuditLog, AuditLogSearchParams};
    use k1s0_auth_server::domain::repository::AuditLogRepository;

    async fn setup_pool() -> (PgPool, testcontainers::ContainerAsync<Postgres>) {
        let container = Postgres::default().start().await.unwrap();
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let connection_string = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );

        let pool = PgPool::connect(&connection_string).await.unwrap();

        // Run migrations
        sqlx::migrate!("../../../database/auth-db/migrations")
            .run(&pool)
            .await
            .unwrap();

        (pool, container)
    }

    #[tokio::test]
    async fn test_audit_log_crud_with_real_db() {
        let (pool, _container) = setup_pool().await;
        let repo = AuditLogPostgresRepository::new(pool);

        let log = AuditLog {
            id: Uuid::new_v4(),
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: "user-tc-1".to_string(),
            ip_address: "10.0.0.1".to_string(),
            user_agent: "test-agent".to_string(),
            resource: "/api/v1/auth/token".to_string(),
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            resource_id: None,
            detail: Some(serde_json::json!({"source": "testcontainers"})),
            trace_id: Some("trace-tc-001".to_string()),
            created_at: chrono::Utc::now(),
        };

        repo.create(&log).await.unwrap();

        let params = AuditLogSearchParams {
            user_id: Some("user-tc-1".to_string()),
            page: 1,
            page_size: 10,
            ..Default::default()
        };

        let (logs, total) = repo.search(&params).await.unwrap();
        assert_eq!(total, 1);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].event_type, "LOGIN_SUCCESS");
        assert_eq!(logs[0].user_id, "user-tc-1");
        assert_eq!(logs[0].detail.as_ref().unwrap()["source"], "testcontainers");
    }

    #[tokio::test]
    async fn test_audit_log_search_pagination_with_real_db() {
        let (pool, _container) = setup_pool().await;
        let repo = AuditLogPostgresRepository::new(pool);

        for i in 0..5 {
            let log = AuditLog {
                id: Uuid::new_v4(),
                event_type: format!("EVENT_{}", i),
                user_id: "user-tc-paginate".to_string(),
                ip_address: "10.0.0.1".to_string(),
                user_agent: "test-agent".to_string(),
                resource: "/test".to_string(),
                action: "GET".to_string(),
                result: "SUCCESS".to_string(),
                resource_id: None,
                detail: None,
                trace_id: None,
                created_at: chrono::Utc::now(),
            };
            repo.create(&log).await.unwrap();
        }

        let params = AuditLogSearchParams {
            user_id: Some("user-tc-paginate".to_string()),
            page: 1,
            page_size: 3,
            ..Default::default()
        };

        let (logs, total) = repo.search(&params).await.unwrap();
        assert_eq!(total, 5);
        assert_eq!(logs.len(), 3);

        let params_page2 = AuditLogSearchParams {
            user_id: Some("user-tc-paginate".to_string()),
            page: 2,
            page_size: 3,
            ..Default::default()
        };

        let (logs2, total2) = repo.search(&params_page2).await.unwrap();
        assert_eq!(total2, 5);
        assert_eq!(logs2.len(), 2);
    }

    #[tokio::test]
    async fn test_audit_log_search_by_event_type_with_real_db() {
        let (pool, _container) = setup_pool().await;
        let repo = AuditLogPostgresRepository::new(pool);

        for event_type in &["LOGIN_SUCCESS", "LOGIN_FAILURE", "TOKEN_VALIDATE"] {
            let log = AuditLog {
                id: Uuid::new_v4(),
                event_type: event_type.to_string(),
                user_id: "user-tc-filter".to_string(),
                ip_address: "10.0.0.1".to_string(),
                user_agent: "test-agent".to_string(),
                resource: "/test".to_string(),
                action: "POST".to_string(),
                result: "SUCCESS".to_string(),
                resource_id: None,
                detail: None,
                trace_id: None,
                created_at: chrono::Utc::now(),
            };
            repo.create(&log).await.unwrap();
        }

        let params = AuditLogSearchParams {
            user_id: Some("user-tc-filter".to_string()),
            event_type: Some("LOGIN_SUCCESS".to_string()),
            page: 1,
            page_size: 10,
            ..Default::default()
        };

        let (logs, total) = repo.search(&params).await.unwrap();
        assert_eq!(total, 1);
        assert_eq!(logs[0].event_type, "LOGIN_SUCCESS");
    }
}
