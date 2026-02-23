//! REST API統合テスト（インメモリリポジトリ使用）
//!
//! dlq-manager の統合テストパターン（tower::ServiceExt + oneshot）を踏襲。

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_saga_server::adapter::handler;
use k1s0_saga_server::adapter::repository::workflow_in_memory::InMemoryWorkflowRepository;
use k1s0_saga_server::domain::entity::saga_state::SagaState;
use k1s0_saga_server::domain::entity::saga_step_log::{SagaStepLog, StepAction};
use k1s0_saga_server::domain::repository::SagaRepository;
use k1s0_saga_server::test_support::{make_test_app_state, InMemorySagaRepository};

/// テスト用ワークフローYAML
const TEST_WORKFLOW_YAML: &str = r#"
name: order-workflow
steps:
  - name: reserve-inventory
    service: inventory-service
    method: InventoryService.Reserve
    compensate: InventoryService.Release
    timeout_secs: 5
    retry:
      max_attempts: 1
      initial_interval_ms: 100
  - name: charge-payment
    service: payment-service
    method: PaymentService.Charge
    compensate: PaymentService.Refund
    timeout_secs: 5
    retry:
      max_attempts: 1
      initial_interval_ms: 100
"#;

/// テスト用 Router を生成するヘルパー。
fn make_app() -> axum::Router {
    let saga_repo = Arc::new(InMemorySagaRepository::new());
    let workflow_repo = Arc::new(InMemoryWorkflowRepository::new());
    let state = make_test_app_state(saga_repo, workflow_repo);
    handler::router(state)
}

/// 共有リポジトリを持つ Router を生成するヘルパー（複数リクエストのテスト用）。
fn make_app_with_repos() -> (
    axum::Router,
    Arc<InMemorySagaRepository>,
    Arc<InMemoryWorkflowRepository>,
) {
    let saga_repo = Arc::new(InMemorySagaRepository::new());
    let workflow_repo = Arc::new(InMemoryWorkflowRepository::new());
    let state = make_test_app_state(saga_repo.clone(), workflow_repo.clone());
    (handler::router(state), saga_repo, workflow_repo)
}

/// 同じリポジトリから新しい Router を再構築する（oneshot 消費後に再利用するため）。
fn rebuild_app(
    saga_repo: Arc<InMemorySagaRepository>,
    workflow_repo: Arc<InMemoryWorkflowRepository>,
) -> axum::Router {
    let state = make_test_app_state(saga_repo, workflow_repo);
    handler::router(state)
}

