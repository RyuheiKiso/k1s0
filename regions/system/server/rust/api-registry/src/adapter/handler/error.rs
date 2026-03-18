use axum::{http::StatusCode, Json};

use k1s0_server_common::error as codes;
use k1s0_server_common::{ErrorResponse, ServiceError};

/// ApiError は api-registry 用のエラーレスポンス生成ヘルパー。
///
/// エラーコードは k1s0-server-common の `SYS_APIREG_*` パターンに従う。
/// `.expect()` による serde_json::to_value を廃止し、Json<ErrorResponse> を直接返す。
pub struct ApiError;

impl ApiError {
    /// 404 Not Found エラーレスポンスを生成する
    pub fn not_found(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
        let err = ErrorResponse::new(codes::api_registry::not_found(), message);
        (StatusCode::NOT_FOUND, Json(err))
    }

    /// 404 スキーマ未検出エラーレスポンスを生成する
    pub fn schema_not_found(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
        let err = ErrorResponse::new(codes::api_registry::schema_not_found(), message);
        (StatusCode::NOT_FOUND, Json(err))
    }

    /// 404 バージョン未検出エラーレスポンスを生成する
    pub fn version_not_found(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
        let err = ErrorResponse::new(codes::api_registry::version_not_found(), message);
        (StatusCode::NOT_FOUND, Json(err))
    }

    /// 400 Bad Request エラーレスポンスを生成する
    pub fn bad_request(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
        let err = ErrorResponse::new(codes::api_registry::bad_request(), message);
        (StatusCode::BAD_REQUEST, Json(err))
    }

    /// 409 Conflict エラーレスポンスを生成する
    pub fn conflict(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
        let err = ErrorResponse::new(codes::api_registry::conflict(), message);
        (StatusCode::CONFLICT, Json(err))
    }

    /// 409 既に存在するエラーレスポンスを生成する
    pub fn already_exists(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
        let err = ErrorResponse::new(codes::api_registry::already_exists(), message);
        (StatusCode::CONFLICT, Json(err))
    }

    /// 409 最新バージョン削除不可エラーレスポンスを生成する
    pub fn cannot_delete_latest(
        message: impl Into<String>,
    ) -> (StatusCode, Json<ErrorResponse>) {
        let err = ErrorResponse::new(codes::api_registry::cannot_delete_latest(), message);
        (StatusCode::CONFLICT, Json(err))
    }

    /// 422 スキーマ不正エラーレスポンスを生成する
    pub fn unprocessable_entity(
        message: impl Into<String>,
    ) -> (StatusCode, Json<ErrorResponse>) {
        let err = ErrorResponse::new(codes::api_registry::schema_invalid(), message);
        (StatusCode::UNPROCESSABLE_ENTITY, Json(err))
    }

    /// 502 バリデーター呼び出し失敗エラーレスポンスを生成する
    pub fn validator_error(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
        let err = ErrorResponse::new(codes::api_registry::validator_error(), message);
        (StatusCode::BAD_GATEWAY, Json(err))
    }

    /// 401 Unauthorized エラーレスポンスを生成する
    pub fn unauthorized(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
        let err = ErrorResponse::new(codes::api_registry::unauthorized(), message);
        (StatusCode::UNAUTHORIZED, Json(err))
    }

    /// 500 Internal Server Error レスポンスを生成する
    pub fn internal(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
        let err = ErrorResponse::new(codes::api_registry::internal_error(), message);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
    }
}

/// ServiceError を (StatusCode, Json<ErrorResponse>) タプル形式に変換する。
/// api-registry ハンドラで使用するヘルパー関数。
/// `.expect()` を排除し、Json<ErrorResponse> を直接返す。
pub fn service_error_to_response(err: ServiceError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match &err {
        ServiceError::NotFound { .. } => StatusCode::NOT_FOUND,
        ServiceError::BadRequest { .. } => StatusCode::BAD_REQUEST,
        ServiceError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
        ServiceError::Forbidden { .. } => StatusCode::FORBIDDEN,
        ServiceError::Conflict { .. } => StatusCode::CONFLICT,
        ServiceError::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
        ServiceError::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
        ServiceError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        ServiceError::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
    };
    let body = err.to_error_response();
    (status, Json(body))
}
