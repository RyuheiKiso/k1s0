// service tier 統合テスト: スタブリポジトリを使いHTTPレイヤーを検証する。
// auth パターンに倣い AppState をスタブで構築し、認証なしモードで動作確認を行う。
// 実 DB 接続テストは #[cfg(feature = "db-tests")] で区分けする。
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_task_server::adapter::handler::{router, AppState};
use sqlx;
use k1s0_task_server::domain::entity::task::{
    AddChecklistItem, CreateTask, Task, TaskChecklistItem, TaskFilter, TaskPriority, TaskStatus,
    UpdateChecklistItem, UpdateTask, UpdateTaskStatus,
};
use k1s0_task_server::domain::error::TaskError;
use k1s0_task_server::domain::repository::task_repository::TaskRepository;

// --- テスト用スタブリポジトリ ---

/// テスト用スタブリポジトリ（インメモリ実装）
struct StubTaskRepository {
    tasks: tokio::sync::RwLock<Vec<Task>>,
}

impl StubTaskRepository {
    fn new() -> Self {
        Self {
            tasks: tokio::sync::RwLock::new(Vec::new()),
        }
    }

    fn with_tasks(tasks: Vec<Task>) -> Self {
        Self {
            tasks: tokio::sync::RwLock::new(tasks),
        }
    }
}

