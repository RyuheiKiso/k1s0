pub mod adapter;
pub mod domain;
pub mod infrastructure;
pub mod proto;
pub mod usecase;

/// workflow-db マイグレーションを起動時に自動適用するためのマイグレーターを定義する
/// `sqlx::migrate`!() はコンパイル時にマイグレーションファイルを埋め込む
pub static MIGRATOR: sqlx::migrate::Migrator =
    sqlx::migrate!("../../../database/workflow-db/migrations");
