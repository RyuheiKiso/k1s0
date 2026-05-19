// workflow.rs — k1s0 tier1 Library: ワークフローエンジン L1+ facade trait
// Temporal 等の OSS 型を公開 API に露出しない（L1+ ラップ規約）。
// ワークフロー起動と結果取得の 2 操作を提供する。
// wall-clock TTL 禁止規約: タイムアウトは HLC ミリ秒で指定する。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;

// WorkflowClient は Temporal を L1+ ラップするワークフロー操作 facade trait。
// 公開 API シグネチャに OSS 型（temporal_sdk::Client 等）を一切含まない。
// 入出力は JSON bytes で表現する（OSS 型依存を排除する）。
#[async_trait]
pub trait WorkflowClient: Send + Sync {
    // start_workflow はワークフローを起動して run_id を返す。
    // workflow_id は実行のべき等性を保証する識別子（UUID v4 推奨）。
    // workflow_type はワークフロー型名（例: "SessionCreationWorkflow"）。
    // input はワークフロー入力の JSON bytes（例: b"{"tenant_id":"t-001"}"）。
    // 戻り値は run_id（Temporal の WorkflowExecution.RunId に対応する）。
    async fn start_workflow(
        &self,
        workflow_id: &str,
        workflow_type: &str,
        input: Vec<u8>,
    ) -> crate::Result<String>;

    // get_result はワークフロー結果を取得する（blocking; タイムアウト: HLC ミリ秒）。
    // run_id は start_workflow の戻り値として得た識別子。
    // timeout_hlc_ms は HLC ミリ秒単位のタイムアウト（wall-clock 禁止規約に準拠する）。
    // 戻り値はワークフロー結果の JSON bytes。
    async fn get_result(&self, run_id: &str, timeout_hlc_ms: u64) -> crate::Result<Vec<u8>>;
}
