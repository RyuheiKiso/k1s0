use async_trait::async_trait;

use crate::{IdempotencyError, IdempotencyRecord, IdempotencyStatus};

#[cfg(feature = "postgres")]
use sqlx::Row;

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait IdempotencyStore: Send + Sync {
    /// Gets a record by key. Expired records should be treated as missing.
    async fn get(&self, key: &str) -> Result<Option<IdempotencyRecord>, IdempotencyError>;

    /// Stores a pending record. This is the canonical "create if absent" operation.
    async fn set(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError>;

    /// Marks a record as completed with the serialized response payload.
    async fn mark_completed(
        &self,
        key: &str,
        response_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError>;

    /// Marks a record as failed.
    async fn mark_failed(
        &self,
        key: &str,
        error_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError>;

    /// Deletes a record. Implementations that do not support delete may return `Ok(false)`.
    async fn delete(&self, key: &str) -> Result<bool, IdempotencyError>;

    /// Backward-compatible alias for legacy call sites.
    async fn insert(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError> {
        self.set(record).await
    }

    /// Backward-compatible alias for legacy call sites.
    async fn update(
        &self,
        key: &str,
        status: IdempotencyStatus,
        response_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        match status {
            IdempotencyStatus::Pending => {
                let mut record =
                    self.get(key)
                        .await?
                        .ok_or_else(|| IdempotencyError::NotFound {
                            key: key.to_string(),
                        })?;
                record.status = IdempotencyStatus::Pending;
                self.set(record).await
            }
            IdempotencyStatus::Completed => {
                self.mark_completed(key, response_body, response_status)
                    .await
            }
            IdempotencyStatus::Failed => {
                self.mark_failed(key, response_body, response_status).await
            }
        }
    }
}

#[derive(Clone)]
pub struct RedisIdempotencyStore {
    pub redis_url: String,
    #[cfg(feature = "redis")]
    pool: deadpool_redis::Pool,
    #[cfg(not(feature = "redis"))]
    inner: crate::memory::InMemoryIdempotencyStore,
}

impl RedisIdempotencyStore {
    pub async fn new(redis_url: impl Into<String>) -> Result<Self, IdempotencyError> {
        let redis_url = redis_url.into();
        if redis_url.trim().is_empty() {
            return Err(IdempotencyError::StorageError(
                "redis url must not be empty".to_string(),
            ));
        }

        #[cfg(feature = "redis")]
        {
            let cfg = deadpool_redis::Config::from_url(redis_url.clone());
            let pool = cfg
                .create_pool(Some(deadpool_redis::Runtime::Tokio1))
                .map_err(|e| {
                    IdempotencyError::StorageError(format!("failed to create redis pool: {e}"))
                })?;

            let mut conn = pool.get().await.map_err(|e| {
                IdempotencyError::StorageError(format!("failed to get redis connection: {e}"))
            })?;
            let _: String = deadpool_redis::redis::cmd("PING")
                .query_async(&mut conn)
                .await
                .map_err(|e| {
                    IdempotencyError::StorageError(format!("failed to ping redis: {e}"))
                })?;

            return Ok(Self { redis_url, pool });
        }

        #[cfg(not(feature = "redis"))]
        {
            Ok(Self {
                redis_url,
                inner: crate::memory::InMemoryIdempotencyStore::new(),
            })
        }
    }

    #[cfg(feature = "redis")]
    async fn connection(&self) -> Result<deadpool_redis::Connection, IdempotencyError> {
        self.pool.get().await.map_err(|e| {
            IdempotencyError::StorageError(format!("failed to get redis connection: {e}"))
        })
    }

    #[cfg(feature = "redis")]
    fn redis_key(key: &str) -> String {
        format!("idempotency:{key}")
    }
}

#[async_trait]
impl IdempotencyStore for RedisIdempotencyStore {
    async fn get(&self, key: &str) -> Result<Option<IdempotencyRecord>, IdempotencyError> {
        #[cfg(feature = "redis")]
        {
            use deadpool_redis::redis::AsyncCommands;

            let mut conn = self.connection().await?;
            let redis_key = Self::redis_key(key);
            let payload: Option<String> = conn.get(&redis_key).await.map_err(redis_error)?;

            let Some(payload) = payload else {
                return Ok(None);
            };

            let record: IdempotencyRecord = serde_json::from_str(&payload)?;
            if record.is_expired() {
                let _: i64 = conn.del(&redis_key).await.map_err(redis_error)?;
                return Ok(None);
            }

            return Ok(Some(record));
        }

        #[cfg(not(feature = "redis"))]
        {
            self.inner.get(key).await
        }
    }

    async fn set(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError> {
        #[cfg(feature = "redis")]
        {
            use deadpool_redis::redis::AsyncCommands;

            let mut conn = self.connection().await?;
            let redis_key = Self::redis_key(&record.key);
            let payload = serde_json::to_string(&record)?;

            let inserted: bool = conn
                .set_nx(&redis_key, payload)
                .await
                .map_err(redis_error)?;
            if !inserted {
                return Err(IdempotencyError::Duplicate {
                    key: record.key.clone(),
                });
            }

            if let Some(ttl) = remaining_ttl_secs(&record) {
                let _: bool = conn.expire(&redis_key, ttl).await.map_err(redis_error)?;
            }

            return Ok(());
        }

        #[cfg(not(feature = "redis"))]
        {
            self.inner.set(record).await
        }
    }

    async fn mark_completed(
        &self,
        key: &str,
        response_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        #[cfg(feature = "redis")]
        {
            use deadpool_redis::redis::Script;

            // M-3 監査対応: GET → 更新 → SET を Lua スクリプトでアトミックに実行し TOCTOU 競合を排除する。
            // 非アトミックな GET/SET 分離では並行リクエストが中間状態を上書きするリスクがあるため、
            // Lua スクリプトにより Redis サーバー側で単一アトミック操作として処理する。
            // KEYS[1]: idempotency プレフィックス付きキー
            // ARGV[1]: response_body（空文字列は JSON null として扱う）
            // ARGV[2]: response_status（"0" は JSON null として扱う）
            // ARGV[3]: completed_at（RFC3339 形式の文字列）
            let script = Script::new(
                r#"
local raw = redis.call('GET', KEYS[1])
if raw == false then
  return redis.error_reply('not_found')
end
local record = cjson.decode(raw)
record['status'] = 'Completed'
if ARGV[1] ~= '' then
  record['response_body'] = ARGV[1]
else
  record['response_body'] = cjson.null
end
if ARGV[2] ~= '0' then
  record['response_status'] = tonumber(ARGV[2])
else
  record['response_status'] = cjson.null
end
record['completed_at'] = ARGV[3]
redis.call('SET', KEYS[1], cjson.encode(record), 'KEEPTTL')
return 1
"#,
            );

            let redis_key = Self::redis_key(key);
            let response_body_arg = response_body.unwrap_or_default();
            let response_status_arg = response_status
                .map(|v| v.to_string())
                .unwrap_or_else(|| "0".to_string());
            let completed_at_arg = chrono::Utc::now().to_rfc3339();
            let mut conn = self.connection().await?;

            let result: Result<i32, _> = script
                .prepare_invoke()
                .key(&redis_key)
                .arg(&response_body_arg)
                .arg(&response_status_arg)
                .arg(&completed_at_arg)
                .invoke_async(&mut *conn)
                .await;

            return match result {
                Ok(_) => Ok(()),
                Err(e) if e.to_string().contains("not_found") => Err(IdempotencyError::NotFound {
                    key: key.to_string(),
                }),
                Err(e) => Err(redis_error(e)),
            };
        }

        #[cfg(not(feature = "redis"))]
        {
            self.inner
                .mark_completed(key, response_body, response_status)
                .await
        }
    }

    async fn mark_failed(
        &self,
        key: &str,
        error_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        #[cfg(feature = "redis")]
        {
            use deadpool_redis::redis::Script;

            // M-3 監査対応: GET → 更新 → SET を Lua スクリプトでアトミックに実行し TOCTOU 競合を排除する。
            // KEYS[1]: idempotency プレフィックス付きキー
            // ARGV[1]: error_body（空文字列は JSON null として扱う）
            // ARGV[2]: response_status（"0" は JSON null として扱う）
            // ARGV[3]: completed_at（RFC3339 形式の文字列）
            let script = Script::new(
                r#"
local raw = redis.call('GET', KEYS[1])
if raw == false then
  return redis.error_reply('not_found')
end
local record = cjson.decode(raw)
record['status'] = 'Failed'
if ARGV[1] ~= '' then
  record['response_body'] = ARGV[1]
else
  record['response_body'] = cjson.null
end
if ARGV[2] ~= '0' then
  record['response_status'] = tonumber(ARGV[2])
else
  record['response_status'] = cjson.null
end
record['completed_at'] = ARGV[3]
redis.call('SET', KEYS[1], cjson.encode(record), 'KEEPTTL')
return 1
"#,
            );

            let redis_key = Self::redis_key(key);
            let error_body_arg = error_body.unwrap_or_default();
            let response_status_arg = response_status
                .map(|v| v.to_string())
                .unwrap_or_else(|| "0".to_string());
            let completed_at_arg = chrono::Utc::now().to_rfc3339();
            let mut conn = self.connection().await?;

            let result: Result<i32, _> = script
                .prepare_invoke()
                .key(&redis_key)
                .arg(&error_body_arg)
                .arg(&response_status_arg)
                .arg(&completed_at_arg)
                .invoke_async(&mut *conn)
                .await;

            return match result {
                Ok(_) => Ok(()),
                Err(e) if e.to_string().contains("not_found") => Err(IdempotencyError::NotFound {
                    key: key.to_string(),
                }),
                Err(e) => Err(redis_error(e)),
            };
        }

        #[cfg(not(feature = "redis"))]
        {
            self.inner
                .mark_failed(key, error_body, response_status)
                .await
        }
    }

    async fn delete(&self, key: &str) -> Result<bool, IdempotencyError> {
        #[cfg(feature = "redis")]
        {
            use deadpool_redis::redis::AsyncCommands;

            let mut conn = self.connection().await?;
            let deleted: i64 = conn.del(Self::redis_key(key)).await.map_err(redis_error)?;
            return Ok(deleted > 0);
        }

        #[cfg(not(feature = "redis"))]
        {
            self.inner.delete(key).await
        }
    }
}

#[derive(Clone)]
pub struct PostgresIdempotencyStore {
    pub database_url: String,
    #[cfg(feature = "postgres")]
    pool: sqlx::PgPool,
    #[cfg(not(feature = "postgres"))]
    inner: crate::memory::InMemoryIdempotencyStore,
}

impl PostgresIdempotencyStore {
    /// 接続プールの最大接続数を指定して新しい PostgresIdempotencyStore を生成する。
    /// max_connections が None の場合はデフォルト値 (10) を使用する。
    pub async fn new(
        database_url: impl Into<String>,
        max_connections: Option<u32>,
    ) -> Result<Self, IdempotencyError> {
        let database_url = database_url.into();
        if database_url.trim().is_empty() {
            return Err(IdempotencyError::StorageError(
                "database url must not be empty".to_string(),
            ));
        }
        // H-02 監査対応: postgres フィーチャー無効時に max_connections が未使用になる警告を抑制する
        let _ = &max_connections;

        #[cfg(feature = "postgres")]
        {
            /// デフォルトの接続プールサイズ（未指定時に使用する）。postgres フィーチャー有効時のみ使用する。
            const DEFAULT_MAX_CONNECTIONS: u32 = 10;
            // 接続プールサイズを設定可能にすることで、環境に応じた最適化を可能にする
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(max_connections.unwrap_or(DEFAULT_MAX_CONNECTIONS))
                .connect(&database_url)
                .await
                .map_err(sqlx_error)?;

            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS idempotency_records (
                    key TEXT PRIMARY KEY,
                    status TEXT NOT NULL,
                    request_hash TEXT NULL,
                    response_body TEXT NULL,
                    response_status INTEGER NULL,
                    created_at TIMESTAMPTZ NOT NULL,
                    expires_at TIMESTAMPTZ NULL,
                    completed_at TIMESTAMPTZ NULL
                )
                "#,
            )
            .execute(&pool)
            .await
            .map_err(sqlx_error)?;

            return Ok(Self { database_url, pool });
        }

        #[cfg(not(feature = "postgres"))]
        {
            Ok(Self {
                database_url,
                inner: crate::memory::InMemoryIdempotencyStore::new(),
            })
        }
    }

    #[cfg(feature = "postgres")]
    fn record_from_row(row: sqlx::postgres::PgRow) -> Result<IdempotencyRecord, IdempotencyError> {
        let status: String = row.try_get("status").map_err(sqlx_error)?;
        let status = parse_status(&status)?;
        let response_status: Option<i32> = row.try_get("response_status").map_err(sqlx_error)?;
        Ok(IdempotencyRecord {
            key: row.try_get("key").map_err(sqlx_error)?,
            status,
            request_hash: row.try_get("request_hash").map_err(sqlx_error)?,
            response_body: row.try_get("response_body").map_err(sqlx_error)?,
            response_status: response_status.map(|v| v as u16),
            created_at: row.try_get("created_at").map_err(sqlx_error)?,
            expires_at: row.try_get("expires_at").map_err(sqlx_error)?,
            completed_at: row.try_get("completed_at").map_err(sqlx_error)?,
        })
    }
}

#[async_trait]
impl IdempotencyStore for PostgresIdempotencyStore {
    async fn get(&self, key: &str) -> Result<Option<IdempotencyRecord>, IdempotencyError> {
        #[cfg(feature = "postgres")]
        {
            let row = sqlx::query(
                r#"
                SELECT key, status, request_hash, response_body, response_status, created_at, expires_at, completed_at
                FROM idempotency_records
                WHERE key = $1
                "#,
            )
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_error)?;

            let Some(row) = row else {
                return Ok(None);
            };

            let record = Self::record_from_row(row)?;
            if record.is_expired() {
                let _ = self.delete(key).await?;
                return Ok(None);
            }
            return Ok(Some(record));
        }

        #[cfg(not(feature = "postgres"))]
        {
            self.inner.get(key).await
        }
    }

    async fn set(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError> {
        #[cfg(feature = "postgres")]
        {
            let result = sqlx::query(
                r#"
                INSERT INTO idempotency_records (
                    key, status, request_hash, response_body, response_status, created_at, expires_at, completed_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(&record.key)
            .bind(status_to_str(record.status.clone()))
            .bind(&record.request_hash)
            .bind(&record.response_body)
            .bind(record.response_status.map(i32::from))
            .bind(record.created_at)
            .bind(record.expires_at)
            .bind(record.completed_at)
            .execute(&self.pool)
            .await;

            match result {
                Ok(_) => Ok(()),
                Err(e) if is_unique_violation(&e) => Err(IdempotencyError::Duplicate {
                    key: record.key.clone(),
                }),
                Err(e) => Err(sqlx_error(e)),
            }?;

            return Ok(());
        }

        #[cfg(not(feature = "postgres"))]
        {
            self.inner.set(record).await
        }
    }

    async fn mark_completed(
        &self,
        key: &str,
        response_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        #[cfg(feature = "postgres")]
        {
            let result = sqlx::query(
                r#"
                UPDATE idempotency_records
                SET status = $2, response_body = $3, response_status = $4, completed_at = $5
                WHERE key = $1
                "#,
            )
            .bind(key)
            .bind(status_to_str(IdempotencyStatus::Completed))
            .bind(response_body)
            .bind(response_status.map(i32::from))
            .bind(Some(chrono::Utc::now()))
            .execute(&self.pool)
            .await
            .map_err(sqlx_error)?;

            if result.rows_affected() == 0 {
                return Err(IdempotencyError::NotFound {
                    key: key.to_string(),
                });
            }
            return Ok(());
        }

        #[cfg(not(feature = "postgres"))]
        {
            self.inner
                .mark_completed(key, response_body, response_status)
                .await
        }
    }

    async fn mark_failed(
        &self,
        key: &str,
        error_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        #[cfg(feature = "postgres")]
        {
            let result = sqlx::query(
                r#"
                UPDATE idempotency_records
                SET status = $2, response_body = $3, response_status = $4, completed_at = $5
                WHERE key = $1
                "#,
            )
            .bind(key)
            .bind(status_to_str(IdempotencyStatus::Failed))
            .bind(error_body)
            .bind(response_status.map(i32::from))
            .bind(Some(chrono::Utc::now()))
            .execute(&self.pool)
            .await
            .map_err(sqlx_error)?;

            if result.rows_affected() == 0 {
                return Err(IdempotencyError::NotFound {
                    key: key.to_string(),
                });
            }
            return Ok(());
        }

        #[cfg(not(feature = "postgres"))]
        {
            self.inner
                .mark_failed(key, error_body, response_status)
                .await
        }
    }

    async fn delete(&self, key: &str) -> Result<bool, IdempotencyError> {
        #[cfg(feature = "postgres")]
        {
            let result = sqlx::query("DELETE FROM idempotency_records WHERE key = $1")
                .bind(key)
                .execute(&self.pool)
                .await
                .map_err(sqlx_error)?;
            return Ok(result.rows_affected() > 0);
        }

        #[cfg(not(feature = "postgres"))]
        {
            self.inner.delete(key).await
        }
    }
}

#[cfg(feature = "redis")]
fn remaining_ttl_secs(record: &IdempotencyRecord) -> Option<i64> {
    record.expires_at.map(|expires_at| {
        let now = chrono::Utc::now();
        if expires_at <= now {
            1
        } else {
            (expires_at - now).num_seconds().max(1)
        }
    })
}

#[cfg(feature = "postgres")]
fn status_to_str(status: IdempotencyStatus) -> &'static str {
    match status {
        IdempotencyStatus::Pending => "pending",
        IdempotencyStatus::Completed => "completed",
        IdempotencyStatus::Failed => "failed",
    }
}

#[cfg(feature = "postgres")]
fn parse_status(raw: &str) -> Result<IdempotencyStatus, IdempotencyError> {
    match raw {
        "pending" | "PENDING" => Ok(IdempotencyStatus::Pending),
        "completed" | "COMPLETED" => Ok(IdempotencyStatus::Completed),
        "failed" | "FAILED" => Ok(IdempotencyStatus::Failed),
        _ => Err(IdempotencyError::StorageError(format!(
            "unknown idempotency status: {raw}"
        ))),
    }
}

#[cfg(feature = "redis")]
fn redis_error(err: deadpool_redis::redis::RedisError) -> IdempotencyError {
    IdempotencyError::StorageError(format!("redis error: {err}"))
}

#[cfg(feature = "postgres")]
fn sqlx_error(err: sqlx::Error) -> IdempotencyError {
    IdempotencyError::StorageError(format!("postgres error: {err}"))
}

#[cfg(feature = "postgres")]
fn is_unique_violation(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => db_err.is_unique_violation(),
        _ => false,
    }
}
