// タスクサーバーのエントリポイント。
use k1s0_task_server::infrastructure;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::startup::run().await
}
