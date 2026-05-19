//! relay.rs — audit_local → audit_chain リレー実装
//! 業務エラー監査 08: PostgreSQL audit_local から ClickHouse audit chain への転送
//! at-least-once + ReplacingMergeTree で idempotent

// UUID 型: audit_event の主キーおよびテナント ID に使用する
use uuid::Uuid;
// 日時型: audit_event の created_at に使用する
use chrono::{DateTime, Utc};
// シリアライズ: ClickHouse への JSON 送信に使用する
use serde::{Deserialize, Serialize};
// HTTP クライアント: ClickHouse HTTP インターフェースへの送信に使用する
use reqwest;
// sqlx PostgreSQL クライアント: audit_local からの SELECT / UPDATE に使用する
use sqlx;

// RelayError: リレー処理中に発生するエラーの列挙型
#[derive(Debug)]
pub enum RelayError {
    // PostgreSQL 接続エラー: audit_local への接続失敗
    PostgresConnection(String),
    // ClickHouse 接続エラー: audit_chain への送信失敗
    ClickHouseConnection(String),
    // シリアライズエラー: JSON 変換失敗
    Serialization(String),
    // クエリエラー: SQL 実行失敗
    Query(String),
}

// impl Display for RelayError: エラーメッセージを文字列化する
impl std::fmt::Display for RelayError {
    // fmt: エラーメッセージを文字列化して返す
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // エラー種別ごとにメッセージを返す
        match self {
            // PostgreSQL 接続エラーメッセージを返す
            RelayError::PostgresConnection(msg) => write!(f, "PostgreSQL 接続エラー: {}", msg),
            // ClickHouse 接続エラーメッセージを返す
            RelayError::ClickHouseConnection(msg) => write!(f, "ClickHouse 接続エラー: {}", msg),
            // シリアライズエラーメッセージを返す
            RelayError::Serialization(msg) => write!(f, "シリアライズエラー: {}", msg),
            // クエリエラーメッセージを返す
            RelayError::Query(msg) => write!(f, "クエリエラー: {}", msg),
        }
    }
}

// PendingAuditEvent: PostgreSQL audit_local から取得する未送信 audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAuditEvent {
    // エントリの主キー (domain_event と同一 UUID)
    pub id: Uuid,
    // テナント ID (RLS FORCE が保証する)
    pub tenant_id: Uuid,
    // アクター識別子 (Keycloak subject)
    pub actor_id: String,
    // セッション目的 (business_op / support 等)
    pub purpose: String,
    // テーブルクラス (TenantScoped / PiiSegregated 等)
    pub table_class: String,
    // 操作内容のペイロード (PII は redact 済み)
    pub payload: serde_json::Value,
    // 書込日時
    pub created_at: DateTime<Utc>,
    // hash chain 用: 直前のエントリの hash_digest
    pub prev_digest: Option<String>,
    // hash chain 用: テナント内での連番
    pub chain_sequence: Option<i64>,
    // relay ステータス: false = 未送信、true = 送信済み
    pub relayed: bool,
}

// AuditRelayConfig: リレー設定構造体
#[derive(Debug, Clone)]
pub struct AuditRelayConfig {
    // PostgreSQL 接続 URL (audit_local テーブルを持つ DB)
    pub postgres_url: String,
    // ClickHouse HTTP エンドポイント URL
    pub clickhouse_url: String,
    // ClickHouse データベース名
    pub clickhouse_database: String,
    // 1 回のリレーバッチサイズ (デフォルト 100 件)
    pub batch_size: usize,
}

// AuditRelay: PostgreSQL audit_local → ClickHouse audit chain リレー実装
// at-least-once 配信 + ReplacingMergeTree で idempotent を保証する
pub struct AuditRelay {
    // リレー設定
    config: AuditRelayConfig,
    // リレー済み件数の Prometheus カウンター (gauge として emit)
    relayed_count: std::sync::atomic::AtomicU64,
}