// ---------------------------------------------------------------------------
// Health / Readiness
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_healthz_returns_ok() {
    let app = make_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_readyz_returns_ok() {
    let app = make_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Workflow registration & listing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_register_and_list_workflows() {
    let (app, saga_repo, workflow_repo) = make_app_with_repos();

    // Register workflow
    let register_body = serde_json::json!({
        "workflow_yaml": TEST_WORKFLOW_YAML,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/workflows")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "order-workflow");
    assert_eq!(json["step_count"], 2);

    // List workflows
    let app2 = rebuild_app(saga_repo, workflow_repo);
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let workflows = json["workflows"].as_array().unwrap();
    assert_eq!(workflows.len(), 1);
    assert_eq!(workflows[0]["name"], "order-workflow");
}

// ---------------------------------------------------------------------------
// Saga start
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_start_saga() {
    let (app, saga_repo, workflow_repo) = make_app_with_repos();

    // まずワークフロー登録
    let register_body = serde_json::json!({
        "workflow_yaml": TEST_WORKFLOW_YAML,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/workflows")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Saga 開始
    let app2 = rebuild_app(saga_repo, workflow_repo);
    let start_body = serde_json::json!({
        "workflow_name": "order-workflow",
        "payload": {"order_id": "order-123"},
        "correlation_id": "corr-001",
        "initiated_by": "test-user",
    });

    let response = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sagas")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&start_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["saga_id"].as_str().is_some());
    assert_eq!(json["status"], "STARTED");
}

// ---------------------------------------------------------------------------
// Saga listing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_sagas() {
    let (app, saga_repo, workflow_repo) = make_app_with_repos();

    // ワークフロー登録
    let register_body = serde_json::json!({ "workflow_yaml": TEST_WORKFLOW_YAML });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/workflows")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Saga 開始
    let app2 = rebuild_app(saga_repo.clone(), workflow_repo.clone());
    let start_body = serde_json::json!({
        "workflow_name": "order-workflow",
        "payload": {"order_id": "order-456"},
    });
    let response = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sagas")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&start_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // バックグラウンド実行のため少し待つ
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Saga 一覧取得
    let app3 = rebuild_app(saga_repo, workflow_repo);
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/v1/sagas")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let sagas = json["sagas"].as_array().unwrap();
    assert!(!sagas.is_empty(), "saga list should not be empty");
    assert!(json["pagination"]["total_count"].as_i64().unwrap() >= 1);
}

// ---------------------------------------------------------------------------
// Get saga by ID
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_saga_by_id() {
    let (app, saga_repo, workflow_repo) = make_app_with_repos();

    // ワークフロー登録
    let register_body = serde_json::json!({ "workflow_yaml": TEST_WORKFLOW_YAML });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/workflows")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Saga 開始
    let app2 = rebuild_app(saga_repo.clone(), workflow_repo.clone());
    let start_body = serde_json::json!({
        "workflow_name": "order-workflow",
        "payload": {},
    });
    let response = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sagas")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&start_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let saga_id = json["saga_id"].as_str().unwrap();

    // バックグラウンド実行のため少し待つ
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Saga 詳細取得
    let app3 = rebuild_app(saga_repo, workflow_repo);
    let response = app3
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sagas/{}", saga_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["saga"]["saga_id"], saga_id);
    assert_eq!(json["saga"]["workflow_name"], "order-workflow");
}

// ---------------------------------------------------------------------------
// Get saga not found
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_saga_not_found() {
    let app = make_app();

    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sagas/{}", fake_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Cancel saga
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_cancel_saga() {
    let (app, saga_repo, workflow_repo) = make_app_with_repos();

    // ワークフロー登録
    let register_body = serde_json::json!({ "workflow_yaml": TEST_WORKFLOW_YAML });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/workflows")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Saga 開始
    let app2 = rebuild_app(saga_repo.clone(), workflow_repo.clone());
    let start_body = serde_json::json!({
        "workflow_name": "order-workflow",
        "payload": {},
    });
    let response = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sagas")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&start_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let saga_id = json["saga_id"].as_str().unwrap();

    // バックグラウンドの saga 実行を待たずにキャンセル（STARTED 状態でキャンセル）
    // 少し待ってからキャンセル
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let app3 = rebuild_app(saga_repo, workflow_repo);
    let response = app3
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sagas/{}/cancel", saga_id))
                .header("content-type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // キャンセルはターミナル状態でなければ成功する
    // バックグラウンド実行がすでに完了している場合は Conflict の可能性あり
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::CONFLICT,
        "expected OK or CONFLICT, got {}",
        status
    );
}

// ---------------------------------------------------------------------------
// Saga 補償トランザクション検証
// ---------------------------------------------------------------------------

/// COMPENSATING 状態の Saga を API で取得すると正しいステータスとエラーメッセージが返ること。
#[tokio::test]
async fn test_get_compensating_saga_returns_compensating_status() {
    let (_, saga_repo, workflow_repo) = make_app_with_repos();

    // COMPENSATING 状態の Saga を直接リポジトリに挿入
    let mut saga = SagaState::new(
        "order-workflow".to_string(),
        serde_json::json!({"order_id": "order-999"}),
        Some("corr-comp-001".to_string()),
        Some("test-user".to_string()),
    );
    saga.start_compensation("reserve-inventory step failed".to_string());
    let saga_id = saga.saga_id;
    saga_repo.create(&saga).await.unwrap();

    // GET /api/v1/sagas/{id}
    let app = rebuild_app(saga_repo, workflow_repo);
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sagas/{}", saga_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["saga"]["status"], "COMPENSATING");
    assert_eq!(
        json["saga"]["error_message"],
        "reserve-inventory step failed"
    );
}

/// FAILED 状態の Saga を API で取得すると error_message が返ること。
#[tokio::test]
async fn test_get_failed_saga_returns_error_message() {
    let (_, saga_repo, workflow_repo) = make_app_with_repos();

    // FAILED 状態の Saga を直接リポジトリに挿入
    let mut saga = SagaState::new(
        "order-workflow".to_string(),
        serde_json::json!({}),
        None,
        None,
    );
    saga.fail("unrecoverable compensation error".to_string());
    let saga_id = saga.saga_id;
    saga_repo.create(&saga).await.unwrap();

    // GET /api/v1/sagas/{id}
    let app = rebuild_app(saga_repo, workflow_repo);
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sagas/{}", saga_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["saga"]["status"], "FAILED");
    assert!(json["saga"]["is_terminal"].is_null() || json["saga"].get("is_terminal").is_none());
    assert_eq!(
        json["saga"]["error_message"],
        "unrecoverable compensation error"
    );
}

/// 補償ステップのログが API レスポンスに含まれること。
#[tokio::test]
async fn test_get_saga_step_logs_include_compensate_action() {
    let (_, saga_repo, workflow_repo) = make_app_with_repos();

    // COMPENSATING 状態の Saga を作成
    let mut saga = SagaState::new(
        "order-workflow".to_string(),
        serde_json::json!({}),
        None,
        None,
    );
    saga.start_compensation("step 0 failed".to_string());
    let saga_id = saga.saga_id;
    saga_repo.create(&saga).await.unwrap();

    // 実行ログ (EXECUTE, FAILED)
    let mut execute_log = SagaStepLog::new_execute(
        saga_id,
        0,
        "reserve-inventory".to_string(),
        Some(serde_json::json!({"item_id": "abc"})),
    );
    execute_log.mark_failed("service timeout".to_string());
    saga_repo
        .update_with_step_log(&saga, &execute_log)
        .await
        .unwrap();

    // 補償ログ (COMPENSATE, SUCCESS)
    let mut compensate_log =
        SagaStepLog::new_compensate(saga_id, 0, "reserve-inventory".to_string(), None);
    compensate_log.mark_success(Some(serde_json::json!({"released": true})));
    saga_repo
        .update_with_step_log(&saga, &compensate_log)
        .await
        .unwrap();

    // GET /api/v1/sagas/{id}
    let app = rebuild_app(saga_repo, workflow_repo);
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sagas/{}", saga_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let step_logs = json["step_logs"].as_array().unwrap();
    assert_eq!(
        step_logs.len(),
        2,
        "EXECUTE と COMPENSATE の 2 件のログが返るべき"
    );

    // COMPENSATE アクションのログが含まれることを確認
    let has_compensate = step_logs
        .iter()
        .any(|l| l["action"].as_str() == Some(StepAction::Compensate.to_string().as_str()));
    assert!(
        has_compensate,
        "step_logs に COMPENSATE アクションが含まれるべき"
    );

    // EXECUTE アクションのログも含まれることを確認
    let has_execute = step_logs
        .iter()
        .any(|l| l["action"].as_str() == Some(StepAction::Execute.to_string().as_str()));
    assert!(has_execute, "step_logs に EXECUTE アクションが含まれるべき");
}

// ---------------------------------------------------------------------------
// Start saga with missing workflow
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_start_saga_missing_workflow() {
    let app = make_app();

    let start_body = serde_json::json!({
        "workflow_name": "nonexistent-workflow",
        "payload": {},
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sagas")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&start_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let message = json["error"]["message"].as_str().unwrap_or("");
    assert!(
        message.contains("workflow not found"),
        "expected 'workflow not found' in error message, got: {}",
        message
    );
}
