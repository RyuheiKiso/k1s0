use k1s0_payment_server::infrastructure;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::startup::run().await
}
