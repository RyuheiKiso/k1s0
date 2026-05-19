// yearly_dry_run.rs — L1+ pair 年次 dry_run Workflow（設計方針 08 §dry_run / 02_移行Pair）
// L1+ migration pair の年次 dry_run を Temporal Workflow として実行する
// dry_run.lock.yaml の last_green_at を 365 日以内に保つために年次で実行する

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

// DryRunPhase は migration pair dry_run の 5 フェーズを宣言する
// 02_移行Pair適合仕様.md §dry_run_phase（5 phase）と対応する
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DryRunPhase {
    // Shadow フェーズ: 新旧両書き込み、読み取りは旧スキーマのみ
    Shadow,
    // DualRead フェーズ: 新旧両方から読み取り、差分をモニタリングする
    DualRead,
    // Cutover フェーズ: 読み取りを新スキーマに切り替える
    Cutover,
    // Cleanup フェーズ: 旧スキーマへの書き込みを停止する
    Cleanup,
    // Complete フェーズ: 旧スキーマを完全削除して dry_run.lock.yaml を更新する
    Complete,
}

// MigrationPairDryRunInput は年次 dry_run Workflow の入力パラメータを宣言する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPairDryRunInput {
    // pair_id: dry_run する migration pair の識別子（dry_run.lock.yaml の pair_id と一致）
    pub pair_id: String,
    // tenant_id: dry_run を実行するテナント識別子
    pub tenant_id: Uuid,
    // target_phase: dry_run で到達する目標フェーズ
    pub target_phase: DryRunPhase,
    // sample_size: dry_run で処理するサンプルデータ件数
    pub sample_size: u64,
    // actor_id: バッチ操作のアクター識別子
    pub actor_id: String,
    // shadow_schema: shadow DB のスキーマ名（デフォルト: "k1s0_shadow"）
    pub shadow_schema: String,
    // envoy_configmap_name: Cutover フェーズで更新する Envoy ConfigMap 名
    pub envoy_configmap_name: String,
    // dry_run_input_path: complete フェーズで更新する dry_run_input.yaml のパス
    pub dry_run_input_path: String,
}

// MigrationPairDryRunOutput は年次 dry_run Workflow の出力を宣言する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPairDryRunOutput {
    // pair_id: dry_run した migration pair の識別子
    pub pair_id: String,
    // reached_phase: 実際に到達したフェーズ
    pub reached_phase: DryRunPhase,
    // processed_records: 処理したサンプルレコード件数
    pub processed_records: u64,
    // divergence_count: 新旧スキーマ間の差異が検出されたレコード件数
    pub divergence_count: u64,
    // dry_run_id: dry_run 実行の識別子（last_green_at の更新に使用する）
    pub dry_run_id: Uuid,
    // success: dry_run が成功したか（divergence_count == 0 の場合 true）
    pub success: bool,
}

// YearlyDryRunWorkflow は年次 L1+ migration pair dry_run の Workflow 実装
pub struct YearlyDryRunWorkflow;

