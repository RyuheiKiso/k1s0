// nightly_close.rs — 夜間 close バッチ Workflow（設計方針 25 §batch）
// 毎日深夜に当日の業務を close して翌営業日に引き継ぐバッチ処理を実装する
// Temporal Workflow として定義し、Argo CronWorkflow から trigger する

// anyhow: Result 型に使用する
use anyhow::Result;
// serde: Workflow 入出力の JSON シリアライズに使用する
use serde::{Deserialize, Serialize};
// uuid: テナント識別子型に使用する
use uuid::Uuid;
// tracing: 構造化ロギングに使用する
use tracing::{info, warn};
// WorkflowError をインポートする
use crate::WorkflowError;
// PgPool: PostgreSQL 接続プールを受け取る
use sqlx::PgPool;
// AtomicTripleWrite: P1〜P4 不変条件を保証する三表同時書込エンジン
use k1s0_tier2::atomic_triple_write::{AtomicTripleWrite, StateChange};
// TableClass: テナントスコープのテーブル分類に使用する
use k1s0_tier2::TableClass;
// TenantContext: RLS GUC を transaction スコープに注入する
use k1s0_tier2::tenant_context::{TenantContext, SessionPurpose};
// HlcClock: wall-clock 代替の HLC タイムスタンプ生成器
use k1s0_hlc::HlcClock;

// NightlyCloseInput は夜間 close Workflow の入力パラメータを宣言する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightlyCloseInput {
    // tenant_id: 処理対象テナント識別子
    pub tenant_id: Uuid,
    // business_date: close する業務日（ISO8601 形式 "YYYY-MM-DD"）
    pub business_date: String,
    // close_timeout_secs: close 処理のタイムアウト秒数（デフォルト 1800 = 30 分）
    pub close_timeout_secs: u64,
    // actor_id: バッチ操作のアクター識別子（Keycloak subject 相当）
    pub actor_id: String,
}

// NightlyCloseOutput は夜間 close Workflow の出力を宣言する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightlyCloseOutput {
    // closed_orders_count: close した受注件数
    pub closed_orders_count: i64,
    // closed_inventory_count: close した在庫件数
    pub closed_inventory_count: i64,
    // audit_event_id: close 完了を記録した audit_event の ID
    pub audit_event_id: Uuid,
    // next_business_date: 翌営業日（ISO8601 形式）
    pub next_business_date: String,
}

// NightlyCloseWorkflow は夜間 close バッチの Workflow 実装
pub struct NightlyCloseWorkflow;

impl NightlyCloseWorkflow {
    // execute は夜間 close バッチの全ステップを実行する
    // 1. オープン状態の受注を close する
    // 2. 在庫確定処理を実行する
    // 3. 翌営業日フラグを更新する
    // 4. close 完了を audit_event に記録する
    pub async fn execute(
        // Workflow 入力パラメータを受け取る
        input: NightlyCloseInput,
        // PostgreSQL 接続プールを Activity 引数として受け取る
        pool: &PgPool,
    ) -> Result<NightlyCloseOutput, WorkflowError> {
        // Workflow 開始をログに記録する
        info!(
            tenant_id = %input.tenant_id,
            business_date = %input.business_date,
            "NightlyClose workflow starting",
        );

        // ---- ステップ 1: オープン受注の close 処理 ----

        // オープン受注の close 処理を実行する
        let closed_orders = close_open_orders(
            // テナント識別子を渡す
            input.tenant_id,
            // 業務日を渡す
            &input.business_date,
            // actor_id を渡す（audit trail に記録する）
            &input.actor_id,
            // 接続プールを渡す
            pool,
        )
        .await
        .map_err(|e| WorkflowError::Database(format!("close_open_orders: {e}")))?;
        // 受注 close 件数をログに記録する
        info!(
            tenant_id = %input.tenant_id,
            closed_orders = closed_orders,
            "open orders closed",
        );

        // ---- ステップ 2: 在庫確定処理 ----

        // 在庫確定処理を実行する
        let closed_inventory = confirm_inventory(
            // テナント識別子を渡す
            input.tenant_id,
            // 業務日を渡す
            &input.business_date,
            // actor_id を渡す（audit trail に記録する）
            &input.actor_id,
            // 接続プールを渡す
            pool,
        )
        .await
        .map_err(|e| WorkflowError::Database(format!("confirm_inventory: {e}")))?;
        // 在庫確定件数をログに記録する
        info!(
            tenant_id = %input.tenant_id,
            closed_inventory = closed_inventory,
            "inventory confirmed",
        );

        // ---- ステップ 3: 翌営業日フラグ更新 ----

        // 翌営業日を計算する（単純に +1 日とする、祝日は別途考慮が必要）
        let next_business_date = calculate_next_business_date(&input.business_date)
            .map_err(|e| WorkflowError::Database(format!("calculate_next_business_date: {e}")))?;
        // 翌営業日更新をログに記録する
        info!(
            tenant_id = %input.tenant_id,
            next_business_date = %next_business_date,
            "next business date calculated",
        );

        // ---- ステップ 4: audit_event 記録 ----

        // close 完了を audit_event に記録する
        let audit_event_id = emit_close_audit_event(
            // テナント識別子を渡す
            input.tenant_id,
            // 業務日を渡す
            &input.business_date,
            // close した受注件数を渡す
            closed_orders,
            // close した在庫件数を渡す
            closed_inventory,
            // actor_id を渡す（audit trail に記録する）
            &input.actor_id,
            // 接続プールを渡す
            pool,
        )
        .await
        .map_err(|e| WorkflowError::Database(format!("emit_close_audit_event: {e}")))?;
        // audit_event 記録をログに記録する
        info!(
            tenant_id = %input.tenant_id,
            audit_event_id = %audit_event_id,
            "nightly close audit event emitted",
        );

        // Workflow 完了をログに記録する
        info!(tenant_id = %input.tenant_id, "NightlyClose workflow completed successfully");

        // 結果を返す
        Ok(NightlyCloseOutput {
            // close した受注件数を設定する
            closed_orders_count: closed_orders,
            // close した在庫件数を設定する
            closed_inventory_count: closed_inventory,
            // audit_event ID を設定する
            audit_event_id,
            // 翌営業日を設定する
            next_business_date,
        })
    }
}

