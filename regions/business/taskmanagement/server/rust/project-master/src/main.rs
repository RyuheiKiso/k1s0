// プロジェクトマスタサーバーのエントリポイント。
// 全初期化処理は infrastructure::startup::run() に委譲する。
use k1s0_project_master_server::infrastructure;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::startup::run().await
}