fn sample_task() -> Task {
    Task {
        id: uuid::Uuid::new_v4(),
        project_id: uuid::Uuid::new_v4(),
        title: "統合テスト用タスク".to_string(),
        description: Some("テスト説明".to_string()),
        status: TaskStatus::Open,
        priority: TaskPriority::Medium,
        assignee_id: None,
        reporter_id: Some("test-user".to_string()),
        due_date: None,
        labels: vec![],
        created_by: "test-user".to_string(),
        updated_by: None,
        version: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[async_trait::async_trait]
impl TaskRepository for StubTaskRepository {
    async fn find_by_id(&self, _tenant_id: &str, id: uuid::Uuid) -> Result<Option<Task>, TaskError> {
        let tasks = self.tasks.read().await;
        Ok(tasks.iter().find(|t| t.id == id).cloned())
    }

    async fn find_all(&self, _tenant_id: &str, _filter: &TaskFilter) -> Result<Vec<Task>, TaskError> {
        let tasks = self.tasks.read().await;
        Ok(tasks.clone())
    }

    async fn count(&self, _tenant_id: &str, _filter: &TaskFilter) -> Result<i64, TaskError> {
        let tasks = self.tasks.read().await;
        Ok(tasks.len() as i64)
    }

    async fn create(&self, _tenant_id: &str, input: &CreateTask, created_by: &str) -> Result<Task, TaskError> {
        let task = Task {
            id: uuid::Uuid::new_v4(),
            project_id: input.project_id,
            title: input.title.clone(),
            description: input.description.clone(),
            status: TaskStatus::Open,
            priority: input.priority.clone(),
            assignee_id: input.assignee_id.clone(),
            reporter_id: input.reporter_id.clone(),
            due_date: input.due_date,
            labels: input.labels.clone(),
            created_by: created_by.to_string(),
            updated_by: None,
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        self.tasks.write().await.push(task.clone());
        Ok(task)
    }

    async fn find_checklist(&self, _tenant_id: &str, _task_id: uuid::Uuid) -> Result<Vec<TaskChecklistItem>, TaskError> {
        Ok(vec![])
    }

    async fn update(&self, _tenant_id: &str, id: uuid::Uuid, input: &UpdateTask, updated_by: &str) -> Result<Task, TaskError> {
        let mut tasks = self.tasks.write().await;
        let task = tasks.iter_mut().find(|t| t.id == id)
            .ok_or_else(|| TaskError::Infrastructure(anyhow::anyhow!("task not found: {}", id)))?;
        if let Some(ref title) = input.title {
            task.title = title.clone();
        }
        if let Some(ref desc) = input.description {
            task.description = Some(desc.clone());
        }
        task.updated_by = Some(updated_by.to_string());
        Ok(task.clone())
    }

    async fn add_checklist_item(&self, _tenant_id: &str, task_id: uuid::Uuid, input: &AddChecklistItem) -> Result<TaskChecklistItem, TaskError> {
        Ok(TaskChecklistItem {
            id: uuid::Uuid::new_v4(),
            task_id,
            title: input.title.clone(),
            is_completed: false,
            sort_order: input.sort_order,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    async fn update_checklist_item(&self, _tenant_id: &str, task_id: uuid::Uuid, item_id: uuid::Uuid, input: &UpdateChecklistItem) -> Result<TaskChecklistItem, TaskError> {
        Ok(TaskChecklistItem {
            id: item_id,
            task_id,
            title: input.title.clone().unwrap_or_default(),
            is_completed: input.is_completed.unwrap_or(false),
            sort_order: input.sort_order.unwrap_or(0),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    async fn delete_checklist_item(&self, _tenant_id: &str, _task_id: uuid::Uuid, _item_id: uuid::Uuid) -> Result<(), TaskError> {
        Ok(())
    }

    async fn update_status(
        &self,
        _tenant_id: &str,
        id: uuid::Uuid,
        input: &UpdateTaskStatus,
        updated_by: &str,
    ) -> Result<Task, TaskError> {
        let mut tasks = self.tasks.write().await;
        let task = tasks.iter_mut().find(|t| t.id == id)
            .ok_or_else(|| TaskError::Infrastructure(anyhow::anyhow!("task not found: {}", id)))?;
        task.status = input.status.clone();
        task.updated_by = Some(updated_by.to_string());
        Ok(task.clone())
    }
}

/// スタブを使ってテスト用 AppState を構築する
fn make_test_app() -> axum::Router {
    let repo = Arc::new(StubTaskRepository::new());
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("task-test"));

    let state = AppState {
        create_task_uc: Arc::new(k1s0_task_server::usecase::create_task::CreateTaskUseCase::new(
            repo.clone(),
        )),
        get_task_uc: Arc::new(k1s0_task_server::usecase::get_task::GetTaskUseCase::new(
            repo.clone(),
        )),
        list_tasks_uc: Arc::new(k1s0_task_server::usecase::list_tasks::ListTasksUseCase::new(
            repo.clone(),
        )),
        update_task_status_uc: Arc::new(
            k1s0_task_server::usecase::update_task_status::UpdateTaskStatusUseCase::new(
                repo.clone(),
            ),
        ),
        update_task_uc: Arc::new(k1s0_task_server::usecase::update_task::UpdateTaskUseCase::new(
            repo.clone(),
        )),
        create_checklist_item_uc: Arc::new(
            k1s0_task_server::usecase::create_checklist_item::CreateChecklistItemUseCase::new(
                repo.clone(),
            ),
        ),
        update_checklist_item_uc: Arc::new(
            k1s0_task_server::usecase::update_checklist_item::UpdateChecklistItemUseCase::new(
                repo.clone(),
            ),
        ),
        delete_checklist_item_uc: Arc::new(
            k1s0_task_server::usecase::delete_checklist_item::DeleteChecklistItemUseCase::new(
                repo.clone(),
            ),
        ),
        metrics,
        auth_state: None,
        // テスト用の遅延接続プール（実際には接続しない。/readyz エンドポイントのテストには使用しない）
        db_pool: sqlx::PgPool::connect_lazy("postgres://localhost/test")
            .expect("テスト用 lazy pool の作成に失敗しました"),
    };
    router(state)
}

/// スタブにタスクを1件含んだ状態でテスト用 AppState を構築する
fn make_test_app_with_task(task: Task) -> axum::Router {
    let repo = Arc::new(StubTaskRepository::with_tasks(vec![task]));
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("task-test"));

    let state = AppState {
        create_task_uc: Arc::new(k1s0_task_server::usecase::create_task::CreateTaskUseCase::new(
            repo.clone(),
        )),
        get_task_uc: Arc::new(k1s0_task_server::usecase::get_task::GetTaskUseCase::new(
            repo.clone(),
        )),
        list_tasks_uc: Arc::new(k1s0_task_server::usecase::list_tasks::ListTasksUseCase::new(
            repo.clone(),
        )),
        update_task_status_uc: Arc::new(
            k1s0_task_server::usecase::update_task_status::UpdateTaskStatusUseCase::new(
                repo.clone(),
            ),
        ),
        update_task_uc: Arc::new(k1s0_task_server::usecase::update_task::UpdateTaskUseCase::new(
            repo.clone(),
        )),
        create_checklist_item_uc: Arc::new(
            k1s0_task_server::usecase::create_checklist_item::CreateChecklistItemUseCase::new(
                repo.clone(),
            ),
        ),
        update_checklist_item_uc: Arc::new(
            k1s0_task_server::usecase::update_checklist_item::UpdateChecklistItemUseCase::new(
                repo.clone(),
            ),
        ),
        delete_checklist_item_uc: Arc::new(
            k1s0_task_server::usecase::delete_checklist_item::DeleteChecklistItemUseCase::new(
                repo.clone(),
            ),
        ),
        metrics,
        auth_state: None,
        // テスト用の遅延接続プール（実際には接続しない。/readyz エンドポイントのテストには使用しない）
        db_pool: sqlx::PgPool::connect_lazy("postgres://localhost/test")
            .expect("テスト用 lazy pool の作成に失敗しました"),
    };
    router(state)
}

// --- 統合テスト ---

/// ヘルスチェックエンドポイント（/healthz）が 200 を返すことを確認する
// /readyz エンドポイントは実際の PostgreSQL 接続を必要とするため、ローカルテストではスキップする
// 統合テスト環境（CI/CD）では sqlx::test マクロを使った接続付きテストで検証する
#[ignore]
#[tokio::test]
async fn test_health_check() {
    let app = make_test_app();

    // /healthz が 200 を返すことを確認する
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .expect("healthz リクエストの構築に失敗");
    let resp = app.clone().oneshot(req).await.expect("healthz リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    // /readyz が 200 を返すことを確認する
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .expect("readyz リクエストの構築に失敗");
    let resp = app.clone().oneshot(req).await.expect("readyz リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    // /metrics が 200 を返すことを確認する
    let req = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .expect("metrics リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("metrics リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
}

/// タスク一覧取得（GET /api/v1/tasks）が 200 と空リストを返すことを確認する
#[tokio::test]
async fn test_list_tasks_empty() {
    let app = make_test_app();

    let req = Request::builder()
        .uri("/api/v1/tasks")
        .body(Body::empty())
        .expect("タスク一覧リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("タスク一覧リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("タスク一覧レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("タスク一覧レスポンスの JSON パースに失敗");
    assert!(json["tasks"].as_array().expect("tasks フィールドが配列でない").is_empty());
    assert_eq!(json["total"], 0);
}

/// タスク作成（POST /api/v1/tasks）が 201 と作成済みタスクを返すことを確認する
#[tokio::test]
async fn test_create_task() {
    let app = make_test_app();

    let project_id = uuid::Uuid::new_v4();
    let body = serde_json::json!({
        "project_id": project_id,
        "title": "新規タスク",
        "description": "テスト用タスク",
        "priority": "medium",
        "labels": [],
        "checklist": []
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("タスク作成ボディの JSON シリアライズに失敗")))
        .expect("タスク作成リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("タスク作成リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("タスク作成レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("タスク作成レスポンスの JSON パースに失敗");
    assert_eq!(json["title"], "新規タスク");
    assert_eq!(json["status"], "open");
    assert!(json["id"].is_string());
}

/// タスク取得（GET /api/v1/tasks/{id}）が既存タスクで 200 を返すことを確認する
#[tokio::test]
async fn test_get_task_found() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let req = Request::builder()
        .uri(format!("/api/v1/tasks/{}", task_id))
        .body(Body::empty())
        .expect("タスク取得リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("タスク取得リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("タスク取得レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("タスク取得レスポンスの JSON パースに失敗");
    assert_eq!(json["id"], task_id.to_string());
    assert_eq!(json["title"], "統合テスト用タスク");
}

/// タスク取得（GET /api/v1/tasks/{id}）が存在しない ID で 404 を返すことを確認する
#[tokio::test]
async fn test_get_task_not_found() {
    let app = make_test_app();

    let req = Request::builder()
        .uri(format!("/api/v1/tasks/{}", uuid::Uuid::new_v4()))
        .body(Body::empty())
        .expect("タスク未検出リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("タスク未検出リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// タスク一覧取得が作成済みタスクを含む場合に正しい件数を返すことを確認する
#[tokio::test]
async fn test_list_tasks_with_data() {
    let task = sample_task();
    let app = make_test_app_with_task(task);

    let req = Request::builder()
        .uri("/api/v1/tasks")
        .body(Body::empty())
        .expect("タスク一覧（データあり）リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("タスク一覧（データあり）リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("タスク一覧（データあり）レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("タスク一覧（データあり）レスポンスの JSON パースに失敗");
    let tasks_arr = json["tasks"].as_array().expect("tasks フィールドが配列でない");
    assert_eq!(tasks_arr.len(), 1);
    assert_eq!(json["total"], 1);
}

/// タスク作成のバリデーション（title が空）で 422 または 400 を返すことを確認する
#[tokio::test]
async fn test_create_task_empty_title() {
    let app = make_test_app();

    let body = serde_json::json!({
        "project_id": uuid::Uuid::new_v4(),
        "title": "",
        "priority": "medium",
        "labels": [],
        "checklist": []
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("バリデーションテストボディの JSON シリアライズに失敗")))
        .expect("バリデーションテストリクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("バリデーションテストリクエストの送信に失敗");
    // タイトルが空の場合はビジネスロジックエラー（500）またはバリデーションエラー（400/422）が返る
    assert_ne!(resp.status(), StatusCode::CREATED);
}

/// タスク作成レスポンスに description フィールドが含まれることを確認する
#[tokio::test]
async fn test_create_task_with_description() {
    let app = make_test_app();

    let body = serde_json::json!({
        "project_id": uuid::Uuid::new_v4(),
        "title": "説明付きタスク",
        "description": "詳細な説明文",
        "priority": "medium",
        "labels": [],
        "checklist": []
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["description"], "詳細な説明文");
}

/// タスク作成で priority=high が正しく反映されることを確認する
#[tokio::test]
async fn test_create_task_with_priority_high() {
    let app = make_test_app();

    let body = serde_json::json!({
        "project_id": uuid::Uuid::new_v4(),
        "title": "高優先度タスク",
        "priority": "high",
        "labels": [],
        "checklist": []
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["priority"], "high");
}

/// タスク作成で assignee_id が正しく反映されることを確認する
#[tokio::test]
async fn test_create_task_with_assignee() {
    let app = make_test_app();

    let body = serde_json::json!({
        "project_id": uuid::Uuid::new_v4(),
        "title": "担当者付きタスク",
        "priority": "medium",
        "assignee_id": "user-abc",
        "labels": [],
        "checklist": []
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["assignee_id"], "user-abc");
}

/// タスク取得レスポンスに project_id が含まれることを確認する
#[tokio::test]
async fn test_get_task_has_project_id() {
    let task = sample_task();
    let task_id = task.id;
    let project_id = task.project_id;
    let app = make_test_app_with_task(task);

    let req = Request::builder()
        .uri(format!("/api/v1/tasks/{}", task_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["project_id"], project_id.to_string());
}

/// タスク取得レスポンスの初期 status が open であることを確認する
#[tokio::test]
async fn test_get_task_initial_status_is_open() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let req = Request::builder()
        .uri(format!("/api/v1/tasks/{}", task_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["status"], "open");
}

/// タスク取得レスポンスに version フィールドが含まれることを確認する
#[tokio::test]
async fn test_get_task_has_version() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let req = Request::builder()
        .uri(format!("/api/v1/tasks/{}", task_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert!(json["version"].is_number());
}

/// タスク更新（PUT /api/v1/tasks/{id}）でタイトルが更新されることを確認する
#[tokio::test]
async fn test_update_task_title() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let body = serde_json::json!({"title": "更新後タイトル"});
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/tasks/{}", task_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["title"], "更新後タイトル");
}

/// タスク更新（PUT /api/v1/tasks/{id}）で存在しない ID に 404 を返すことを確認する
#[tokio::test]
async fn test_update_task_not_found() {
    let app = make_test_app();

    let body = serde_json::json!({"title": "更新後タイトル"});
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/tasks/{}", uuid::Uuid::new_v4()))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::OK);
}

/// ステータス更新（PUT /api/v1/tasks/{id}/status）で in_progress に遷移できることを確認する
#[tokio::test]
async fn test_update_task_status_to_in_progress() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let body = serde_json::json!({"status": "in_progress"});
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/tasks/{}/status", task_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["status"], "in_progress");
}

/// ステータス更新で done に遷移できることを確認する
#[tokio::test]
async fn test_update_task_status_to_done() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let body = serde_json::json!({"status": "done"});
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/tasks/{}/status", task_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["status"], "done");
}

/// ステータス更新で存在しない ID に対してエラーを返すことを確認する
#[tokio::test]
async fn test_update_task_status_not_found() {
    let app = make_test_app();

    let body = serde_json::json!({"status": "done"});
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/tasks/{}/status", uuid::Uuid::new_v4()))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::OK);
}

/// タスク一覧の total フィールドが件数と一致することを確認する
#[tokio::test]
async fn test_list_tasks_total_count_matches_array() {
    let tasks = vec![sample_task(), sample_task(), sample_task()];
    let repo = Arc::new(StubTaskRepository::with_tasks(tasks));
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("task-test"));
    let state = AppState {
        create_task_uc: Arc::new(k1s0_task_server::usecase::create_task::CreateTaskUseCase::new(repo.clone())),
        get_task_uc: Arc::new(k1s0_task_server::usecase::get_task::GetTaskUseCase::new(repo.clone())),
        list_tasks_uc: Arc::new(k1s0_task_server::usecase::list_tasks::ListTasksUseCase::new(repo.clone())),
        update_task_status_uc: Arc::new(k1s0_task_server::usecase::update_task_status::UpdateTaskStatusUseCase::new(repo.clone())),
        update_task_uc: Arc::new(k1s0_task_server::usecase::update_task::UpdateTaskUseCase::new(repo.clone())),
        create_checklist_item_uc: Arc::new(k1s0_task_server::usecase::create_checklist_item::CreateChecklistItemUseCase::new(repo.clone())),
        update_checklist_item_uc: Arc::new(k1s0_task_server::usecase::update_checklist_item::UpdateChecklistItemUseCase::new(repo.clone())),
        delete_checklist_item_uc: Arc::new(k1s0_task_server::usecase::delete_checklist_item::DeleteChecklistItemUseCase::new(repo.clone())),
        metrics,
        auth_state: None,
        db_pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").expect("テスト用 lazy pool の作成に失敗しました"),
    };
    let app = k1s0_task_server::adapter::handler::router(state);

    let req = Request::builder()
        .uri("/api/v1/tasks")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let tasks_arr = json["tasks"].as_array().expect("tasks フィールドが配列でない");
    assert_eq!(tasks_arr.len(), 3);
    assert_eq!(json["total"], 3);
}

/// チェックリスト取得（GET /api/v1/tasks/{id}/checklist）が 200 と空リストを返すことを確認する
#[tokio::test]
async fn test_get_checklist_empty() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let req = Request::builder()
        .uri(format!("/api/v1/tasks/{}/checklist", task_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert!(json.as_array().expect("チェックリストが配列でない").is_empty());
}

/// チェックリストアイテム追加（POST /api/v1/tasks/{id}/checklist）が 201 を返すことを確認する
#[tokio::test]
async fn test_create_checklist_item() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let body = serde_json::json!({
        "title": "チェックリストアイテム1",
        "sort_order": 0
    });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/checklist", task_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["title"], "チェックリストアイテム1");
    assert_eq!(json["is_completed"], false);
}

/// チェックリストアイテム更新（PUT /api/v1/tasks/{id}/checklist/{item_id}）が 200 を返すことを確認する
#[tokio::test]
async fn test_update_checklist_item() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let item_id = uuid::Uuid::new_v4();
    let body = serde_json::json!({
        "title": "更新済みアイテム",
        "is_completed": true
    });
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/tasks/{}/checklist/{}", task_id, item_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["is_completed"], true);
}

/// チェックリストアイテム削除（DELETE /api/v1/tasks/{id}/checklist/{item_id}）が 204 を返すことを確認する
#[tokio::test]
async fn test_delete_checklist_item() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let item_id = uuid::Uuid::new_v4();
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/tasks/{}/checklist/{}", task_id, item_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

/// タスク作成で labels 配列が正しく反映されることを確認する
#[tokio::test]
async fn test_create_task_with_labels() {
    let app = make_test_app();

    let body = serde_json::json!({
        "project_id": uuid::Uuid::new_v4(),
        "title": "ラベル付きタスク",
        "priority": "low",
        "labels": ["bug", "feature"],
        "checklist": []
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let labels = json["labels"].as_array().expect("labels フィールドが配列でない");
    assert_eq!(labels.len(), 2);
}

/// タスク作成のデフォルト status が open であることを確認する
#[tokio::test]
async fn test_create_task_default_status_is_open() {
    let app = make_test_app();

    let body = serde_json::json!({
        "project_id": uuid::Uuid::new_v4(),
        "title": "ステータス確認タスク",
        "priority": "medium",
        "labels": [],
        "checklist": []
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["status"], "open");
}

/// タスク一覧に query parameter を付けても 200 を返すことを確認する
#[tokio::test]
async fn test_list_tasks_with_query_params() {
    let task = sample_task();
    let project_id = task.project_id;
    let app = make_test_app_with_task(task);

    let req = Request::builder()
        .uri(format!("/api/v1/tasks?project_id={}&limit=10&offset=0", project_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
}

/// タスク取得レスポンスに created_at フィールドが含まれることを確認する
#[tokio::test]
async fn test_get_task_has_created_at() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let req = Request::builder()
        .uri(format!("/api/v1/tasks/{}", task_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert!(json["created_at"].is_string());
}

/// タスク一覧レスポンスの各アイテムに id フィールドが含まれることを確認する
#[tokio::test]
async fn test_list_tasks_items_have_id() {
    let task = sample_task();
    let app = make_test_app_with_task(task);

    let req = Request::builder()
        .uri("/api/v1/tasks")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let tasks_arr = json["tasks"].as_array().expect("tasks フィールドが配列でない");
    assert!(tasks_arr[0]["id"].is_string());
}

/// タスク更新で description を変更できることを確認する
#[tokio::test]
async fn test_update_task_description() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let body = serde_json::json!({"description": "更新後の説明"});
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/tasks/{}", task_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["description"], "更新後の説明");
}

/// タスク作成で project_id が欠如した場合にバリデーションエラーを返すことを確認する
#[tokio::test]
async fn test_create_task_missing_project_id() {
    let app = make_test_app();

    let body = serde_json::json!({
        "title": "プロジェクトIDなし",
        "priority": "medium",
        "labels": [],
        "checklist": []
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::CREATED);
}

/// チェックリストアイテム作成レスポンスに task_id が含まれることを確認する
#[tokio::test]
async fn test_create_checklist_item_has_task_id() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let body = serde_json::json!({"title": "アイテム確認", "sort_order": 1});
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/checklist", task_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["task_id"], task_id.to_string());
}

/// タスク作成で reporter_id が正しく反映されることを確認する
#[tokio::test]
async fn test_create_task_with_reporter() {
    let app = make_test_app();

    let body = serde_json::json!({
        "project_id": uuid::Uuid::new_v4(),
        "title": "報告者付きタスク",
        "priority": "medium",
        "reporter_id": "reporter-xyz",
        "labels": [],
        "checklist": []
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/tasks")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["reporter_id"], "reporter-xyz");
}

/// GET /api/v1/tasks/{id} で不正な UUID フォーマットに対してエラーを返すことを確認する
#[tokio::test]
async fn test_get_task_invalid_uuid() {
    let app = make_test_app();

    let req = Request::builder()
        .uri("/api/v1/tasks/not-a-uuid")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::OK);
}

/// タスク一覧レスポンスの各アイテムに status フィールドが含まれることを確認する
#[tokio::test]
async fn test_list_tasks_items_have_status() {
    let task = sample_task();
    let app = make_test_app_with_task(task);

    let req = Request::builder()
        .uri("/api/v1/tasks")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let tasks_arr = json["tasks"].as_array().expect("tasks フィールドが配列でない");
    assert!(tasks_arr[0]["status"].is_string());
}

/// ステータス更新で closed に遷移できることを確認する
#[tokio::test]
async fn test_update_task_status_to_closed() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let body = serde_json::json!({"status": "closed"});
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/tasks/{}/status", task_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["status"], "closed");
}

/// タスク取得レスポンスに priority フィールドが含まれることを確認する
#[tokio::test]
async fn test_get_task_has_priority() {
    let task = sample_task();
    let task_id = task.id;
    let app = make_test_app_with_task(task);

    let req = Request::builder()
        .uri(format!("/api/v1/tasks/{}", task_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert!(json["priority"].is_string());
}

/// タスク一覧レスポンスに tasks と total の両フィールドが含まれることを確認する
#[tokio::test]
async fn test_list_tasks_response_structure() {
    let app = make_test_app();

    let req = Request::builder()
        .uri("/api/v1/tasks")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert!(json["tasks"].is_array());
    assert!(json["total"].is_number());
}

// --- 実 DB を使ったテスト（db-tests feature が有効な場合のみ実行）---
// 現時点では #[ignore] で区分けし、CI の db-tests ジョブで有効化する

#[tokio::test]
#[cfg(feature = "db-tests")]
async fn test_task_crud_with_real_db() {
    // 実 DB を使った CRUD テスト（Phase 4 以降で実装予定）
    // TODO: testcontainers を使って PostgreSQL コンテナを起動し、
    //       リポジトリ実装の CRUD を検証する
}
