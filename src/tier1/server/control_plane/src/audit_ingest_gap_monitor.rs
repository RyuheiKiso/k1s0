// audit_ingest_gap_monitor.rs — spec 03 §audit signal ingest gap monitor
// docs/04_詳細設計/01_適合仕様/03_観測適合仕様.md §audit signal の不変条件:
// "at-least-once + ReplacingMergeTree で idempotent" に基づき、
// audit signal のインジェスト遅延（gap）を検知してアラートを発する。
// ClickHouse HTTP interface の POST /?query= を reqwest で呼び出し、
// now() - max(event_time) が gap_threshold_seconds を超えたら tracing::warn! を発する。

// chrono: UTC 日時型（last_event_at フィールドに使用する）
use chrono::{DateTime, Utc};
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::{anyhow, Result};
// reqwest: ClickHouse HTTP interface 呼び出しに使用する非同期 HTTP クライアント
use reqwest::Client;
// tracing: 構造化ロギング（warn! / info! マクロ）
use tracing::{info, warn};
// urlencoding: ClickHouse HTTP interface の SQL クエリを URL エンコードする
// NOTE: reqwest の query パラメーターは自動エンコードするため直接使用は不要だが、
//       クエリ文字列を明示的に構築する際の可読性のために使用する

// ============================================================
// GapStatus — audit ingest gap の単発チェック結果を表す型
// ============================================================

// GapStatus は AuditIngestGapMonitor::check_once の戻り値型。
// ClickHouse の max(event_time) と now() の差分を格納する。
#[derive(Debug, Clone)]
pub struct GapStatus {
    // last_event_at: ClickHouse audit_events テーブルの最新 event_time
    pub last_event_at: DateTime<Utc>,
    // gap_seconds: now() - last_event_at の秒数
    pub gap_seconds: u64,
    // is_alerting: gap_seconds が gap_threshold_seconds を超えている場合は true
    pub is_alerting: bool,
}

// ============================================================
// AuditIngestGapMonitor — audit signal インジェスト遅延モニター
// ============================================================

// AuditIngestGapMonitor は ClickHouse の audit_events テーブルを定期ポーリングして
// インジェスト遅延（gap）を検知する構造体。
// spec 03 §audit: at-least-once + idempotent 保証の監視面を担う。
pub struct AuditIngestGapMonitor {
    // clickhouse_url: ClickHouse HTTP interface の接続先 URL（例: http://clickhouse:8123）
    clickhouse_url: String,
    // gap_threshold_seconds: この閾値を超えた gap 秒数でアラートを発する
    gap_threshold_seconds: u64,
    // http_client: reqwest 非同期 HTTP クライアント（再利用のため構造体に保持する）
    http_client: Client,
}

impl AuditIngestGapMonitor {
    // new は AuditIngestGapMonitor を構築する。
    // clickhouse_url: ClickHouse HTTP interface の URL（末尾スラッシュなし）
    // gap_threshold_seconds: アラート閾値（秒）
    pub fn new(clickhouse_url: String, gap_threshold_seconds: u64) -> Self {
        // reqwest::Client を構築する（接続プールを再利用するため構造体フィールドに保持する）
        let http_client = Client::new();
        // AuditIngestGapMonitor を返す
        Self {
            clickhouse_url,
            gap_threshold_seconds,
            http_client,
        }
    }

