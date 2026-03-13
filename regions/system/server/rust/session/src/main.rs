mod adapter;
mod domain;
mod error;
mod infrastructure;
#[allow(dead_code)]
mod proto;
mod usecase;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::startup::run().await
}
