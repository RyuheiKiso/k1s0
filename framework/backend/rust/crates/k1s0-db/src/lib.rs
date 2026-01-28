//! k1s0-db: データベースアクセス標準化
//!
//! このクレートは、k1s0 フレームワークにおけるデータベースアクセスの
//! 標準化されたインターフェースを提供する。
//!
//! ## 機能
//!
//! - **接続設定**: データベース接続、プール、タイムアウトの設定
//! - **トランザクション**: トランザクション境界とUnit of Workパターン
//! - **エラー処理**: 統一されたエラー型とステータスコード変換
//! - **メトリクス**: クエリの計測とパフォーマンス監視
//! - **マイグレーション**: スキーママイグレーションの管理
//!
//! ## 設計原則
//!
//! このクレートは domain/application 層で使用するインターフェースを定義する。
//! 実際のデータベース実装（sqlx、diesel など）は infrastructure 層で行う。
//!
//! ## 使用例
//!
//! ```rust,ignore
//! use k1s0_db::{DbConfig, DbError, DbResult};
//! use k1s0_db::tx::{TransactionOptions, IsolationLevel};
//! use k1s0_db::metrics::QueryTimer;
//!
//! // 設定の作成
//! let config = DbConfig::builder()
//!     .host("localhost")
//!     .database("myapp")
//!     .username("app_user")
//!     .password_file("/run/secrets/db_password")
//!     .build()?;
//!
//! // トランザクションオプション
//! let options = TransactionOptions::new()
//!     .with_isolation_level(IsolationLevel::Serializable);
//!
//! // クエリ計測
//! let timer = QueryTimer::start(QueryType::Select);
//! // ... クエリ実行 ...
//! let metrics = timer.finish_success();
//! ```

pub mod config;
pub mod error;
pub mod health;
pub mod metrics;
pub mod migration;
pub mod pool;
pub mod query;
pub mod repository;
pub mod setting;
pub mod testing;
pub mod tx;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "postgres")]
pub mod uow;

// 主要な型の再エクスポート
pub use config::{
    DbConfig, DbConfigBuilder, PoolConfig, SslMode, TimeoutConfig, DEFAULT_CONNECT_TIMEOUT_MS,
    DEFAULT_IDLE_TIMEOUT_SECS, DEFAULT_MAX_CONNECTIONS, DEFAULT_MAX_LIFETIME_SECS,
    DEFAULT_MIN_CONNECTIONS, DEFAULT_QUERY_TIMEOUT_MS,
};
pub use error::{DbError, DbResult};
pub use metrics::{DbMetrics, DbSpanLabels, QueryMetrics, QueryResult, QueryTimer, QueryType};
pub use migration::{
    load_migrations, AppliedMigration, Migration, MigrationConfig, MigrationDirection,
    MigrationResult, MigrationRunner,
};
pub use pool::{from_env, from_env_with_prefix, DbPoolBuilder};
pub use tx::{
    IsolationLevel, TransactionExecutor, TransactionMode, TransactionOptions, TransactionState,
    UnitOfWork,
};

// クエリビルダー
pub use query::{
    BuiltQuery, DeleteBuilder, InsertBuilder, Operator, SelectBuilder, UpdateBuilder, WhereClause,
};

// リポジトリパターン
pub use repository::{
    BulkRepository, FilterableRepository, PagedResult, Pagination, Repository,
    SoftDeleteRepository, SortBy, SortDirection, SortableRepository,
};

// ヘルスチェック
pub use health::{DbHealthConfig, DbHealthStatus, HealthCheckable};

// テスト支援
pub use testing::{generate_test_db_name, Fixture, TestDbConfig};

// PostgreSQL 固有の型
#[cfg(feature = "postgres")]
pub use postgres::{create_pool, create_pool_from_url, PgPool, PoolStatus, PostgresPool};

#[cfg(feature = "postgres")]
pub use uow::{
    execute_in_transaction, execute_with_retry, MultiTableUnitOfWork, PostgresUnitOfWork,
    RepositoryContext, ScopedUnitOfWork, UnitOfWorkFactory,
};

#[cfg(feature = "postgres")]
pub use health::DbHealthChecker;

#[cfg(feature = "postgres")]
pub use testing::{FixtureLoader, TestTransaction};

// fw_m_setting 設定リポジトリ
pub use setting::{MIGRATION_SQL as SETTING_MIGRATION_SQL, ROLLBACK_SQL as SETTING_ROLLBACK_SQL};

#[cfg(all(feature = "postgres", feature = "config"))]
pub use setting::{
    PostgresSettingRepository, PostgresSettingRepositoryBuilder, PostgresSettingWriter,
};
