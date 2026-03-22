// タスクサーバークレートルート。
pub mod adapter;
pub mod domain;
pub mod infrastructure;
pub mod proto;
pub mod usecase;

// タスクサービス DB マイグレーター
pub static MIGRATOR: sqlx::migrate::Migrator =
    sqlx::migrate!("../../../database/postgres/migrations");
