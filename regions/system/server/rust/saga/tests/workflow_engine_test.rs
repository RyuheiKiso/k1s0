//! ワークフローエンジン統合テスト（モックgRPC caller使用）
//!
//! ExecuteSagaUseCase の前方実行・補償ロジックのテストは
//! src/usecase/execute_saga.rs 内のユニットテストで MockSagaRepository /
//! MockGrpcStepCaller を使って網羅的に実施済み。
//!
//! このファイルは外部テストから InMemorySagaRepository + NoOpGrpcCaller を使った
//! エンドツーエンドのワークフロー実行テストの追加先として残している。
//! REST API 経由の統合テストは tests/integration_test.rs を参照。

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "covered by src/usecase/execute_saga.rs unit tests and tests/integration_test.rs"]
    async fn test_full_workflow_success() {
        // ExecuteSagaUseCase::run で全ステップ成功 → Completed
        // see: src/usecase/execute_saga.rs::tests::test_successful_execution
    }

    #[tokio::test]
    #[ignore = "covered by src/usecase/execute_saga.rs unit tests"]
    async fn test_workflow_with_payment_failure() {
        // ステップ失敗 → 補償処理実行 → Failed
        // see: src/usecase/execute_saga.rs::tests::test_step_failure_triggers_compensation
    }
}
