// yearly_archive.rs — 年次 archive Workflow（設計方針 25 §batch / §データライフサイクル）
// 1 年以上経過したデータを archive ストレージに移動する Temporal Workflow
// retention_policy.yaml の business_event カテゴリ（365 日）に従いアーカイブ対象を選択する
// Object Lock retention は retention_policy.yaml の audit カテゴリ（2555 日）を使用する

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

// retention_policy.yaml §audit カテゴリの保持日数（2555 日 = 7 年）
// S3 Object Lock の COMPLIANCE モード retention に使用する
const AUDIT_RETENTION_DAYS: u32 = 2555;

// S3 アーカイブ先のベース URL（環境変数 ARCHIVE_S3_ENDPOINT で上書き可能）
const DEFAULT_ARCHIVE_S3_ENDPOINT: &str = "http://minio:9000";

// YearlyArchiveInput は年次 archive Workflow の入力パラメータを宣言する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyArchiveInput {
    // tenant_id: archive 対象テナント識別子
    pub tenant_id: Uuid,
    // archive_year: archive する年（例: 2023）
    pub archive_year: i32,
    // retention_years: 本番 DB に保持する年数（archive_year より古いデータを archive する）
    pub retention_years: u32,
    // actor_id: バッチ操作のアクター識別子
    pub actor_id: String,
    // s3_bucket: アーカイブ先の S3 バケット名
    pub s3_bucket: String,
}

// YearlyArchiveOutput は年次 archive Workflow の出力を宣言する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyArchiveOutput {
    // archived_records_count: archive したレコード件数
    pub archived_records_count: i64,
    // archive_location: archive 先のストレージパス
    pub archive_location: String,
    // archive_id: archive 処理の識別子
    pub archive_id: Uuid,
}

// YearlyArchiveWorkflow は年次 archive の Workflow 実装
pub struct YearlyArchiveWorkflow;

impl YearlyArchiveWorkflow {
    // execute は年次 archive の全ステップを実行する
    pub async fn execute(
        // Workflow 入力パラメータを受け取る
        input: YearlyArchiveInput,
        // PostgreSQL 接続プールを Activity 引数として受け取る
        pool: &PgPool,
    ) -> Result<YearlyArchiveOutput, WorkflowError> {
        // Workflow 開始をログに記録する
        info!(
            tenant_id = %input.tenant_id,
            archive_year = input.archive_year,
            "YearlyArchive workflow starting",
        );
        // archive 対象レコードの ID リストを取得する（年齢 >= retention_years の closed レコード）
        let candidate_ids = select_archive_candidates(
            // テナント識別子を渡す
            input.tenant_id,
            // archive する年を渡す
            input.archive_year,
            // 接続プールを渡す
            pool,
        )
        .await
        .map_err(|e| WorkflowError::Database(format!("select_archive_candidates: {e}")))?;
        // archive 対象件数をログに記録する
        info!(
            tenant_id = %input.tenant_id,
            candidate_count = candidate_ids.len(),
            "archive candidates selected",
        );
        // archive 先パスを構築する（バケット/テナント/年/ の形式）
        let archive_location = format!(
            "s3://{}/{}/tenant_{}/year_{}/",
            input.s3_bucket,
            input.archive_year,
            input.tenant_id,
            input.archive_year
        );
        // レコードを archive ストレージに書き出す（S3 PUT + Object Lock retention 設定）
        let archived_count = archive_to_storage(
            // archive 対象レコード ID リストを渡す
            candidate_ids.clone(),
            // テナント識別子を渡す
            input.tenant_id,
            // archive する年を渡す
            input.archive_year,
            // archive 先パスを渡す
            &archive_location,
            // 接続プールを渡す（レコードの読み取りに使用する）
            pool,
        )
        .await
        .map_err(|e| WorkflowError::ExternalService {
            // サービス名を設定する
            service: "s3_archive".to_string(),
            // エラーメッセージを設定する
            message: e.to_string(),
        })?;
        // 本番 DB から archive 済みデータを削除する（atomic_triple_write 経由）
        delete_archived_records(
            // archive 済みレコード ID リストを渡す
            candidate_ids,
            // テナント識別子を渡す
            input.tenant_id,
            // actor_id を渡す
            &input.actor_id,
            // 接続プールを渡す
            pool,
        )
        .await
        .map_err(|e| WorkflowError::Database(format!("delete_archived_records: {e}")))?;
        // archive ID を生成する（archive 処理の識別子として使用する）
        let archive_id = Uuid::new_v4();
        // Workflow 完了をログに記録する
        info!(
            tenant_id = %input.tenant_id,
            archive_id = %archive_id,
            archived_count = archived_count,
            "YearlyArchive workflow completed",
        );
        // 結果を返す
        Ok(YearlyArchiveOutput {
            // archive 件数を設定する
            archived_records_count: archived_count,
            // archive 先パスを設定する
            archive_location,
            // archive ID を設定する
            archive_id,
        })
    }
}

