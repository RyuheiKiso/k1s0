use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use super::{ErrorDetail, ErrorResponse};
use crate::usecase::delete_config::DeleteConfigError;
use crate::usecase::get_config::GetConfigError;
use crate::usecase::get_service_config::GetServiceConfigError;
use crate::usecase::list_configs::ListConfigsError;
use crate::usecase::update_config::UpdateConfigError;

/// GetConfigError を HTTP レスポンスに変換する。
impl IntoResponse for GetConfigError {
    fn into_response(self) -> axum::response::Response {
        match self {
            GetConfigError::NotFound(ns, key) => {
                let err = ErrorResponse::new(
                    "SYS_CONFIG_KEY_NOT_FOUND",
                    &format!("指定された設定キーが見つかりません: {}/{}", ns, key),
                );
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            }
            GetConfigError::Internal(msg) => {
                let err = ErrorResponse::new("SYS_CONFIG_INTERNAL_ERROR", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// ListConfigsError を HTTP レスポンスに変換する。
impl IntoResponse for ListConfigsError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ListConfigsError::Validation(msg) => {
                let err = ErrorResponse::new("SYS_CONFIG_VALIDATION_FAILED", &msg);
                (StatusCode::BAD_REQUEST, Json(err)).into_response()
            }
            ListConfigsError::Internal(msg) => {
                let err = ErrorResponse::new("SYS_CONFIG_INTERNAL_ERROR", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// UpdateConfigError を HTTP レスポンスに変換する。
impl IntoResponse for UpdateConfigError {
    fn into_response(self) -> axum::response::Response {
        match self {
            UpdateConfigError::NotFound(ns, key) => {
                let err = ErrorResponse::new(
                    "SYS_CONFIG_KEY_NOT_FOUND",
                    &format!("指定された設定キーが見つかりません: {}/{}", ns, key),
                );
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            }
            UpdateConfigError::VersionConflict { expected, current } => {
                let err = ErrorResponse::with_details(
                    "SYS_CONFIG_VERSION_CONFLICT",
                    "設定値が他のユーザーによって更新されています。最新のバージョンを取得してください",
                    vec![ErrorDetail {
                        field: "version".to_string(),
                        message: format!("期待値: {}, 現在値: {}", expected, current),
                    }],
                );
                (StatusCode::CONFLICT, Json(err)).into_response()
            }
            UpdateConfigError::Validation(msg) => {
                let err = ErrorResponse::new("SYS_CONFIG_VALIDATION_FAILED", &msg);
                (StatusCode::BAD_REQUEST, Json(err)).into_response()
            }
            UpdateConfigError::Internal(msg) => {
                let err = ErrorResponse::new("SYS_CONFIG_INTERNAL_ERROR", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// DeleteConfigError を HTTP レスポンスに変換する。
impl IntoResponse for DeleteConfigError {
    fn into_response(self) -> axum::response::Response {
        match self {
            DeleteConfigError::NotFound(ns, key) => {
                let err = ErrorResponse::new(
                    "SYS_CONFIG_KEY_NOT_FOUND",
                    &format!("指定された設定キーが見つかりません: {}/{}", ns, key),
                );
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            }
            DeleteConfigError::Internal(msg) => {
                let err = ErrorResponse::new("SYS_CONFIG_INTERNAL_ERROR", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// GetServiceConfigError を HTTP レスポンスに変換する。
impl IntoResponse for GetServiceConfigError {
    fn into_response(self) -> axum::response::Response {
        match self {
            GetServiceConfigError::NotFound(name) => {
                let err = ErrorResponse::new(
                    "SYS_CONFIG_SERVICE_NOT_FOUND",
                    &format!("指定されたサービスの設定が見つかりません: {}", name),
                );
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            }
            GetServiceConfigError::Internal(msg) => {
                let err = ErrorResponse::new("SYS_CONFIG_INTERNAL_ERROR", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn response_to_json(
        resp: axum::response::Response,
    ) -> (StatusCode, serde_json::Value) {
        let status = resp.status();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        (status, json)
    }

    #[tokio::test]
    async fn test_get_config_error_not_found_response() {
        let err = GetConfigError::NotFound("system.auth".to_string(), "missing".to_string());
        let resp = err.into_response();
        let (status, json) = response_to_json(resp).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["error"]["code"], "SYS_CONFIG_KEY_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_update_config_error_version_conflict_response() {
        let err = UpdateConfigError::VersionConflict {
            expected: 3,
            current: 4,
        };
        let resp = err.into_response();
        let (status, json) = response_to_json(resp).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(json["error"]["code"], "SYS_CONFIG_VERSION_CONFLICT");
        assert!(!json["error"]["details"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_delete_config_error_not_found_response() {
        let err = DeleteConfigError::NotFound("system.auth".to_string(), "missing".to_string());
        let resp = err.into_response();
        let (status, json) = response_to_json(resp).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["error"]["code"], "SYS_CONFIG_KEY_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_get_service_config_error_not_found_response() {
        let err = GetServiceConfigError::NotFound("unknown-service".to_string());
        let resp = err.into_response();
        let (status, json) = response_to_json(resp).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["error"]["code"], "SYS_CONFIG_SERVICE_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_list_configs_error_validation_response() {
        let err = ListConfigsError::Validation("page must be >= 1".to_string());
        let resp = err.into_response();
        let (status, json) = response_to_json(resp).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["error"]["code"], "SYS_CONFIG_VALIDATION_FAILED");
    }
}
