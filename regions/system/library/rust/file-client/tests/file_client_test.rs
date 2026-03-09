use std::time::Duration;

use k1s0_file_client::{
    FileClient, FileClientConfig, FileClientError, InMemoryFileClient,
};

fn test_config() -> FileClientConfig {
    FileClientConfig::server_mode("http://file-server:8080")
}

fn make_client() -> InMemoryFileClient {
    InMemoryFileClient::new(test_config())
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[test]
fn config_server_mode_defaults() {
    let cfg = FileClientConfig::server_mode("http://localhost:9000");
    assert_eq!(cfg.server_url, Some("http://localhost:9000".to_string()));
    assert!(cfg.s3_endpoint.is_none());
    assert!(cfg.bucket.is_none());
    assert!(cfg.region.is_none());
    assert_eq!(cfg.timeout, Duration::from_secs(30));
}

#[test]
fn config_direct_mode_fields() {
    let cfg = FileClientConfig::direct_mode("http://minio:9000", "my-bucket", "us-east-1");
    assert!(cfg.server_url.is_none());
    assert_eq!(cfg.s3_endpoint, Some("http://minio:9000".to_string()));
    assert_eq!(cfg.bucket, Some("my-bucket".to_string()));
    assert_eq!(cfg.region, Some("us-east-1".to_string()));
}

#[test]
fn config_with_timeout_overrides_default() {
    let cfg = FileClientConfig::server_mode("http://localhost:8080")
        .with_timeout(Duration::from_secs(120));
    assert_eq!(cfg.timeout, Duration::from_secs(120));
}

// ---------------------------------------------------------------------------
// Upload URL generation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn generate_upload_url_returns_put_method() {
    let client = make_client();
    let url = client
        .generate_upload_url("docs/readme.md", "text/markdown", Duration::from_secs(600))
        .await
        .unwrap();
    assert_eq!(url.method, "PUT");
}

#[tokio::test]
async fn generate_upload_url_contains_path() {
    let client = make_client();
    let url = client
        .generate_upload_url("images/photo.jpg", "image/jpeg", Duration::from_secs(60))
        .await
        .unwrap();
    assert!(url.url.contains("images/photo.jpg"));
}

#[tokio::test]
async fn generate_upload_url_sets_expiration_in_future() {
    let before = chrono::Utc::now();
    let client = make_client();
    let url = client
        .generate_upload_url("a.txt", "text/plain", Duration::from_secs(3600))
        .await
        .unwrap();
    assert!(url.expires_at > before);
}

#[tokio::test]
async fn generate_upload_url_registers_file_in_store() {
    let client = make_client();
    client
        .generate_upload_url("data/file.csv", "text/csv", Duration::from_secs(60))
        .await
        .unwrap();

    let meta = client.get_metadata("data/file.csv").await.unwrap();
    assert_eq!(meta.path, "data/file.csv");
    assert_eq!(meta.content_type, "text/csv");
}

// ---------------------------------------------------------------------------
// Download URL generation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn generate_download_url_for_existing_file() {
    let client = make_client();
    client
        .generate_upload_url("report.pdf", "application/pdf", Duration::from_secs(60))
        .await
        .unwrap();

    let url = client
        .generate_download_url("report.pdf", Duration::from_secs(300))
        .await
        .unwrap();
    assert_eq!(url.method, "GET");
    assert!(url.url.contains("report.pdf"));
}

#[tokio::test]
async fn generate_download_url_not_found_for_missing_file() {
    let client = make_client();
    let result = client
        .generate_download_url("does-not-exist.bin", Duration::from_secs(60))
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, FileClientError::NotFound(_)));
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_existing_file_succeeds() {
    let client = make_client();
    client
        .generate_upload_url("tmp/to-delete.txt", "text/plain", Duration::from_secs(60))
        .await
        .unwrap();

    client.delete("tmp/to-delete.txt").await.unwrap();

    // Verify it's gone
    let result = client.get_metadata("tmp/to-delete.txt").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn delete_nonexistent_file_returns_not_found() {
    let client = make_client();
    let result = client.delete("nonexistent.txt").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FileClientError::NotFound(_)));
}

#[tokio::test]
async fn delete_then_upload_same_path() {
    let client = make_client();
    client
        .generate_upload_url("cycle.txt", "text/plain", Duration::from_secs(60))
        .await
        .unwrap();
    client.delete("cycle.txt").await.unwrap();
    client
        .generate_upload_url("cycle.txt", "application/json", Duration::from_secs(60))
        .await
        .unwrap();

    let meta = client.get_metadata("cycle.txt").await.unwrap();
    assert_eq!(meta.content_type, "application/json");
}

// ---------------------------------------------------------------------------
// Get Metadata
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_metadata_returns_correct_content_type() {
    let client = make_client();
    client
        .generate_upload_url("img/banner.png", "image/png", Duration::from_secs(60))
        .await
        .unwrap();

    let meta = client.get_metadata("img/banner.png").await.unwrap();
    assert_eq!(meta.content_type, "image/png");
    assert_eq!(meta.path, "img/banner.png");
}

#[tokio::test]
async fn get_metadata_not_found() {
    let client = make_client();
    let result = client.get_metadata("no-such-file").await;
    assert!(matches!(result.unwrap_err(), FileClientError::NotFound(_)));
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_filters_by_prefix() {
    let client = make_client();
    let dur = Duration::from_secs(60);
    client
        .generate_upload_url("prefix/a.txt", "text/plain", dur)
        .await
        .unwrap();
    client
        .generate_upload_url("prefix/b.txt", "text/plain", dur)
        .await
        .unwrap();
    client
        .generate_upload_url("other/c.txt", "text/plain", dur)
        .await
        .unwrap();

    let files = client.list("prefix/").await.unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.iter().all(|f| f.path.starts_with("prefix/")));
}

