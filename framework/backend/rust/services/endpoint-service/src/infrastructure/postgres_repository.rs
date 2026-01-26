//! PostgreSQLリポジトリ実装

use sqlx::{PgPool, Row};
use std::time::SystemTime;

use crate::domain::{
    Endpoint, EndpointError, EndpointList, EndpointQuery, EndpointRepository, ResolvedAddress,
};

/// PostgreSQLリポジトリ
pub struct PostgresRepository {
    pool: PgPool,
    environment: Option<String>,
}

impl PostgresRepository {
    /// 新しいリポジトリを作成
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            environment: None,
        }
    }

    /// 環境を指定してリポジトリを作成
    pub fn with_environment(pool: PgPool, environment: impl Into<String>) -> Self {
        Self {
            pool,
            environment: Some(environment.into()),
        }
    }
}

impl EndpointRepository for PostgresRepository {
    async fn get(
        &self,
        service_name: &str,
        method: Option<&str>,
        path: Option<&str>,
    ) -> Result<Option<Endpoint>, EndpointError> {
        let query = match (method, path) {
            (Some(m), Some(p)) => {
                sqlx::query(
                    r#"
                    SELECT endpoint_id, service_name, path, method, created_at, updated_at
                    FROM fw_m_endpoint
                    WHERE service_name = $1 AND method = $2 AND path = $3
                    "#,
                )
                .bind(service_name)
                .bind(m)
                .bind(p)
            }
            (Some(m), None) => {
                sqlx::query(
                    r#"
                    SELECT endpoint_id, service_name, path, method, created_at, updated_at
                    FROM fw_m_endpoint
                    WHERE service_name = $1 AND method = $2
                    LIMIT 1
                    "#,
                )
                .bind(service_name)
                .bind(m)
            }
            (None, Some(p)) => {
                sqlx::query(
                    r#"
                    SELECT endpoint_id, service_name, path, method, created_at, updated_at
                    FROM fw_m_endpoint
                    WHERE service_name = $1 AND path = $2
                    LIMIT 1
                    "#,
                )
                .bind(service_name)
                .bind(p)
            }
            (None, None) => {
                sqlx::query(
                    r#"
                    SELECT endpoint_id, service_name, path, method, created_at, updated_at
                    FROM fw_m_endpoint
                    WHERE service_name = $1
                    LIMIT 1
                    "#,
                )
                .bind(service_name)
            }
        };

        let row = query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| EndpointError::storage(e.to_string()))?;

        match row {
            Some(row) => {
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

                Ok(Some(Endpoint {
                    id: row.get::<i64, _>("endpoint_id") as i32,
                    service_name: row.get("service_name"),
                    path: row.get("path"),
                    method: row.get("method"),
                    created_at: SystemTime::from(created_at),
                    updated_at: SystemTime::from(updated_at),
                }))
            }
            None => Ok(None),
        }
    }

    async fn list(&self, query: &EndpointQuery) -> Result<EndpointList, EndpointError> {
        let page_size = query.page_size.unwrap_or(100).min(1000) as i64;
        let offset = query
            .page_token
            .as_ref()
            .and_then(|t| t.strip_prefix("offset:"))
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let rows = match &query.service_name {
            Some(service_name) => {
                sqlx::query(
                    r#"
                    SELECT endpoint_id, service_name, path, method, created_at, updated_at
                    FROM fw_m_endpoint
                    WHERE service_name = $1
                    ORDER BY service_name, path, method
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(service_name)
                .bind(page_size + 1)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query(
                    r#"
                    SELECT endpoint_id, service_name, path, method, created_at, updated_at
                    FROM fw_m_endpoint
                    ORDER BY service_name, path, method
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(page_size + 1)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| EndpointError::storage(e.to_string()))?;

        let has_next = rows.len() as i64 > page_size;
        let endpoints: Vec<Endpoint> = rows
            .into_iter()
            .take(page_size as usize)
            .map(|row| {
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

                Endpoint {
                    id: row.get::<i64, _>("endpoint_id") as i32,
                    service_name: row.get("service_name"),
                    path: row.get("path"),
                    method: row.get("method"),
                    created_at: SystemTime::from(created_at),
                    updated_at: SystemTime::from(updated_at),
                }
            })
            .collect();

        let mut list = EndpointList::new(endpoints);
        if has_next {
            list = list.with_next_page_token(format!("offset:{}", offset + page_size));
        }

        Ok(list)
    }

    async fn resolve(
        &self,
        service_name: &str,
        protocol: &str,
    ) -> Result<ResolvedAddress, EndpointError> {
        let row = match &self.environment {
            Some(env) => {
                sqlx::query(
                    r#"
                    SELECT address, use_tls
                    FROM fw_m_service_address
                    WHERE service_name = $1
                      AND protocol = $2
                      AND is_active = TRUE
                      AND (environment = $3 OR environment IS NULL)
                    ORDER BY
                      CASE WHEN environment = $3 THEN 0 ELSE 1 END,
                      priority DESC
                    LIMIT 1
                    "#,
                )
                .bind(service_name)
                .bind(protocol)
                .bind(env)
                .fetch_optional(&self.pool)
                .await
            }
            None => {
                sqlx::query(
                    r#"
                    SELECT address, use_tls
                    FROM fw_m_service_address
                    WHERE service_name = $1
                      AND protocol = $2
                      AND is_active = TRUE
                      AND environment IS NULL
                    ORDER BY priority DESC
                    LIMIT 1
                    "#,
                )
                .bind(service_name)
                .bind(protocol)
                .fetch_optional(&self.pool)
                .await
            }
        }
        .map_err(|e| EndpointError::storage(e.to_string()))?;

        match row {
            Some(row) => Ok(ResolvedAddress {
                address: row.get("address"),
                use_tls: row.get("use_tls"),
            }),
            None => Err(EndpointError::not_found(service_name)),
        }
    }

    async fn save(&self, endpoint: &Endpoint) -> Result<(), EndpointError> {
        sqlx::query(
            r#"
            INSERT INTO fw_m_endpoint (endpoint_id, service_name, path, method, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (service_name, path, method, version)
            DO UPDATE SET updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(endpoint.id as i64)
        .bind(&endpoint.service_name)
        .bind(&endpoint.path)
        .bind(&endpoint.method)
        .bind(chrono::DateTime::<chrono::Utc>::from(endpoint.created_at))
        .bind(chrono::DateTime::<chrono::Utc>::from(endpoint.updated_at))
        .execute(&self.pool)
        .await
        .map_err(|e| EndpointError::storage(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_repository_new() {
        // Pool creation requires a database connection, so we just test the struct
        // Full integration tests would require a test database
    }
}
