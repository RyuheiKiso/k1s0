// saga_long_running.rs — Saga 長時間 Workflow 実装（設計方針 13 / 18 §Saga）
// 業務トランザクションの Saga compensation pattern を実装する
// 各ステップが失敗した場合は補償トランザクションを逆順で実行する

// anyhow: Result 型に使用する
use anyhow::Result;
// serde: Workflow 入出力の JSON シリアライズに使用する
use serde::{Deserialize, Serialize};
// uuid: Workflow ID 生成に使用する
use uuid::Uuid;
// tracing: 構造化ロギングに使用する
use tracing::{info, warn, error};
// WorkflowError をインポートする
use crate::WorkflowError;

// SagaStep は Saga の 1 ステップを表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep {
    // step_id: ステップの一意識別子
    pub step_id: String,
    // step_name: ステップ名（ログ・監査用）
    pub step_name: String,
    // compensation_step_id: このステップの補償ステップ ID
    pub compensation_step_id: Option<String>,
}

// SagaWorkflowInput は Saga Workflow の入力パラメータを宣言する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaWorkflowInput {
    // workflow_id: Workflow の一意識別子（Temporal Workflow ID と一致させる）
    pub workflow_id: String,
    // tenant_id: 対象テナント識別子
    pub tenant_id: Uuid,
    // steps: 実行する Saga ステップのリスト（順番に実行する）
    pub steps: Vec<SagaStep>,
    // payload: ステップ間で共有されるペイロード（JSON）
    pub payload: serde_json::Value,
    // max_compensation_retry: 補償トランザクションの最大リトライ回数
    pub max_compensation_retry: u32,
}

// SagaWorkflowOutput は Saga Workflow の出力を宣言する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaWorkflowOutput {
    // completed_steps: 正常完了したステップ ID のリスト
    pub completed_steps: Vec<String>,
    // compensated_steps: 補償実行したステップ ID のリスト（失敗時のみ）
    pub compensated_steps: Vec<String>,
    // success: 全ステップが正常完了した場合 true
    pub success: bool,
    // final_payload: 最終的なペイロード（全ステップ処理後の状態）
    pub final_payload: serde_json::Value,
}

// SagaContext は Saga 実行中の状態を保持する構造体
#[derive(Debug)]
pub struct SagaContext {
    // completed: 完了したステップのスタック（補償時に逆順で使用する）
    completed: Vec<SagaStep>,
    // payload: 現在のペイロード状態
    payload: serde_json::Value,
    // tenant_id: テナント識別子
    tenant_id: Uuid,
}

impl SagaContext {
    // new は SagaContext を初期化する
    pub fn new(tenant_id: Uuid, initial_payload: serde_json::Value) -> Self {
        // 空の completed スタックで初期化する
        Self {
            // completed スタックを空で初期化する
            completed: Vec::new(),
            // 初期ペイロードを設定する
            payload: initial_payload,
            // テナント ID を設定する
            tenant_id,
        }
    }

    // execute_step は 1 ステップを実行してスタックに積む
    // step_fn は実際のビジネスロジック（DB 書込等）を実行するコールバック
    pub async fn execute_step<F, Fut>(
        &mut self,
        step: SagaStep,
        step_fn: F,
    ) -> Result<(), WorkflowError>
    where
        // step_fn: ペイロードを受け取って更新したペイロードを返す非同期関数
        F: Fn(Uuid, serde_json::Value) -> Fut,
        // Fut: 非同期の Result を返す Future
        Fut: std::future::Future<Output = Result<serde_json::Value, String>>,
    {
        // ステップ開始をログに記録する
        info!(
            step_id = %step.step_id,
            step_name = %step.step_name,
            tenant_id = %self.tenant_id,
            "Saga step starting",
        );
        // ビジネスロジックを実行する
        match step_fn(self.tenant_id, self.payload.clone()).await {
            // ステップ成功: ペイロードを更新してスタックに積む
            Ok(new_payload) => {
                // ペイロードを更新する
                self.payload = new_payload;
                // 完了スタックにステップを積む
                self.completed.push(step.clone());
                // ステップ完了をログに記録する
                info!(step_id = %step.step_id, "Saga step completed");
                // 成功を返す
                Ok(())
            }
            // ステップ失敗: 補償を実行する
            Err(cause) => {
                // ステップ失敗をログに記録する
                error!(
                    step_id = %step.step_id,
                    cause = %cause,
                    "Saga step failed — starting compensation",
                );
                // Database エラーとして返す
                Err(WorkflowError::Database(format!(
                    "step {} failed: {}",
                    step.step_id, cause
                )))
            }
        }
    }