// close_open_orders はオープン受注を HLC タイムスタンプで close し、atomic_triple_write で三表書込する
async fn close_open_orders(
    // 処理対象テナント識別子
    tenant_id: Uuid,
    // close する業務日（ISO8601 形式）
    business_date: &str,
    // バッチ操作の actor_id
    actor_id: &str,
    // PostgreSQL 接続プール
    pool: &PgPool,
) -> Result<i64> {
    // HLC クロックを環境変数から初期化する
    let clock = HlcClock::from_env();
    // HLC タイムスタンプを取得する（wall-clock 禁止規約に従う）
    let hlc_now = clock.now();
    // HLC タイムスタンプを compact 文字列に変換する
    let hlc_str = hlc_now.format_compact();
    // HLC wall_ms を chrono::DateTime<Utc> に変換する（timestamptz バインド用）
    let closed_at = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(hlc_now.wall_ms as i64)
        .unwrap_or_else(|| chrono::Utc::now());
    // TenantContext を生成する（BusinessOp purpose でバッチ操作を実行する）
    let ctx = TenantContext::from_auth(
        // テナント識別子を設定する
        tenant_id,
        // actor_id を設定する
        actor_id.to_string(),
        // 業務操作 purpose を使用する
        SessionPurpose::BusinessOp,
    );
    // SET LOCAL GUC SQL を取得する（RLS FORCE がこの GUC を参照する）
    let set_guc_sql = ctx.to_set_local_sql();
    // AtomicTripleWrite エンジンを生成する（TenantContext と PgPool を渡す）
    let writer = AtomicTripleWrite::new(ctx.clone(), pool.clone());
    // PostgreSQL トランザクションを開始する
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| anyhow::anyhow!("tx begin error: {e}"))?;
    // SET LOCAL GUC を注入する（RLS FORCE がこの GUC を参照する）
    sqlx::query(&set_guc_sql)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("SET LOCAL GUC error: {e}"))?;
    // オープン受注を HLC タイムスタンプで close する（業務日が一致する行のみ対象）
    let result = sqlx::query(
        r#"
        UPDATE k1s0.orders
        SET status = 'closed',
            closed_at = $1,
            closed_hlc = $2
        WHERE tenant_id = current_setting('app.tenant_id')::uuid
          AND status = 'open'
          AND created_date = $3::date
        "#,
    )
    // closed_at に HLC 由来の timestamptz をバインドする
    .bind(closed_at)
    // closed_hlc に HLC compact 文字列をバインドする
    .bind(&hlc_str)
    // 業務日を date 型にバインドする
    .bind(business_date)
    // 同一トランザクション内で UPDATE を実行する
    .execute(&mut *tx)
    .await
    .map_err(|e| anyhow::anyhow!("UPDATE orders error: {e}"))?;
    // UPDATE された行数を取得する
    let closed_count = result.rows_affected() as i64;
    // 閉鎖件数が 0 の場合は警告ログを記録する
    if closed_count == 0 {
        // 閉鎖対象が 0 件であることを警告する
        warn!(
            tenant_id = %tenant_id,
            business_date = %business_date,
            "close_open_orders: no open orders found for business_date",
        );
    }
    // atomic_triple_write で Outbox + Audit も同一 tx に書き込む
    let change = StateChange {
        // aggregate_id として新規 UUID を生成する（close バッチ操作の識別子）
        aggregate_id: Uuid::new_v4(),
        // テナント識別子を設定する
        tenant_id,
        // テナントスコープのテーブルクラスを使用する
        table_class: TableClass::TenantScoped,
        // ペイロードに close 情報を JSON 形式で設定する
        payload: serde_json::json!({
            "event": "nightly_close_orders",
            "business_date": business_date,
            "closed_count": closed_count,
            "closed_hlc": hlc_str,
        }),
        // バージョン 1 を設定する（新規イベントのため）
        version: 1,
    };
    // execute() で外部トランザクションに Outbox + Audit を書き込む
    writer
        .execute(&change, &mut tx)
        .await
        .map_err(|e| anyhow::anyhow!("atomic_triple_write error: {e:?}"))?;
    // トランザクションをコミットする（3 INSERT + UPDATE を同一 txn で確定する）
    tx.commit()
        .await
        .map_err(|e| anyhow::anyhow!("tx commit error: {e}"))?;
    // close した受注件数を返す
    Ok(closed_count)
}

