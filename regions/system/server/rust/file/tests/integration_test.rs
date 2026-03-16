// router 初期化と基本エンドポイントの smoke test
// file サーバーの REST API ルーターが正しく構築され、
// ヘルスチェックおよび認証ミドルウェアが期待どおり動作することを検証する。

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_file_server::adapter::handler::{router, AppState};
use k1s0_file_server::adapter::middleware::auth::FileAuthState;
use k1s0_file_server::domain::entity::file::FileMetadata;
use k1s0_file_server::domain::repository::{FileMetadataRepository, FileStorageRepository};
use k1s0_file_server::infrastructure::kafka_producer::FileEventPublisher;
use k1s0_file_server::usecase::*;

// ---------------------------------------------------------------------------
// テスト用スタブ: FileMetadataRepository（全メソッドが空の結果を返す）
// ---------------------------------------------------------------------------
struct StubMetadataRepo;

#[async_trait]
impl FileMetadataRepository for StubMetadataRepo {
    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<FileMetadata>> {
        Ok(None)
    }
    async fn find_all(
        &self,
        _tenant_id: Option<String>,
        _uploaded_by: Option<String>,
        _content_type: Option<String>,
        _tag: Option<(String, String)>,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<FileMetadata>, u64)> {
        Ok((vec![], 0))
    }
    async fn create(&self, _file: &FileMetadata) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _file: &FileMetadata) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete(&self, _id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

// ---------------------------------------------------------------------------
// テスト用スタブ: FileStorageRepository（全メソッドがダミー値を返す）
// ---------------------------------------------------------------------------
struct StubStorageRepo;

#[async_trait]
impl FileStorageRepository for StubStorageRepo {
    async fn generate_upload_url(
        &self,
        _storage_key: &str,
        _content_type: &str,
        _expires_in_seconds: u32,
    ) -> anyhow::Result<String> {
        Ok("https://example.com/upload".to_string())
    }
    async fn generate_download_url(
        &self,
        _storage_key: &str,
        _expires_in_seconds: u32,
    ) -> anyhow::Result<String> {
        Ok("https://example.com/download".to_string())
    }
    async fn delete_object(&self, _storage_key: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn get_object_metadata(
        &self,
        _storage_key: &str,
    ) -> anyhow::Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }
}

// ---------------------------------------------------------------------------
// テスト用スタブ: FileEventPublisher（何もしないダミー実装）
// ---------------------------------------------------------------------------
struct StubFilePublisher;

#[async_trait]
impl FileEventPublisher for StubFilePublisher {
    async fn publish(
        &self,
        _event_type: &str,
        _payload: &serde_json::Value,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// テスト用アプリケーション構築ヘルパー（認証なしモード）
// ---------------------------------------------------------------------------
fn make_test_app() -> axum::Router {
    let metadata_repo: Arc<dyn FileMetadataRepository> = Arc::new(StubMetadataRepo);
    let storage_repo: Arc<dyn FileStorageRepository> = Arc::new(StubStorageRepo);
    let event_publisher: Arc<dyn FileEventPublisher> = Arc::new(StubFilePublisher);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("test"));

    // 各ユースケースをスタブリポジトリで構築
    let state = AppState {
        list_files_uc: Arc::new(ListFilesUseCase::new(metadata_repo.clone())),
        generate_upload_url_uc: Arc::new(GenerateUploadUrlUseCase::new(
            metadata_repo.clone(),
            storage_repo.clone(),
        )),
        complete_upload_uc: Arc::new(CompleteUploadUseCase::new(
            metadata_repo.clone(),
            event_publisher.clone(),
        )),
        get_file_metadata_uc: Arc::new(GetFileMetadataUseCase::new(metadata_repo.clone())),
        generate_download_url_uc: Arc::new(GenerateDownloadUrlUseCase::new(
            metadata_repo.clone(),
            storage_repo.clone(),
        )),
        delete_file_uc: Arc::new(DeleteFileUseCase::new(
            metadata_repo.clone(),
            storage_repo.clone(),
            event_publisher.clone(),
        )),
        update_file_tags_uc: Arc::new(UpdateFileTagsUseCase::new(metadata_repo.clone())),
        metrics,
        auth_state: None,
    };

    router(state)
}

// ---------------------------------------------------------------------------
// テスト: /healthz と /readyz が 200 を返すことを確認
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app();

    // /healthz への GET リクエスト
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "/healthz は 200 を返すべき");

    // /readyz への GET リクエスト
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "/readyz は 200 を返すべき");
}

// ---------------------------------------------------------------------------
// テスト: 認証有効時に token なしで保護エンドポイントにアクセスすると 401 を返す
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_unauthorized_without_token() {
    let metadata_repo: Arc<dyn FileMetadataRepository> = Arc::new(StubMetadataRepo);
    let storage_repo: Arc<dyn FileStorageRepository> = Arc::new(StubStorageRepo);
    let event_publisher: Arc<dyn FileEventPublisher> = Arc::new(StubFilePublisher);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("test"));

    // 認証ありの AppState を構築（不正な JWKS URL でダミー verifier を生成）
    let verifier = Arc::new(k1s0_auth::JwksVerifier::new(
        "https://invalid.example.com/.well-known/jwks.json",
        "https://invalid.example.com",
        "test-audience",
        std::time::Duration::from_secs(60),
    ));
    let auth_state = FileAuthState { verifier };

    let state = AppState {
        list_files_uc: Arc::new(ListFilesUseCase::new(metadata_repo.clone())),
        generate_upload_url_uc: Arc::new(GenerateUploadUrlUseCase::new(
            metadata_repo.clone(),
            storage_repo.clone(),
        )),
        complete_upload_uc: Arc::new(CompleteUploadUseCase::new(
            metadata_repo.clone(),
            event_publisher.clone(),
        )),
        get_file_metadata_uc: Arc::new(GetFileMetadataUseCase::new(metadata_repo.clone())),
        generate_download_url_uc: Arc::new(GenerateDownloadUrlUseCase::new(
            metadata_repo.clone(),
            storage_repo.clone(),
        )),
        delete_file_uc: Arc::new(DeleteFileUseCase::new(
            metadata_repo.clone(),
            storage_repo.clone(),
            event_publisher.clone(),
        )),
        update_file_tags_uc: Arc::new(UpdateFileTagsUseCase::new(metadata_repo.clone())),
        metrics,
        auth_state: Some(auth_state),
    };

    let app = router(state);

    // 保護されたエンドポイントに Authorization ヘッダーなしでアクセス
    let req = Request::builder()
        .uri("/api/v1/files")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "token なしで保護エンドポイントは 401 を返すべき"
    );
}