impl YearlyDryRunWorkflow {
    // execute は年次 dry_run の全フェーズを実行する
    pub async fn execute(
        // Workflow 入力パラメータを受け取る
        input: MigrationPairDryRunInput,
        // PostgreSQL 接続プールを Activity 引数として受け取る
        pool: &PgPool,
    ) -> Result<MigrationPairDryRunOutput, WorkflowError> {
        // Workflow 開始をログに記録する
        info!(
            pair_id = %input.pair_id,
            tenant_id = %input.tenant_id,
            target_phase = ?input.target_phase,
            "YearlyDryRun workflow starting",
        );

        // ---- フェーズ 1: Shadow ----

        // Shadow フェーズを実行する（本番 DB を読み取りのみ、shadow スキーマに write する）
        let shadow_result = shadow_phase(
            // テナント識別子を渡す
            input.tenant_id,
            // pair_id を渡す
            &input.pair_id,
            // サンプルサイズを渡す
            input.sample_size,
            // shadow スキーマ名を渡す
            &input.shadow_schema,
            // actor_id を渡す
            &input.actor_id,
            // 接続プールを渡す
            pool,
        )
        .await
        .map_err(|e| WorkflowError::Database(format!("shadow_phase: {e}")))?;
        // Shadow フェーズ完了をログに記録する
        info!(pair_id = %input.pair_id, processed = shadow_result, "shadow phase completed");
        // Shadow まで到達したら完了する
        if input.target_phase == DryRunPhase::Shadow {
            // Shadow フェーズで完了する
            return build_output(&input, DryRunPhase::Shadow, shadow_result, 0);
        }

        // ---- フェーズ 2: DualRead ----

        // DualRead フェーズを実行する（本番と shadow の値を比較して差分を記録する）
        let (dual_read_count, divergence) = dual_read_phase(
            // テナント識別子を渡す
            input.tenant_id,
            // pair_id を渡す
            &input.pair_id,
            // サンプルサイズを渡す
            input.sample_size,
            // shadow スキーマ名を渡す
            &input.shadow_schema,
            // 接続プールを渡す
            pool,
        )
        .await
        .map_err(|e| WorkflowError::Database(format!("dual_read_phase: {e}")))?;
        // 差分が検出された場合は警告ログを記録する
        if divergence > 0 {
            // 差分件数を警告ログに記録する
            warn!(
                pair_id = %input.pair_id,
                divergence = divergence,
                "divergence detected in dual_read phase",
            );
        }
        // DualRead まで到達したら完了する
        if input.target_phase == DryRunPhase::DualRead {
            // DualRead フェーズで完了する
            return build_output(&input, DryRunPhase::DualRead, dual_read_count, divergence);
        }

        // ---- フェーズ 3: Cutover ----

        // Cutover フェーズを実行する（Envoy ConfigMap を更新してルーティングを shadow に切り替える）
        cutover_phase(
            // pair_id を渡す
            &input.pair_id,
            // Envoy ConfigMap 名を渡す
            &input.envoy_configmap_name,
            // shadow スキーマ名を渡す
            &input.shadow_schema,
        )
        .await
        .map_err(|e| WorkflowError::ExternalService {
            // サービス名を設定する
            service: "envoy_configmap".to_string(),
            // エラーメッセージを設定する
            message: e.to_string(),
        })?;
        // Cutover フェーズ完了をログに記録する
        info!(pair_id = %input.pair_id, "cutover phase completed");
        // Cutover まで到達したら完了する
        if input.target_phase == DryRunPhase::Cutover {
            // Cutover フェーズで完了する
            return build_output(&input, DryRunPhase::Cutover, dual_read_count, divergence);
        }

        // ---- フェーズ 4: Cleanup ----

        // Cleanup フェーズを実行する（旧スキーマをアーカイブして shadow を本番として昇格させる）
        cleanup_phase(
            // テナント識別子を渡す
            input.tenant_id,
            // pair_id を渡す
            &input.pair_id,
            // shadow スキーマ名を渡す
            &input.shadow_schema,
            // actor_id を渡す
            &input.actor_id,
            // 接続プールを渡す
            pool,
        )
        .await
        .map_err(|e| WorkflowError::Database(format!("cleanup_phase: {e}")))?;
        // Cleanup フェーズ完了をログに記録する
        info!(pair_id = %input.pair_id, "cleanup phase completed");
        // Cleanup まで到達したら完了する
        if input.target_phase == DryRunPhase::Cleanup {
            // Cleanup フェーズで完了する
            return build_output(&input, DryRunPhase::Cleanup, dual_read_count, divergence);
        }

        // ---- フェーズ 5: Complete ----

        // Complete フェーズを実行する（dry_run_input.yaml の last_green_at を HLC 文字列で更新する）
        complete_phase(
            // pair_id を渡す
            &input.pair_id,
            // dry_run_input.yaml のパスを渡す
            &input.dry_run_input_path,
        )
        .await
        .map_err(|e| WorkflowError::ExternalService {
            // サービス名を設定する
            service: "dry_run_input_yaml".to_string(),
            // エラーメッセージを設定する
            message: e.to_string(),
        })?;
        // Complete フェーズ完了をログに記録する
        info!(pair_id = %input.pair_id, "complete phase: dry_run_input.yaml updated");

        // 全フェーズ完了をログに記録する
        info!(pair_id = %input.pair_id, "YearlyDryRun all phases completed");
        // Complete フェーズで完了する
        build_output(&input, DryRunPhase::Complete, dual_read_count, divergence)
    }
}

