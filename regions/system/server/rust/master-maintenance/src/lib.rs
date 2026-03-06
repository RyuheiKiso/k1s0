pub mod adapter;
pub mod domain;
pub mod infrastructure;
pub mod proto;
pub mod usecase;

pub static MIGRATOR: sqlx::migrate::Migrator =
    sqlx::migrate!("../../../database/master-maintenance-db/migrations");
