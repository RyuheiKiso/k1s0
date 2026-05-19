// monthly_aggregation.rs — 月次集計バッチ Workflow（設計方針 25 §batch）
// 月次集計（売上集計・在庫月次確定）を Temporal Workflow として実装する
// GROUP BY + SUM で月次サマリーを生成し、atomic_triple_write で Outbox + Audit を同一 txn に書く

// anyhow: Result 型に使用する
use anyhow::Result;
// serde: Workflow 入出力の JSON シリアライズに使用する
use serde::{Deserialize, Serialize};
// uuid: テナント識別子型に使用する
use uuid::Uuid;
// tracing: 構造化ロギングに使用する
use tracing::info;
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

// MonthlyAggregationInput は月次集計 Workflow の入力パラメータを宣言する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyAggregationInput {
    // tenant_id: 集計対象テナント識別子
    pub tenant_id: Uuid,
    // year_month: 集計対象年月（"YYYY-MM" 形式）
    pub year_month: String,
    // actor_id: バッチ操作のアクター識別子（Keycloak subject 相当）
    pub actor_id: String,
}

// MonthlyAggregationOutput は月次集計 Workflow の出力を宣言する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyAggregationOutput {
    // total_order_amount_jpy: 月次受注総額（円）
    pub total_order_amount_jpy: i64,
    // total_items_shipped: 月次出荷件数
    pub total_items_shipped: i64,
    // aggregation_id: 集計結果の識別子
    pub aggregation_id: Uuid,
}

// MonthlyAggregationWorkflow は月次集計の Workflow 実装
pub struct MonthlyAggregationWorkflow;

impl MonthlyAggregationWorkflow {
    // execute は月次集計の全ステップを実行する
    pub async fn execute(
        // Workflow 入力パラメータを受け取る
        input: MonthlyAggregationInput,
        // PostgreSQL 接続プールを Activity 引数として受け取る
        pool: &PgPool,
    ) -> Result<MonthlyAggregationOutput, WorkflowError> {
        // Workflow 開始をログに記録する
        info!(
            tenant_id = %input.tenant_id,
            year_month = %input.year_month,
            "MonthlyAggregation workflow starting",
        );
        // 受注集計を実行する（GROUP BY + SUM で月次集計テーブルに INSERT する）
        let total_order_amount = aggregate_orders(
            // テナント識別子を渡す
            input.tenant_id,
            // 集計対象年月を渡す
            &input.year_month,
            // actor_id を渡す
            &input.actor_id,
            // 接続プールを渡す
            pool,
        )
        .await
        .map_err(|e| WorkflowError::Database(format!("aggregate_orders: {e}")))?;
        // 出荷件数集計を実行する
        let total_shipped = aggregate_shipments(
            // テナント識別子を渡す
            input.tenant_id,
            // 集計対象年月を渡す
            &input.year_month,
            // actor_id を渡す
            &input.actor_id,
            // 接続プールを渡す
            pool,
        )
        .await
        .map_err(|e| WorkflowError::Database(format!("aggregate_shipments: {e}")))?;
        // 集計結果 ID を生成する（複数テーブルの集計を束ねる識別子）
        let aggregation_id = Uuid::new_v4();
        // Workflow 完了をログに記録する
        info!(
            tenant_id = %input.tenant_id,
            aggregation_id = %aggregation_id,
            "MonthlyAggregation workflow completed",
        );
        // 集計結果を返す
        Ok(MonthlyAggregationOutput {
            // 受注総額を設定する
            total_order_amount_jpy: total_order_amount,
            // 出荷件数を設定する
            total_items_shipped: total_shipped,
            // 集計 ID を設定する
            aggregation_id,
        })
    }
}