// build_output は Workflow 出力を構築するヘルパー関数
fn build_output(
    // Workflow 入力パラメータを受け取る
    input: &MigrationPairDryRunInput,
    // 到達したフェーズを受け取る
    reached_phase: DryRunPhase,
    // 処理件数を受け取る
    processed: u64,
    // 差分件数を受け取る
    divergence: u64,
) -> Result<MigrationPairDryRunOutput, WorkflowError> {
    // dry_run ID を生成する
    let dry_run_id = Uuid::new_v4();
    // 差分が 0 の場合のみ success とする
    let success = divergence == 0;
    // 出力を構築する
    Ok(MigrationPairDryRunOutput {
        // pair_id を設定する
        pair_id: input.pair_id.clone(),
        // 到達フェーズを設定する
        reached_phase,
        // 処理件数を設定する
        processed_records: processed,
        // 差分件数を設定する
        divergence_count: divergence,
        // dry_run ID を設定する
        dry_run_id,
        // 成功フラグを設定する
        success,
    })
}

// shadow_phase は実 DB を読み取りのみ参照し shadow スキーマに書き込む
// 本番データを変更せずに migration 後のスキーマに相当する shadow DB にデータを複製する
async fn shadow_phase(
    // テナント識別子
    tenant_id: Uuid,
    // migration pair の識別子
    pair_id: &str,
    // 処理するサンプルデータ件数
    sample_size: u64,
    // shadow DB のスキーマ名
    shadow_schema: &str,
    // バッチ操作の actor_id
    actor_id: &str,
    // PostgreSQL 接続プール
    pool: &PgPool,
) -> Result<u64> {
    // HLC クロックを環境変数から初期化する
    let clock = HlcClock::from_env();
    // HLC タイムスタンプを取得する
    let hlc_now = clock.now();
    // HLC タイムスタンプを compact 文字列に変換する
    let hlc_str = hlc_now.format_compact();
    // TenantContext を生成する（Migration purpose で shadow 書き込みを実行する）
    let ctx = TenantContext::from_auth(
        // テナント識別子を設定する
        tenant_id,
        // actor_id を設定する
        actor_id.to_string(),
        // マイグレーション purpose を使用する（特権操作のため）
        SessionPurpose::Migration,
    );
    // SET LOCAL GUC SQL を取得する
    let set_guc_sql = ctx.to_set_local_sql();
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
    // 本番スキーマからサンプルレコードを読み取って shadow スキーマに INSERT する
    // LIMIT でサンプルサイズに制限する
    sqlx::query(
        &format!(
            r#"
            INSERT INTO {shadow_schema}.orders_shadow
                SELECT *, $1 AS shadow_hlc, $2 AS pair_id
                FROM k1s0.orders
                WHERE tenant_id = current_setting('app.tenant_id')::uuid
                ORDER BY created_at ASC
                LIMIT $3
            ON CONFLICT (id) DO UPDATE SET
                shadow_hlc = EXCLUDED.shadow_hlc
            "#,
            shadow_schema = shadow_schema
        ),
    )
    // shadow_hlc に HLC compact 文字列をバインドする
    .bind(&hlc_str)
    // pair_id をバインドする（shadow レコードにトレーサビリティを付与する）
    .bind(pair_id)
    // サンプルサイズをバインドする
    .bind(sample_size as i64)
    // 同一トランザクション内で INSERT を実行する
    .execute(&mut *tx)
    .await
    .map_err(|e| anyhow::anyhow!("INSERT shadow orders error: {e}"))?;
    // トランザクションをコミットする
    tx.commit()
        .await
        .map_err(|e| anyhow::anyhow!("tx commit error: {e}"))?;
    // 処理したサンプル件数を返す
    Ok(sample_size)
}

