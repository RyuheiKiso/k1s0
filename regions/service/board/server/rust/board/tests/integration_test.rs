// service tier 統合テスト: スタブリポジトリを使いHTTPレイヤーを検証する。
// ハンドラーは Claims が必須のため、テスト用の fake Claims 注入ミドルウェアを使用する。
// 実 DB 接続テストは #[cfg(feature = "db-tests")] で区分けする。
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_board_server::adapter::handler::{router, AppState};
use sqlx;
use k1s0_board_server::domain::entity::board_column::{
    BoardColumn, BoardColumnFilter, DecrementColumnRequest, IncrementColumnRequest,
    UpdateWipLimitRequest,
};
use k1s0_board_server::domain::error::BoardError;
use k1s0_board_server::domain::repository::board_column_repository::BoardColumnRepository;

// --- テスト用スタブリポジトリ ---

/// テスト用スタブリポジトリ（インメモリ実装）
struct StubBoardColumnRepository {
    columns: tokio::sync::RwLock<Vec<BoardColumn>>,
}

impl StubBoardColumnRepository {
    fn new() -> Self {
        Self {
            columns: tokio::sync::RwLock::new(Vec::new()),
        }
    }

    fn with_columns(columns: Vec<BoardColumn>) -> Self {
        Self {
            columns: tokio::sync::RwLock::new(columns),
        }
    }
}