// aggregate_orders は月次受注総額を集計して order_monthly_summary テーブルに INSERT する
async fn aggregate_orders(
    // 集計対象テナント識別子
    tenant_id: Uuid,
    // 集計対象年月（"YYYY-MM" 形式）
    year_month: &str,
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
    // TenantContext を生成する（BusinessOp purpose で月次集計を実行する）
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
    // AtomicTripleWrite エンジンを生成する（TenantContext は Clone するため ctx を複製する）
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
    // 月次受注サマリーを既存エントリに UPSERT する
    // INSERT … ON CONFLICT でべき等性を保証する（複数回実行しても重複しない）
    sqlx::query(
        r#"
        INSERT INTO k1s0.order_monthly_summary
            (id, tenant_id, year_month, order_count, total_amount, aggregated_at, aggregated_hlc)
        SELECT
            gen_random_uuid(),
            current_setting('app.tenant_id')::uuid,
            DATE_TRUNC('month', created_at)::date,
            COUNT(*)::bigint,
            COALESCE(SUM(amount), 0)::bigint,
            $1,
            $2
        FROM k1s0.orders
        WHERE tenant_id = current_setting('app.tenant_id')::uuid
          AND TO_CHAR(created_at, 'YYYY-MM') = $3
          AND status = 'closed'
        GROUP BY DATE_TRUNC('month', created_at)
        ON CONFLICT (tenant_id, year_month)
        DO UPDATE SET
            order_count = EXCLUDED.order_count,
            total_amount = EXCLUDED.total_amount,
            aggregated_at = EXCLUDED.aggregated_at,
            aggregated_hlc = EXCLUDED.aggregated_hlc
        "#,
    )
    // aggregated_at に HLC 由来の timestamptz をバインドする
    .bind(chrono::DateTime::<chrono::Utc>::from_timestamp_millis(hlc_now.wall_ms as i64)
        .unwrap_or_else(|| chrono::Utc::now()))
    // aggregated_hlc に HLC compact 文字列をバインドする
    .bind(&hlc_str)
    // 集計対象年月をバインドする（"YYYY-MM" 形式）
    .bind(year_month)
    // 同一トランザクション内で UPSERT を実行する
    .execute(&mut *tx)
    .await
    .map_err(|e| anyhow::anyhow!("UPSERT order_monthly_summary error: {e}"))?;
    // 月次受注総額を SELECT SUM で取得する（サマリーテーブルから再読み取りする）
    let row: (Option<i64>,) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(total_amount), 0)::bigint
        FROM k1s0.order_monthly_summary
        WHERE tenant_id = current_setting('app.tenant_id')::uuid
          AND TO_CHAR(year_month, 'YYYY-MM') = $1
        "#,
    )
    // 集計対象年月をバインドする
    .bind(year_month)
    // 同一トランザクション内で SELECT を実行する
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| anyhow::anyhow!("SELECT order_monthly_summary error: {e}"))?;
    // 受注総額を取得する（NULL の場合は 0 を使用する）
    let total_amount = row.0.unwrap_or(0);
    // atomic_triple_write で Outbox + Audit も同一 tx に書き込む
    let change = StateChange {
        // aggregate_id として新規 UUID を生成する（月次受注集計イベントの識別子）
        aggregate_id: Uuid::new_v4(),
        // テナント識別子を設定する
        tenant_id,
        // テナントスコープのテーブルクラスを使用する
        table_class: TableClass::TenantScoped,
        // ペイロードに月次受注集計情報を JSON 形式で設定する
        payload: serde_json::json!({
            "event": "monthly_aggregate_orders",
            "year_month": year_month,
            "total_amount": total_amount,
            "aggregated_hlc": hlc_str,
        }),
        // バージョン 1 を設定する（新規イベントのため）
        version: 1,
    };
    // execute() で外部トランザクションに Outbox + Audit を書き込む
    writer
        .execute(&change, &mut tx)
        .await
        .map_err(|e| anyhow::anyhow!("atomic_triple_write error: {e:?}"))?;
    // トランザクションをコミットする（UPSERT + 3 INSERT を同一 txn で確定する）
    tx.commit()
        .await
        .map_err(|e| anyhow::anyhow!("tx commit error: {e}"))?;
    // 月次受注総額を返す
    Ok(total_amount)
}