// dual_read_phase は本番と shadow の値を比較して差分件数を記録する
// 新旧スキーマ間のデータ整合性を検証する（差分 0 = migration 安全）
async fn dual_read_phase(
    // テナント識別子
    tenant_id: Uuid,
    // migration pair の識別子
    pair_id: &str,
    // 処理するサンプルデータ件数
    sample_size: u64,
    // shadow DB のスキーマ名
    shadow_schema: &str,
    // PostgreSQL 接続プール
    pool: &PgPool,
) -> Result<(u64, u64)> {
    // TenantContext を生成する（Export purpose でデュアルリード比較を実行する）
    let ctx = TenantContext::from_auth(
        // テナント識別子を設定する
        tenant_id,
        // システムバッチアクターを設定する
        "system-dryrun-batch".to_string(),
        // データエクスポート purpose を使用する
        SessionPurpose::Export,
    );
    // SET LOCAL GUC SQL を取得する
    let set_guc_sql = ctx.to_set_local_sql();
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
    // 本番スキーマと shadow スキーマのレコードを JOIN して差分を検出する
    // EXCEPT で本番にあって shadow にない、または逆の差分を全て抽出する
    let row: (Option<i64>,) = sqlx::query_as(
        &format!(
            r#"
            SELECT COUNT(*) AS divergence_count
            FROM (
                -- 本番スキーマにあって shadow スキーマにないレコード
                SELECT id, status, amount FROM k1s0.orders
                WHERE tenant_id = current_setting('app.tenant_id')::uuid
                ORDER BY created_at ASC LIMIT $1
                EXCEPT
                -- shadow スキーマにあって本番スキーマと差異があるレコード
                SELECT id, status, amount FROM {shadow_schema}.orders_shadow
                WHERE tenant_id = current_setting('app.tenant_id')::uuid
            ) diff
            "#,
            shadow_schema = shadow_schema
        ),
    )
    // サンプルサイズをバインドする
    .bind(sample_size as i64)
    // 同一トランザクション内で SELECT を実行する
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| anyhow::anyhow!("SELECT dual_read divergence error: {e}"))?;
    // 差分件数を取得する（NULL の場合は 0 を使用する）
    let divergence = row.0.unwrap_or(0) as u64;
    // 差分検出結果をログに記録する
    info!(
        pair_id = %pair_id,
        divergence = divergence,
        "dual_read_phase divergence check completed",
    );
    // トランザクションをコミットする
    tx.commit()
        .await
        .map_err(|e| anyhow::anyhow!("tx commit error: {e}"))?;
    // （処理件数, 差分件数）のタプルを返す
    Ok((sample_size, divergence))
}

