mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

/// H-02 監査対応: bin クレートのルートで MIGRATOR を定義し startup.rs の crate::MIGRATOR 参照を解決する
/// lib.rs にも同様の定義があるが、bin クレートと lib クレートはコンパイル単位が異なるため両方に必要
pub static MIGRATOR: sqlx::migrate::Migrator =
    sqlx::migrate!("../../../database/scheduler-db/migrations");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::startup::run().await
}
