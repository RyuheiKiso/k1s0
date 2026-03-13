use k1s0_master_maintenance_server::infrastructure;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::startup::run().await
}
