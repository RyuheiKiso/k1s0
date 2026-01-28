//! k1s0-db トランザクションテスト
//!
//! トランザクション分離レベル、ロールバック、設定のテスト。

use k1s0_db::{
    DbConfig, DbError, IsolationLevel, TransactionMode, TransactionOptions, TransactionState,
};

/// 設定のテスト
mod config_tests {
    use super::*;
    use k1s0_db::{PoolConfig, SslMode, TimeoutConfig};

    #[test]
    fn test_db_config_builder() {
        let config = DbConfig::builder()
            .host("localhost")
            .port(5432)
            .database("testdb")
            .username("user")
            .password_file("/run/secrets/db_password")
            .build()
            .unwrap();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "testdb");
        assert_eq!(config.username, "user");
    }

    #[test]
    fn test_db_config_defaults() {
        let config = DbConfig::builder()
            .host("localhost")
            .database("testdb")
            .username("user")
            .build()
            .unwrap();

        assert_eq!(config.port, 5432);
        assert_eq!(config.pool.max_connections, k1s0_db::DEFAULT_MAX_CONNECTIONS);
    }

    #[test]
    fn test_pool_config() {
        let pool = PoolConfig {
            max_connections: 50,
            min_connections: 5,
            ..Default::default()
        };

        assert_eq!(pool.max_connections, 50);
        assert_eq!(pool.min_connections, 5);
    }

    #[test]
    fn test_timeout_config() {
        let timeout = TimeoutConfig {
            connect_timeout_ms: 5000,
            query_timeout_ms: 30000,
            idle_timeout_secs: 600,
            max_lifetime_secs: 1800,
        };

        assert_eq!(timeout.connect_timeout_ms, 5000);
        assert_eq!(timeout.query_timeout_ms, 30000);
    }

    #[test]
    fn test_ssl_mode() {
        assert_eq!(SslMode::Disable.as_str(), "disable");
        assert_eq!(SslMode::Prefer.as_str(), "prefer");
        assert_eq!(SslMode::Require.as_str(), "require");
        assert_eq!(SslMode::VerifyCa.as_str(), "verify-ca");
        assert_eq!(SslMode::VerifyFull.as_str(), "verify-full");
    }
}

/// トランザクションオプションのテスト
mod transaction_options_tests {
    use super::*;

    #[test]
    fn test_transaction_options_default() {
        let options = TransactionOptions::default();

        assert_eq!(options.isolation_level, IsolationLevel::ReadCommitted);
        assert_eq!(options.mode, TransactionMode::ReadWrite);
    }

    #[test]
    fn test_transaction_options_builder() {
        let options = TransactionOptions::new()
            .with_isolation_level(IsolationLevel::Serializable)
            .with_mode(TransactionMode::ReadOnly);

        assert_eq!(options.isolation_level, IsolationLevel::Serializable);
        assert_eq!(options.mode, TransactionMode::ReadOnly);
    }

    #[test]
    fn test_isolation_levels() {
        assert_eq!(IsolationLevel::ReadUncommitted.as_str(), "READ UNCOMMITTED");
        assert_eq!(IsolationLevel::ReadCommitted.as_str(), "READ COMMITTED");
        assert_eq!(IsolationLevel::RepeatableRead.as_str(), "REPEATABLE READ");
        assert_eq!(IsolationLevel::Serializable.as_str(), "SERIALIZABLE");
    }

    #[test]
    fn test_transaction_mode() {
        assert_eq!(TransactionMode::ReadOnly.as_str(), "READ ONLY");
        assert_eq!(TransactionMode::ReadWrite.as_str(), "READ WRITE");
    }

    #[test]
    fn test_transaction_state() {
        assert_eq!(TransactionState::Active.as_str(), "active");
        assert_eq!(TransactionState::Committed.as_str(), "committed");
        assert_eq!(TransactionState::RolledBack.as_str(), "rolled_back");
    }

    #[test]
    fn test_read_only_options() {
        let options = TransactionOptions::read_only();

        assert_eq!(options.mode, TransactionMode::ReadOnly);
        assert_eq!(options.isolation_level, IsolationLevel::ReadCommitted);
    }

    #[test]
    fn test_serializable_options() {
        let options = TransactionOptions::serializable();

        assert_eq!(options.isolation_level, IsolationLevel::Serializable);
        assert_eq!(options.mode, TransactionMode::ReadWrite);
    }
}

/// エラーのテスト
mod error_tests {
    use super::*;

    #[test]
    fn test_db_error_codes() {
        let conn_err = DbError::connection("Connection refused");
        assert_eq!(conn_err.error_code(), "DB_CONNECTION_ERROR");
        assert!(conn_err.is_retryable());

        let query_err = DbError::query("Syntax error");
        assert_eq!(query_err.error_code(), "DB_QUERY_ERROR");
        assert!(!query_err.is_retryable());

        let tx_err = DbError::transaction("Deadlock detected");
        assert_eq!(tx_err.error_code(), "DB_TRANSACTION_ERROR");
        assert!(tx_err.is_retryable());
    }

