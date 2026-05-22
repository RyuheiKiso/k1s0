// k1s0 tier2 audit relay — Rust 実装
// 0007_audit_local_view_or_table.sql で定義した audit_local view から未転送 events を取得し
// ClickHouse に転送する relay エンジン
// spec 10 §pii_segregated.outbox_pii_redact = true に基づき PII を含まない view 列のみを扱う
// clickhouse_sink.yaml の HTTP API 経由（reqwest を使用: 新規クライアントライブラリ追加なし）

// sqlx PgPool: PostgreSQL 接続プールを使って audit_local を操作する
use sqlx::PgPool;
// anyhow: エラー型（Result<T, anyhow::Error> の略記 Result<T>）
use anyhow::Result;
// uuid: audit_event の主キー UUID
use uuid::Uuid;
// chrono: 書込日時の型（UTC タイムゾーン付き）
use chrono::{DateTime, Utc};
// reqwest: ClickHouse HTTP API への転送に使用する（新規クライアントライブラリ追加なし）
use reqwest::Client as HttpClient;
// serde_json: ClickHouse 向け JSON ペイロードの構築に使用する
use serde_json::{json, Value as JsonValue};

// PendingAuditEvent: audit_local view から取得する未転送 event の型
// relay.rs が SELECT する列に 1:1 対応する（0007 migration の view 定義を参照）
// payload カラムは view から除外されているため PII 不在を保証する
#[derive(Debug, Clone)]
pub struct PendingAuditEvent {
    // イベント識別子（domain_event と同一 UUID）
    pub id: Uuid,
    // テナント識別子（RLS FORCE が保証する、view 列）
    pub tenant_id: Uuid,
    // アクター識別子（Keycloak subject、view 列）
    pub actor_id: String,
    // セッション目的（business_op / support 等、view 列）
    pub purpose: String,
    // テーブルクラス（TenantScoped / PiiSegregated 等、view 列）
    pub table_class: String,
    // 書込日時（UTC タイムゾーン付き、view 列）
    pub created_at: DateTime<Utc>,
    // 直前エントリの hash_digest（hash chain 連続性保証、view 列）
    pub prev_digest: Option<String>,
    // テナント内での連番（hash chain 順序保証、view 列）
    pub chain_sequence: Option<i64>,
    // ClickHouse 転送済みフラグ（false = 未送信、view 列）
    pub relayed: bool,
}

// ClickHouseRow: ClickHouse に送信する 1 行のデータ型
// PendingAuditEvent から PII を除いた列のみを含む
#[derive(Debug, Clone)]
struct ClickHouseRow {
    // イベント識別子（UUID 文字列）
    id: String,
    // テナント識別子（UUID 文字列）
    tenant_id: String,
    // アクター識別子（Keycloak subject）
    actor_id: String,
    // セッション目的
    purpose: String,
    // テーブルクラス
    table_class: String,
    // 書込日時（ISO 8601 文字列: ClickHouse DateTime64 と互換）
    created_at: String,
    // 直前エントリの hash_digest（None の場合は空文字列）
    prev_digest: String,
    // テナント内での連番（None の場合は -1）
    chain_sequence: i64,
}

// ClickHouseRow を PendingAuditEvent から変換する
impl From<&PendingAuditEvent> for ClickHouseRow {
    // from: PendingAuditEvent から ClickHouseRow に変換する
    fn from(event: &PendingAuditEvent) -> Self {
        // 各フィールドを変換して ClickHouseRow を生成する
        ClickHouseRow {
            // UUID を文字列に変換する
            id: event.id.to_string(),
            // テナント UUID を文字列に変換する
            tenant_id: event.tenant_id.to_string(),
            // アクター識別子をコピーする
            actor_id: event.actor_id.clone(),
            // セッション目的をコピーする
            purpose: event.purpose.clone(),
            // テーブルクラスをコピーする
            table_class: event.table_class.clone(),
            // 書込日時を ISO 8601 文字列に変換する（ClickHouse DateTime64 と互換）
            created_at: event.created_at.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            // prev_digest が None の場合は空文字列を使用する
            prev_digest: event.prev_digest.clone().unwrap_or_default(),
            // chain_sequence が None の場合は -1 を使用する
            chain_sequence: event.chain_sequence.unwrap_or(-1),
        }
    }
}

