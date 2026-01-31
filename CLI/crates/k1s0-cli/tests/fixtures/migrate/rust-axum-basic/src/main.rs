use axum::{routing::get, Router};
use std::net::SocketAddr;

mod db;
mod handlers;
mod models;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = db::pool::create_pool(&database_url).await;

    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/items", get(handlers::items::list_items).post(handlers::items::create_item))
        .route("/items/:id", get(handlers::items::get_item).delete(handlers::items::delete_item))
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