impl AuditRelay {
    // リレーを初期化する: PostgreSQL 接続と ClickHouse 接続を設定する
    pub fn new(config: AuditRelayConfig) -> Self {
        // 設定を保持するインスタンスを生成する
        Self {
            config,
            // リレー済み件数カウンターをゼロで初期化する
            relayed_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    // audit_local から未送信イベントを取得して ClickHouse に送信する
    // 戻り値: 今回のバッチで relay した件数
    pub async fn relay_pending(&self) -> Result<usize, RelayError> {
        // PostgreSQL から未送信 audit_event を batch_size 件取得する
        let pending = self.fetch_pending_events().await?;
        // 未送信イベントが空の場合はスキップする
        if pending.is_empty() {
            // 送信件数 0 を返す
            return Ok(0);
        }
        // ClickHouse に at-least-once で送信する (ReplacingMergeTree が冪等性を保証する)
        let relayed = self.send_to_clickhouse(&pending).await?;
        // PostgreSQL の relayed フラグを更新する
        self.mark_as_relayed(&pending).await?;
        // relay 済み件数を加算する
        self.relayed_count.fetch_add(
            relayed as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        // relay した件数を返す
        Ok(relayed)
    }

    // PostgreSQL audit_local から未送信イベントを取得する
    // relayed = false のレコードを chain_sequence 昇順で batch_size 件取得する
    // compile-time 型チェック: sqlx::query_as! マクロを使用する（CI で cargo sqlx prepare --check を実行する）
    // SQLX_OFFLINE=true でビルドする場合は .sqlx/ ディレクトリのメタデータを参照する
    async fn fetch_pending_events(&self) -> Result<Vec<PendingAuditEvent>, RelayError> {
        // PostgreSQL 接続プールを生成する（シングルコネクションのライフタイムを制御する）
        let pool = sqlx::postgres::PgPoolOptions::new()
            // 最大コネクション数をバッチサイズに合わせて 2 に制限する
            .max_connections(2)
            // postgres_url で接続する
            .connect(&self.config.postgres_url)
            .await
            // 接続失敗を RelayError::PostgresConnection に変換する
            .map_err(|e| RelayError::PostgresConnection(e.to_string()))?;
        // バッチサイズを i64 にキャストする（sqlx::query_as! の型パラメータに合わせる）
        let limit = self.config.batch_size as i64;
        // audit_event から relayed = false のレコードを chain_sequence 昇順で取得する
        // audit_local view は payload を除外するため、audit_event 本体から直接取得する
        // sqlx::query_as! は compile-time に戻り値型と SQL カラムの対応を検証する
        let events = sqlx::query_as!(
            PendingAuditEvent,
            r#"
            SELECT id, tenant_id, actor_id, purpose, table_class, payload,
                   created_at, prev_digest, chain_sequence, relayed
            FROM k1s0.audit_event
            WHERE relayed = false
            ORDER BY chain_sequence ASC NULLS LAST
            LIMIT $1
            "#,
            // バッチサイズを i64 としてバインドする（PostgreSQL LIMIT は bigint を受け取る）
            limit
        )
        // 接続プールを使ってクエリを実行する
        .fetch_all(&pool)
        .await
        // クエリエラーを RelayError::Query に変換する
        .map_err(|e| RelayError::Query(e.to_string()))?;
        // 取得件数をデバッグログに出力する
        tracing::debug!(
            batch_size = self.config.batch_size,
            fetched = events.len(),
            "audit_event から未送信イベントを取得した"
        );
        // 取得したイベントリストを返す
        Ok(events)
    }

    // ClickHouse audit_event テーブルに送信する
    // ReplacingMergeTree の冪等性により at-least-once でも安全に送信できる
    // ClickHouse HTTP インターフェースの JSONEachRow フォーマットを使用する
    async fn send_to_clickhouse(
        &self,
        events: &[PendingAuditEvent],
    ) -> Result<usize, RelayError> {
        // 送信するイベントを JSONEachRow 形式の文字列に変換する（1 行 1 JSON オブジェクト）
        let json_body: String = events
            .iter()
            .map(|e| {
                // 各イベントを JSON オブジェクトとしてシリアライズする
                serde_json::json!({
                    // イベント主キー
                    "id": e.id,
                    // テナント ID
                    "tenant_id": e.tenant_id,
                    // アクター識別子
                    "actor_id": e.actor_id,
                    // セッション目的
                    "purpose": e.purpose,
                    // テーブルクラス
                    "table_class": e.table_class,
                    // ペイロード（PII は redact 済み）
                    "payload": e.payload,
                    // 書込日時（RFC3339 形式）
                    "created_at": e.created_at.to_rfc3339(),
                    // 直前のハッシュダイジェスト
                    "prev_digest": e.prev_digest,
                    // チェーン連番
                    "chain_sequence": e.chain_sequence,
                })
                // JSON オブジェクトを 1 行の文字列に変換する
                .to_string()
            })
            // 行を改行で結合して JSONEachRow フォーマットにする
            .collect::<Vec<_>>()
            .join("\n");
        // ClickHouse HTTP エンドポイントの URL を組み立てる
        let url = format!(
            "{}/{}?query=INSERT+INTO+k1s0_audit.audit_event+FORMAT+JSONEachRow",
            self.config.clickhouse_url,
            self.config.clickhouse_database,
        );
        // reqwest HTTP クライアントを生成する（コネクション再利用のため毎回生成しない設計が望ましいが、
        // デモ実装のためシンプルに都度生成する）
        let client = reqwest::Client::new();
        // ClickHouse HTTP インターフェースに POST リクエストで JSONEachRow を送信する
        let response = client
            .post(&url)
            // リクエストボディに JSONEachRow 文字列を設定する
            .body(json_body)
            // Content-Type を text/plain に設定する（ClickHouse の期待するフォーマット）
            .header("Content-Type", "text/plain")
            // リクエストを送信する
            .send()
            .await
            // 送信失敗を RelayError::ClickHouseConnection に変換する
            .map_err(|e| RelayError::ClickHouseConnection(e.to_string()))?;
        // ClickHouse のレスポンスステータスを確認する
        if !response.status().is_success() {
            // エラーレスポンスのボディを取得してエラーメッセージに含める
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "(body 取得失敗)".to_string());
            // ClickHouse からのエラーを RelayError::ClickHouseConnection として返す
            return Err(RelayError::ClickHouseConnection(format!(
                "ClickHouse HTTP error: status={}, body={}",
                status, body
            )));
        }
        // 送信件数をデバッグログに出力する
        tracing::debug!(
            clickhouse_url = %self.config.clickhouse_url,
            event_count = events.len(),
            "ClickHouse に audit_event を送信した"
        );
        // 送信したイベント数を返す
        Ok(events.len())
    }

    // PostgreSQL audit_local の relayed フラグを true に更新する
    // 送信済みマーキングは冪等なので at-least-once でも安全に実行できる
    async fn mark_as_relayed(
        &self,
        events: &[PendingAuditEvent],
    ) -> Result<(), RelayError> {
        // 対象イベントの ID リストを取得する
        let ids: Vec<Uuid> = events.iter().map(|e| e.id).collect();
        // ID が空の場合は更新をスキップする
        if ids.is_empty() {
            return Ok(());
        }
        // PostgreSQL 接続プールを生成する
        let pool = sqlx::postgres::PgPoolOptions::new()
            // 更新専用に最大コネクション数を 2 に制限する
            .max_connections(2)
            // postgres_url で接続する
            .connect(&self.config.postgres_url)
            .await
            // 接続失敗を RelayError::PostgresConnection に変換する
            .map_err(|e| RelayError::PostgresConnection(e.to_string()))?;
        // audit_event の relayed = true に更新して送信済みにマークする
        // compile-time 型チェック: sqlx::query! マクロを使用する（CI で cargo sqlx prepare --check を実行する）
        // audit_local view は UPDATE 対象にできないため audit_event 本体を直接 UPDATE する（migration 0007 参照）
        // relayed_at カラムは migration 0007 に存在しないため relayed フラグのみ更新する
        // UUID 配列は sqlx では &[Uuid] として渡す（PostgreSQL ANY($1) 構文に対応する）
        sqlx::query!(
            r#"
            UPDATE k1s0.audit_event
            SET relayed = true
            WHERE id = ANY($1)
            "#,
            // ids を UUID スライスとしてバインドする（PostgreSQL uuid[] 型に対応する）
            &ids as &[Uuid]
        )
        // 接続プールを使ってクエリを実行する
        .execute(&pool)
        .await
        // クエリエラーを RelayError::Query に変換する
        .map_err(|e| RelayError::Query(e.to_string()))?;
        // 更新件数をデバッグログに出力する
        tracing::debug!(
            ids = ?ids,
            "audit_local の relayed フラグを更新した"
        );
        // 更新成功を返す
        Ok(())
    }

    // heartbeat を prometheus metric として emit する
    // audit_ingest_last_event_ts gauge を現在時刻で更新する
    pub fn emit_heartbeat_metric(&self) {
        // 現在の UNIX タイムスタンプを取得する
        let now_ts = chrono::Utc::now().timestamp();
        // relay 済み件数を取得する
        let relayed = self.relayed_count.load(std::sync::atomic::Ordering::Relaxed);
        // Prometheus metric として emit する (実装では prometheus crate を使用する)
        tracing::info!(
            audit_ingest_last_event_ts = now_ts,
            total_relayed = relayed,
            "heartbeat metric を emit した"
        );
    }
}

// tracing マクロのために tracing クレートをインポートする
use tracing;

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    // AuditRelay の生成が正常に動作することを確認する
    #[test]
    fn test_audit_relay_new() {
        // テスト用のリレー設定を作成する
        let config = AuditRelayConfig {
            postgres_url: "postgres://tier2:pass@localhost:5432/tier2".to_string(),
            clickhouse_url: "http://localhost:8123".to_string(),
            clickhouse_database: "k1s0_audit".to_string(),
            batch_size: 100,
        };
        // AuditRelay を生成する
        let relay = AuditRelay::new(config);
        // relay_count が 0 で初期化されていることを確認する
        assert_eq!(
            relay.relayed_count.load(std::sync::atomic::Ordering::Relaxed),
            0
        );
    }

    // emit_heartbeat_metric が panic しないことを確認する
    #[test]
    fn test_emit_heartbeat_metric_no_panic() {
        // テスト用のリレーを生成する
        let relay = AuditRelay::new(AuditRelayConfig {
            postgres_url: "postgres://localhost/test".to_string(),
            clickhouse_url: "http://localhost:8123".to_string(),
            clickhouse_database: "test".to_string(),
            batch_size: 10,
        });
        // heartbeat metric の emit が panic しないことを確認する
        relay.emit_heartbeat_metric();
    }
}