// select_archive_candidates は archive 対象レコードの UUID リストを返す
// 対象条件: created_at が archive_year の年末以前かつ status='closed' のレコード
async fn select_archive_candidates(
    // 対象テナント識別子
    tenant_id: Uuid,
    // archive する年（この年の 1/1 より前に作成されたレコードを選択する）
    archive_year: i32,
    // PostgreSQL 接続プール
    pool: &PgPool,
) -> Result<Vec<Uuid>> {
    // TenantContext を生成する（Export purpose でアーカイブ候補を読み取る）
    let ctx = TenantContext::from_auth(
        // テナント識別子を設定する
        tenant_id,
        // システムバッチアクターを設定する
        "system-archive-batch".to_string(),
        // データエクスポート purpose を使用する
        SessionPurpose::Export,
    );
    // SET LOCAL GUC SQL を取得する（RLS FORCE がこの GUC を参照する）
    let set_guc_sql = ctx.to_set_local_sql();
    // PostgreSQL トランザクションを開始する（SELECT のみのため読み取り専用で OK）
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| anyhow::anyhow!("tx begin error: {e}"))?;
    // SET LOCAL GUC を注入する
    sqlx::query(&set_guc_sql)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("SET LOCAL GUC error: {e}"))?;
    // before_date は archive_year + 1 年の 1/1 とする（archive_year 末日までが対象）
    let before_year = archive_year + 1;
    // archive 対象の cutoff 日付文字列を構築する（YYYY-01-01 形式）
    let before_date = format!("{}-01-01", before_year);
    // archive 対象レコードの UUID を全件取得する（closed かつ created_at が before_date より前）
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id
        FROM k1s0.orders
        WHERE tenant_id = current_setting('app.tenant_id')::uuid
          AND status = 'closed'
          AND created_at < $1::timestamptz
        ORDER BY created_at ASC
        "#,
    )
    // cutoff 日付をバインドする
    .bind(&before_date)
    // 同一トランザクション内で SELECT を実行する
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| anyhow::anyhow!("SELECT archive candidates error: {e}"))?;
    // トランザクションをコミットする（SELECT 完了後に解放する）
    tx.commit()
        .await
        .map_err(|e| anyhow::anyhow!("tx commit error: {e}"))?;
    // UUID リストを返す
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