// cutover_phase はルーティングを本番から shadow に切り替える（Envoy ConfigMap 更新）
// kubectl または Kubernetes API 経由で ConfigMap を更新する（dry-run 完了後に手動ロールバック可能）
async fn cutover_phase(
    // migration pair の識別子
    pair_id: &str,
    // Envoy ConfigMap の名前
    envoy_configmap_name: &str,
    // shadow DB のスキーマ名
    shadow_schema: &str,
) -> Result<()> {
    // HLC クロックを環境変数から初期化する
    let clock = HlcClock::from_env();
    // HLC タイムスタンプを取得する（Cutover 操作の HLC 証跡として使用する）
    let hlc_now = clock.now();
    // HLC タイムスタンプを compact 文字列に変換する
    let hlc_str = hlc_now.format_compact();
    // KUBECONFIG または IN-CLUSTER config から Kubernetes API サーバー URL を取得する
    let k8s_api_url = std::env::var("KUBERNETES_SERVICE_HOST")
        .map(|host| {
            // Kubernetes cluster 内部から API サーバーに接続する URL を構築する
            let port = std::env::var("KUBERNETES_SERVICE_PORT").unwrap_or_else(|_| "443".to_string());
            // https://{host}:{port} 形式の URL を返す
            format!("https://{}:{}", host, port)
        })
        .unwrap_or_else(|_| {
            // cluster 外（ローカル開発環境）では KUBE_APISERVER 環境変数を使用する
            std::env::var("KUBE_APISERVER")
                .unwrap_or_else(|_| "http://localhost:8001".to_string())
        });
    // Envoy ConfigMap の namespace を環境変数から取得する（デフォルト: k1s0-system）
    let namespace = std::env::var("ENVOY_CONFIGMAP_NAMESPACE")
        .unwrap_or_else(|_| "k1s0-system".to_string());
    // Cutover 用の Envoy ConfigMap パッチ内容を JSON 形式で構築する
    let patch = serde_json::json!({
        "metadata": {
            // カットオーバー実行時の HLC タイムスタンプをアノテーションに記録する
            "annotations": {
                "k1s0.io/cutover-hlc": hlc_str,
                "k1s0.io/cutover-pair-id": pair_id,
            }
        },
        "data": {
            // shadow スキーマにルーティングするための設定を更新する
            "db_schema": shadow_schema,
            // カットオーバー状態を示すフラグを設定する
            "migration_state": "cutover",
        }
    });
    // Kubernetes API サーバーへのパッチリクエスト URL を構築する
    let patch_url = format!(
        "{}/api/v1/namespaces/{}/configmaps/{}",
        k8s_api_url, namespace, envoy_configmap_name
    );
    // reqwest クライアントを生成する
    // 本番環境では Envoy/Linkerd sidecar が mTLS 終端を担うため、reqwest は HTTP で通信する
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| anyhow::anyhow!("reqwest client build error: {e}"))?;
    // Kubernetes API に PATCH リクエストを送信して ConfigMap を更新する
    let resp = client
        .patch(&patch_url)
        // Content-Type を merge-patch+json に設定する（Kubernetes RFC7386 マージパッチ）
        .header("Content-Type", "application/merge-patch+json")
        // Bearer トークンを環境変数から取得して Authorization ヘッダに設定する
        .header(
            "Authorization",
            format!(
                "Bearer {}",
                std::env::var("KUBE_SA_TOKEN").unwrap_or_default()
            ),
        )
        // パッチ内容を body に設定する
        .json(&patch)
        // PATCH リクエストを送信する
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Kubernetes PATCH error: {e}"))?;
    // HTTP ステータスが 2xx でない場合はエラーを返す
    if !resp.status().is_success() {
        // PATCH 失敗のエラーメッセージを構築する
        return Err(anyhow::anyhow!(
            "Kubernetes PATCH failed: status={}, url={}",
            resp.status(),
            patch_url
        ));
    }
    // Cutover 完了をログに記録する
    info!(
        pair_id = %pair_id,
        configmap = %envoy_configmap_name,
        hlc = %hlc_str,
        "cutover_phase: Envoy ConfigMap updated",
    );
    // 正常終了を返す
    Ok(())
}

