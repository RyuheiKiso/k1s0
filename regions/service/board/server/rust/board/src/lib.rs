// ボードサーバークレートルート。
pub mod adapter;
pub mod domain;
pub mod infrastructure;
pub mod proto;
pub mod usecase;

// ボードサービス DB マイグレーター
pub static MIGRATOR: sqlx::migrate::Migrator =
    sqlx::migrate!("../../../database/postgres/migrations");
