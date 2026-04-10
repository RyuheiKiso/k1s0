// service tier 統合テスト: スタブリポジトリを使いHTTPレイヤーを検証する。
// ハンドラーは Claims が必須のため、テスト用の fake Claims 注入ミドルウェアを使用する。
// 実 DB 接続テストは #[cfg(feature = "db-tests")] で区分けする。
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_activity_server::adapter::handler::{router, AppState};
use sqlx;
use k1s0_activity_server::domain::entity::activity::{
    Activity, ActivityFilter, ActivityStatus, ActivityType, CreateActivity,
};
use k1s0_activity_server::domain::error::ActivityError;
use k1s0_activity_server::domain::repository::activity_repository::ActivityRepository;

/// テスト用の fake Claims 注入ミドルウェア。
/// ハンドラーは `Option<Extension<Claims>>` が None の場合に 401 を返すため、
/// テスト環境でも Claims を Extension として挿入する必要がある。
async fn fake_claims_middleware(
    mut req: axum::http::Request<Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use std::collections::HashMap;
    // テスト用の Claims を生成して Extension として挿入する
    let claims = k1s0_auth::Claims {
        sub: "test-user-uuid".to_string(),
        iss: "test-issuer".to_string(),
        aud: k1s0_auth::Audience(vec!["test-audience".to_string()]),
        exp: u64::MAX,
        iat: 0,
        jti: None,
        typ: None,
        azp: None,
        scope: None,
        preferred_username: Some("test-user".to_string()),
        email: None,
        realm_access: Some(k1s0_auth::RealmAccess {
            roles: vec!["service_user".to_string()],
        }),
        resource_access: Some(HashMap::new()),
        tier_access: Some(vec!["service".to_string()]),
        tenant_id: "test-tenant".to_string(),
    };
    req.extensions_mut().insert(claims);
    next.run(req).await
}

// --- テスト用スタブリポジトリ ---

/// テスト用スタブリポジトリ（インメモリ実装）
struct StubActivityRepository {
    activities: tokio::sync::RwLock<Vec<Activity>>,
}

impl StubActivityRepository {
    fn new() -> Self {
        Self {
            activities: tokio::sync::RwLock::new(Vec::new()),
        }
    }

    fn with_activities(activities: Vec<Activity>) -> Self {
        Self {
            activities: tokio::sync::RwLock::new(activities),
        }
    }
}

