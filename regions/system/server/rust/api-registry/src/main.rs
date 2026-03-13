#[tokio::main]
async fn main() -> anyhow::Result<()> {
    k1s0_api_registry_server::infrastructure::startup::run().await
}