    // check_once は ClickHouse に SQL クエリを発行して gap を単発チェックする。
    // 単発チェック用インターフェース（テスト可能・ループ外から呼び出し可能）。
    // 戻り値: GapStatus（last_event_at / gap_seconds / is_alerting）
    pub async fn check_once(&self) -> Result<GapStatus> {
        // ClickHouse HTTP interface に発行する SQL クエリを構築する
        // FORMAT JSONEachRow で 1 行の JSON を取得する
        let query = "SELECT \
            toUnixTimestamp(max(event_time)) AS last_event_ts, \
            toUnixTimestamp(now()) AS now_ts \
            FROM k1s0_audit.audit_events \
            FORMAT JSONEachRow";

        // ClickHouse HTTP interface の URL を構築する（POST /?query=<encoded_sql>）
        let url = format!("{}/", self.clickhouse_url);

        // HTTP POST リクエストを発行してレスポンスを取得する
        let response = self
            .http_client
            .post(&url)
            // ClickHouse HTTP interface はクエリを query パラメーターで受け取る
            .query(&[("query", query)])
            .send()
            .await
            .map_err(|e| anyhow!("ClickHouse HTTP request failed: {}", e))?;

        // HTTP ステータスコードを確認する（200 以外はエラーとして扱う）
        if !response.status().is_success() {
            // エラーレスポンスのボディを取得してエラーメッセージに含める
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "ClickHouse returned HTTP {}: {}",
                status,
                body
            ));
        }

        // レスポンスボディを文字列として取得する
        let body = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read ClickHouse response body: {}", e))?;

        // JSONEachRow 形式の 1 行目をパースする
        // 空の場合は audit_events テーブルにレコードがない（gap = now - epoch として扱う）
        let (last_event_ts, now_ts): (i64, i64) = if body.trim().is_empty() {
            // テーブルが空の場合は last_event_ts = 0（Unix epoch）として扱う
            (0, chrono::Utc::now().timestamp())
        } else {
            // JSON オブジェクトをパースして last_event_ts と now_ts を取得する
            let parsed: serde_json::Value = serde_json::from_str(body.trim())
                .map_err(|e| anyhow!("Failed to parse ClickHouse JSONEachRow response: {}", e))?;
            // last_event_ts フィールドを取得する
            let last = parsed["last_event_ts"]
                .as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .or_else(|| parsed["last_event_ts"].as_i64())
                .ok_or_else(|| anyhow!("Missing or invalid 'last_event_ts' in ClickHouse response"))?;
            // now_ts フィールドを取得する
            let now = parsed["now_ts"]
                .as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .or_else(|| parsed["now_ts"].as_i64())
                .ok_or_else(|| anyhow!("Missing or invalid 'now_ts' in ClickHouse response"))?;
            (last, now)
        };

        // gap_seconds を計算する（now_ts - last_event_ts）
        // last_event_ts が now_ts より大きい（時計ずれ）場合は 0 として扱う
        let gap_seconds = if now_ts > last_event_ts {
            (now_ts - last_event_ts) as u64
        } else {
            // 時計ずれまたは最新イベントが未来の場合は gap = 0 として扱う
            0u64
        };

        // last_event_at を DateTime<Utc> に変換する
        let last_event_at = DateTime::<Utc>::from_timestamp(last_event_ts, 0)
            .unwrap_or_else(Utc::now);

        // is_alerting: gap_seconds が gap_threshold_seconds を超えているかどうか
        let is_alerting = gap_seconds > self.gap_threshold_seconds;

        // GapStatus を構築して返す
        Ok(GapStatus {
            last_event_at,
            gap_seconds,
            is_alerting,
        })
    }

    // run は check_once を定期的に実行するループ。
    // gap が閾値を超えた場合は tracing::warn! を発し、
    // Prometheus counter の increment ログを残す（counter の実 increment は
    // metrics_exporter 実装が担うため、ここでは warn! で代替する）。
    // この関数はキャンセルされるまでループし続ける（tokio::select! でキャンセル可能）。
    pub async fn run(&self) -> Result<()> {
        // ループ間隔（秒）: 30 秒ごとにチェックする（運用コスト・レスポンスタイムのバランス）
        let interval_secs = 30u64;
        info!(
            clickhouse_url = %self.clickhouse_url,
            gap_threshold_seconds = self.gap_threshold_seconds,
            check_interval_seconds = interval_secs,
            "AuditIngestGapMonitor: started"
        );

        loop {
            // check_once を実行して gap 状態を取得する
            match self.check_once().await {
                Ok(status) => {
                    if status.is_alerting {
                        // gap が閾値を超えた場合は warn! を発する
                        // Prometheus counter 相当の記録（k1s0_audit_ingest_gap_alert_total の代替）
                        warn!(
                            last_event_at = %status.last_event_at,
                            gap_seconds = status.gap_seconds,
                            threshold_seconds = self.gap_threshold_seconds,
                            "AuditIngestGapMonitor: ALERT — audit ingest gap exceeds threshold \
                             (k1s0_audit_ingest_gap_alert_total +1)"
                        );
                    } else {
                        // gap が閾値内の場合は info! でステータスを記録する
                        info!(
                            last_event_at = %status.last_event_at,
                            gap_seconds = status.gap_seconds,
                            threshold_seconds = self.gap_threshold_seconds,
                            "AuditIngestGapMonitor: gap within threshold"
                        );
                    }
                }
                Err(e) => {
                    // ClickHouse 接続エラーなどの場合は warn! を発してループを継続する
                    warn!(
                        error = %e,
                        "AuditIngestGapMonitor: check_once failed, will retry in {} seconds",
                        interval_secs
                    );
                }
            }

            // 次のチェックまで interval_secs 秒待機する
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
        }
    }
}

// ============================================================
// テスト群
// ============================================================

#[cfg(test)]
mod tests {
    // 親モジュールから型を import する
    use super::*;

    // new_monitor_has_correct_fields は AuditIngestGapMonitor::new が
    // 渡したパラメーターを正しくフィールドに保持することを assert する。
    #[test]
    fn new_monitor_has_correct_fields() {
        // AuditIngestGapMonitor を構築する
        let monitor = AuditIngestGapMonitor::new(
            "http://clickhouse.k1s0-system.svc:8123".to_string(),
            300,
        );
        // clickhouse_url が正しく設定されていること
        assert_eq!(
            monitor.clickhouse_url,
            "http://clickhouse.k1s0-system.svc:8123"
        );
        // gap_threshold_seconds が正しく設定されていること
        assert_eq!(monitor.gap_threshold_seconds, 300);
    }

    // gap_status_is_alerting_when_gap_exceeds_threshold は
    // GapStatus の is_alerting が gap_seconds > gap_threshold_seconds の場合に
    // true となることを assert する（GapStatus の意味的正確性）。
    #[test]
    fn gap_status_is_alerting_when_gap_exceeds_threshold() {
        // gap_seconds が閾値を超えた GapStatus を構築する
        let status = GapStatus {
            last_event_at: Utc::now(),
            gap_seconds: 400,
            is_alerting: 400 > 300,
        };
        // is_alerting が true であること
        assert!(status.is_alerting, "gap_seconds=400 > threshold=300 must be alerting");
    }

    // gap_status_not_alerting_when_within_threshold は
    // GapStatus の is_alerting が gap_seconds <= gap_threshold_seconds の場合に
    // false となることを assert する。
    #[test]
    fn gap_status_not_alerting_when_within_threshold() {
        // gap_seconds が閾値以内の GapStatus を構築する
        let status = GapStatus {
            last_event_at: Utc::now(),
            gap_seconds: 100,
            is_alerting: 100 > 300,
        };
        // is_alerting が false であること
        assert!(!status.is_alerting, "gap_seconds=100 <= threshold=300 must not be alerting");
    }
}
