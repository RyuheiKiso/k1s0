// プロジェクトマスタサーバーのクレートルート。
// 各レイヤーのモジュールを公開し、DB マイグレーターを定義する。
pub mod adapter;
pub mod domain;
pub mod infrastructure;
pub mod proto;
pub mod usecase;

// タスク管理ドメインのプロジェクトマスタ DB マイグレーター
pub static MIGRATOR: sqlx::migrate::Migrator =
    sqlx::migrate!("../../../database/project-master-db/migrations");