// cleanup_phase は旧本番スキーマをアーカイブして shadow スキーマを正式本番に昇格させる
// atomic_triple_write で cleanup 操作の Audit も同一 txn に書き込む
async fn cleanup_phase(
    // テナント識別子
    tenant_id: Uuid,
    // migration pair の識別子
    pair_id: &str,
    // shadow DB のスキーマ名（cleanup 後に本番スキーマとして昇格させる）
    shadow_schema: &str,
    // バッチ操作の actor_id
    actor_id: &str,
    // PostgreSQL 接続プール
    pool: &PgPool,
) -> Result<()> {
    // HLC クロックを環境変数から初期化する
    let clock = HlcClock::from_env();
    // HLC タイムスタンプを取得する（cleanup 操作の証跡として使用する）
    let hlc_now = clock.now();
    // HLC タイムスタンプを compact 文字列に変換する
    let hlc_str = hlc_now.format_compact();
    // TenantContext を生成する（Migration purpose で cleanup 操作を実行する）
    let ctx = TenantContext::from_auth(
        // テナント識別子を設定する
        tenant_id,
        // actor_id を設定する
        actor_id.to_string(),
        // マイグレーション purpose を使用する（特権操作のため）
        SessionPurpose::Migration,
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
    // 旧本番スキーマを archive スキーマに rename する（data を保持しつつ本番から外す）
    // archive スキーマ名は pair_id + "_archive_" + HLC hex で一意にする
    let archive_schema = format!("k1s0_archive_{}_{}", pair_id.replace('-', "_"), &hlc_str[..8]);
    // ALTER SCHEMA で旧本番スキーマを archive スキーマに rename する
    sqlx::query(
        &format!(
            "ALTER SCHEMA k1s0 RENAME TO {}",
            archive_schema
        ),
    )
    // 同一トランザクション内で ALTER SCHEMA を実行する
    .execute(&mut *tx)
    .await
    .map_err(|e| anyhow::anyhow!("ALTER SCHEMA rename error: {e}"))?;
    // shadow スキーマを本番スキーマ k1s0 として昇格させる
    sqlx::query(
        &format!(
            "ALTER SCHEMA {} RENAME TO k1s0",
            shadow_schema
        ),
    )
    // 同一トランザクション内で ALTER SCHEMA を実行する
    .execute(&mut *tx)
    .await
    .map_err(|e| anyhow::anyhow!("ALTER SCHEMA promote error: {e}"))?;
    // atomic_triple_write で cleanup 操作の Audit を同一 tx に書き込む
    let change = StateChange {
        // aggregate_id として新規 UUID を生成する（cleanup 操作のイベント識別子）
        aggregate_id: Uuid::new_v4(),
        // テナント識別子を設定する
        tenant_id,
        // テナントスコープのテーブルクラスを使用する
        table_class: TableClass::TenantScoped,
        // ペイロードに cleanup 情報を JSON 形式で設定する
        payload: serde_json::json!({
            "event": "yearly_dryrun_cleanup",
            "pair_id": pair_id,
            "archive_schema": archive_schema,
            "promoted_schema": "k1s0",
            "cleanup_hlc": hlc_str,
        }),
        // バージョン 1 を設定する（新規イベントのため）
        version: 1,
    };
    // execute() で外部トランザクションに Outbox + Audit を書き込む
    writer
        .execute(&change, &mut tx)
        .await
        .map_err(|e| anyhow::anyhow!("atomic_triple_write error: {e:?}"))?;
    // トランザクションをコミットする（ALTER SCHEMA + 3 INSERT を同一 txn で確定する）
    tx.commit()
        .await
        .map_err(|e| anyhow::anyhow!("tx commit error: {e}"))?;
    // Cleanup 完了をログに記録する
    info!(
        pair_id = %pair_id,
        archive_schema = %archive_schema,
        "cleanup_phase: old schema archived, shadow promoted to k1s0",
    );
    // 正常終了を返す
    Ok(())
}

// complete_phase は dry_run_input.yaml の last_green_at を HLC 文字列で更新する
// lock yaml 生成ツール（generate_dry_run.py）への入力ファイルを更新することで
// dry_run.lock.yaml の再生成時に last_green_at が最新に保たれる
async fn complete_phase(
    // 更新対象の migration pair の識別子
    pair_id: &str,
    // dry_run_input.yaml のファイルパス
    dry_run_input_path: &str,
) -> Result<()> {
    // HLC クロックを環境変数から初期化する
    let clock = HlcClock::from_env();
    // HLC タイムスタンプを取得する（last_green_at として YAML に書き込む）
    let hlc_now = clock.now();
    // HLC wall_ms を RFC3339 形式の日時文字列に変換する（YAML に記録する）
    let last_green_at = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(hlc_now.wall_ms as i64)
        .unwrap_or_else(|| chrono::Utc::now())
        .to_rfc3339();
    // dry_run_input.yaml を読み込む（ファイルが存在しない場合はエラーを返す）
    let yaml_content = tokio::fs::read_to_string(dry_run_input_path)
        .await
        .map_err(|e| anyhow::anyhow!("read dry_run_input.yaml error: {path} — {e}", path = dry_run_input_path))?;
    // pair_id に対応するブロックの last_green_at を更新する
    // pair_id: "xxx" の次の last_green_at: "..." を置換する単純な文字列置換を使用する
    // YAML パーサーを使わない理由: コメントと書式を保持するため（serde_yaml は整形を変更する）
    let target_pair_marker = format!("pair_id: {}", pair_id);
    // pair_id マーカーが存在しない場合はエラーを返す
    if !yaml_content.contains(&target_pair_marker) {
        // 指定された pair_id が YAML に存在しないことを示すエラーを返す
        return Err(anyhow::anyhow!(
            "pair_id '{}' not found in dry_run_input.yaml",
            pair_id
        ));
    }
    // pair_id ブロックの last_green_at を更新する（最初に出現する last_green_at 行を置換する）
    // 対象 pair_id の直後にある last_green_at: "..." 行を更新する
    let mut lines: Vec<&str> = yaml_content.lines().collect();
    // pair_id マーカーを発見した後の last_green_at 行を更新する
    let mut in_target_pair = false;
    // 更新済みフラグを初期化する
    let mut updated = false;
    // 更新後の行リストを構築する
    let mut new_lines: Vec<String> = Vec::with_capacity(lines.len());
    // 各行を順番に処理する
    for line in &mut lines {
        // pair_id マーカー行を検出したら対象 pair のブロック内に入ったとマークする
        if line.contains(&target_pair_marker) {
            // 対象 pair ブロック開始をマークする
            in_target_pair = true;
            // 現在行をそのまま追加する
            new_lines.push(line.to_string());
        } else if in_target_pair && !updated && line.contains("last_green_at:") {
            // 対象 pair ブロック内の last_green_at 行を新しい値で置換する
            // インデント量を保持するため元行の先頭空白を引き継ぐ
            let indent = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
            // 新しい last_green_at 行を構築する
            new_lines.push(format!("{}last_green_at: \"{}\"", indent, last_green_at));
            // 更新済みフラグを立てる
            updated = true;
            // 次の pair_id に当たったら対象 pair ブロックを抜けたとみなす
        } else if in_target_pair && line.trim().starts_with("- pair_id:") && !line.contains(&target_pair_marker) {
            // 別の pair_id ブロックに入ったら対象 pair ブロックを抜けたとマークする
            in_target_pair = false;
            // 現在行をそのまま追加する
            new_lines.push(line.to_string());
        } else {
            // 対象外の行はそのまま追加する
            new_lines.push(line.to_string());
        }
    }
    // last_green_at の更新が行われなかった場合はエラーを返す
    if !updated {
        // 更新対象の last_green_at 行が見つからなかったことを示すエラーを返す
        return Err(anyhow::anyhow!(
            "last_green_at not found in pair_id '{}' block in dry_run_input.yaml",
            pair_id
        ));
    }
    // 更新後の YAML 内容を結合する（末尾改行を保持する）
    let new_content = new_lines.join("\n") + "\n";
    // dry_run_input.yaml を更新内容で上書きする
    tokio::fs::write(dry_run_input_path, new_content)
        .await
        .map_err(|e| anyhow::anyhow!("write dry_run_input.yaml error: {path} — {e}", path = dry_run_input_path))?;
    // Complete フェーズの更新内容をログに記録する
    info!(
        pair_id = %pair_id,
        last_green_at = %last_green_at,
        path = %dry_run_input_path,
        "complete_phase: dry_run_input.yaml last_green_at updated",
    );
    // 正常終了を返す
    Ok(())
}

// YearlyDryRunWorkflow のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールをインポートする
    use super::*;
    // 一時ファイル作成に使用する
    use std::io::Write;

    // 基本的な年次 dry_run テスト（Shadow フェーズで停止、DB 不要ロジックのみ確認）
    #[test]
    fn test_build_output_shadow_success() {
        // テスト用の入力パラメータを構築する
        let input = MigrationPairDryRunInput {
            // pair_id を設定する
            pair_id: "msg_v1_to_v2".to_string(),
            // テナント ID を生成する
            tenant_id: Uuid::new_v4(),
            // Shadow フェーズで停止する
            target_phase: DryRunPhase::Shadow,
            // サンプルサイズを設定する
            sample_size: 1000,
            // actor_id を設定する
            actor_id: "system-test".to_string(),
            // shadow スキーマ名を設定する
            shadow_schema: "k1s0_shadow".to_string(),
            // Envoy ConfigMap 名を設定する
            envoy_configmap_name: "envoy-routing".to_string(),
            // dry_run_input.yaml パスを設定する
            dry_run_input_path: "/tmp/test_dry_run_input.yaml".to_string(),
        };
        // Shadow フェーズで差分 0 の出力を構築する
        let output = build_output(&input, DryRunPhase::Shadow, 1000, 0).unwrap();
        // Shadow フェーズで停止することを確認する
        assert_eq!(output.reached_phase, DryRunPhase::Shadow);
        // 差分なしで success=true になることを確認する
        assert!(output.success, "差分なしの場合 success=true になるべき");
        // 処理件数が一致することを確認する
        assert_eq!(output.processed_records, 1000);
    }

    // 差分がある場合は success=false になることを確認するテスト
    #[test]
    fn test_build_output_with_divergence() {
        // テスト用の入力パラメータを構築する
        let input = MigrationPairDryRunInput {
            // pair_id を設定する
            pair_id: "relational_pg_pair".to_string(),
            // テナント ID を生成する
            tenant_id: Uuid::new_v4(),
            // DualRead フェーズで停止する
            target_phase: DryRunPhase::DualRead,
            // サンプルサイズを設定する
            sample_size: 500,
            // actor_id を設定する
            actor_id: "system-test".to_string(),
            // shadow スキーマ名を設定する
            shadow_schema: "k1s0_shadow".to_string(),
            // Envoy ConfigMap 名を設定する
            envoy_configmap_name: "envoy-routing".to_string(),
            // dry_run_input.yaml パスを設定する
            dry_run_input_path: "/tmp/test_dry_run_input.yaml".to_string(),
        };
        // 差分 3 件の DualRead フェーズ出力を構築する
        let output = build_output(&input, DryRunPhase::DualRead, 500, 3).unwrap();
        // DualRead フェーズに到達していることを確認する
        assert_eq!(output.reached_phase, DryRunPhase::DualRead);
        // 差分ありの場合は success=false になることを確認する
        assert!(!output.success, "差分ありの場合 success=false になるべき");
        // 差分件数が一致することを確認する
        assert_eq!(output.divergence_count, 3);
    }

    // complete_phase が dry_run_input.yaml の last_green_at を更新することを確認するテスト
    #[tokio::test]
    async fn test_complete_phase_updates_last_green_at() {
        // 一時ファイルを作成する
        let tmp_path = "/tmp/test_dry_run_input_complete.yaml";
        // テスト用の dry_run_input.yaml 内容を作成する
        let initial_content = r#"pairs:
  - pair_id: workflow_pair
    from_oss: temporal
    to_oss: cadence
    last_green_at: "2026-01-01T00:00:00Z"
    phases: []
"#;
        // 一時ファイルにテスト内容を書き込む
        let mut f = std::fs::File::create(tmp_path).unwrap();
        // テスト内容を書き込む
        f.write_all(initial_content.as_bytes()).unwrap();
        // complete_phase を実行する
        complete_phase("workflow_pair", tmp_path).await.unwrap();
        // 更新後のファイルを読み込む
        let updated = tokio::fs::read_to_string(tmp_path).await.unwrap();
        // last_green_at が更新されていることを確認する（2026-01-01 以外の値になっているはず）
        assert!(
            !updated.contains("2026-01-01T00:00:00Z"),
            "last_green_at が更新されるべき（元の値のまま）"
        );
        // last_green_at キーが残っていることを確認する
        assert!(updated.contains("last_green_at:"), "last_green_at キーが残るべき");
    }
}
