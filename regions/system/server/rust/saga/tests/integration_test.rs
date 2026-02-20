//! REST API統合テスト（インメモリリポジトリ使用）
//!
//! Note: MockSagaRepository/MockGrpcStepCaller は #[cfg(test)] で生成されるため、
//! 外部テストファイルからは利用できない。完全な統合テストは InMemorySagaRepository を
//! lib クレートに移動した後に実装する。

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "requires InMemorySagaRepository in lib crate"]
    async fn test_healthz() {
        // let state = create_test_state().await;
        // let server = TestServer::new(router(state)).unwrap();
        // let response = server.get("/healthz").await;
        // assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore = "requires InMemorySagaRepository in lib crate"]
    async fn test_register_and_list_workflows() {
        // let state = create_test_state().await;
        // let server = TestServer::new(router(state)).unwrap();
        // Register + List workflows
    }
}