// confirm_inventory は当日の在庫確定処理を実行し、atomic_triple_write で三表書込する
async fn confirm_inventory(
    // 処理対象テナント識別子
    tenant_id: Uuid,
    // 在庫確定する業務日（ISO8601 形式）
    business_date: &str,
    // バッチ操作の actor_id
    actor_id: &str,
    // PostgreSQL 接続プール
    pool: &PgPool,
) -> Result<i64> {
    // HLC クロックを環境変数から初期化する
    let clock = HlcClock::from_env();
    // HLC タイムスタンプを取得する（wall-clock 禁止規約に従う）
    let hlc_now = clock.now();
    // HLC タイムスタンプを compact 文字列に変換する
    let hlc_str = hlc_now.format_compact();
    // HLC wall_ms を chrono::DateTime<Utc> に変換する（timestamptz バインド用）
    let confirmed_at = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(hlc_now.wall_ms as i64)
        .unwrap_or_else(|| chrono::Utc::now());
    // TenantContext を生成する（BusinessOp purpose で在庫確定を実行する）
    let ctx = TenantContext::from_auth(
        // テナント識別子を設定する
        tenant_id,
        // actor_id を設定する
        actor_id.to_string(),
        // 業務操作 purpose を使用する
        SessionPurpose::BusinessOp,
    );
    // SET LOCAL GUC SQL を取得する（RLS FORCE がこの GUC を参照する）
    let set_guc_sql = ctx.to_set_local_sql();
    // AtomicTripleWrite エンジンを生成する
    let writer = AtomicTripleWrite::new(ctx.clone(), pool.clone());
    // PostgreSQL トランザクションを開始する
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| anyhow::anyhow!("tx begin error: {e}"))?;
    // SET LOCAL GUC を注入する
    sqlx::query(&set_guc_sql)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("SET LOCAL GUC error: {e}"))?;
    // 当日の在庫を確定する（pending 状態の inventory_item を confirmed に更新する）
    let result = sqlx::query(
        r#"
        UPDATE k1s0.inventory_items
        SET status = 'confirmed',
            confirmed_at = $1,
            confirmed_hlc = $2
        WHERE tenant_id = current_setting('app.tenant_id')::uuid
          AND status = 'pending'
          AND business_date = $3::date
        "#,
    )
    // confirmed_at に HLC 由来の timestamptz をバインドする
    .bind(confirmed_at)
    // confirmed_hlc に HLC compact 文字列をバインドする
    .bind(&hlc_str)
    // 業務日をバインドする
    .bind(business_date)
    // 同一トランザクション内で UPDATE を実行する
    .execute(&mut *tx)
    .await
    .map_err(|e| anyhow::anyhow!("UPDATE inventory_items error: {e}"))?;
    // 確定した在庫件数を取得する
    let confirmed_count = result.rows_affected() as i64;
    // atomic_triple_write で Outbox + Audit も同一 tx に書き込む
    let change = StateChange {
        // aggregate_id として新規 UUID を生成する（在庫確定バッチ操作の識別子）
        aggregate_id: Uuid::new_v4(),
        // テナント識別子を設定する
        tenant_id,
        // テナントスコープのテーブルクラスを使用する
        table_class: TableClass::TenantScoped,
        // ペイロードに在庫確定情報を JSON 形式で設定する
        payload: serde_json::json!({
            "event": "nightly_confirm_inventory",
            "business_date": business_date,
            "confirmed_count": confirmed_count,
            "confirmed_hlc": hlc_str,
        }),
        // バージョン 1 を設定する（新規イベントのため）
        version: 1,
    };
    // execute() で外部トランザクションに Outbox + Audit を書き込む
    writer
        .execute(&change, &mut tx)
        .await
        .map_err(|e| anyhow::anyhow!("atomic_triple_write error: {e:?}"))?;
    // トランザクションをコミットする（3 INSERT + UPDATE を同一 txn で確定する）
    tx.commit()
        .await
        .map_err(|e| anyhow::anyhow!("tx commit error: {e}"))?;
    // 確定した在庫件数を返す
    Ok(confirmed_count)
}