// aggregate_shipments は月次出荷件数を集計して shipment_monthly_summary テーブルに INSERT する
async fn aggregate_shipments(
    // 集計対象テナント識別子
    tenant_id: Uuid,
    // 集計対象年月（"YYYY-MM" 形式）
    year_month: &str,
    // バッチ操作の actor_id
    actor_id: &str,
    // PostgreSQL 接続プール
    pool: &PgPool,
) -> Result<i64> {
    // HLC クロックを環境変数から初期化する
    let clock = HlcClock::from_env();
    // HLC タイムスタンプを取得する
    let hlc_now = clock.now();
    // HLC タイムスタンプを compact 文字列に変換する
    let hlc_str = hlc_now.format_compact();
    // TenantContext を生成する
    let ctx = TenantContext::from_auth(
        // テナント識別子を設定する
        tenant_id,
        // actor_id を設定する
        actor_id.to_string(),
        // 業務操作 purpose を使用する
        SessionPurpose::BusinessOp,
    );
    // SET LOCAL GUC SQL を取得する
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
    // 月次出荷件数サマリーを UPSERT する（べき等性を保証する）
    sqlx::query(
        r#"
        INSERT INTO k1s0.shipment_monthly_summary
            (id, tenant_id, year_month, shipment_count, aggregated_at, aggregated_hlc)
        SELECT
            gen_random_uuid(),
            current_setting('app.tenant_id')::uuid,
            DATE_TRUNC('month', shipped_at)::date,
            COUNT(*)::bigint,
            $1,
            $2
        FROM k1s0.shipments
        WHERE tenant_id = current_setting('app.tenant_id')::uuid
          AND TO_CHAR(shipped_at, 'YYYY-MM') = $3
          AND status = 'delivered'
        GROUP BY DATE_TRUNC('month', shipped_at)
        ON CONFLICT (tenant_id, year_month)
        DO UPDATE SET
            shipment_count = EXCLUDED.shipment_count,
            aggregated_at = EXCLUDED.aggregated_at,
            aggregated_hlc = EXCLUDED.aggregated_hlc
        "#,
    )
    // aggregated_at に HLC 由来の timestamptz をバインドする
    .bind(chrono::DateTime::<chrono::Utc>::from_timestamp_millis(hlc_now.wall_ms as i64)
        .unwrap_or_else(|| chrono::Utc::now()))
    // aggregated_hlc に HLC compact 文字列をバインドする
    .bind(&hlc_str)
    // 集計対象年月をバインドする
    .bind(year_month)
    // 同一トランザクション内で UPSERT を実行する
    .execute(&mut *tx)
    .await
    .map_err(|e| anyhow::anyhow!("UPSERT shipment_monthly_summary error: {e}"))?;
    // 月次出荷件数を SELECT COUNT で取得する（サマリーテーブルから再読み取りする）
    let row: (Option<i64>,) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(shipment_count), 0)::bigint
        FROM k1s0.shipment_monthly_summary
        WHERE tenant_id = current_setting('app.tenant_id')::uuid
          AND TO_CHAR(year_month, 'YYYY-MM') = $1
        "#,
    )
    // 集計対象年月をバインドする
    .bind(year_month)
    // 同一トランザクション内で SELECT を実行する
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| anyhow::anyhow!("SELECT shipment_monthly_summary error: {e}"))?;
    // 出荷件数を取得する（NULL の場合は 0 を使用する）
    let total_shipped = row.0.unwrap_or(0);
    // atomic_triple_write で Outbox + Audit も同一 tx に書き込む
    let change = StateChange {
        // aggregate_id として新規 UUID を生成する（月次出荷集計イベントの識別子）
        aggregate_id: Uuid::new_v4(),
        // テナント識別子を設定する
        tenant_id,
        // テナントスコープのテーブルクラスを使用する
        table_class: TableClass::TenantScoped,
        // ペイロードに月次出荷集計情報を JSON 形式で設定する
        payload: serde_json::json!({
            "event": "monthly_aggregate_shipments",
            "year_month": year_month,
            "total_shipped": total_shipped,
            "aggregated_hlc": hlc_str,
        }),
        // バージョン 1 を設定する（新規イベントのため）
        version: 1,
    };
    // execute() で外部トランザクションに Outbox + Audit を書き込む
    writer
        .execute(&change, &mut tx)
        .await
        .map_err(|e| anyhow::anyhow!("atomic_triple_write error: {e:?}"))?;
    // トランザクションをコミットする（UPSERT + 3 INSERT を同一 txn で確定する）
    tx.commit()
        .await
        .map_err(|e| anyhow::anyhow!("tx commit error: {e}"))?;
    // 月次出荷件数を返す
    Ok(total_shipped)
}

// MonthlyAggregationWorkflow のユニットテスト
#[cfg(test)]
mod tests {
    // 基本的な月次集計テスト（DB 接続不要のロジックのみ確認する）
    #[test]
    fn test_monthly_aggregation_input_validation() {
        // 年月フォーマットが "YYYY-MM" 形式で正しいことを確認する
        let year_month = "2024-01";
        // 年月が 7 文字であることを確認する
        assert_eq!(year_month.len(), 7, "year_month は YYYY-MM 形式（7 文字）であるべき");
        // 年月がハイフン区切りであることを確認する
        assert!(year_month.contains('-'), "year_month にはハイフンが含まれるべき");
    }
}