// percent_encode_query: URL クエリパラメータ用のパーセントエンコード（標準ライブラリのみ使用する）
// RFC 3986 の unreserved characters（英数字・ハイフン・アンダースコア・ドット・チルダ）は変換しない
// それ以外の ASCII バイトは %XX 形式にエンコードする
fn percent_encode_query(input: &str) -> String {
    // 出力バッファを確保する（入力の 3 倍のキャパシティを確保して再割り当てを抑える）
    let mut output = String::with_capacity(input.len() * 3);
    // 入力文字列を UTF-8 バイト列として処理する
    for byte in input.as_bytes() {
        // 英数字とハイフン・アンダースコア・ドット・チルダはそのまま追加する
        if byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_' || *byte == b'.' || *byte == b'~' {
            // 安全な文字はそのまま出力バッファに追加する
            output.push(*byte as char);
        } else {
            // その他の文字は %XX 形式に変換して出力バッファに追加する
            output.push_str(&format!("%{:02X}", byte));
        }
    }
    // エンコードされた文字列を返す
    output
}

// AuditRelay: audit_local への接続と ClickHouse への転送を担う構造体
// fetch_pending_events → ClickHouse 転送 → mark_as_relayed の 3 ステップを担当する
pub struct AuditRelay {
    // PostgreSQL 接続プール（audit_local view への SELECT と audit_event への UPDATE に使用する）
    pool: PgPool,
    // ClickHouse エンドポイント URL（clickhouse_sink.yaml の host:port から構築する）
    clickhouse_endpoint: String,
    // ClickHouse 書込先データベース名（clickhouse_sink.yaml の database から取得する）
    clickhouse_database: String,
    // ClickHouse 書込先テーブル名（clickhouse_sink.yaml の table_mapping.table_name から取得する）
    clickhouse_table: String,
    // ClickHouse 認証ユーザー名（clickhouse_sink.yaml の user から取得する）
    clickhouse_user: String,
    // ClickHouse 認証パスワード（clickhouse_sink.yaml の password から取得する）
    clickhouse_password: String,
    // reqwest HTTP クライアント（ClickHouse HTTP API への転送に再利用する）
    http_client: HttpClient,
}

impl AuditRelay {
    // new: AuditRelay を生成する（PgPool と ClickHouse 接続設定を受け取る）
    // clickhouse_sink.yaml の設定値を環境変数経由で受け取ることを想定する
    pub fn new(
        // PostgreSQL 接続プール
        pool: PgPool,
        // ClickHouse HTTP エンドポイント URL（例: "http://localhost:8123"）
        clickhouse_endpoint: impl Into<String>,
        // ClickHouse 書込先データベース名（例: "k1s0_audit"）
        clickhouse_database: impl Into<String>,
        // ClickHouse 書込先テーブル名（例: "audit_events"）
        clickhouse_table: impl Into<String>,
        // ClickHouse 認証ユーザー名
        clickhouse_user: impl Into<String>,
        // ClickHouse 認証パスワード
        clickhouse_password: impl Into<String>,
    ) -> Self {
        // HTTP クライアントを生成する（タイムアウト設定付き）
        let http_client = HttpClient::builder()
            // 接続タイムアウトを 30 秒に設定する
            .connect_timeout(std::time::Duration::from_secs(30))
            // レスポンスタイムアウトを 60 秒に設定する
            .timeout(std::time::Duration::from_secs(60))
            // HTTP クライアントを構築する
            .build()
            // HTTP クライアントの構築失敗は致命的エラーとして panic する
            .expect("reqwest HTTP クライアントの構築に失敗しました");
        // AuditRelay を生成して返す
        Self {
            // PostgreSQL 接続プールを格納する
            pool,
            // ClickHouse エンドポイント URL を格納する
            clickhouse_endpoint: clickhouse_endpoint.into(),
            // ClickHouse データベース名を格納する
            clickhouse_database: clickhouse_database.into(),
            // ClickHouse テーブル名を格納する
            clickhouse_table: clickhouse_table.into(),
            // ClickHouse ユーザー名を格納する
            clickhouse_user: clickhouse_user.into(),
            // ClickHouse パスワードを格納する
            clickhouse_password: clickhouse_password.into(),
            // HTTP クライアントを格納する
            http_client,
        }
    }

