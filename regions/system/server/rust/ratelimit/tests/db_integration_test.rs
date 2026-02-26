//! Database integration tests using testcontainers.
//! These tests spin up a real PostgreSQL container and run migrations.
//! Requires Docker to be available.

#[cfg(test)]
mod testcontainers_db_tests {
    use sqlx::PgPool;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;

    use k1s0_ratelimit_server::adapter::repository::ratelimit_postgres::RateLimitPostgresRepository;
    use k1s0_ratelimit_server::domain::entity::{Algorithm, RateLimitRule};
    use k1s0_ratelimit_server::domain::repository::RateLimitRepository;

    async fn setup_pool() -> (PgPool, testcontainers::ContainerAsync<Postgres>) {
        let container = Postgres::default().start().await.unwrap();
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let connection_string = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );

        let pool = PgPool::connect(&connection_string).await.unwrap();

        // Run migrations
        sqlx::migrate!("../../../database/ratelimit-db/migrations")
            .run(&pool)
            .await
            .unwrap();

        (pool, container)
    }

    #[tokio::test]
    async fn test_rate_limit_rule_crud() {
        let (pool, _container) = setup_pool().await;
        let repo = RateLimitPostgresRepository::new(pool);

        let rule = RateLimitRule::new(
            "api-service".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        );

        let created = repo.create(&rule).await.unwrap();
        assert_eq!(created.scope, "api-service");
        assert_eq!(created.identifier_pattern, "global");
        assert_eq!(created.limit, 100);
        assert_eq!(created.window_seconds, 60);
        assert_eq!(created.algorithm, Algorithm::TokenBucket);
        assert!(created.enabled);
    }

    #[tokio::test]
    async fn test_find_rate_limit_rule_by_id() {
        let (pool, _container) = setup_pool().await;
        let repo = RateLimitPostgresRepository::new(pool);

        let rule = RateLimitRule::new(
            "find-by-id-service".to_string(),
            "user:*".to_string(),
            50,
            30,
            Algorithm::FixedWindow,
        );

        let created = repo.create(&rule).await.unwrap();
        let found = repo.find_by_id(&created.id).await.unwrap();

        assert_eq!(found.id, created.id);
        assert_eq!(found.scope, "find-by-id-service");
        assert_eq!(found.algorithm, Algorithm::FixedWindow);
    }

    #[tokio::test]
    async fn test_find_rate_limit_rule_by_name() {
        let (pool, _container) = setup_pool().await;
        let repo = RateLimitPostgresRepository::new(pool);

        let rule = RateLimitRule::new(
            "named-rule".to_string(),
            "ip:*".to_string(),
            200,
            3600,
            Algorithm::SlidingWindow,
        );

        repo.create(&rule).await.unwrap();

        let found = repo.find_by_name("named-rule").await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.scope, "named-rule");
        assert_eq!(found.algorithm, Algorithm::SlidingWindow);
    }

    #[tokio::test]
    async fn test_find_all_rate_limit_rules() {
        let (pool, _container) = setup_pool().await;
        let repo = RateLimitPostgresRepository::new(pool);

        for i in 0..3 {
            let rule = RateLimitRule::new(
                format!("list-service-{}", i),
                "global".to_string(),
                100,
                60,
                Algorithm::TokenBucket,
            );
            repo.create(&rule).await.unwrap();
        }

        let all = repo.find_all().await.unwrap();
        assert!(all.len() >= 3);
    }

    #[tokio::test]
    async fn test_update_rate_limit_rule() {
        let (pool, _container) = setup_pool().await;
        let repo = RateLimitPostgresRepository::new(pool);

        let rule = RateLimitRule::new(
            "update-service".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        );

        let mut created = repo.create(&rule).await.unwrap();
        created.limit = 200;
        created.enabled = false;

        repo.update(&created).await.unwrap();

        let updated = repo.find_by_id(&created.id).await.unwrap();
        assert_eq!(updated.limit, 200);
        assert!(!updated.enabled);
    }

    #[tokio::test]
    async fn test_delete_rate_limit_rule() {
        let (pool, _container) = setup_pool().await;
        let repo = RateLimitPostgresRepository::new(pool);

        let rule = RateLimitRule::new(
            "delete-service".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        );

        let created = repo.create(&rule).await.unwrap();
        let deleted = repo.delete(&created.id).await.unwrap();
        assert!(deleted);

        let not_found = repo.find_by_name("delete-service").await.unwrap();
        assert!(not_found.is_none());
    }
}