#[tokio::test]
async fn list_empty_prefix_returns_all() {
    let client = make_client();
    let dur = Duration::from_secs(60);
    client
        .generate_upload_url("x.txt", "text/plain", dur)
        .await
        .unwrap();
    client
        .generate_upload_url("y.txt", "text/plain", dur)
        .await
        .unwrap();

    let files = client.list("").await.unwrap();
    assert_eq!(files.len(), 2);
}

#[tokio::test]
async fn list_no_matches_returns_empty() {
    let client = make_client();
    let files = client.list("nonexistent/").await.unwrap();
    assert!(files.is_empty());
}

// ---------------------------------------------------------------------------
// Copy
// ---------------------------------------------------------------------------

#[tokio::test]
async fn copy_creates_file_at_destination() {
    let client = make_client();
    client
        .generate_upload_url("src/original.txt", "text/plain", Duration::from_secs(60))
        .await
        .unwrap();

    client
        .copy("src/original.txt", "dst/copied.txt")
        .await
        .unwrap();

    let meta = client.get_metadata("dst/copied.txt").await.unwrap();
    assert_eq!(meta.path, "dst/copied.txt");
    assert_eq!(meta.content_type, "text/plain");
}

#[tokio::test]
async fn copy_preserves_content_type() {
    let client = make_client();
    client
        .generate_upload_url("media/video.mp4", "video/mp4", Duration::from_secs(60))
        .await
        .unwrap();

    client
        .copy("media/video.mp4", "archive/video.mp4")
        .await
        .unwrap();

    let src_meta = client.get_metadata("media/video.mp4").await.unwrap();
    let dst_meta = client.get_metadata("archive/video.mp4").await.unwrap();
    assert_eq!(src_meta.content_type, dst_meta.content_type);
}

#[tokio::test]
async fn copy_source_not_found() {
    let client = make_client();
    let result = client.copy("ghost.txt", "target.txt").await;
    assert!(matches!(result.unwrap_err(), FileClientError::NotFound(_)));
}

#[tokio::test]
async fn copy_does_not_remove_source() {
    let client = make_client();
    client
        .generate_upload_url("keep-me.txt", "text/plain", Duration::from_secs(60))
        .await
        .unwrap();

    client.copy("keep-me.txt", "other.txt").await.unwrap();

    // Source should still exist
    let result = client.get_metadata("keep-me.txt").await;
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// stored_files helper
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stored_files_reflects_current_state() {
    let client = make_client();
    assert!(client.stored_files().await.is_empty());

    client
        .generate_upload_url("one.txt", "text/plain", Duration::from_secs(60))
        .await
        .unwrap();
    assert_eq!(client.stored_files().await.len(), 1);

    client.delete("one.txt").await.unwrap();
    assert!(client.stored_files().await.is_empty());
}

// ---------------------------------------------------------------------------
// PresignedUrl / FileMetadata struct checks
// ---------------------------------------------------------------------------

#[tokio::test]
async fn presigned_url_has_empty_headers_for_inmemory() {
    let client = make_client();
    let url = client
        .generate_upload_url("h.txt", "text/plain", Duration::from_secs(60))
        .await
        .unwrap();
    assert!(url.headers.is_empty());
}

#[tokio::test]
async fn file_metadata_initial_size_is_zero() {
    let client = make_client();
    client
        .generate_upload_url("zero.bin", "application/octet-stream", Duration::from_secs(60))
        .await
        .unwrap();

    let meta = client.get_metadata("zero.bin").await.unwrap();
    assert_eq!(meta.size_bytes, 0);
}

// ---------------------------------------------------------------------------
// Error variant checks
// ---------------------------------------------------------------------------

#[test]
fn error_display_messages() {
    let err = FileClientError::NotFound("missing.txt".to_string());
    assert!(err.to_string().contains("missing.txt"));

    let err = FileClientError::ConnectionError("timeout".to_string());
    assert!(err.to_string().contains("timeout"));

    let err = FileClientError::Unauthorized("no token".to_string());
    assert!(err.to_string().contains("no token"));

    let err = FileClientError::QuotaExceeded("over limit".to_string());
    assert!(err.to_string().contains("over limit"));

    let err = FileClientError::InvalidConfig("bad".to_string());
    assert!(err.to_string().contains("bad"));

    let err = FileClientError::Internal("crash".to_string());
    assert!(err.to_string().contains("crash"));
}

// ---------------------------------------------------------------------------
// Overwrite semantics: upload same path twice
// ---------------------------------------------------------------------------

#[tokio::test]
async fn upload_same_path_overwrites_metadata() {
    let client = make_client();
    let dur = Duration::from_secs(60);

    client
        .generate_upload_url("overwrite.txt", "text/plain", dur)
        .await
        .unwrap();
    client
        .generate_upload_url("overwrite.txt", "application/json", dur)
        .await
        .unwrap();

    let meta = client.get_metadata("overwrite.txt").await.unwrap();
    assert_eq!(meta.content_type, "application/json");

    // Only one entry
    let all = client.stored_files().await;
    let count = all.iter().filter(|f| f.path == "overwrite.txt").count();
    assert_eq!(count, 1);
}
