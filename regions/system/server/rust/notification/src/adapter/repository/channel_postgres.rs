use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::notification_channel::NotificationChannel;
use crate::domain::repository::NotificationChannelRepository;

/// C-005 監査対応: AES-256-GCM 暗号化・復号化関数をインポートする
use k1s0_encryption::aes::{aes_decrypt, aes_encrypt};

/// チャンネルの PostgreSQL リポジトリ実装
/// C-005: channels.config を AES-256-GCM で暗号化して保存する
/// H-012: tenant_id カラムによるマルチテナント分離を実装する
/// H-010: RLS（Row Level Security）と set_config によるテナント境界を強制する
pub struct ChannelPostgresRepository {
    pool: Arc<PgPool>,
    /// C-005 監査対応: チャンネル設定の暗号化キー（32バイト）。None の場合は暗号化を省略する
    encryption_key: Option<[u8; 32]>,
}

impl ChannelPostgresRepository {
    /// 暗号化キー付きでリポジトリを作成する
    /// encryption_key が None の場合、設定は平文で保存される（開発環境のみ許可）
    pub fn new(pool: Arc<PgPool>, encryption_key: Option<[u8; 32]>) -> Self {
        Self {
            pool,
            encryption_key,
        }
    }

    /// PostgreSQL セッション変数 app.current_tenant_id を設定して RLS ポリシーを有効化する
    /// set_config の第3引数 true は SET LOCAL（トランザクションスコープ）を意味する
    async fn set_tenant_context(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: &str,
    ) -> anyhow::Result<()> {
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    /// config JSON を AES-256-GCM で暗号化する
    /// 暗号化キーが設定されていない場合は None を返す（平文 config カラムを使用）
    fn encrypt_config(&self, config: &serde_json::Value) -> anyhow::Result<Option<String>> {
        match &self.encryption_key {
            Some(key) => {
                let plaintext = serde_json::to_vec(config)
                    .map_err(|e| anyhow::anyhow!("設定の JSON シリアライズに失敗: {}", e))?;
                let encrypted = aes_encrypt(key, &plaintext)
                    .map_err(|e| anyhow::anyhow!("設定の AES-256-GCM 暗号化に失敗: {}", e))?;
                Ok(Some(encrypted))
            }
            None => Ok(None),
        }
    }

    /// 暗号化済み設定を復号化して serde_json::Value に変換する
    fn decrypt_config(&self, encrypted: &str) -> anyhow::Result<serde_json::Value> {
        match &self.encryption_key {
            Some(key) => {
                let plaintext = aes_decrypt(key, encrypted)
                    .map_err(|e| anyhow::anyhow!("設定の AES-256-GCM 復号化に失敗: {}", e))?;
                serde_json::from_slice(&plaintext)
                    .map_err(|e| anyhow::anyhow!("復号化済み設定の JSON 解析に失敗: {}", e))
            }
            None => Err(anyhow::anyhow!(
                "暗号化された設定データがありますが、暗号化キーが設定されていません。\
                 NOTIFICATION_CHANNEL_ENCRYPTION_KEY 環境変数を設定してください。"
            )),
        }
    }

    /// ChannelRow をドメインエンティティに変換する
    /// encrypted_config が存在する場合は復号化を優先する（dual-read 移行戦略）
    fn row_to_channel(&self, row: ChannelRow) -> anyhow::Result<NotificationChannel> {
        let config = if let Some(ref encrypted) = row.encrypted_config {
            // C-005: 暗号化済みデータが存在する場合は復号化して使用する
            self.decrypt_config(encrypted)?
        } else {
            // 移行前のデータまたは暗号化キーなしの場合は平文 config を使用する
            row.config
        };

        Ok(NotificationChannel {
            id: row.id,
            name: row.name,
            channel_type: row.channel_type,
            config,
            tenant_id: row.tenant_id,
            enabled: row.enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// DB から取得する生データ構造（sqlx::FromRow で自動マッピング）
#[derive(sqlx::FromRow)]
struct ChannelRow {
    id: String,
    name: String,
    channel_type: String,
    /// 平文の設定値（移行前データまたは暗号化なしの場合）
    config: serde_json::Value,
    /// C-005: AES-256-GCM 暗号化済み設定値（Base64: nonce[12B] || ciphertext）
    encrypted_config: Option<String>,
    /// H-012: テナント識別子
    tenant_id: String,
    enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[async_trait]
impl NotificationChannelRepository for ChannelPostgresRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<NotificationChannel>> {
        // H-010: トランザクション内で RLS セッション変数を設定する
        // NOTE: 現時点ではシステムチャンネル（tenant_id='system'）をデフォルトとして使用する
        //       JWT クレームからのテナント伝播は ADR-0056 に記載の実装ロードマップで対応する
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, "system").await?;

        let row: Option<ChannelRow> = sqlx::query_as(
            "SELECT id, name, channel_type, config, encrypted_config, tenant_id, enabled, created_at, updated_at \
             FROM notification.channels WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        row.map(|r| self.row_to_channel(r)).transpose()
    }

    async fn find_all(&self) -> anyhow::Result<Vec<NotificationChannel>> {
        let mut tx = self.pool.begin().await?;
        // H-010: RLS セッション変数を設定してテナント分離を強制する
        Self::set_tenant_context(&mut tx, "system").await?;

        let rows: Vec<ChannelRow> = sqlx::query_as(
            "SELECT id, name, channel_type, config, encrypted_config, tenant_id, enabled, created_at, updated_at \
             FROM notification.channels ORDER BY created_at DESC",
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        rows.into_iter().map(|r| self.row_to_channel(r)).collect()
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
        enabled_only: bool,
    ) -> anyhow::Result<(Vec<NotificationChannel>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        let mut conditions = Vec::new();
        let mut bind_index = 1u32;

        if channel_type.is_some() {
            conditions.push(format!("channel_type = ${}", bind_index));
            bind_index += 1;
        }
        if enabled_only {
            conditions.push("enabled = true".to_string());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_query = format!(
            "SELECT COUNT(*) FROM notification.channels {}",
            where_clause
        );
        let data_query = format!(
            "SELECT id, name, channel_type, config, encrypted_config, tenant_id, enabled, created_at, updated_at \
             FROM notification.channels {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause,
            bind_index,
            bind_index + 1
        );

        // H-010: RLS セッション変数を設定してテナント分離を強制する
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, "system").await?;

        let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(ref v) = channel_type {
            count_q = count_q.bind(v);
        }
        let total_count = count_q.fetch_one(&mut *tx).await?;

        let mut data_q = sqlx::query_as::<_, ChannelRow>(&data_query);
        if let Some(ref v) = channel_type {
            data_q = data_q.bind(v);
        }
        data_q = data_q.bind(limit);
        data_q = data_q.bind(offset);

        let rows: Vec<ChannelRow> = data_q.fetch_all(&mut *tx).await?;
        tx.commit().await?;

        let channels: anyhow::Result<Vec<NotificationChannel>> =
            rows.into_iter().map(|r| self.row_to_channel(r)).collect();
        Ok((channels?, total_count as u64))
    }

    async fn create(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        // C-005: 設定を暗号化する（暗号化キーが設定されている場合）
        let encrypted_config = self.encrypt_config(&channel.config)?;

        // H-010: RLS セッション変数を設定してテナント分離を強制する
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, &channel.tenant_id).await?;

        sqlx::query(
            "INSERT INTO notification.channels \
             (id, name, channel_type, config, encrypted_config, tenant_id, enabled, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(&channel.id)
        .bind(&channel.name)
        .bind(&channel.channel_type)
        // 暗号化キーがある場合は config を空 JSON にして encrypted_config を使用する
        .bind(if encrypted_config.is_some() {
            serde_json::json!({})
        } else {
            channel.config.clone()
        })
        .bind(&encrypted_config)
        .bind(&channel.tenant_id)
        .bind(channel.enabled)
        .bind(channel.created_at)
        .bind(channel.updated_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn update(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        // C-005: 設定を暗号化する（暗号化キーが設定されている場合）
        let encrypted_config = self.encrypt_config(&channel.config)?;

        // H-010: RLS セッション変数を設定してテナント分離を強制する
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, &channel.tenant_id).await?;

        sqlx::query(
            "UPDATE notification.channels \
             SET name = $2, config = $3, encrypted_config = $4, enabled = $5, updated_at = $6 \
             WHERE id = $1",
        )
        .bind(&channel.id)
        .bind(&channel.name)
        .bind(if encrypted_config.is_some() {
            serde_json::json!({})
        } else {
            channel.config.clone()
        })
        .bind(&encrypted_config)
        .bind(channel.enabled)
        .bind(channel.updated_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        // H-010: RLS セッション変数を設定してテナント分離を強制する
        let mut tx = self.pool.begin().await?;
        Self::set_tenant_context(&mut tx, "system").await?;

        let result = sqlx::query("DELETE FROM notification.channels WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }
}
