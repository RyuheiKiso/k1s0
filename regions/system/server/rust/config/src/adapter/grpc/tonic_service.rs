//! tonic gRPC サービス登録ヘルパー。
//!
//! proto 生成コード (tonic-build) が未生成のため、
//! tonic::transport::Server に登録するためのサービスラッパーを手動で定義する。
//! tonic-build による生成後はこのファイルは不要となる。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use super::config_grpc::{
    ConfigGrpcService, DeleteConfigRequest, DeleteConfigResponse, GetConfigRequest,
    GetConfigResponse, GetServiceConfigRequest, GetServiceConfigResponse, GrpcError,
    ListConfigsRequest, ListConfigsResponse, UpdateConfigRequest, UpdateConfigResponse,
};
use super::watch_stream::{WatchConfigRequest, WatchConfigStreamHandler};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- ConfigService tonic ラッパー ---

/// ConfigServiceTonic は tonic の gRPC サービスとして ConfigGrpcService をラップする。
pub struct ConfigServiceTonic {
    inner: Arc<ConfigGrpcService>,
}

impl ConfigServiceTonic {
    pub fn new(inner: Arc<ConfigGrpcService>) -> Self {
        Self { inner }
    }

    pub async fn get_config(
        &self,
        request: Request<GetConfigRequest>,
    ) -> Result<Response<GetConfigResponse>, Status> {
        let resp = self.inner.get_config(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    pub async fn list_configs(
        &self,
        request: Request<ListConfigsRequest>,
    ) -> Result<Response<ListConfigsResponse>, Status> {
        let resp = self.inner.list_configs(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    pub async fn get_service_config(
        &self,
        request: Request<GetServiceConfigRequest>,
    ) -> Result<Response<GetServiceConfigResponse>, Status> {
        let resp = self.inner.get_service_config(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    pub async fn update_config(
        &self,
        request: Request<UpdateConfigRequest>,
    ) -> Result<Response<UpdateConfigResponse>, Status> {
        let resp = self.inner.update_config(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    pub async fn delete_config(
        &self,
        request: Request<DeleteConfigRequest>,
    ) -> Result<Response<DeleteConfigResponse>, Status> {
        let resp = self.inner.delete_config(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    /// 設定変更監視ストリームハンドラを生成して返す。
    ///
    /// gRPC サーバーストリーミングが利用可能になった後は、返された
    /// `WatchConfigStreamHandler` を `next()` でポーリングして
    /// レスポンスストリームに書き出す。
    ///
    /// watch 機能が無効の場合は `Status::unimplemented` を返す。
    #[allow(clippy::result_large_err)]
    pub fn watch_config(
        &self,
        request: Request<WatchConfigRequest>,
    ) -> Result<WatchConfigStreamHandler, Status> {
        self.inner
            .watch_config(request.into_inner())
            .map_err(Status::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("config not found: system/key".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::NotFound);
        assert!(status.message().contains("config not found"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("namespace is required".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::InvalidArgument);
        assert!(status.message().contains("namespace is required"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("database error"));
    }

    #[tokio::test]
    async fn test_config_service_tonic_get_config() {
        use crate::domain::entity::config_entry::ConfigEntry;
        use crate::domain::repository::config_repository::MockConfigRepository;
        use crate::usecase::delete_config::DeleteConfigUseCase;
        use crate::usecase::get_config::GetConfigUseCase;
        use crate::usecase::get_service_config::GetServiceConfigUseCase;
        use crate::usecase::list_configs::ListConfigsUseCase;
        use crate::usecase::update_config::UpdateConfigUseCase;

        let mut mock_repo = MockConfigRepository::new();
        let entry = ConfigEntry {
            id: uuid::Uuid::new_v4(),
            namespace: "system.auth".to_string(),
            key: "pool_size".to_string(),
            value_json: serde_json::json!(25),
            version: 3,
            description: Some("DB pool size".to_string()),
            created_by: "admin".to_string(),
            updated_by: "admin".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let expected_id = entry.id;

        mock_repo
            .expect_find_by_namespace_and_key()
            .withf(|ns, key| ns == "system.auth" && key == "pool_size")
            .returning(move |_, _| Ok(Some(entry.clone())));

        let repo = Arc::new(mock_repo);
        let get_config_uc = Arc::new(GetConfigUseCase::new(repo.clone()));
        let list_configs_uc = Arc::new(ListConfigsUseCase::new(repo.clone()));
        let get_service_config_uc = Arc::new(GetServiceConfigUseCase::new(repo.clone()));
        let update_config_uc = Arc::new(UpdateConfigUseCase::new(repo.clone()));
        let delete_config_uc = Arc::new(DeleteConfigUseCase::new(repo));

        let config_svc = Arc::new(ConfigGrpcService::new(
            get_config_uc,
            list_configs_uc,
            get_service_config_uc,
            update_config_uc,
            delete_config_uc,
        ));
        let tonic_svc = ConfigServiceTonic::new(config_svc);

        let req = Request::new(GetConfigRequest {
            namespace: "system.auth".to_string(),
            key: "pool_size".to_string(),
        });
        let resp = tonic_svc.get_config(req).await.unwrap();
        let inner = resp.into_inner();
        let pb_entry = inner.entry.unwrap();

        assert_eq!(pb_entry.id, expected_id.to_string());
        assert_eq!(pb_entry.namespace, "system.auth");
        assert_eq!(pb_entry.key, "pool_size");
        assert_eq!(pb_entry.version, 3);
    }

    #[tokio::test]
    async fn test_config_service_tonic_get_config_invalid_argument() {
        use crate::domain::repository::config_repository::MockConfigRepository;
        use crate::usecase::delete_config::DeleteConfigUseCase;
        use crate::usecase::get_config::GetConfigUseCase;
        use crate::usecase::get_service_config::GetServiceConfigUseCase;
        use crate::usecase::list_configs::ListConfigsUseCase;
        use crate::usecase::update_config::UpdateConfigUseCase;

        let mock_repo = MockConfigRepository::new();
        let repo = Arc::new(mock_repo);
        let get_config_uc = Arc::new(GetConfigUseCase::new(repo.clone()));
        let list_configs_uc = Arc::new(ListConfigsUseCase::new(repo.clone()));
        let get_service_config_uc = Arc::new(GetServiceConfigUseCase::new(repo.clone()));
        let update_config_uc = Arc::new(UpdateConfigUseCase::new(repo.clone()));
        let delete_config_uc = Arc::new(DeleteConfigUseCase::new(repo));

        let config_svc = Arc::new(ConfigGrpcService::new(
            get_config_uc,
            list_configs_uc,
            get_service_config_uc,
            update_config_uc,
            delete_config_uc,
        ));
        let tonic_svc = ConfigServiceTonic::new(config_svc);

        // Empty namespace and key should return InvalidArgument
        let req = Request::new(GetConfigRequest {
            namespace: "".to_string(),
            key: "".to_string(),
        });
        let result = tonic_svc.get_config(req).await;
        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_watch_config_not_enabled_returns_internal_status() {
        use crate::domain::repository::config_repository::MockConfigRepository;
        use crate::usecase::delete_config::DeleteConfigUseCase;
        use crate::usecase::get_config::GetConfigUseCase;
        use crate::usecase::get_service_config::GetServiceConfigUseCase;
        use crate::usecase::list_configs::ListConfigsUseCase;
        use crate::usecase::update_config::UpdateConfigUseCase;

        let mock_repo = MockConfigRepository::new();
        let repo = Arc::new(mock_repo);
        let get_config_uc = Arc::new(GetConfigUseCase::new(repo.clone()));
        let list_configs_uc = Arc::new(ListConfigsUseCase::new(repo.clone()));
        let get_service_config_uc = Arc::new(GetServiceConfigUseCase::new(repo.clone()));
        let update_config_uc = Arc::new(UpdateConfigUseCase::new(repo.clone()));
        let delete_config_uc = Arc::new(DeleteConfigUseCase::new(repo));

        // watch_sender なしの ConfigGrpcService
        let config_svc = Arc::new(ConfigGrpcService::new(
            get_config_uc,
            list_configs_uc,
            get_service_config_uc,
            update_config_uc,
            delete_config_uc,
        ));
        let tonic_svc = ConfigServiceTonic::new(config_svc);

        let req = Request::new(WatchConfigRequest {
            namespace: "system.auth".to_string(),
        });
        let result = tonic_svc.watch_config(req);

        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), Code::Internal);
    }

    #[tokio::test]
    async fn test_watch_config_enabled_returns_handler() {
        use crate::domain::repository::config_repository::MockConfigRepository;
        use crate::usecase::delete_config::DeleteConfigUseCase;
        use crate::usecase::get_config::GetConfigUseCase;
        use crate::usecase::get_service_config::GetServiceConfigUseCase;
        use crate::usecase::list_configs::ListConfigsUseCase;
        use crate::usecase::update_config::UpdateConfigUseCase;
        use crate::usecase::watch_config::WatchConfigUseCase;

        let mock_repo = MockConfigRepository::new();
        let repo = Arc::new(mock_repo);
        let get_config_uc = Arc::new(GetConfigUseCase::new(repo.clone()));
        let list_configs_uc = Arc::new(ListConfigsUseCase::new(repo.clone()));
        let get_service_config_uc = Arc::new(GetServiceConfigUseCase::new(repo.clone()));
        let update_config_uc = Arc::new(UpdateConfigUseCase::new(repo.clone()));
        let delete_config_uc = Arc::new(DeleteConfigUseCase::new(repo));
        let (_watch_uc, tx) = WatchConfigUseCase::new();

        let config_svc = Arc::new(ConfigGrpcService::new_with_watch(
            get_config_uc,
            list_configs_uc,
            get_service_config_uc,
            update_config_uc,
            delete_config_uc,
            tx,
        ));
        let tonic_svc = ConfigServiceTonic::new(config_svc);

        let req = Request::new(WatchConfigRequest {
            namespace: "system.auth".to_string(),
        });
        let result = tonic_svc.watch_config(req);

        assert!(result.is_ok());
    }
}
