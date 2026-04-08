/// STATIC-HIGH-003 監査対応: ローカルファイルシステムからファイルを提供する内部ストレージハンドラー。
/// Content-Disposition: attachment ヘッダーによりブラウザの自動実行を防止する。
/// infer クレートのマジックバイト検証でコンテンツタイプを正確に判定し、
/// X-Content-Type-Options: nosniff で MIME スニッフィングを無効化する。
use std::path::PathBuf;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

use super::AppState;
use crate::domain::service::FileDomainService;

/// GET /internal/storage/{*key} - ローカル PV からファイルを提供する。
/// production では S3 pre-signed URL を使用するため、このハンドラーは dev/test 環境向け。
pub async fn serve_internal_storage(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    // ローカルストレージが未設定（S3 モード等）の場合は 501 を返す
    let root = match &state.storage_root_path {
        Some(p) => p.clone(),
        None => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                "Local storage is not configured",
            )
                .into_response();
        }
    };

    // パストラバーサル攻撃を防ぐため、絶対パスや ".." を含むキーを拒否する
    let key_path = PathBuf::from(&key);
    if key_path.is_absolute()
        || key_path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            )
        })
    {
        return (StatusCode::BAD_REQUEST, "Invalid storage key").into_response();
    }

    let full_path = root.join(&key_path);

    // ファイルバイト列を読み込む（存在しない場合は 404）
    let bytes = match tokio::fs::read(&full_path).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };

    // infer によるマジックバイト検出でコンテンツタイプを決定する。
    // 拡張子ベースの推定は偽装可能なため、実際のファイルバイトで検証する。
    let detected_type = infer::get(&bytes).map(|t| t.mime_type().to_string());

    // 拡張子ベースの期待コンテンツタイプ（allowlist から取得）
    let extension_type = full_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map_or("application/octet-stream", |ext| match ext {
            "pdf" => "application/pdf",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "txt" => "text/plain",
            "csv" => "text/csv",
            "json" => "application/json",
            "zip" => "application/zip",
            "gz" => "application/gzip",
            _ => "application/octet-stream",
        });

    // STATIC-HIGH-003 監査対応: マジックバイトで検出した型が拡張子の期待型と一致しない場合は拒否する。
    // ただし infer が判定できない場合（バイナリ等）は拡張子ベースにフォールバックする。
    let content_type = match &detected_type {
        Some(detected) => {
            if detected != extension_type && extension_type != "application/octet-stream" {
                tracing::warn!(
                    key = %key,
                    detected = %detected,
                    expected = %extension_type,
                    "マジックバイトと拡張子のコンテンツタイプが一致しません"
                );
                return (
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "File content type does not match the declared type",
                )
                    .into_response();
            }
            // マジックバイト検出結果が許可リストにない場合は拒否する
            if !FileDomainService::is_allowed_content_type(detected) {
                tracing::warn!(key = %key, detected = %detected, "許可リスト外のコンテンツタイプを拒否");
                return (StatusCode::FORBIDDEN, "This content type is not allowed")
                    .into_response();
            }
            detected.clone()
        }
        // infer が判定できない場合は拡張子ベースの型を使用する
        None => extension_type.to_string(),
    };

    // ファイル名を storage_key の末尾から取得する
    let filename = full_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download")
        .to_string();

    // RFC 5987 形式に限定する（ASCIIのみ）
    let safe_filename = filename
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect::<String>();
    let safe_filename = if safe_filename.is_empty() {
        "download".to_string()
    } else {
        safe_filename
    };

    Response::builder()
        .status(StatusCode::OK)
        // STATIC-HIGH-003 監査対応: ブラウザによるコンテンツの自動実行を防止する
        .header(header::CONTENT_TYPE, &content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{safe_filename}\""),
        )
        // MIME スニッフィングを完全に無効化する
        .header("X-Content-Type-Options", "nosniff")
        .body(Body::from(bytes))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()
        })
}