    // new_with_defaults: 環境変数から ClickHouse 設定を読み込んで AuditRelay を生成する
    // clickhouse_sink.yaml の default 値を環境変数の fallback として使用する
    pub fn new_with_defaults(pool: PgPool) -> Self {
        // CLICKHOUSE_HOST 環境変数を読み込む（デフォルト: "localhost"）
        let host = std::env::var("CLICKHOUSE_HOST").unwrap_or_else(|_| "localhost".to_string());
        // CLICKHOUSE_PORT 環境変数を読み込む（デフォルト: "8123"）
        let port = std::env::var("CLICKHOUSE_PORT").unwrap_or_else(|_| "8123".to_string());
        // ClickHouse HTTP エンドポイント URL を構築する
        let endpoint = format!("http://{}:{}", host, port);
        // CLICKHOUSE_USER 環境変数を読み込む（デフォルト: "default"）
        let user = std::env::var("CLICKHOUSE_USER").unwrap_or_else(|_| "default".to_string());
        // CLICKHOUSE_PASSWORD 環境変数を読み込む（デフォルト: 空文字列）
        let password = std::env::var("CLICKHOUSE_PASSWORD").unwrap_or_default();
        // AuditRelay を生成して返す
        Self::new(
            // PostgreSQL 接続プール
            pool,
            // ClickHouse HTTP エンドポイント URL
            endpoint,
            // データベース名（clickhouse_sink.yaml の database 値）
            "k1s0_audit",
            // テーブル名（clickhouse_sink.yaml の table_mapping.table_name 値）
            "audit_events",
            // ユーザー名
            user,
            // パスワード
            password,
        )
    }

    // fetch_pending_events: audit_local view から未転送（relayed = false）の events を取得する
    // 戻り値: PendingAuditEvent の Vec（取得件数は limit で上限を設定する）
    //
    // SQL 等価クエリ（0007 migration コメント参照）:
    //   SELECT id, tenant_id, actor_id, purpose, table_class, created_at,
    //          prev_digest, chain_sequence, relayed
    //   FROM k1s0.audit_local
    //   WHERE relayed = false
    //   ORDER BY chain_sequence ASC NULLS LAST
    //   LIMIT $1
    pub async fn fetch_pending_events(
        // PostgreSQL 接続プール（self を借用して使用する）
        conn: &PgPool,
        // 取得上限件数（ClickHouse 転送のバッチサイズを制御する）
        limit: i64,
    ) -> Result<Vec<PendingAuditEvent>> {
        // sqlx::query_as! を使って audit_local view から未転送 events を取得する
        // SQLX_OFFLINE=true の場合は .sqlx/ キャッシュを使用する
        // 未使用のため sqlx::query! ではなく sqlx::query_as を runtime 版で使用する
        let rows = sqlx::query_as!(
            // 取得する型を指定する（PendingAuditEvent と列名が一致する必要がある）
            PendingAuditEvent,
            // audit_local view から未転送 events を chain_sequence 昇順で取得する
            r#"
            SELECT
                id,
                tenant_id,
                actor_id,
                purpose,
                table_class,
                created_at,
                prev_digest,
                chain_sequence,
                relayed
            FROM k1s0.audit_local
            WHERE relayed = false
            ORDER BY chain_sequence ASC NULLS LAST
            LIMIT $1
            "#,
            // 取得上限件数をバインドする
            limit,
        )
        // PostgreSQL 接続プールから実行する（接続プールから自動的に接続を借用する）
        .fetch_all(conn)
        // sqlx エラーを anyhow エラーに変換する
        .await?;
        // 取得した events を返す
        Ok(rows)
    }

