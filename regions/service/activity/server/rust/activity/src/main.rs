// アクティビティサーバーのエントリポイント。
use k1s0_activity_server::infrastructure;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::startup::run().await
}