    // compensate は完了済みステップを逆順で補償する
    // compensation_fn は各ステップの補償ロジックを実行するコールバック
    pub async fn compensate<F, Fut>(
        &mut self,
        compensation_fn: F,
    ) -> Result<Vec<String>, WorkflowError>
    where
        // compensation_fn: ステップと現在のペイロードを受け取る補償関数
        F: Fn(SagaStep, Uuid, serde_json::Value) -> Fut,
        // Fut: 補償結果を返す Future
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        // 補償済みステップのリストを初期化する
        let mut compensated = Vec::new();
        // 完了済みスタックを逆順で処理する（LIFO 順で補償する）
        while let Some(step) = self.completed.pop() {
            // 補償が必要なステップのみ実行する（compensation_step_id が設定されている場合）
            if step.compensation_step_id.is_none() {
                // 補償ステップがない場合はスキップする
                warn!(step_id = %step.step_id, "no compensation step — skipping");
                // 次のステップへ進む
                continue;
            }
            // 補償ロジックを実行する
            match compensation_fn(step.clone(), self.tenant_id, self.payload.clone()).await {
                // 補償成功
                Ok(()) => {
                    // 補償完了をログに記録する
                    info!(step_id = %step.step_id, "Saga compensation completed");
                    // 補償済みリストに追加する
                    compensated.push(step.step_id);
                }
                // 補償失敗
                Err(cause) => {
                    // 補償失敗をエラーログに記録する
                    error!(
                        step_id = %step.step_id,
                        cause = %cause,
                        "Saga compensation FAILED — manual intervention required",
                    );
                    // CompensationFailed エラーを返す
                    return Err(WorkflowError::CompensationFailed {
                        // 失敗ステップ ID を設定する
                        step: step.step_id,
                        // 原因メッセージを設定する
                        cause,
                    });
                }
            }
        }
        // 補償済みステップのリストを返す
        Ok(compensated)
    }

    // completed_steps は完了済みステップ ID のリストを返す
    pub fn completed_step_ids(&self) -> Vec<String> {
        // completed スタックからステップ ID を収集する
        self.completed.iter().map(|s| s.step_id.clone()).collect()
    }

    // current_payload は現在のペイロードを返す
    pub fn current_payload(&self) -> &serde_json::Value {
        // 現在のペイロードへの参照を返す
        &self.payload
    }
}

// SagaLongRunningWorkflow の単体テスト
#[cfg(test)]
mod tests {
    // 親モジュールをインポートする
    use super::*;
    // serde_json::json! マクロをインポートする
    use serde_json::json;

    // Saga の基本的な実行フローテスト（全ステップ成功）
    #[tokio::test]
    async fn test_saga_all_steps_succeed() {
        // テスト用テナント ID を生成する
        let tenant_id = Uuid::new_v4();
        // テスト用初期ペイロードを生成する
        let initial = json!({"order_id": "test-001"});
        // SagaContext を初期化する
        let mut ctx = SagaContext::new(tenant_id, initial);
        // ステップ 1 を実行する（成功）
        let step1 = SagaStep {
            // ステップ ID を設定する
            step_id: "step_1".to_string(),
            // ステップ名を設定する
            step_name: "deduct_inventory".to_string(),
            // 補償ステップ ID を設定する
            compensation_step_id: Some("compensate_step_1".to_string()),
        };
        // ステップ 1 を非同期実行する
        ctx.execute_step(step1, |_tid, payload| async move {
            // ペイロードを更新して返す（成功）
            let mut p = payload;
            // step_1_done フラグを設定する
            p["step_1_done"] = json!(true);
            // 更新済みペイロードを返す
            Ok(p)
        }).await.unwrap();
        // 完了ステップが 1 件であることを確認する
        assert_eq!(ctx.completed_step_ids().len(), 1);
        // ペイロードに step_1_done が設定されていることを確認する
        assert_eq!(ctx.current_payload()["step_1_done"], json!(true));
    }

    // Saga の補償フローテスト（ステップ失敗後に補償実行）
    #[tokio::test]
    async fn test_saga_compensation_on_failure() {
        // テスト用テナント ID を生成する
        let tenant_id = Uuid::new_v4();
        // SagaContext を初期化する
        let mut ctx = SagaContext::new(tenant_id, json!({}));
        // ステップ 1 を成功させる
        let step1 = SagaStep {
            // ステップ ID を設定する
            step_id: "s1".to_string(),
            // ステップ名を設定する
            step_name: "step_one".to_string(),
            // 補償ステップ ID を設定する
            compensation_step_id: Some("compensate_s1".to_string()),
        };
        // ステップ 1 を実行する（成功）
        ctx.execute_step(step1, |_, p| async move { Ok(p) }).await.unwrap();
        // ステップ 2 を失敗させる
        let step2 = SagaStep {
            // ステップ ID を設定する
            step_id: "s2".to_string(),
            // ステップ名を設定する
            step_name: "step_two".to_string(),
            // 補償ステップ ID を設定する
            compensation_step_id: Some("compensate_s2".to_string()),
        };
        // ステップ 2 は失敗する
        let result = ctx.execute_step(step2, |_, _| async move {
            // 失敗を返す
            Err("database error".to_string())
        }).await;
        // ステップ 2 が失敗することを確認する
        assert!(result.is_err(), "ステップ 2 は失敗するべき");
        // 補償を実行する
        let compensated = ctx.compensate(|step, _, _| async move {
            // ステップ s1 の補償は成功する
            let _ = step;
            // 成功を返す
            Ok(())
        }).await.unwrap();
        // ステップ s1 が補償されることを確認する
        assert!(compensated.contains(&"s1".to_string()), "s1 が補償されるべき");
    }
}