    #[test]
    fn test_db_error_specific_types() {
        let timeout = DbError::timeout(30000);
        assert_eq!(timeout.error_code(), "DB_TIMEOUT");
        assert!(timeout.is_retryable());

        let constraint = DbError::constraint_violation("unique_email", "Email already exists");
        assert_eq!(constraint.error_code(), "DB_CONSTRAINT_VIOLATION");
        assert!(!constraint.is_retryable());

        let not_found = DbError::not_found("User", "123");
        assert_eq!(not_found.error_code(), "DB_NOT_FOUND");
        assert!(!not_found.is_retryable());
    }

    #[test]
    fn test_db_error_serialization_conflict() {
        let err = DbError::serialization_conflict();
        assert_eq!(err.error_code(), "DB_SERIALIZATION_CONFLICT");
        assert!(err.is_retryable());
    }

    #[test]
    fn test_db_error_deadlock() {
        let err = DbError::deadlock();
        assert_eq!(err.error_code(), "DB_DEADLOCK");
        assert!(err.is_retryable());
    }
}

/// クエリビルダーのテスト
mod query_builder_tests {
    use k1s0_db::{SelectBuilder, InsertBuilder, UpdateBuilder, DeleteBuilder, Operator};

    #[test]
    fn test_select_builder() {
        let query = SelectBuilder::new("users")
            .columns(&["id", "name", "email"])
            .where_clause("status", Operator::Eq, "active")
            .where_clause("age", Operator::Gte, "18")
            .order_by("created_at", false)
            .limit(10)
            .offset(0)
            .build();

        assert!(query.sql.contains("SELECT"));
        assert!(query.sql.contains("users"));
        assert!(query.sql.contains("WHERE"));
        assert!(query.sql.contains("ORDER BY"));
        assert!(query.sql.contains("LIMIT"));
    }

    #[test]
    fn test_insert_builder() {
        let query = InsertBuilder::new("users")
            .column("name", "Alice")
            .column("email", "alice@example.com")
            .returning(&["id", "created_at"])
            .build();

        assert!(query.sql.contains("INSERT INTO"));
        assert!(query.sql.contains("users"));
        assert!(query.sql.contains("RETURNING"));
    }

    #[test]
    fn test_update_builder() {
        let query = UpdateBuilder::new("users")
            .set("name", "Bob")
            .set("email", "bob@example.com")
            .where_clause("id", Operator::Eq, "123")
            .returning(&["id", "updated_at"])
            .build();

        assert!(query.sql.contains("UPDATE"));
        assert!(query.sql.contains("SET"));
        assert!(query.sql.contains("WHERE"));
    }

    #[test]
    fn test_delete_builder() {
        let query = DeleteBuilder::new("users")
            .where_clause("id", Operator::Eq, "123")
            .returning(&["id"])
            .build();

        assert!(query.sql.contains("DELETE FROM"));
        assert!(query.sql.contains("WHERE"));
    }

    #[test]
    fn test_operators() {
        assert_eq!(Operator::Eq.as_str(), "=");
        assert_eq!(Operator::Ne.as_str(), "<>");
        assert_eq!(Operator::Lt.as_str(), "<");
        assert_eq!(Operator::Lte.as_str(), "<=");
        assert_eq!(Operator::Gt.as_str(), ">");
        assert_eq!(Operator::Gte.as_str(), ">=");
        assert_eq!(Operator::Like.as_str(), "LIKE");
        assert_eq!(Operator::In.as_str(), "IN");
        assert_eq!(Operator::IsNull.as_str(), "IS NULL");
        assert_eq!(Operator::IsNotNull.as_str(), "IS NOT NULL");
    }
}

/// リポジトリパターンのテスト
mod repository_tests {
    use k1s0_db::{Pagination, PagedResult, SortBy, SortDirection};

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(2, 20);

        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 20);
        assert_eq!(pagination.offset(), 20);
    }

    #[test]
    fn test_pagination_default() {
        let pagination = Pagination::default();

        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.per_page, 20);
        assert_eq!(pagination.offset(), 0);
    }

    #[test]
    fn test_paged_result() {
        let items = vec!["a", "b", "c"];
        let result = PagedResult::new(items.clone(), 100, Pagination::new(1, 20));

        assert_eq!(result.items, items);
        assert_eq!(result.total, 100);
        assert_eq!(result.page, 1);
        assert_eq!(result.per_page, 20);
        assert_eq!(result.total_pages, 5);
        assert!(result.has_next);
        assert!(!result.has_prev);
    }

    #[test]
    fn test_paged_result_last_page() {
        let items = vec!["x"];
        let result = PagedResult::new(items.clone(), 41, Pagination::new(3, 20));

        assert_eq!(result.total_pages, 3);
        assert!(!result.has_next);
        assert!(result.has_prev);
    }

    #[test]
    fn test_sort_by() {
        let sort = SortBy::new("created_at", SortDirection::Desc);

        assert_eq!(sort.column, "created_at");
        assert_eq!(sort.direction, SortDirection::Desc);
    }

    #[test]
    fn test_sort_direction() {
        assert_eq!(SortDirection::Asc.as_str(), "ASC");
        assert_eq!(SortDirection::Desc.as_str(), "DESC");
    }
}

