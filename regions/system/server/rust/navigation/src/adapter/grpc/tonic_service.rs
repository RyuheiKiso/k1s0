use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::navigation::v1::{
    navigation_service_server::NavigationService,
    GetNavigationRequest as ProtoGetNavigationRequest,
    GetNavigationResponse as ProtoGetNavigationResponse,
};

use super::navigation_grpc::{GrpcError, NavigationGrpcService};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        // HIGH-001 監査対応: 同一ボディのマッチアームを OR パターンで統合する
        match e {
            GrpcError::ConfigLoad(msg) | GrpcError::Internal(msg) => Status::internal(msg),
            GrpcError::Unauthenticated(msg) => Status::unauthenticated(msg),
        }
    }
}

/// gRPC メタデータの Authorization ヘッダーから Bearer トークンを取得する。
/// HIGH-015対応: `bearer_token` フィールドをプロトメッセージから削除し、
/// 標準の gRPC メタデータ経由でトークンを受け取る設計に移行する。
/// トークンが存在しない場合は空文字列を返す（公開ルート取得を可能にするため）。
fn bearer_token_from_metadata<T>(request: &Request<T>) -> String {
    request
        .metadata()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            // "Bearer <token>" または "bearer <token>" の形式に対応する（大文字小文字を問わない）
            let lower = v.to_lowercase();
            if lower.starts_with("bearer ") {
                Some(v[7..].to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

pub struct NavigationServiceTonic {
    inner: Arc<NavigationGrpcService>,
}

impl NavigationServiceTonic {
    #[must_use]
    pub fn new(inner: Arc<NavigationGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl NavigationService for NavigationServiceTonic {
    async fn get_navigation(
        &self,
        request: Request<ProtoGetNavigationRequest>,
    ) -> Result<Response<ProtoGetNavigationResponse>, Status> {
        // HIGH-015対応: Authorization メタデータヘッダーから Bearer トークンを取得する
        // bearer_token フィールドはプロトメッセージから削除済みのため、メタデータを使用する
        let bearer_token = bearer_token_from_metadata(&request);
        let result = self
            .inner
            .get_navigation(&bearer_token)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetNavigationResponse::from(result)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::presentation::NavigationResponseBody;

    #[test]
    fn test_grpc_error_config_load_to_status() {
        let err = GrpcError::ConfigLoad("file not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("file not found"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("unexpected".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
    }

    #[test]
    fn test_grpc_error_unauthenticated_to_status() {
        let err = GrpcError::Unauthenticated("invalid token".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn filtered_navigation_maps_to_rest_body() {
        let body = NavigationResponseBody {
            routes: vec![],
            guards: vec![],
        };
        assert!(body.routes.is_empty());
    }
}
