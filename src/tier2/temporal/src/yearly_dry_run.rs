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
    // Complete フェーズ: 旧スキーマを完全削除する
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
        input: MigrationPairDryRunInput,
    ) -> Result<MigrationPairDryRunOutput, WorkflowError> {
        // Workflow 開始をログに記録する
        info!(
            pair_id = %input.pair_id,
            tenant_id = %input.tenant_id,
            target_phase = ?input.target_phase,
            "YearlyDryRun workflow starting",
        );

        // ---- フェーズ 1: Shadow ----

        // Shadow フェーズを実行する（新旧両書き込み、読み取りは旧のみ）
        let shadow_result = run_shadow_phase(input.tenant_id, &input.pair_id, input.sample_size).await
            .map_err(|e| WorkflowError::Database(format!("shadow_phase: {e}")))?;
        // Shadow フェーズ完了をログに記録する
        info!(pair_id = %input.pair_id, "shadow phase completed");
        // Shadow まで到達したら完了する
        if input.target_phase == DryRunPhase::Shadow {
            // Shadow フェーズで完了する
            return build_output(&input, DryRunPhase::Shadow, shadow_result, 0);
        }

        // ---- フェーズ 2: DualRead ----

        // DualRead フェーズを実行する（新旧両方から読み取り差分チェック）
        let (dual_read_count, divergence) = run_dual_read_phase(input.tenant_id, &input.pair_id, input.sample_size).await
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

        // ---- フェーズ 3 〜 5: Cutover / Cleanup / Complete は将来実装 ----

        // Cutover 以降は destructive な操作を含むため、dry_run では DualRead で停止する
        // 実際の Cutover は別の Temporal Workflow（cutover_workflow.rs）で実行する
        info!(pair_id = %input.pair_id, "YearlyDryRun stopping at DualRead phase");
        // DualRead フェーズで完了する
        build_output(&input, DryRunPhase::DualRead, dual_read_count, divergence)
    }
}

// build_output は Workflow 出力を構築するヘルパー関数
fn build_output(
    input: &MigrationPairDryRunInput,
    reached_phase: DryRunPhase,
    processed: u64,
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

// run_shadow_phase は Shadow フェーズを実行する（stub 実装）
async fn run_shadow_phase(tenant_id: Uuid, pair_id: &str, sample_size: u64) -> Result<u64> {
    // stub として sample_size を返す
    let _ = (tenant_id, pair_id);
    // 処理件数を返す
    Ok(sample_size)
}

// run_dual_read_phase は DualRead フェーズを実行して（処理件数, 差分件数）を返す（stub 実装）
async fn run_dual_read_phase(
    tenant_id: Uuid,
    pair_id: &str,
    sample_size: u64,
) -> Result<(u64, u64)> {
    // stub として（sample_size, 0 差分）を返す
    let _ = (tenant_id, pair_id);
    // 差分なしで返す
    Ok((sample_size, 0))
}

// YearlyDryRunWorkflow のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールをインポートする
    use super::*;

    // 基本的な年次 dry_run テスト（Shadow フェーズで停止）
    #[tokio::test]
    async fn test_yearly_dry_run_shadow() {
        // テスト用入力を生成する
        let input = MigrationPairDryRunInput {
            // pair_id を設定する
            pair_id: "msg_v1_to_v2".to_string(),
            // テナント ID を生成する
            tenant_id: Uuid::new_v4(),
            // Shadow フェーズで停止する
            target_phase: DryRunPhase::Shadow,
            // サンプルサイズを設定する
            sample_size: 1000,
        };
        // Workflow を実行する
        let output = YearlyDryRunWorkflow::execute(input).await.unwrap();
        // Shadow フェーズで停止することを確認する
        assert_eq!(output.reached_phase, DryRunPhase::Shadow);
        // 差分なしで成功することを確認する
        assert!(output.success, "差分なしの場合 success=true になるべき");
    }
}