    // mark_as_relayed: 転送済みの audit_event を relayed = true に更新する
    // 戻り値: Ok(()) = 更新成功、Err = DB エラー
    //
    // SQL 等価クエリ（0007 migration コメント参照）:
    //   UPDATE k1s0.audit_event SET relayed = true WHERE id = ANY($1)
    //   （audit_event 本体を直接 UPDATE する。view 経由ではない）
    pub async fn mark_as_relayed(
        // PostgreSQL 接続プール（self を借用して使用する）
        conn: &PgPool,
        // 転送済みにする audit_event の UUID スライス（空スライスの場合は UPDATE を実行しない）
        event_ids: &[Uuid],
    ) -> Result<()> {
        // event_ids が空の場合は早期リターンする（UPDATE を実行しない）
        if event_ids.is_empty() {
            // 空のスライスの場合は何もしない
            return Ok(());
        }

        // sqlx::query! を使って audit_event テーブルの relayed フラグを true に更新する
        // ANY($1) で UUID スライスを一括 UPDATE する（N+1 クエリを避ける）
        sqlx::query!(
            // audit_event 本体を直接 UPDATE する（view 経由は不可）
            r#"
            UPDATE k1s0.audit_event
            SET relayed = true
            WHERE id = ANY($1)
            "#,
            // UUID スライスを PostgreSQL の UUID 配列としてバインドする
            event_ids,
        )
        // PostgreSQL 接続プールから実行する
        .execute(conn)
        // sqlx エラーを anyhow エラーに変換する
        .await?;
        // 更新成功を返す
        Ok(())
    }

    // send_to_clickhouse: PendingAuditEvent のバッチを ClickHouse HTTP API で転送する
    // ClickHouse の JSONEachRow 形式で INSERT クエリを構築し HTTP POST する
    // clickhouse_sink.yaml の HTTP インターフェース（port: 8123）経由で送信する
    async fn send_to_clickhouse(&self, events: &[PendingAuditEvent]) -> Result<()> {
        // 空の events は送信不要なため早期リターンする
        if events.is_empty() {
            // 空の場合は何もしない
            return Ok(());
        }

        // ClickHouse JSONEachRow 形式のボディを構築する
        // 各行を JSON オブジェクトに変換して改行区切りで結合する（NDJSON 形式）
        let json_lines: Vec<String> = events
            .iter()
            // 各 PendingAuditEvent を ClickHouseRow に変換する
            .map(ClickHouseRow::from)
            // 各 ClickHouseRow を JSON オブジェクトに変換して文字列化する
            .map(|row| {
                // ClickHouse に送信する JSON オブジェクトを構築する
                let obj: JsonValue = json!({
                    // イベント識別子
                    "id": row.id,
                    // テナント識別子
                    "tenant_id": row.tenant_id,
                    // アクター識別子
                    "actor_id": row.actor_id,
                    // セッション目的
                    "purpose": row.purpose,
                    // テーブルクラス
                    "table_class": row.table_class,
                    // 書込日時
                    "created_at": row.created_at,
                    // 直前 hash_digest（空文字列の場合は null に変換する）
                    "prev_digest": if row.prev_digest.is_empty() { JsonValue::Null } else { JsonValue::String(row.prev_digest) },
                    // テナント内連番（-1 の場合は null に変換する）
                    "chain_sequence": if row.chain_sequence == -1 { JsonValue::Null } else { json!(row.chain_sequence) },
                });
                // JSON オブジェクトを文字列化して返す
                obj.to_string()
            })
            // 全行を Vec<String> に収集する
            .collect();

        // NDJSON 形式でボディを構築する（各行を改行区切りで結合する）
        let body = json_lines.join("\n");

        // ClickHouse HTTP API の INSERT エンドポイント URL を構築する
        // 形式: http://{host}:{port}/?query=INSERT%20INTO%20{db}.{table}%20FORMAT%20JSONEachRow
        let insert_query = format!(
            "INSERT INTO {}.{} FORMAT JSONEachRow",
            self.clickhouse_database, self.clickhouse_table
        );
        // URL クエリパラメータ用にパーセントエンコードする（標準ライブラリのみ使用する）
        let encoded_query = percent_encode_query(&insert_query);
        // ClickHouse HTTP API URL を構築する
        let url = format!("{}/?query={}", self.clickhouse_endpoint, encoded_query);

        // reqwest で ClickHouse HTTP API に POST する
        let response = self
            .http_client
            // HTTP POST メソッドで URL にリクエストを送信する
            .post(&url)
            // Basic 認証ヘッダを設定する（clickhouse_sink.yaml の user / password）
            .basic_auth(&self.clickhouse_user, Some(&self.clickhouse_password))
            // Content-Type を application/octet-stream に設定する（JSONEachRow 形式）
            .header("Content-Type", "application/octet-stream")
            // ボディに NDJSON データを設定する
            .body(body)
            // HTTP リクエストを送信する（タイムアウトは HttpClient のデフォルト値を使用する）
            .send()
            // reqwest エラーを anyhow エラーに変換する
            .await
            .map_err(|e| anyhow::anyhow!("ClickHouse HTTP POST エラー: {}", e))?;

        // ClickHouse からのレスポンスを確認する
        if !response.status().is_success() {
            // ClickHouse エラーレスポンスのボディを取得する（エラーメッセージを含む）
            let status = response.status();
            // レスポンスボディを文字列として取得する（エラー詳細を含む）
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<レスポンスボディ取得失敗>".to_string());
            // ClickHouse エラーを anyhow エラーとして返す
            return Err(anyhow::anyhow!(
                "ClickHouse INSERT 失敗: HTTP {} — {}",
                status,
                body
            ));
        }

        // 転送成功を返す
        Ok(())
    }

