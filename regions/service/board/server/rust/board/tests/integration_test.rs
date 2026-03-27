// service tier 統合テスト: スタブリポジトリを使いHTTPレイヤーを検証する。
// auth パターンに倣い AppState をスタブで構築し、認証なしモードで動作確認を行う。
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

/// スタブを使ってテスト用 AppState を構築する
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
    router(state)
}

/// スタブにカラムを1件含んだ状態でテスト用 AppState を構築する
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

// --- 実 DB を使ったテスト（db-tests feature が有効な場合のみ実行）---
// 現時点では #[cfg(feature = "db-tests")] で区分けし、CI の db-tests ジョブで有効化する

#[tokio::test]
#[cfg(feature = "db-tests")]
async fn test_board_crud_with_real_db() {
    // 実 DB を使った CRUD テスト（Phase 4 以降で実装予定）
    // TODO: testcontainers を使って PostgreSQL コンテナを起動し、
    //       リポジトリ実装の CRUD を検証する
}
