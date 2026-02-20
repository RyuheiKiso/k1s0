//! ワークフローエンジン統合テスト（モックgRPC caller使用）
//!
//! Note: MockSagaRepository/MockGrpcStepCaller は #[cfg(test)] で生成されるため、
//! 外部テストファイルからは利用できない。同等のテストは
//! src/usecase/execute_saga.rs 内のユニットテストで実施している。

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "mock types not available from external tests; see execute_saga.rs unit tests"]
    async fn test_full_workflow_success() {
        // See src/usecase/execute_saga.rs::tests::test_successful_execution
    }

    #[tokio::test]
    #[ignore = "mock types not available from external tests; see execute_saga.rs unit tests"]
    async fn test_workflow_with_payment_failure() {
        // See src/usecase/execute_saga.rs::tests::test_step_failure_triggers_compensation
    }
}
