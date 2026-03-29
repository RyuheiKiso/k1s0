//! ワークフローエンジン統合テスト（InMemoryリポジトリ + NoOpGrpcCaller使用）
//! test-utils フィーチャーが有効な場合のみコンパイルする（L-07 監査対応）
#![cfg(feature = "test-utils")]
//!
//! ExecuteSagaUseCase の前方実行・補償ロジックのモックベースのユニットテストは
//! src/usecase/execute_saga.rs 内で MockSagaRepository / MockGrpcStepCaller を使って
//! 網羅的に実施済み。
//!
//! このファイルでは InMemorySagaRepository + NoOpGrpcCaller を使った
//! エンドツーエンドのワークフロー実行テストを実装し、実際のリポジトリ操作を
//! 含む結合レベルの動作を検証する。
//! REST API 経由の統合テストは tests/integration_test.rs を参照。
#![allow(clippy::unwrap_used)]

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use k1s0_saga_server::domain::entity::saga_state::{SagaState, SagaStatus};
    use k1s0_saga_server::domain::entity::saga_step_log::{StepAction, StepStatus};
    use k1s0_saga_server::domain::entity::workflow::WorkflowDefinition;
    use k1s0_saga_server::domain::repository::SagaRepository;
    use k1s0_saga_server::infrastructure::grpc_caller::GrpcStepCaller;
    use k1s0_saga_server::infrastructure::kafka_producer::SagaEventPublisher;
    use k1s0_saga_server::test_support::{InMemorySagaRepository, NoOpGrpcCaller, NoOpPublisher};
    use k1s0_saga_server::usecase::ExecuteSagaUseCase;

    /// テスト用のワークフロー定義（2ステップ、補償あり）を作成するヘルパー
    fn make_two_step_workflow() -> WorkflowDefinition {
        WorkflowDefinition::from_yaml(
            r#"
name: task-assignment
steps:
  - name: create-task
    service: task-server
    method: TaskService.CreateTask
    compensate: TaskService.CancelTask
    timeout_secs: 5
    retry:
      max_attempts: 1
      initial_interval_ms: 100
  - name: increment-board-column
    service: board-server
    method: BoardService.IncrementColumn
    compensate: BoardService.DecrementColumn
    timeout_secs: 5
    retry:
      max_attempts: 1
      initial_interval_ms: 100
"#,
        )
        .unwrap()
    }

    /// テスト用のワークフロー定義（3ステップ、一部補償なし）を作成するヘルパー
    fn make_three_step_workflow() -> WorkflowDefinition {
        WorkflowDefinition::from_yaml(
            r#"
name: three-step-workflow
steps:
  - name: step-1
    service: svc-a
    method: SvcA.Do
    compensate: SvcA.Undo
    timeout_secs: 5
    retry:
      max_attempts: 1
      initial_interval_ms: 100
  - name: step-2
    service: svc-b
    method: SvcB.Do
    timeout_secs: 5
    retry:
      max_attempts: 1
      initial_interval_ms: 100
  - name: step-3
    service: svc-c
    method: SvcC.Do
    compensate: SvcC.Undo
    timeout_secs: 5
    retry:
      max_attempts: 1
      initial_interval_ms: 100
"#,
        )
        .unwrap()
    }

    /// テスト用のSagaStateを作成しリポジトリに永続化するヘルパー
    async fn create_saga(repo: &InMemorySagaRepository, workflow_name: &str) -> SagaState {
        let saga = SagaState::new(
            workflow_name.to_string(),
            serde_json::json!({"task_id": "TASK-001", "assignee_id": "user-001"}),
            Some("corr-test".to_string()),
            Some("test-user".to_string()),
        );
        repo.create(&saga).await.unwrap();
        saga
    }

    /// 常に失敗するGrpcStepCaller。ステップ失敗→補償のテストに使用する。
    struct FailingGrpcCaller;

    #[async_trait::async_trait]
    impl GrpcStepCaller for FailingGrpcCaller {
        async fn call_step(
            &self,
            _service_name: &str,
            _method: &str,
            _payload: &serde_json::Value,
        ) -> anyhow::Result<serde_json::Value> {
            Err(anyhow::anyhow!("service unavailable"))
        }
    }

    /// 指定ステップでのみ失敗するGrpcStepCaller。部分的な失敗テストに使用する。
    struct SelectiveFailCaller {
        /// 失敗させるメソッド名のリスト
        fail_methods: Vec<String>,
    }

    #[async_trait::async_trait]
    impl GrpcStepCaller for SelectiveFailCaller {
        async fn call_step(
            &self,
            _service_name: &str,
            method: &str,
            _payload: &serde_json::Value,
        ) -> anyhow::Result<serde_json::Value> {
            if self.fail_methods.iter().any(|m| m == method) {
                Err(anyhow::anyhow!("step {} failed", method))
            } else {
                Ok(serde_json::json!({"status": "ok"}))
            }
        }
    }

    /// 発行されたイベントを記録するSagaEventPublisher。イベント発行の検証に使用する。
    struct RecordingPublisher {
        events: tokio::sync::Mutex<Vec<(String, String, serde_json::Value)>>,
    }

    impl RecordingPublisher {
        fn new() -> Self {
            Self {
                events: tokio::sync::Mutex::new(Vec::new()),
            }
        }

        /// 記録されたイベント一覧を取得する
        async fn recorded_events(&self) -> Vec<(String, String, serde_json::Value)> {
            self.events.lock().await.clone()
        }
    }

    #[async_trait::async_trait]
    impl SagaEventPublisher for RecordingPublisher {
        async fn publish_saga_event(
            &self,
            saga_id: &str,
            event_type: &str,
            payload: &serde_json::Value,
        ) -> anyhow::Result<()> {
            self.events.lock().await.push((
                saga_id.to_string(),
                event_type.to_string(),
                payload.clone(),
            ));
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    // =========================================================================
    // ユニットテスト（外部サービス不要）
    // =========================================================================

    /// ワークフロー定義のYAMLパースとバリデーションを検証する
    #[test]
    fn test_workflow_definition_parsing() {
        let workflow = make_two_step_workflow();
        assert_eq!(workflow.name, "task-assignment");
        assert_eq!(workflow.steps.len(), 2);
        assert_eq!(workflow.steps[0].name, "create-task");
        assert_eq!(workflow.steps[0].service, "task-server");
        assert_eq!(
            workflow.steps[0].compensate.as_deref(),
            Some("TaskService.CancelTask")
        );
        assert_eq!(workflow.steps[1].name, "increment-board-column");
    }

    /// ワークフロー定義のバリデーション: 空の名前はエラー
    #[test]
    fn test_workflow_validation_empty_name() {
        let yaml = r#"
name: ""
steps:
  - name: step1
    service: svc
    method: Svc.Do
"#;
        let result = WorkflowDefinition::from_yaml(yaml);
        assert!(result.is_err(), "空のワークフロー名はバリデーションエラー");
    }

    /// ワークフロー定義のバリデーション: ステップなしはエラー
    #[test]
    fn test_workflow_validation_no_steps() {
        let yaml = r#"
name: empty-workflow
steps: []
"#;
        let result = WorkflowDefinition::from_yaml(yaml);
        assert!(result.is_err(), "ステップなしはバリデーションエラー");
    }

    /// ワークフロー定義のバリデーション: ステップのservice空はエラー
    #[test]
    fn test_workflow_validation_empty_service() {
        let yaml = r#"
name: bad-workflow
steps:
  - name: step1
    service: ""
    method: Svc.Do
"#;
        let result = WorkflowDefinition::from_yaml(yaml);
        assert!(result.is_err(), "空のサービス名はバリデーションエラー");
    }

    /// ワークフロー定義のデフォルト値を検証する
    #[test]
    fn test_workflow_defaults() {
        let yaml = r#"
name: minimal
steps:
  - name: step1
    service: svc
    method: Svc.Do
"#;
        let workflow = WorkflowDefinition::from_yaml(yaml).unwrap();
        assert_eq!(workflow.version, 1, "デフォルトバージョンは1");
        assert!(workflow.enabled, "デフォルトで有効");
        assert_eq!(
            workflow.total_timeout_secs, 300,
            "デフォルトタイムアウトは300秒"
        );
        assert_eq!(
            workflow.steps[0].timeout_secs, 30,
            "ステップのデフォルトタイムアウトは30秒"
        );
        assert!(
            workflow.steps[0].compensate.is_none(),
            "補償メソッドはデフォルトでなし"
        );
        assert!(
            workflow.steps[0].retry.is_none(),
            "リトライ設定はデフォルトでなし"
        );
    }

    /// 全ステップ成功時にSagaが COMPLETED になることを検証する
    #[tokio::test]
    async fn test_full_workflow_success() {
        let repo = Arc::new(InMemorySagaRepository::new());
        let caller: Arc<dyn GrpcStepCaller> = Arc::new(NoOpGrpcCaller);
        let publisher = Arc::new(RecordingPublisher::new());

        let uc = ExecuteSagaUseCase::new(
            repo.clone(),
            caller,
            Some(publisher.clone() as Arc<dyn SagaEventPublisher>),
        );

        let workflow = make_two_step_workflow();
        let saga = create_saga(&repo, "task-assignment").await;
        let saga_id = saga.saga_id;

        // Sagaを実行する
        let result = uc.run(saga_id, &workflow).await;
        assert!(result.is_ok(), "Saga実行が成功する");

        // 最終ステータスが COMPLETED であることを検証する
        let final_state = repo.find_by_id(saga_id).await.unwrap().unwrap();
        assert_eq!(final_state.status, SagaStatus::Completed);
        assert!(final_state.error_message.is_none());

        // 各ステップのログが記録されていることを検証する
        let logs = repo.find_step_logs(saga_id).await.unwrap();
        assert_eq!(logs.len(), 2, "2ステップのログが記録される");
        assert!(
            logs.iter().all(|l| l.status == StepStatus::Success),
            "全ステップが成功"
        );
        assert!(
            logs.iter().all(|l| l.action == StepAction::Execute),
            "全ステップが実行アクション"
        );

        // イベントが発行されていることを検証する
        let events = publisher.recorded_events().await;
        let event_types: Vec<&str> = events.iter().map(|(_, t, _)| t.as_str()).collect();
        assert!(
            event_types.contains(&"SAGA_RUNNING"),
            "SAGA_RUNNING イベントが発行される"
        );
        assert!(
            event_types.contains(&"SAGA_COMPLETED"),
            "SAGA_COMPLETED イベントが発行される"
        );
    }

    /// ステップ失敗時に補償処理が実行されSagaが FAILED になることを検証する
    #[tokio::test]
    async fn test_step_failure_triggers_compensation() {
        let repo = Arc::new(InMemorySagaRepository::new());
        // 2番目のステップ（BoardService.IncrementColumn）で失敗するcallerを作成する
        let caller: Arc<dyn GrpcStepCaller> = Arc::new(SelectiveFailCaller {
            fail_methods: vec!["BoardService.IncrementColumn".to_string()],
        });
        let publisher = Arc::new(RecordingPublisher::new());

        let uc = ExecuteSagaUseCase::new(
            repo.clone(),
            caller,
            Some(publisher.clone() as Arc<dyn SagaEventPublisher>),
        );

        let workflow = make_two_step_workflow();
        let saga = create_saga(&repo, "task-assignment").await;
        let saga_id = saga.saga_id;

        // Sagaを実行する（ステップ2で失敗→補償）
        let result = uc.run(saga_id, &workflow).await;
        assert!(result.is_ok(), "補償後にエラーなく完了する");

        // 最終ステータスが FAILED であることを検証する
        let final_state = repo.find_by_id(saga_id).await.unwrap().unwrap();
        assert_eq!(final_state.status, SagaStatus::Failed);
        assert!(
            final_state.error_message.is_some(),
            "エラーメッセージが設定される"
        );

        // ステップログに実行と補償の両方が記録されていることを検証する
        let logs = repo.find_step_logs(saga_id).await.unwrap();
        // ステップ0成功 + ステップ1失敗 + ステップ0補償 = 3件
        assert!(logs.len() >= 3, "実行ログと補償ログが記録される");

        // 補償アクションのログが存在することを検証する
        let compensation_logs: Vec<_> = logs
            .iter()
            .filter(|l| l.action == StepAction::Compensate)
            .collect();
        assert!(
            !compensation_logs.is_empty(),
            "補償ステップのログが記録される"
        );

        // イベントに SAGA_COMPENSATING が含まれることを検証する
        let events = publisher.recorded_events().await;
        let event_types: Vec<&str> = events.iter().map(|(_, t, _)| t.as_str()).collect();
        assert!(
            event_types.contains(&"SAGA_COMPENSATING"),
            "SAGA_COMPENSATING イベントが発行される"
        );
        assert!(
            event_types.contains(&"SAGA_FAILED"),
            "SAGA_FAILED イベントが発行される"
        );
    }

    /// 最初のステップで失敗した場合の補償処理を検証する（補償不要のケース）
    #[tokio::test]
    async fn test_first_step_failure() {
        let repo = Arc::new(InMemorySagaRepository::new());
        // 全てのステップで失敗するcallerを作成する
        let caller: Arc<dyn GrpcStepCaller> = Arc::new(FailingGrpcCaller);

        let uc = ExecuteSagaUseCase::new(repo.clone(), caller, None);

        let workflow = make_two_step_workflow();
        let saga = create_saga(&repo, "task-assignment").await;
        let saga_id = saga.saga_id;

        // Sagaを実行する（ステップ0で失敗）
        let result = uc.run(saga_id, &workflow).await;
        assert!(result.is_ok(), "補償完了後にエラーなく返る");

        // 最終ステータスが FAILED であることを検証する
        let final_state = repo.find_by_id(saga_id).await.unwrap().unwrap();
        assert_eq!(final_state.status, SagaStatus::Failed);
    }

    /// 既に終端状態のSagaに対してrunを呼んでもスキップされることを検証する
    #[tokio::test]
    async fn test_terminal_state_skipped() {
        let repo = Arc::new(InMemorySagaRepository::new());
        let caller: Arc<dyn GrpcStepCaller> = Arc::new(NoOpGrpcCaller);

        let uc = ExecuteSagaUseCase::new(repo.clone(), caller, None);

        // COMPLETED状態のSagaを作成する
        let mut saga = SagaState::new(
            "task-assignment".to_string(),
            serde_json::json!({}),
            None,
            None,
        );
        saga.complete();
        let saga_id = saga.saga_id;
        repo.create(&saga).await.unwrap();

        let workflow = make_two_step_workflow();

        // 終端状態のSaga実行を試みる
        let result = uc.run(saga_id, &workflow).await;
        assert!(result.is_ok(), "終端状態はスキップされる");

        // ステータスが変わっていないことを検証する
        let state = repo.find_by_id(saga_id).await.unwrap().unwrap();
        assert_eq!(state.status, SagaStatus::Completed, "COMPLETED のまま");
    }

    /// 3ステップワークフローの中間ステップ失敗時の補償を検証する
    #[tokio::test]
    async fn test_three_step_mid_failure_compensation() {
        let repo = Arc::new(InMemorySagaRepository::new());
        // ステップ2（SvcB.Do）で失敗するcallerを作成する
        let caller: Arc<dyn GrpcStepCaller> = Arc::new(SelectiveFailCaller {
            fail_methods: vec!["SvcB.Do".to_string()],
        });

        let uc = ExecuteSagaUseCase::new(repo.clone(), caller, None);

        let workflow = make_three_step_workflow();
        let saga = create_saga(&repo, "three-step-workflow").await;
        let saga_id = saga.saga_id;

        // Sagaを実行する
        let result = uc.run(saga_id, &workflow).await;
        assert!(result.is_ok());

        // FAILED になることを検証する
        let final_state = repo.find_by_id(saga_id).await.unwrap().unwrap();
        assert_eq!(final_state.status, SagaStatus::Failed);

        // ステップログを確認: step-1成功 + step-2失敗 + 補償ログ
        let logs = repo.find_step_logs(saga_id).await.unwrap();

        // 実行ログの検証
        let execute_logs: Vec<_> = logs
            .iter()
            .filter(|l| l.action == StepAction::Execute)
            .collect();
        assert_eq!(
            execute_logs.len(),
            2,
            "step-1成功 + step-2失敗 = 2件の実行ログ"
        );

        // step-1 は成功していることを検証する
        assert_eq!(execute_logs[0].step_name, "step-1");
        assert_eq!(execute_logs[0].status, StepStatus::Success);

        // step-2 は失敗していることを検証する
        assert_eq!(execute_logs[1].step_name, "step-2");
        assert_eq!(execute_logs[1].status, StepStatus::Failed);

        // 補償ログが存在することを検証する（step-2は補償なし→SKIPPED、step-1は補償あり→SUCCESS）
        let compensate_logs: Vec<_> = logs
            .iter()
            .filter(|l| l.action == StepAction::Compensate)
            .collect();
        assert!(!compensate_logs.is_empty(), "補償ステップのログが存在する");
    }

    /// publisher なしでもSaga実行が正常に完了することを検証する
    #[tokio::test]
    async fn test_execution_without_publisher() {
        let repo = Arc::new(InMemorySagaRepository::new());
        let caller: Arc<dyn GrpcStepCaller> = Arc::new(NoOpGrpcCaller);

        // publisher を None にして作成する
        let uc = ExecuteSagaUseCase::new(repo.clone(), caller, None);

        let workflow = make_two_step_workflow();
        let saga = create_saga(&repo, "task-assignment").await;
        let saga_id = saga.saga_id;

        // publisherなしでも正常に実行できることを検証する
        let result = uc.run(saga_id, &workflow).await;
        assert!(result.is_ok());

        let final_state = repo.find_by_id(saga_id).await.unwrap().unwrap();
        assert_eq!(final_state.status, SagaStatus::Completed);
    }

    /// NoOpPublisher の動作を検証する
    #[tokio::test]
    async fn test_noop_publisher() {
        let publisher = NoOpPublisher;
        let payload = serde_json::json!({"test": true});

        // publish_saga_event が成功を返すことを検証する
        let result = publisher
            .publish_saga_event("saga-001", "SAGA_STARTED", &payload)
            .await;
        assert!(result.is_ok());

        // close が成功を返すことを検証する
        let result = publisher.close().await;
        assert!(result.is_ok());
    }

    /// NoOpGrpcCaller の動作を検証する
    #[tokio::test]
    async fn test_noop_grpc_caller() {
        let caller = NoOpGrpcCaller;
        let payload = serde_json::json!({"task_id": "123"});

        // call_step が成功レスポンスを返すことを検証する
        let result = caller
            .call_step("task-server", "TaskService.CreateTask", &payload)
            .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["status"], "ok");
    }

    // =========================================================================
    // 統合テスト（REST API / 外部サービス経由のエンドツーエンド）
    // =========================================================================

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    #[ignore = "covered by src/usecase/execute_saga.rs unit tests and tests/integration_test.rs"]
    async fn test_rest_api_full_workflow() {
        // REST API 経由の完全なワークフロー実行テスト
        // POST /api/sagas でSaga開始 → GET /api/sagas/:id で結果確認
        // see: tests/integration_test.rs
    }
}