// calculate_next_business_date は翌営業日を計算する
fn calculate_next_business_date(business_date: &str) -> Result<String> {
    // chrono で日付をパースして +1 日を計算する
    use chrono::NaiveDate;
    // 日付をパースする
    let date = NaiveDate::parse_from_str(business_date, "%Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("date parse error: {e}"))?;
    // 翌日を計算する
    let next = date.succ_opt().ok_or_else(|| anyhow::anyhow!("date overflow"))?;
    // ISO8601 形式で返す
    Ok(next.format("%Y-%m-%d").to_string())
}

// emit_close_audit_event は close 完了の audit_event を atomic_triple_write 経由で記録する
async fn emit_close_audit_event(
    // 処理対象テナント識別子
    tenant_id: Uuid,
    // close した業務日
    business_date: &str,
    // close した受注件数
    closed_orders: i64,
    // close した在庫件数
    closed_inventory: i64,
    // バッチ操作の actor_id
    actor_id: &str,
    // PostgreSQL 接続プール
    pool: &PgPool,
) -> Result<Uuid> {
    // HLC クロックを環境変数から初期化する
    let clock = HlcClock::from_env();
    // HLC タイムスタンプを取得する（audit emit 時刻として HLC を使用する）
    let hlc_now = clock.now();
    // HLC タイムスタンプを compact 文字列に変換する
    let hlc_str = hlc_now.format_compact();
    // TenantContext を生成する（BusinessOp purpose で audit を記録する）
    let ctx = TenantContext::from_auth(
        // テナント識別子を設定する
        tenant_id,
        // actor_id を設定する
        actor_id.to_string(),
        // 業務操作 purpose を使用する
        SessionPurpose::BusinessOp,
    );
    // AtomicTripleWrite エンジンを生成する
    let writer = AtomicTripleWrite::new(ctx, pool.clone());
    // 夜間 close 完了サマリーを Audit ペイロードとして組み立てる
    let aggregate_id = Uuid::new_v4();
    // StateChange を構築する（夜間 close 完了サマリーを Audit ペイロードとして含む）
    let change = StateChange {
        // aggregate_id として新規 UUID を設定する（夜間 close 完了イベントの識別子）
        aggregate_id,
        // テナント識別子を設定する
        tenant_id,
        // テナントスコープのテーブルクラスを使用する
        table_class: TableClass::TenantScoped,
        // ペイロードに夜間 close サマリーを JSON 形式で設定する
        payload: serde_json::json!({
            "event": "nightly_close_completed",
            "business_date": business_date,
            "closed_orders": closed_orders,
            "closed_inventory": closed_inventory,
            "emit_hlc": hlc_str,
        }),
        // バージョン 1 を設定する（新規イベントのため）
        version: 1,
    };
    // execute_triple_write() で独立したトランザクションで三表書込する
    writer
        .execute_triple_write(&change)
        .await
        .map_err(|e| anyhow::anyhow!("atomic_triple_write error: {e:?}"))?;
    // 三表書込が完了した aggregate_id を audit_event_id として返す
    Ok(aggregate_id)
}

// NightlyCloseWorkflow のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールをインポートする
    use super::*;

    // 翌営業日計算の単体テスト（DB 接続不要）
    #[test]
    fn test_calculate_next_business_date() {
        // 業務日 "2024-01-15" の翌日が "2024-01-16" であることを確認する
        let result = calculate_next_business_date("2024-01-15").unwrap();
        // 翌営業日が正しく計算されることを確認する
        assert_eq!(result, "2024-01-16");
    }

    // 月末の翌日計算テスト
    #[test]
    fn test_calculate_next_business_date_month_end() {
        // 1月末 "2024-01-31" の翌日が "2024-02-01" であることを確認する
        let result = calculate_next_business_date("2024-01-31").unwrap();
        // 月をまたぐ翌日計算が正しいことを確認する
        assert_eq!(result, "2024-02-01");
    }

    // 不正な日付入力でエラーを返すことを確認するテスト
    #[test]
    fn test_calculate_next_business_date_invalid() {
        // 不正な日付フォーマットでエラーが返ることを確認する
        let result = calculate_next_business_date("not-a-date");
        // エラーが返ることを確認する
        assert!(result.is_err(), "不正な日付でエラーを返すべき");
    }
}
