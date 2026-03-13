mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::startup::run().await
}