fn sample_activity() -> Activity {
    Activity {
        id: uuid::Uuid::new_v4(),
        task_id: uuid::Uuid::new_v4(),
        actor_id: "test-user".to_string(),
        activity_type: ActivityType::Comment,
        content: Some("統合テスト用コメント".to_string()),
        duration_minutes: None,
        status: ActivityStatus::Active,
        idempotency_key: None,
        version: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[async_trait::async_trait]
impl ActivityRepository for StubActivityRepository {
    async fn find_by_id(&self, _tenant_id: &str, id: uuid::Uuid) -> Result<Option<Activity>, ActivityError> {
        let activities = self.activities.read().await;
        Ok(activities.iter().find(|a| a.id == id).cloned())
    }

    async fn find_by_idempotency_key(&self, _tenant_id: &str, key: &str) -> Result<Option<Activity>, ActivityError> {
        let activities = self.activities.read().await;
        Ok(activities.iter().find(|a| a.idempotency_key.as_deref() == Some(key)).cloned())
    }

    async fn find_all(&self, _tenant_id: &str, _filter: &ActivityFilter) -> Result<Vec<Activity>, ActivityError> {
        let activities = self.activities.read().await;
        Ok(activities.clone())
    }

    async fn count(&self, _tenant_id: &str, _filter: &ActivityFilter) -> Result<i64, ActivityError> {
        let activities = self.activities.read().await;
        Ok(activities.len() as i64)
    }

    async fn create(&self, _tenant_id: &str, input: &CreateActivity, actor_id: &str) -> Result<Activity, ActivityError> {
        let activity = Activity {
            id: uuid::Uuid::new_v4(),
            task_id: input.task_id,
            actor_id: actor_id.to_string(),
            activity_type: input.activity_type.clone(),
            content: input.content.clone(),
            duration_minutes: input.duration_minutes,
            status: ActivityStatus::Active,
            idempotency_key: input.idempotency_key.clone(),
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        self.activities.write().await.push(activity.clone());
        Ok(activity)
    }

    async fn update_status(
        &self,
        _tenant_id: &str,
        id: uuid::Uuid,
        status: &str,
        updated_by: Option<String>,
    ) -> Result<Activity, ActivityError> {
        let mut activities = self.activities.write().await;
        let activity = activities.iter_mut().find(|a| a.id == id)
            .ok_or_else(|| ActivityError::NotFound(format!("activity not found: {}", id)))?;
        activity.status = status.parse().map_err(|_| {
            ActivityError::ValidationFailed(format!("invalid status: {}", status))
        })?;
        let _ = updated_by; // updated_by は将来のバージョン追跡のために予約
        Ok(activity.clone())
    }
}

/// スタブを使ってテスト用 AppState を構築する（fake Claims ミドルウェア付き）
fn make_test_app() -> axum::Router {
    let repo = Arc::new(StubActivityRepository::new());
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("activity-test"));

    let state = AppState {
        create_activity_uc: Arc::new(
            k1s0_activity_server::usecase::create_activity::CreateActivityUseCase::new(repo.clone()),
        ),
        get_activity_uc: Arc::new(
            k1s0_activity_server::usecase::get_activity::GetActivityUseCase::new(repo.clone()),
        ),
        list_activities_uc: Arc::new(
            k1s0_activity_server::usecase::list_activities::ListActivitiesUseCase::new(repo.clone()),
        ),
        submit_activity_uc: Arc::new(
            k1s0_activity_server::usecase::submit_activity::SubmitActivityUseCase::new(repo.clone()),
        ),
        approve_activity_uc: Arc::new(
            k1s0_activity_server::usecase::approve_activity::ApproveActivityUseCase::new(repo.clone()),
        ),
        reject_activity_uc: Arc::new(
            k1s0_activity_server::usecase::reject_activity::RejectActivityUseCase::new(repo.clone()),
        ),
        metrics,
        auth_state: None,
        // テスト用の遅延接続プール（実際には接続しない。/readyz エンドポイントのテストには使用しない）
        db_pool: sqlx::PgPool::connect_lazy("postgres://localhost/test")
            .expect("テスト用 lazy pool の作成に失敗しました"),
    };
    // fake Claims ミドルウェアを追加して、ハンドラーが Claims を取得できるようにする
    router(state).layer(axum::middleware::from_fn(fake_claims_middleware))
}

/// スタブにアクティビティを1件含んだ状態でテスト用 AppState を構築する（fake Claims ミドルウェア付き）
fn make_test_app_with_activity(activity: Activity) -> axum::Router {
    let repo = Arc::new(StubActivityRepository::with_activities(vec![activity]));
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("activity-test"));

    let state = AppState {
        create_activity_uc: Arc::new(
            k1s0_activity_server::usecase::create_activity::CreateActivityUseCase::new(repo.clone()),
        ),
        get_activity_uc: Arc::new(
            k1s0_activity_server::usecase::get_activity::GetActivityUseCase::new(repo.clone()),
        ),
        list_activities_uc: Arc::new(
            k1s0_activity_server::usecase::list_activities::ListActivitiesUseCase::new(repo.clone()),
        ),
        submit_activity_uc: Arc::new(
            k1s0_activity_server::usecase::submit_activity::SubmitActivityUseCase::new(repo.clone()),
        ),
        approve_activity_uc: Arc::new(
            k1s0_activity_server::usecase::approve_activity::ApproveActivityUseCase::new(repo.clone()),
        ),
        reject_activity_uc: Arc::new(
            k1s0_activity_server::usecase::reject_activity::RejectActivityUseCase::new(repo.clone()),
        ),
        metrics,
        auth_state: None,
        // テスト用の遅延接続プール（実際には接続しない。/readyz エンドポイントのテストには使用しない）
        db_pool: sqlx::PgPool::connect_lazy("postgres://localhost/test")
            .expect("テスト用 lazy pool の作成に失敗しました"),
    };
    // fake Claims ミドルウェアを追加して、ハンドラーが Claims を取得できるようにする
    router(state).layer(axum::middleware::from_fn(fake_claims_middleware))
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

/// アクティビティ一覧取得（GET /api/v1/activities）が空リストを返すことを確認する
#[tokio::test]
async fn test_list_activities_empty() {
    let app = make_test_app();

    let req = Request::builder()
        .uri("/api/v1/activities")
        .body(Body::empty())
        .expect("アクティビティ一覧リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("アクティビティ一覧リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("アクティビティ一覧レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("アクティビティ一覧レスポンスの JSON パースに失敗");
    assert!(json["activities"].as_array().expect("activities フィールドが配列でない").is_empty());
    assert_eq!(json["total"], 0);
}

/// アクティビティ作成（POST /api/v1/activities）が 201 と作成済みアクティビティを返すことを確認する
#[tokio::test]
async fn test_create_activity() {
    let app = make_test_app();

    let task_id = uuid::Uuid::new_v4();
    let body = serde_json::json!({
        "task_id": task_id,
        "activity_type": "comment",
        "content": "テストコメント"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/activities")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("アクティビティ作成ボディの JSON シリアライズに失敗")))
        .expect("アクティビティ作成リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("アクティビティ作成リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("アクティビティ作成レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("アクティビティ作成レスポンスの JSON パースに失敗");
    assert_eq!(json["activity_type"], "comment");
    assert_eq!(json["status"], "active");
    assert!(json["id"].is_string());
}

/// アクティビティ取得（GET /api/v1/activities/{id}）が既存アクティビティで 200 を返すことを確認する
#[tokio::test]
async fn test_get_activity_found() {
    let activity = sample_activity();
    let activity_id = activity.id;
    let app = make_test_app_with_activity(activity);

    let req = Request::builder()
        .uri(format!("/api/v1/activities/{}", activity_id))
        .body(Body::empty())
        .expect("アクティビティ取得リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("アクティビティ取得リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("アクティビティ取得レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("アクティビティ取得レスポンスの JSON パースに失敗");
    assert_eq!(json["id"], activity_id.to_string());
}

/// アクティビティ取得（GET /api/v1/activities/{id}）が存在しない ID で 404 を返すことを確認する
#[tokio::test]
async fn test_get_activity_not_found() {
    let app = make_test_app();

    let req = Request::builder()
        .uri(format!("/api/v1/activities/{}", uuid::Uuid::new_v4()))
        .body(Body::empty())
        .expect("アクティビティ未検出リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("アクティビティ未検出リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// アクティビティ一覧取得がデータあり状態で正しい件数を返すことを確認する
#[tokio::test]
async fn test_list_activities_with_data() {
    let activity = sample_activity();
    let app = make_test_app_with_activity(activity);

    let req = Request::builder()
        .uri("/api/v1/activities")
        .body(Body::empty())
        .expect("アクティビティ一覧（データあり）リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("アクティビティ一覧（データあり）リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("アクティビティ一覧（データあり）レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("アクティビティ一覧（データあり）レスポンスの JSON パースに失敗");
    let activities_arr = json["activities"].as_array().expect("activities フィールドが配列でない");
    assert_eq!(activities_arr.len(), 1);
    assert_eq!(json["total"], 1);
}

/// time_entry アクティビティ作成で duration_minutes が必須であることを確認する
#[tokio::test]
async fn test_create_time_entry_requires_duration() {
    let app = make_test_app();

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "activity_type": "time_entry"
        // duration_minutes が未指定
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/activities")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("time_entry バリデーションテストボディの JSON シリアライズに失敗")))
        .expect("time_entry バリデーションテストリクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("time_entry バリデーションテストリクエストの送信に失敗");
    // duration_minutes が未指定の場合は CREATED にならないことを確認する
    assert_ne!(resp.status(), StatusCode::CREATED);
}

/// アクティビティ作成レスポンスに task_id が含まれることを確認する
#[tokio::test]
async fn test_create_activity_has_task_id() {
    let app = make_test_app();

    let task_id = uuid::Uuid::new_v4();
    let body = serde_json::json!({
        "task_id": task_id,
        "activity_type": "comment",
        "content": "task_id 確認テスト"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/activities")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["task_id"], task_id.to_string());
}

/// アクティビティ取得レスポンスに actor_id フィールドが含まれることを確認する
#[tokio::test]
async fn test_get_activity_has_actor_id() {
    let activity = sample_activity();
    let activity_id = activity.id;
    let app = make_test_app_with_activity(activity);

    let req = Request::builder()
        .uri(format!("/api/v1/activities/{}", activity_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert!(json["actor_id"].is_string());
}

/// アクティビティ一覧の total フィールドが件数と一致することを確認する
#[tokio::test]
async fn test_list_activities_total_count() {
    let activities = vec![sample_activity(), sample_activity()];
    let repo = Arc::new(StubActivityRepository::with_activities(activities));
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("activity-test"));
    let state = AppState {
        create_activity_uc: Arc::new(k1s0_activity_server::usecase::create_activity::CreateActivityUseCase::new(repo.clone())),
        get_activity_uc: Arc::new(k1s0_activity_server::usecase::get_activity::GetActivityUseCase::new(repo.clone())),
        list_activities_uc: Arc::new(k1s0_activity_server::usecase::list_activities::ListActivitiesUseCase::new(repo.clone())),
        submit_activity_uc: Arc::new(k1s0_activity_server::usecase::submit_activity::SubmitActivityUseCase::new(repo.clone())),
        approve_activity_uc: Arc::new(k1s0_activity_server::usecase::approve_activity::ApproveActivityUseCase::new(repo.clone())),
        reject_activity_uc: Arc::new(k1s0_activity_server::usecase::reject_activity::RejectActivityUseCase::new(repo.clone())),
        metrics,
        auth_state: None,
        db_pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").expect("テスト用 lazy pool の作成に失敗しました"),
    };
    // fake Claims ミドルウェアを追加して、ハンドラーが Claims を取得できるようにする
    let app = k1s0_activity_server::adapter::handler::router(state)
        .layer(axum::middleware::from_fn(fake_claims_middleware));

    let req = Request::builder()
        .uri("/api/v1/activities")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let arr = json["activities"].as_array().expect("activities フィールドが配列でない");
    assert_eq!(arr.len(), 2);
    assert_eq!(json["total"], 2);
}

/// アクティビティ作成で idempotency_key を指定できることを確認する
#[tokio::test]
async fn test_create_activity_with_idempotency_key() {
    let app = make_test_app();

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "activity_type": "comment",
        "content": "べき等キー付き",
        "idempotency_key": "unique-key-001"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/activities")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert!(json["id"].is_string());
}

/// アクティビティ作成で task_id が欠如した場合にエラーを返すことを確認する
#[tokio::test]
async fn test_create_activity_missing_task_id() {
    let app = make_test_app();

    let body = serde_json::json!({
        "activity_type": "comment",
        "content": "タスクIDなし"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/activities")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::CREATED);
}

/// アクティビティ作成で activity_type が欠如した場合にエラーを返すことを確認する
#[tokio::test]
async fn test_create_activity_missing_activity_type() {
    let app = make_test_app();

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "content": "タイプなし"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/activities")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::CREATED);
}

/// submit エンドポイント（PUT /api/v1/activities/{id}/submit）が 200 を返すことを確認する
#[tokio::test]
async fn test_submit_activity() {
    let activity = sample_activity();
    let activity_id = activity.id;
    let app = make_test_app_with_activity(activity);

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/activities/{}/submit", activity_id))
        .header("content-type", "application/json")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
}

/// approve エンドポイント（PUT /api/v1/activities/{id}/approve）が 200 を返すことを確認する
/// approve は Submitted → Approved の遷移なので、submitted 状態のアクティビティを使用する
#[tokio::test]
async fn test_approve_activity() {
    // Active → Approved は無効な遷移のため、submitted 状態のアクティビティを作成する
    let mut activity = sample_activity();
    activity.status = ActivityStatus::Submitted;
    let activity_id = activity.id;
    let app = make_test_app_with_activity(activity);

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/activities/{}/approve", activity_id))
        .header("content-type", "application/json")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
}

/// reject エンドポイント（PUT /api/v1/activities/{id}/reject）が 200 を返すことを確認する
/// reject は Submitted → Rejected の遷移なので、submitted 状態のアクティビティを使用する
#[tokio::test]
async fn test_reject_activity() {
    // Active → Rejected は無効な遷移のため、submitted 状態のアクティビティを作成する
    let mut activity = sample_activity();
    activity.status = ActivityStatus::Submitted;
    let activity_id = activity.id;
    let app = make_test_app_with_activity(activity);

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/activities/{}/reject", activity_id))
        .header("content-type", "application/json")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
}

/// submit エンドポイントが存在しない ID に対してエラーを返すことを確認する
#[tokio::test]
async fn test_submit_activity_not_found() {
    let app = make_test_app();

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/activities/{}/submit", uuid::Uuid::new_v4()))
        .header("content-type", "application/json")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::OK);
}

/// approve エンドポイントが存在しない ID に対してエラーを返すことを確認する
#[tokio::test]
async fn test_approve_activity_not_found() {
    let app = make_test_app();

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/activities/{}/approve", uuid::Uuid::new_v4()))
        .header("content-type", "application/json")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::OK);
}

/// reject エンドポイントが存在しない ID に対してエラーを返すことを確認する
#[tokio::test]
async fn test_reject_activity_not_found() {
    let app = make_test_app();

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/activities/{}/reject", uuid::Uuid::new_v4()))
        .header("content-type", "application/json")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::OK);
}

/// アクティビティ取得レスポンスに version フィールドが含まれることを確認する
#[tokio::test]
async fn test_get_activity_has_version() {
    let activity = sample_activity();
    let activity_id = activity.id;
    let app = make_test_app_with_activity(activity);

    let req = Request::builder()
        .uri(format!("/api/v1/activities/{}", activity_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert!(json["version"].is_number());
}

/// アクティビティ取得レスポンスに created_at フィールドが含まれることを確認する
#[tokio::test]
async fn test_get_activity_has_created_at() {
    let activity = sample_activity();
    let activity_id = activity.id;
    let app = make_test_app_with_activity(activity);

    let req = Request::builder()
        .uri(format!("/api/v1/activities/{}", activity_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert!(json["created_at"].is_string());
}

/// アクティビティ作成時の初期 status が active であることを確認する
#[tokio::test]
async fn test_create_activity_initial_status_is_active() {
    let app = make_test_app();

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "activity_type": "comment",
        "content": "ステータス確認"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/activities")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["status"], "active");
}

/// アクティビティ一覧に query parameter を付けても 200 を返すことを確認する
#[tokio::test]
async fn test_list_activities_with_query_params() {
    let activity = sample_activity();
    let task_id = activity.task_id;
    let app = make_test_app_with_activity(activity);

    let req = Request::builder()
        .uri(format!("/api/v1/activities?task_id={}&limit=10&offset=0", task_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
}

/// time_entry タイプで duration_minutes が反映されることを確認する
#[tokio::test]
async fn test_create_time_entry_with_duration() {
    let app = make_test_app();

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "activity_type": "time_entry",
        "duration_minutes": 60
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/activities")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["duration_minutes"], 60);
}

/// アクティビティ一覧レスポンスの各アイテムに activity_type が含まれることを確認する
#[tokio::test]
async fn test_list_activities_items_have_activity_type() {
    let activity = sample_activity();
    let app = make_test_app_with_activity(activity);

    let req = Request::builder()
        .uri("/api/v1/activities")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let arr = json["activities"].as_array().expect("activities フィールドが配列でない");
    assert!(arr[0]["activity_type"].is_string());
}

/// アクティビティ取得レスポンスの id が UUID 文字列であることを確認する
#[tokio::test]
async fn test_get_activity_id_is_uuid_string() {
    let activity = sample_activity();
    let activity_id = activity.id;
    let app = make_test_app_with_activity(activity);

    let req = Request::builder()
        .uri(format!("/api/v1/activities/{}", activity_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let id_str = json["id"].as_str().expect("id フィールドが文字列でない");
    assert!(uuid::Uuid::parse_str(id_str).is_ok());
}

/// アクティビティ一覧レスポンスに activities と total の両フィールドが含まれることを確認する
#[tokio::test]
async fn test_list_activities_response_structure() {
    let app = make_test_app();

    let req = Request::builder()
        .uri("/api/v1/activities")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert!(json["activities"].is_array());
    assert!(json["total"].is_number());
}

/// GET /api/v1/activities/{id} で不正な UUID フォーマットに対してエラーを返すことを確認する
#[tokio::test]
async fn test_get_activity_invalid_uuid() {
    let app = make_test_app();

    let req = Request::builder()
        .uri("/api/v1/activities/not-a-valid-uuid")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::OK);
}

/// アクティビティ一覧レスポンスの各アイテムに status フィールドが含まれることを確認する
#[tokio::test]
async fn test_list_activities_items_have_status() {
    let activity = sample_activity();
    let app = make_test_app_with_activity(activity);

    let req = Request::builder()
        .uri("/api/v1/activities")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let arr = json["activities"].as_array().expect("activities フィールドが配列でない");
    assert!(arr[0]["status"].is_string());
}

/// status_change タイプのアクティビティ作成が 201 を返すことを確認する
#[tokio::test]
async fn test_create_activity_status_change_type() {
    let app = make_test_app();

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "activity_type": "status_change",
        "content": "open -> in_progress"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/activities")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["activity_type"], "status_change");
}

/// アクティビティ作成レスポンスに id フィールドが含まれることを確認する
#[tokio::test]
async fn test_create_activity_returns_id() {
    let app = make_test_app();

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "activity_type": "comment",
        "content": "ID確認テスト"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/activities")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let id_str = json["id"].as_str().expect("id フィールドが文字列でない");
    assert!(uuid::Uuid::parse_str(id_str).is_ok());
}

// --- 実 DB を使ったテスト（db-tests feature が有効な場合のみ実行）---
// 現時点では #[cfg(feature = "db-tests")] で区分けし、CI の db-tests ジョブで有効化する

#[tokio::test]
#[cfg(feature = "db-tests")]
async fn test_activity_crud_with_real_db() {
    // 実 DB を使った CRUD テスト（Phase 4 以降で実装予定）
    // TODO: testcontainers を使って PostgreSQL コンテナを起動し、
    //       リポジトリ実装の CRUD を検証する
}