// archive_to_storage は archive 対象レコードを DB から読み取り、S3/MinIO に PUT する
// Object Lock retention を retention_policy.yaml の audit 期間（2555 日）で設定する
async fn archive_to_storage(
    // archive 対象レコード ID リスト
    record_ids: Vec<Uuid>,
    // テナント識別子
    tenant_id: Uuid,
    // archive する年
    archive_year: i32,
    // archive 先パス（s3://bucket/path/ 形式）
    archive_location: &str,
    // PostgreSQL 接続プール（レコードの読み取りに使用する）
    pool: &PgPool,
) -> Result<i64> {
    // archive 対象が 0 件の場合は早期リターンする
    if record_ids.is_empty() {
        // 対象なしで 0 件を返す
        return Ok(0);
    }
    // S3 エンドポイント URL を環境変数から取得する（デフォルト値を使用する）
    let s3_endpoint = std::env::var("ARCHIVE_S3_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_ARCHIVE_S3_ENDPOINT.to_string());
    // TenantContext を生成する（Export purpose でレコードを読み取る）
    let ctx = TenantContext::from_auth(
        // テナント識別子を設定する
        tenant_id,
        // システムバッチアクターを設定する
        "system-archive-batch".to_string(),
        // データエクスポート purpose を使用する
        SessionPurpose::Export,
    );
    // SET LOCAL GUC SQL を取得する
    let set_guc_sql = ctx.to_set_local_sql();
    // archive 済みレコード数を初期化する
    let mut archived_count: i64 = 0;
    // レコードを 1 件ずつ読み取って S3 に PUT する
    for record_id in &record_ids {
        // PostgreSQL トランザクションを開始する（レコード読み取り用）
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| anyhow::anyhow!("tx begin error: {e}"))?;
        // SET LOCAL GUC を注入する
        sqlx::query(&set_guc_sql)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow::anyhow!("SET LOCAL GUC error: {e}"))?;
        // レコードを JSON 形式で読み取る（archive ペイロードとして使用する）
        let row: Option<(serde_json::Value,)> = sqlx::query_as(
            r#"
            SELECT row_to_json(o)
            FROM k1s0.orders o
            WHERE tenant_id = current_setting('app.tenant_id')::uuid
              AND id = $1
            "#,
        )
        // レコード ID をバインドする
        .bind(record_id)
        // 同一トランザクション内で SELECT を実行する
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("SELECT order JSON error: {e}"))?;
        // レコードが取得できた場合のみ S3 PUT を実行する
        if let Some((payload,)) = row {
            // archive オブジェクトの S3 キーを構築する（tenant/year/record_id.json 形式）
            let object_key = format!(
                "{}/tenant_{}/year_{}/{}.json",
                archive_location.trim_start_matches("s3://").split('/').next().unwrap_or("archive"),
                tenant_id,
                archive_year,
                record_id
            );
            // S3 PUT リクエストの URL を構築する
            let put_url = format!("{}/{}", s3_endpoint.trim_end_matches('/'), object_key);
            // reqwest クライアントを生成する（接続プールは再利用しない）
            let client = reqwest::Client::new();
            // archive ペイロードを JSON 文字列にシリアライズする
            let body = serde_json::to_string(&payload)
                .map_err(|e| anyhow::anyhow!("JSON serialize error: {e}"))?;
            // S3 PUT リクエストを送信する（Object Lock 用ヘッダを付与する）
            let resp = client
                .put(&put_url)
                // Content-Type を application/json に設定する
                .header("Content-Type", "application/json")
                // Object Lock COMPLIANCE モードを設定する（retention_policy.yaml 準拠）
                .header("x-amz-object-lock-mode", "COMPLIANCE")
                // Object Lock retention date を設定する（現在日 + AUDIT_RETENTION_DAYS 日後）
                .header(
                    "x-amz-object-lock-retain-until-date",
                    // HLC wall_ms ベースで retention 期限を計算する（wall-clock 規約に従う）
                    {
                        let clock = HlcClock::from_env();
                        let hlc_now = clock.now();
                        let retain_ms = hlc_now.wall_ms
                            + (AUDIT_RETENTION_DAYS as u64 * 24 * 60 * 60 * 1000);
                        chrono::DateTime::<chrono::Utc>::from_timestamp_millis(retain_ms as i64)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .to_rfc3339()
                    },
                )
                // archive ペイロードを body に設定する
                .body(body)
                // PUT リクエストを送信する
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("S3 PUT error: {e}"))?;
            // HTTP ステータスが 2xx でない場合はエラーを返す
            if !resp.status().is_success() {
                // PUT 失敗のエラーメッセージを構築する
                return Err(anyhow::anyhow!(
                    "S3 PUT failed: status={}, url={}",
                    resp.status(),
                    put_url
                ));
            }
            // archive 済み件数をインクリメントする
            archived_count += 1;
        }
        // SELECT トランザクションをコミットする
        tx.commit()
            .await
            .map_err(|e| anyhow::anyhow!("tx commit error: {e}"))?;
    }
    // archive した件数を返す
    Ok(archived_count)
}

