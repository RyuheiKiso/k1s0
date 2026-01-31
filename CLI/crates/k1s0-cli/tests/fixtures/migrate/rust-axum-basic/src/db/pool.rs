use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> PgPool {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    println!("JWT secret loaded: {jwt_secret}");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to create pool")
}
