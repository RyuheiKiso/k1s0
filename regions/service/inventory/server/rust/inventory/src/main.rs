use k1s0_inventory_server::infrastructure;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::startup::run().await
}