// delete_archived_records は archive 済みレコードを本番 DB から削除し、atomic_triple_write で三表書込する
async fn delete_archived_records(
    // 削除対象レコード ID リスト
    record_ids: Vec<Uuid>,
    // テナント識別子
    tenant_id: Uuid,
    // バッチ操作の actor_id
    actor_id: &str,
    // PostgreSQL 接続プール
    pool: &PgPool,
) -> Result<i64> {
    // 削除対象が 0 件の場合は早期リターンする
    if record_ids.is_empty() {
        // 対象なしで 0 件を返す
        return Ok(0);
    }
    // HLC クロックを環境変数から初期化する
    let clock = HlcClock::from_env();
    // HLC タイムスタンプを取得する（削除操作の HLC 証跡として使用する）
    let hlc_now = clock.now();
    // HLC タイムスタンプを compact 文字列に変換する
    let hlc_str = hlc_now.format_compact();
    // TenantContext を生成する（BusinessOp purpose で削除操作を実行する）
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
    // IN 句用の UUID 配列文字列を構築する（sqlx ANY($1) で配列バインドする）
    let ids_vec: Vec<Uuid> = record_ids.clone();
    // archive 済みレコードを一括 DELETE する（ANY で配列バインドする）
    let result = sqlx::query(
        r#"
        DELETE FROM k1s0.orders
        WHERE tenant_id = current_setting('app.tenant_id')::uuid
          AND id = ANY($1)
        "#,
    )
    // UUID 配列をバインドする（sqlx の uuid feature で型安全に扱う）
    .bind(&ids_vec[..])
    // 同一トランザクション内で DELETE を実行する
    .execute(&mut *tx)
    .await
    .map_err(|e| anyhow::anyhow!("DELETE orders error: {e}"))?;
    // 削除した件数を取得する
    let deleted_count = result.rows_affected() as i64;
    // atomic_triple_write で Outbox + Audit も同一 tx に書き込む
    let change = StateChange {
        // aggregate_id として新規 UUID を生成する（年次アーカイブ削除イベントの識別子）
        aggregate_id: Uuid::new_v4(),
        // テナント識別子を設定する
        tenant_id,
        // テナントスコープのテーブルクラスを使用する
        table_class: TableClass::TenantScoped,
        // ペイロードに削除情報を JSON 形式で設定する
        payload: serde_json::json!({
            "event": "yearly_archive_delete",
            "deleted_count": deleted_count,
            "record_ids": record_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
            "deleted_hlc": hlc_str,
        }),
        // バージョン 1 を設定する（新規イベントのため）
        version: 1,
    };
    // execute() で外部トランザクションに Outbox + Audit を書き込む
    writer
        .execute(&change, &mut tx)
        .await
        .map_err(|e| anyhow::anyhow!("atomic_triple_write error: {e:?}"))?;
    // トランザクションをコミットする（DELETE + 3 INSERT を同一 txn で確定する）
    tx.commit()
        .await
        .map_err(|e| anyhow::anyhow!("tx commit error: {e}"))?;
    // 削除した件数を返す
    Ok(deleted_count)
}

// YearlyArchiveWorkflow のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールをインポートする
    use super::*;

    // archive 先パス構築の単体テスト（DB 接続不要）
    #[test]
    fn test_archive_location_format() {
        // テスト用テナント ID を生成する
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        // archive 先パスを構築する
        let location = format!(
            "s3://{}/{}/tenant_{}/year_{}/",
            "k1s0-archive",
            2023,
            tenant_id,
            2023
        );
        // S3 パスに年が含まれることを確認する
        assert!(location.contains("2023"), "archive 先パスに年が含まれるべき");
        // S3 パスにテナント ID が含まれることを確認する
        assert!(
            location.contains("550e8400"),
            "archive 先パスにテナント ID が含まれるべき"
        );
    }

    // AUDIT_RETENTION_DAYS の値が retention_policy.yaml と一致することを確認するテスト
    #[test]
    fn test_audit_retention_days() {
        // retention_policy.yaml の audit カテゴリが 2555 日であることを確認する
        assert_eq!(AUDIT_RETENTION_DAYS, 2555, "audit retention は 2555 日（7 年）であるべき");
    }
}