/// メトリクスのテスト
mod metrics_tests {
    use k1s0_db::{QueryTimer, QueryType, QueryMetrics, QueryResult, DbMetrics};

    #[test]
    fn test_query_type() {
        assert_eq!(QueryType::Select.as_str(), "SELECT");
        assert_eq!(QueryType::Insert.as_str(), "INSERT");
        assert_eq!(QueryType::Update.as_str(), "UPDATE");
        assert_eq!(QueryType::Delete.as_str(), "DELETE");
        assert_eq!(QueryType::Transaction.as_str(), "TRANSACTION");
    }

    #[test]
    fn test_query_result() {
        assert_eq!(QueryResult::Success.as_str(), "success");
        assert_eq!(QueryResult::Error.as_str(), "error");
        assert_eq!(QueryResult::Timeout.as_str(), "timeout");
    }

    #[test]
    fn test_query_timer() {
        let metrics = DbMetrics::new();
        let timer = QueryTimer::start(&metrics, QueryType::Select);

        // 完了
        let query_metrics = timer.finish(QueryResult::Success, 1);

        assert_eq!(query_metrics.query_type, QueryType::Select);
        assert_eq!(query_metrics.result, QueryResult::Success);
        assert_eq!(query_metrics.rows_affected, 1);
        assert!(query_metrics.duration_ms > 0.0 || query_metrics.duration_ms == 0.0);
    }

    #[test]
    fn test_db_metrics() {
        let metrics = DbMetrics::new();

        metrics.record_query(QueryType::Select, QueryResult::Success, 10.0, 5);
        metrics.record_query(QueryType::Insert, QueryResult::Success, 5.0, 1);
        metrics.record_query(QueryType::Select, QueryResult::Error, 1.0, 0);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_queries, 3);
        assert_eq!(snapshot.successful_queries, 2);
        assert_eq!(snapshot.failed_queries, 1);
    }
}

/// ヘルスチェックのテスト
mod health_tests {
    use k1s0_db::{DbHealthConfig, DbHealthStatus};

    #[test]
    fn test_health_config_default() {
        let config = DbHealthConfig::default();

        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.check_interval_secs, 30);
    }

    #[test]
    fn test_health_status() {
        let status = DbHealthStatus {
            healthy: true,
            latency_ms: Some(5.0),
            connection_count: 10,
            max_connections: 50,
            message: None,
        };

        assert!(status.healthy);
        assert_eq!(status.connection_count, 10);
    }

    #[test]
    fn test_health_status_unhealthy() {
        let status = DbHealthStatus {
            healthy: false,
            latency_ms: None,
            connection_count: 0,
            max_connections: 50,
            message: Some("Connection refused".to_string()),
        };

        assert!(!status.healthy);
        assert!(status.message.is_some());
    }
}

/// マイグレーションのテスト
mod migration_tests {
    use k1s0_db::{Migration, MigrationConfig, MigrationDirection};

    #[test]
    fn test_migration() {
        let migration = Migration {
            version: 1,
            name: "create_users".to_string(),
            description: Some("Create users table".to_string()),
            up_sql: "CREATE TABLE users (id SERIAL PRIMARY KEY);".to_string(),
            down_sql: Some("DROP TABLE users;".to_string()),
            checksum: "abc123".to_string(),
        };

        assert_eq!(migration.version, 1);
        assert_eq!(migration.name, "create_users");
        assert!(migration.down_sql.is_some());
    }

    #[test]
    fn test_migration_config_default() {
        let config = MigrationConfig::default();

        assert_eq!(config.table_name, "schema_migrations");
        assert_eq!(config.migrations_dir, "migrations");
    }

    #[test]
    fn test_migration_direction() {
        assert_eq!(MigrationDirection::Up.as_str(), "up");
        assert_eq!(MigrationDirection::Down.as_str(), "down");
    }
}

/// テスト支援のテスト
mod testing_support_tests {
    use k1s0_db::{TestDbConfig, generate_test_db_name, Fixture};

    #[test]
    fn test_generate_test_db_name() {
        let name = generate_test_db_name("my_test");

        assert!(name.starts_with("test_my_test_"));
        assert!(name.len() > "test_my_test_".len());
    }

    #[test]
    fn test_test_db_config() {
        let config = TestDbConfig::new("localhost", 5432, "test_user", "test_password");

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.username, "test_user");
    }

    #[test]
    fn test_fixture() {
        let fixture = Fixture::new("users")
            .column("id", "1")
            .column("name", "Test User")
            .column("email", "test@example.com");

        assert_eq!(fixture.table, "users");
        assert_eq!(fixture.columns.len(), 3);
    }
}
