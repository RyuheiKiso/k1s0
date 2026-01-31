use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;

use crate::models::item::Item;

pub async fn list_items(State(pool): State<PgPool>) -> Result<Json<Vec<Item>>, StatusCode> {
    let items = sqlx::query_as!(Item, "SELECT id, name, description FROM items")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(items))
}

pub async fn get_item(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<Item>, StatusCode> {
    let item = sqlx::query_as!(Item, "SELECT id, name, description FROM items WHERE id = $1", id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(item))
}

pub async fn create_item(
    State(pool): State<PgPool>,
    Json(input): Json<Item>,
) -> Result<(StatusCode, Json<Item>), StatusCode> {
    let item = sqlx::query_as!(
        Item,
        "INSERT INTO items (name, description) VALUES ($1, $2) RETURNING id, name, description",
        input.name,
        input.description
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn delete_item(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query!("DELETE FROM items WHERE id = $1", id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