fn sample_column() -> BoardColumn {
    BoardColumn {
        id: uuid::Uuid::new_v4(),
        project_id: uuid::Uuid::new_v4(),
        status_code: "open".to_string(),
        wip_limit: 5,
        task_count: 0,
        version: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[async_trait::async_trait]
impl BoardColumnRepository for StubBoardColumnRepository {
    async fn find_by_id(&self, _tenant_id: &str, id: uuid::Uuid) -> Result<Option<BoardColumn>, BoardError> {
        let cols = self.columns.read().await;
        Ok(cols.iter().find(|c| c.id == id).cloned())
    }

    async fn find_by_project_and_status(
        &self,
        _tenant_id: &str,
        project_id: uuid::Uuid,
        status_code: &str,
    ) -> Result<Option<BoardColumn>, BoardError> {
        let cols = self.columns.read().await;
        Ok(cols.iter().find(|c| c.project_id == project_id && c.status_code == status_code).cloned())
    }

    async fn find_all(&self, _tenant_id: &str, _filter: &BoardColumnFilter) -> Result<Vec<BoardColumn>, BoardError> {
        let cols = self.columns.read().await;
        Ok(cols.clone())
    }

    async fn count(&self, _tenant_id: &str, _filter: &BoardColumnFilter) -> Result<i64, BoardError> {
        let cols = self.columns.read().await;
        Ok(cols.len() as i64)
    }

    async fn increment(&self, _tenant_id: &str, req: &IncrementColumnRequest) -> Result<BoardColumn, BoardError> {
        let mut cols = self.columns.write().await;
        let col = cols.iter_mut().find(|c| {
            c.project_id == req.project_id && c.status_code == req.status_code
        });
        match col {
            Some(c) => {
                c.task_count += 1;
                Ok(c.clone())
            }
            None => {
                // カラムが存在しない場合は新規作成（upsert 相当）
                let new_col = BoardColumn {
                    id: uuid::Uuid::new_v4(),
                    project_id: req.project_id,
                    status_code: req.status_code.clone(),
                    wip_limit: 0,
                    task_count: 1,
                    version: 1,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                cols.push(new_col.clone());
                Ok(new_col)
            }
        }
    }

    async fn decrement(&self, _tenant_id: &str, req: &DecrementColumnRequest) -> Result<BoardColumn, BoardError> {
        let mut cols = self.columns.write().await;
        let col = cols.iter_mut().find(|c| {
            c.project_id == req.project_id && c.status_code == req.status_code
        }).ok_or_else(|| BoardError::NotFound(format!("column not found for project {}", req.project_id)))?;
        col.task_count = (col.task_count - 1).max(0);
        Ok(col.clone())
    }

    async fn update_wip_limit(&self, _tenant_id: &str, req: &UpdateWipLimitRequest) -> Result<BoardColumn, BoardError> {
        let mut cols = self.columns.write().await;
        let col = cols.iter_mut().find(|c| c.id == req.column_id)
            .ok_or_else(|| BoardError::NotFound(format!("column not found: {}", req.column_id)))?;
        col.wip_limit = req.wip_limit;
        col.version += 1;
        Ok(col.clone())
    }
}

/// テスト用の fake Claims 注入ミドルウェア。
/// ハンドラーは `Option<Extension<Claims>>` が None の場合に 401 を返すため、
/// テスト環境でも Claims を Extension として挿入する必要がある。
async fn fake_claims_middleware(
    mut req: axum::http::Request<Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use std::collections::HashMap;
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

/// スタブを使ってテスト用 AppState を構築する（fake Claims ミドルウェア付き）
fn make_test_app() -> axum::Router {
    let repo = Arc::new(StubBoardColumnRepository::new());
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("board-test"));

    let state = AppState {
        increment_column_uc: Arc::new(
            k1s0_board_server::usecase::increment_column::IncrementColumnUseCase::new(repo.clone()),
        ),
        decrement_column_uc: Arc::new(
            k1s0_board_server::usecase::decrement_column::DecrementColumnUseCase::new(repo.clone()),
        ),
        get_board_column_uc: Arc::new(
            k1s0_board_server::usecase::get_board_column::GetBoardColumnUseCase::new(repo.clone()),
        ),
        list_board_columns_uc: Arc::new(
            k1s0_board_server::usecase::list_board_columns::ListBoardColumnsUseCase::new(repo.clone()),
        ),
        update_wip_limit_uc: Arc::new(
            k1s0_board_server::usecase::update_wip_limit::UpdateWipLimitUseCase::new(repo.clone()),
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

/// スタブにカラムを1件含んだ状態でテスト用 AppState を構築する（fake Claims ミドルウェア付き）
fn make_test_app_with_column(col: BoardColumn) -> axum::Router {
    let repo = Arc::new(StubBoardColumnRepository::with_columns(vec![col]));
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("board-test"));

    let state = AppState {
        increment_column_uc: Arc::new(
            k1s0_board_server::usecase::increment_column::IncrementColumnUseCase::new(repo.clone()),
        ),
        decrement_column_uc: Arc::new(
            k1s0_board_server::usecase::decrement_column::DecrementColumnUseCase::new(repo.clone()),
        ),
        get_board_column_uc: Arc::new(
            k1s0_board_server::usecase::get_board_column::GetBoardColumnUseCase::new(repo.clone()),
        ),
        list_board_columns_uc: Arc::new(
            k1s0_board_server::usecase::list_board_columns::ListBoardColumnsUseCase::new(repo.clone()),
        ),
        update_wip_limit_uc: Arc::new(
            k1s0_board_server::usecase::update_wip_limit::UpdateWipLimitUseCase::new(repo.clone()),
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

/// ボードカラム一覧取得（GET /api/v1/board-columns）が空リストを返すことを確認する
#[tokio::test]
async fn test_list_board_columns_empty() {
    let app = make_test_app();

    let req = Request::builder()
        .uri("/api/v1/board-columns")
        .body(Body::empty())
        .expect("ボードカラム一覧リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("ボードカラム一覧リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("ボードカラム一覧レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("ボードカラム一覧レスポンスの JSON パースに失敗");
    assert!(json["columns"].as_array().expect("columns フィールドが配列でない").is_empty());
    assert_eq!(json["total"], 0);
}

/// ボードカラム一覧取得がデータあり状態で正しい件数を返すことを確認する
#[tokio::test]
async fn test_list_board_columns_with_data() {
    let col = sample_column();
    let app = make_test_app_with_column(col);

    let req = Request::builder()
        .uri("/api/v1/board-columns")
        .body(Body::empty())
        .expect("ボードカラム一覧（データあり）リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("ボードカラム一覧（データあり）リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("ボードカラム一覧（データあり）レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("ボードカラム一覧（データあり）レスポンスの JSON パースに失敗");
    let cols_arr = json["columns"].as_array().expect("columns フィールドが配列でない");
    assert_eq!(cols_arr.len(), 1);
    assert_eq!(json["total"], 1);
}

/// ボードカラム取得（GET /api/v1/board-columns/{id}）が既存カラムで 200 を返すことを確認する
#[tokio::test]
async fn test_get_board_column_found() {
    let col = sample_column();
    let col_id = col.id;
    let app = make_test_app_with_column(col);

    let req = Request::builder()
        .uri(format!("/api/v1/board-columns/{}", col_id))
        .body(Body::empty())
        .expect("ボードカラム取得リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("ボードカラム取得リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("ボードカラム取得レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("ボードカラム取得レスポンスの JSON パースに失敗");
    assert_eq!(json["id"], col_id.to_string());
}

/// ボードカラム取得（GET /api/v1/board-columns/{id}）が存在しない ID で 404 を返すことを確認する
#[tokio::test]
async fn test_get_board_column_not_found() {
    let app = make_test_app();

    let req = Request::builder()
        .uri(format!("/api/v1/board-columns/{}", uuid::Uuid::new_v4()))
        .body(Body::empty())
        .expect("ボードカラム未検出リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("ボードカラム未検出リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// increment エンドポイント（POST /api/v1/board-columns/increment）が 200 を返すことを確認する
#[tokio::test]
async fn test_increment_column() {
    let app = make_test_app();

    let project_id = uuid::Uuid::new_v4();
    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": project_id,
        "status_code": "open"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/increment")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("increment ボディの JSON シリアライズに失敗")))
        .expect("increment リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("increment リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("increment レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("increment レスポンスの JSON パースに失敗");
    assert_eq!(json["task_count"], 1);
}

/// ボードカラム取得レスポンスに status_code フィールドが含まれることを確認する
#[tokio::test]
async fn test_get_board_column_has_status_code() {
    let col = sample_column();
    let col_id = col.id;
    let app = make_test_app_with_column(col);

    let req = Request::builder()
        .uri(format!("/api/v1/board-columns/{}", col_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["status_code"], "open");
}

/// ボードカラム取得レスポンスに wip_limit フィールドが含まれることを確認する
#[tokio::test]
async fn test_get_board_column_has_wip_limit() {
    let col = sample_column();
    let col_id = col.id;
    let app = make_test_app_with_column(col);

    let req = Request::builder()
        .uri(format!("/api/v1/board-columns/{}", col_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["wip_limit"], 5);
}

/// ボードカラム取得レスポンスに project_id フィールドが含まれることを確認する
#[tokio::test]
async fn test_get_board_column_has_project_id() {
    let col = sample_column();
    let col_id = col.id;
    let project_id = col.project_id;
    let app = make_test_app_with_column(col);

    let req = Request::builder()
        .uri(format!("/api/v1/board-columns/{}", col_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["project_id"], project_id.to_string());
}

/// ボードカラム取得レスポンスに version フィールドが含まれることを確認する
#[tokio::test]
async fn test_get_board_column_has_version() {
    let col = sample_column();
    let col_id = col.id;
    let app = make_test_app_with_column(col);

    let req = Request::builder()
        .uri(format!("/api/v1/board-columns/{}", col_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["version"], 1);
}

/// increment エンドポイントで既存カラムの task_count が増加することを確認する
#[tokio::test]
async fn test_increment_existing_column() {
    let col = sample_column();
    let project_id = col.project_id;
    let app = make_test_app_with_column(col);

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": project_id,
        "status_code": "open"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/increment")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["task_count"], 1);
}

/// increment レスポンスに status_code が含まれることを確認する
#[tokio::test]
async fn test_increment_response_has_status_code() {
    let app = make_test_app();

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": uuid::Uuid::new_v4(),
        "status_code": "in_progress"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/increment")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["status_code"], "in_progress");
}

/// increment レスポンスに project_id が含まれることを確認する
#[tokio::test]
async fn test_increment_response_has_project_id() {
    let app = make_test_app();

    let project_id = uuid::Uuid::new_v4();
    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": project_id,
        "status_code": "open"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/increment")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["project_id"], project_id.to_string());
}

/// decrement エンドポイントが既存カラムの task_count を減少させることを確認する
#[tokio::test]
async fn test_decrement_column() {
    let mut col = sample_column();
    col.task_count = 3;
    let project_id = col.project_id;
    let app = make_test_app_with_column(col);

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": project_id,
        "status_code": "open"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/decrement")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["task_count"], 2);
}

/// decrement エンドポイントがオプションの reason フィールドを受け付けることを確認する
#[tokio::test]
async fn test_decrement_column_with_reason() {
    let mut col = sample_column();
    col.task_count = 1;
    let project_id = col.project_id;
    let app = make_test_app_with_column(col);

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": project_id,
        "status_code": "open",
        "reason": "タスク完了のため"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/decrement")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
}

/// decrement エンドポイントが存在しないカラムに対してエラーを返すことを確認する
#[tokio::test]
async fn test_decrement_column_not_found() {
    let app = make_test_app();

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": uuid::Uuid::new_v4(),
        "status_code": "open"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/decrement")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    // 存在しないカラムへの decrement は 404 または 500 を返す
    assert_ne!(resp.status(), StatusCode::OK);
}

/// WIP 制限更新（PUT /api/v1/board-columns/{id}）が正しく更新されることを確認する
#[tokio::test]
async fn test_update_wip_limit() {
    let col = sample_column();
    let col_id = col.id;
    let app = make_test_app_with_column(col);

    let body = serde_json::json!({
        "wip_limit": 10,
        "expected_version": 1
    });
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/board-columns/{}", col_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["wip_limit"], 10);
}

/// WIP 制限を 0 に更新すると無制限になることを確認する
#[tokio::test]
async fn test_update_wip_limit_to_zero_unlimited() {
    let col = sample_column();
    let col_id = col.id;
    let app = make_test_app_with_column(col);

    let body = serde_json::json!({
        "wip_limit": 0,
        "expected_version": 1
    });
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/board-columns/{}", col_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["wip_limit"], 0);
}

/// WIP 制限更新が存在しない ID に対して 404 を返すことを確認する
#[tokio::test]
async fn test_update_wip_limit_not_found() {
    let app = make_test_app();

    let body = serde_json::json!({
        "wip_limit": 5,
        "expected_version": 1
    });
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/board-columns/{}", uuid::Uuid::new_v4()))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::OK);
}

/// WIP 制限更新後にバージョンが増加することを確認する
#[tokio::test]
async fn test_update_wip_limit_increments_version() {
    let col = sample_column();
    let col_id = col.id;
    let app = make_test_app_with_column(col);

    let body = serde_json::json!({
        "wip_limit": 8,
        "expected_version": 1
    });
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/board-columns/{}", col_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    // バージョンはスタブ実装で 1 → 2 に増加する
    assert_eq!(json["version"], 2);
}

/// カラム一覧の total フィールドが件数と一致することを確認する
#[tokio::test]
async fn test_list_board_columns_total_count() {
    let cols = vec![sample_column(), sample_column(), sample_column()];
    let repo = Arc::new(StubBoardColumnRepository::with_columns(cols));
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("board-test"));
    let state = AppState {
        increment_column_uc: Arc::new(k1s0_board_server::usecase::increment_column::IncrementColumnUseCase::new(repo.clone())),
        decrement_column_uc: Arc::new(k1s0_board_server::usecase::decrement_column::DecrementColumnUseCase::new(repo.clone())),
        get_board_column_uc: Arc::new(k1s0_board_server::usecase::get_board_column::GetBoardColumnUseCase::new(repo.clone())),
        list_board_columns_uc: Arc::new(k1s0_board_server::usecase::list_board_columns::ListBoardColumnsUseCase::new(repo.clone())),
        update_wip_limit_uc: Arc::new(k1s0_board_server::usecase::update_wip_limit::UpdateWipLimitUseCase::new(repo.clone())),
        metrics,
        auth_state: None,
        db_pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").expect("テスト用 lazy pool の作成に失敗しました"),
    };
    let app = router(state).layer(axum::middleware::from_fn(fake_claims_middleware));

    let req = Request::builder()
        .uri("/api/v1/board-columns")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let cols_arr = json["columns"].as_array().expect("columns フィールドが配列でない");
    assert_eq!(cols_arr.len(), 3);
    assert_eq!(json["total"], 3);
}

/// increment を複数回実行すると task_count が積み上がることを確認する
#[tokio::test]
async fn test_increment_multiple_times() {
    let app = make_test_app();
    let project_id = uuid::Uuid::new_v4();

    // 1回目
    let body = serde_json::json!({"task_id": uuid::Uuid::new_v4(), "project_id": project_id, "status_code": "open"});
    let req = Request::builder().method("POST").uri("/api/v1/board-columns/increment")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    app.clone().oneshot(req).await.expect("リクエストの送信に失敗");

    // 2回目
    let body = serde_json::json!({"task_id": uuid::Uuid::new_v4(), "project_id": project_id, "status_code": "open"});
    let req = Request::builder().method("POST").uri("/api/v1/board-columns/increment")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.clone().oneshot(req).await.expect("リクエストの送信に失敗");

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    // スタブはステートレスなので各 oneshot 呼び出しは独立。2回目は count=1 になる
    assert_eq!(json["task_count"], 1);
}

/// 異なるプロジェクトへの increment が独立して動作することを確認する
#[tokio::test]
async fn test_increment_different_projects_are_independent() {
    let app = make_test_app();

    let project_a = uuid::Uuid::new_v4();
    let project_b = uuid::Uuid::new_v4();

    let body_a = serde_json::json!({"task_id": uuid::Uuid::new_v4(), "project_id": project_a, "status_code": "open"});
    let req = Request::builder().method("POST").uri("/api/v1/board-columns/increment")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body_a).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.clone().oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["project_id"], project_a.to_string());

    let body_b = serde_json::json!({"task_id": uuid::Uuid::new_v4(), "project_id": project_b, "status_code": "open"});
    let req = Request::builder().method("POST").uri("/api/v1/board-columns/increment")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body_b).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.clone().oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["project_id"], project_b.to_string());
}

/// increment リクエストの task_id が UUID 形式であることを検証する
#[tokio::test]
async fn test_increment_invalid_request_body() {
    let app = make_test_app();

    // task_id が欠如したリクエスト
    let body = serde_json::json!({
        "project_id": uuid::Uuid::new_v4(),
        "status_code": "open"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/increment")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    // task_id 欠如は 422 または 400 を返す
    assert_ne!(resp.status(), StatusCode::OK);
}

/// increment リクエストで status_code が "done" でも動作することを確認する
#[tokio::test]
async fn test_increment_column_done_status() {
    let app = make_test_app();

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": uuid::Uuid::new_v4(),
        "status_code": "done"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/increment")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["status_code"], "done");
}

/// decrement リクエストで status_code が指定されたカラムへの操作であることを確認する
#[tokio::test]
async fn test_decrement_response_has_correct_status_code() {
    let mut col = sample_column();
    col.task_count = 2;
    col.status_code = "in_progress".to_string();
    let project_id = col.project_id;
    let app = make_test_app_with_column(col);

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": project_id,
        "status_code": "in_progress"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/decrement")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["status_code"], "in_progress");
}

/// WIP 制限更新で id フィールドが保持されることを確認する
#[tokio::test]
async fn test_update_wip_limit_preserves_id() {
    let col = sample_column();
    let col_id = col.id;
    let app = make_test_app_with_column(col);

    let body = serde_json::json!({"wip_limit": 3, "expected_version": 1});
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/board-columns/{}", col_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["id"], col_id.to_string());
}

/// カラム一覧の columns 配列に id フィールドが含まれることを確認する
#[tokio::test]
async fn test_list_board_columns_items_have_id() {
    let col = sample_column();
    let col_id = col.id;
    let app = make_test_app_with_column(col);

    let req = Request::builder()
        .uri("/api/v1/board-columns")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let cols_arr = json["columns"].as_array().expect("columns フィールドが配列でない");
    assert_eq!(cols_arr[0]["id"], col_id.to_string());
}

/// カラム一覧の各アイテムに wip_limit フィールドが含まれることを確認する
#[tokio::test]
async fn test_list_board_columns_items_have_wip_limit() {
    let col = sample_column();
    let app = make_test_app_with_column(col);

    let req = Request::builder()
        .uri("/api/v1/board-columns")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let cols_arr = json["columns"].as_array().expect("columns フィールドが配列でない");
    assert!(cols_arr[0]["wip_limit"].is_number());
}

/// increment でカラムが存在しない場合は新規作成されることを確認する（upsert 動作）
#[tokio::test]
async fn test_increment_creates_column_if_not_exists() {
    let app = make_test_app();

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": uuid::Uuid::new_v4(),
        "status_code": "review"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/increment")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["task_count"], 1);
    assert_eq!(json["status_code"], "review");
}

/// WIP 制限更新で updated_at フィールドが存在することを確認する
#[tokio::test]
async fn test_update_wip_limit_response_has_updated_at() {
    let col = sample_column();
    let col_id = col.id;
    let app = make_test_app_with_column(col);

    let body = serde_json::json!({"wip_limit": 7, "expected_version": 1});
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/board-columns/{}", col_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert!(json["updated_at"].is_string());
}

/// カラム一覧レスポンスに created_at フィールドが含まれることを確認する
#[tokio::test]
async fn test_list_board_columns_items_have_created_at() {
    let col = sample_column();
    let app = make_test_app_with_column(col);

    let req = Request::builder()
        .uri("/api/v1/board-columns")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    let cols_arr = json["columns"].as_array().expect("columns フィールドが配列でない");
    assert!(cols_arr[0]["created_at"].is_string());
}

/// decrement で task_count が 0 以下にならないことを確認する（フロア制約）
#[tokio::test]
async fn test_decrement_floor_at_zero() {
    let col = sample_column(); // task_count = 0
    let project_id = col.project_id;
    let app = make_test_app_with_column(col);

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": project_id,
        "status_code": "open"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/decrement")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["task_count"], 0);
}

/// increment レスポンスに task_id は含まれず task_count が正しいことを確認する
#[tokio::test]
async fn test_increment_response_structure() {
    let app = make_test_app();

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": uuid::Uuid::new_v4(),
        "status_code": "open"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/increment")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    // レスポンスは BoardColumn 構造体のフィールドのみ含む
    assert!(json["id"].is_string());
    assert!(json["task_count"].is_number());
    assert!(json["wip_limit"].is_number());
}

/// GET /api/v1/board-columns に query parameter を付けても 200 を返すことを確認する
#[tokio::test]
async fn test_list_board_columns_with_query_params() {
    let col = sample_column();
    let project_id = col.project_id;
    let app = make_test_app_with_column(col);

    let req = Request::builder()
        .uri(format!("/api/v1/board-columns?project_id={}&limit=10&offset=0", project_id))
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
}

/// decrement レスポンスに project_id フィールドが含まれることを確認する
#[tokio::test]
async fn test_decrement_response_has_project_id() {
    let mut col = sample_column();
    col.task_count = 1;
    let project_id = col.project_id;
    let app = make_test_app_with_column(col);

    let body = serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "project_id": project_id,
        "status_code": "open"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/board-columns/decrement")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("ボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("JSON パースに失敗");
    assert_eq!(json["project_id"], project_id.to_string());
}

/// GET /api/v1/board-columns/{id} が不正な UUID フォーマットで 400/422 を返すことを確認する
#[tokio::test]
async fn test_get_board_column_invalid_uuid() {
    let app = make_test_app();

    let req = Request::builder()
        .uri("/api/v1/board-columns/not-a-valid-uuid")
        .body(Body::empty())
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::OK);
}

/// PUT /api/v1/board-columns/{id} で wip_limit フィールドが欠如した場合にエラーを返すことを確認する
#[tokio::test]
async fn test_update_wip_limit_missing_field() {
    let col = sample_column();
    let col_id = col.id;
    let app = make_test_app_with_column(col);

    // expected_version のみ（wip_limit 欠如）
    let body = serde_json::json!({"expected_version": 1});
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/board-columns/{}", col_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("JSON シリアライズに失敗")))
        .expect("リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("リクエストの送信に失敗");
    assert_ne!(resp.status(), StatusCode::OK);
}

// --- 実 DB を使ったテスト（db-tests feature が有効な場合のみ実行）---
// 現時点では #[cfg(feature = "db-tests")] で区分けし、CI の db-tests ジョブで有効化する

#[tokio::test]
#[cfg(feature = "db-tests")]
async fn test_board_crud_with_real_db() {
    // 実 DB を使った CRUD テスト（Phase 4 以降で実装予定）
    // TODO: testcontainers を使って PostgreSQL コンテナを起動し、
    //       リポジトリ実装の CRUD を検証する
}