    // relay_batch: fetch_pending_events → ClickHouse 転送 → mark_as_relayed を一括実行する
    // 戻り値: 転送した event 数
    pub async fn relay_batch(&self, batch_size: i64) -> Result<usize> {
        // 未転送 events を取得する
        let pending = Self::fetch_pending_events(&self.pool, batch_size).await?;
        // 未転送 events が 0 件の場合は早期リターンする
        if pending.is_empty() {
            // 転送件数 0 を返す
            return Ok(0);
        }

        // ClickHouse HTTP API 経由で events を転送する（reqwest 使用）
        self.send_to_clickhouse(&pending).await?;

        // 転送済みの event ID リストを構築する
        let event_ids: Vec<Uuid> = pending.iter().map(|e| e.id).collect();
        // 転送件数を記録する
        let count = event_ids.len();

        // mark_as_relayed: 転送済みフラグを true に更新する
        // ClickHouse 転送が成功した場合のみ relayed フラグを立てる（at-least-once 保証）
        Self::mark_as_relayed(&self.pool, &event_ids).await?;

        // 転送した event 数を返す
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部のインポート
    use super::*;

    // テスト用の PgPool を生成するヘルパー（接続は遅延初期化）
    async fn make_pool_for_test() -> PgPool {
        // TEST_DATABASE_URL が設定されている場合は実際の DB に接続する
        let url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/k1s0_test".to_string());
        // PgPool を接続せずに生成する（unit test では pool を使わないため）
        sqlx::PgPool::connect_lazy(&url)
            .expect("PgPool::connect_lazy should not fail on valid URL format")
    }

    #[tokio::test]
    // AuditRelay::new: pool と endpoint を受け取って構築できることを確認する
    async fn test_new_creates_relay() {
        // テスト用 PgPool を生成する
        let pool = make_pool_for_test().await;
        // AuditRelay を生成する
        let relay = AuditRelay::new(
            pool,
            // ClickHouse エンドポイント URL を設定する
            "http://clickhouse:8123",
            // データベース名を設定する
            "k1s0_audit",
            // テーブル名を設定する
            "audit_events",
            // ユーザー名を設定する
            "default",
            // パスワードを設定する
            "",
        );
        // clickhouse_endpoint が正しく設定されていることを確認する
        assert_eq!(relay.clickhouse_endpoint, "http://clickhouse:8123");
        // clickhouse_database が正しく設定されていることを確認する
        assert_eq!(relay.clickhouse_database, "k1s0_audit");
        // clickhouse_table が正しく設定されていることを確認する
        assert_eq!(relay.clickhouse_table, "audit_events");
    }

    #[tokio::test]
    // mark_as_relayed: 空スライスを渡した場合は Ok(()) を返すことを確認する
    async fn test_mark_as_relayed_empty_ids_returns_ok() {
        // テスト用 PgPool を生成する
        let pool = make_pool_for_test().await;
        // 空の UUID スライスを渡す（DB に接続しないため Ok が返る）
        let result = AuditRelay::mark_as_relayed(&pool, &[]).await;
        // Ok(()) が返ることを確認する
        assert!(result.is_ok(), "空スライスの場合は Ok が返る");
    }

    #[tokio::test]
    // ClickHouseRow::from: PendingAuditEvent から ClickHouseRow への変換を確認する
    async fn test_clickhouse_row_from_pending_event() {
        // テスト用の PendingAuditEvent を生成する
        let event_id = Uuid::new_v4();
        // テナント UUID を生成する
        let tenant_id = Uuid::new_v4();
        // 書込日時を現在時刻で生成する
        let now = Utc::now();
        // PendingAuditEvent を生成する
        let event = PendingAuditEvent {
            // イベント識別子を設定する
            id: event_id,
            // テナント識別子を設定する
            tenant_id,
            // アクター識別子を設定する
            actor_id: "actor-001".to_string(),
            // セッション目的を設定する
            purpose: "business_op".to_string(),
            // テーブルクラスを設定する
            table_class: "TenantScoped".to_string(),
            // 書込日時を設定する
            created_at: now,
            // 直前 hash_digest を設定する
            prev_digest: Some("abc123".to_string()),
            // テナント内連番を設定する
            chain_sequence: Some(42),
            // 転送済みフラグを設定する
            relayed: false,
        };
        // ClickHouseRow に変換する
        let row = ClickHouseRow::from(&event);
        // 変換結果を確認する
        assert_eq!(row.id, event_id.to_string());
        // テナント識別子が正しく変換されたことを確認する
        assert_eq!(row.tenant_id, tenant_id.to_string());
        // アクター識別子が正しく変換されたことを確認する
        assert_eq!(row.actor_id, "actor-001");
        // prev_digest が正しく変換されたことを確認する
        assert_eq!(row.prev_digest, "abc123");
        // chain_sequence が正しく変換されたことを確認する
        assert_eq!(row.chain_sequence, 42);
    }

    #[tokio::test]
    // ClickHouseRow::from: prev_digest が None の場合は空文字列に変換されることを確認する
    async fn test_clickhouse_row_none_prev_digest_becomes_empty() {
        // テスト用の PendingAuditEvent を生成する（prev_digest = None）
        let event = PendingAuditEvent {
            // イベント識別子を設定する
            id: Uuid::new_v4(),
            // テナント識別子を設定する
            tenant_id: Uuid::new_v4(),
            // アクター識別子を設定する
            actor_id: "actor-001".to_string(),
            // セッション目的を設定する
            purpose: "business_op".to_string(),
            // テーブルクラスを設定する
            table_class: "TenantScoped".to_string(),
            // 書込日時を設定する
            created_at: Utc::now(),
            // prev_digest を None に設定する
            prev_digest: None,
            // chain_sequence を None に設定する
            chain_sequence: None,
            // 転送済みフラグを設定する
            relayed: false,
        };
        // ClickHouseRow に変換する
        let row = ClickHouseRow::from(&event);
        // prev_digest が空文字列に変換されたことを確認する
        assert_eq!(row.prev_digest, "");
        // chain_sequence が -1 に変換されたことを確認する
        assert_eq!(row.chain_sequence, -1);
    }
}
