// ボードサーバーのエントリポイント。
use k1s0_board_server::infrastructure;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::startup::run().await
}
