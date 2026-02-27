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

// --- Proto 生成コードの ConfigService トレイト実装 ---
//
// ConfigServiceTonic は手動定義型を使った gRPC サービスラッパー。
// proto 生成コードの ConfigService トレイトを実装し、
// tonic::transport::Server に add_service で登録可能にする。

use crate::proto::k1s0::system::config::v1 as pb;

#[tonic::async_trait]
impl pb::config_service_server::ConfigService for ConfigServiceTonic {
    async fn get_config(
        &self,
        request: Request<pb::GetConfigRequest>,
    ) -> Result<Response<pb::GetConfigResponse>, Status> {
        let req = request.into_inner();
        let hand_req = GetConfigRequest {
            namespace: req.namespace,
            key: req.key,
        };
        let resp = self.inner.get_config(hand_req).await?;
        Ok(Response::new(pb::GetConfigResponse {
            entry: resp.entry.map(|e| hand_pb_to_proto_entry(&e)),
        }))
    }

    async fn list_configs(
        &self,
        request: Request<pb::ListConfigsRequest>,
    ) -> Result<Response<pb::ListConfigsResponse>, Status> {
        let req = request.into_inner();
        let hand_req = ListConfigsRequest {
            namespace: req.namespace,
            pagination: req.pagination.map(|p| super::config_grpc::PbPagination {
                page: p.page,
                page_size: p.page_size,
            }),
            search: req.search,
        };
        let resp = self.inner.list_configs(hand_req).await?;
        Ok(Response::new(pb::ListConfigsResponse {
            entries: resp.entries.iter().map(hand_pb_to_proto_entry).collect(),
            pagination: resp.pagination.map(|p| {
                crate::proto::k1s0::system::common::v1::PaginationResult {
                    total_count: p.total_count as i32,
                    page: p.page,
                    page_size: p.page_size,
                    has_next: p.has_next,
                }
            }),
        }))
    }

    async fn update_config(
        &self,
        request: Request<pb::UpdateConfigRequest>,
    ) -> Result<Response<pb::UpdateConfigResponse>, Status> {
        let req = request.into_inner();
        let value_json = String::from_utf8(req.value).unwrap_or_default();
        let hand_req = UpdateConfigRequest {
            namespace: req.namespace,
            key: req.key,
            value_json,
            version: req.version,
            description: req.description,
            updated_by: req.updated_by,
        };
        let resp = self.inner.update_config(hand_req).await?;
        Ok(Response::new(pb::UpdateConfigResponse {
            entry: resp.entry.map(|e| hand_pb_to_proto_entry(&e)),
        }))
    }

    async fn delete_config(
        &self,
        request: Request<pb::DeleteConfigRequest>,
    ) -> Result<Response<pb::DeleteConfigResponse>, Status> {
        let req = request.into_inner();
        let hand_req = DeleteConfigRequest {
            namespace: req.namespace,
            key: req.key,
            deleted_by: req.deleted_by,
        };
        let resp = self.inner.delete_config(hand_req).await?;
        Ok(Response::new(pb::DeleteConfigResponse {
            success: resp.success,
        }))
    }

    async fn get_service_config(
        &self,
        request: Request<pb::GetServiceConfigRequest>,
    ) -> Result<Response<pb::GetServiceConfigResponse>, Status> {
        let req = request.into_inner();
        let hand_req = GetServiceConfigRequest {
            service_name: req.service_name,
        };
        let resp = self.inner.get_service_config(hand_req).await?;
        let configs: std::collections::HashMap<String, String> = resp
            .entries
            .into_iter()
            .map(|e| (format!("{}.{}", e.namespace, e.key), e.value_json))
            .collect();
        Ok(Response::new(pb::GetServiceConfigResponse { configs }))
    }

    async fn get_config_schema(
        &self,
        _request: Request<pb::GetConfigSchemaRequest>,
    ) -> Result<Response<pb::GetConfigSchemaResponse>, Status> {
        Err(Status::unimplemented(
            "get_config_schema is not yet implemented via gRPC; use REST API",
        ))
    }

    async fn upsert_config_schema(
        &self,
        _request: Request<pb::UpsertConfigSchemaRequest>,
    ) -> Result<Response<pb::UpsertConfigSchemaResponse>, Status> {
        Err(Status::unimplemented(
            "upsert_config_schema is not yet implemented via gRPC; use REST API",
        ))
    }

    type WatchConfigStream =
        tokio_stream::wrappers::ReceiverStream<Result<pb::WatchConfigResponse, Status>>;

    async fn watch_config(
        &self,
        request: Request<pb::WatchConfigRequest>,
    ) -> Result<Response<Self::WatchConfigStream>, Status> {
        let req = request.into_inner();
        let hand_req = WatchConfigRequest {
            namespaces: req.namespaces,
        };
        let mut handler = self.inner.watch_config(hand_req).map_err(Status::from)?;

        let (tx, rx) = tokio::sync::mpsc::channel(128);

        tokio::spawn(async move {
            while let Some(notif) = handler.next().await {
                let resp = pb::WatchConfigResponse {
                    namespace: notif.namespace,
                    key: notif.key,
                    old_value: vec![],
                    new_value: notif.value_json.into_bytes(),
                    old_version: 0,
                    new_version: notif.version,
                    changed_by: notif.updated_by,
                    change_type: "UPDATED".to_string(),
                    changed_at: None,
                };
                if tx.send(Ok(resp)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }
}

/// 手動 PbConfigEntry を proto 生成 ConfigEntry に変換する。
fn hand_pb_to_proto_entry(e: &super::config_grpc::PbConfigEntry) -> pb::ConfigEntry {
    pb::ConfigEntry {
        id: e.id.clone(),
        namespace: e.namespace.clone(),
        key: e.key.clone(),
        value: e.value_json.as_bytes().to_vec(),
        version: e.version,
        description: e.description.clone(),
        created_by: e.created_by.clone(),
        updated_by: e.updated_by.clone(),
        created_at: e.created_at.as_ref().map(|t| {
            crate::proto::k1s0::system::common::v1::Timestamp {
                seconds: t.seconds,
                nanos: t.nanos,
            }
        }),
        updated_at: e.updated_at.as_ref().map(|t| {
            crate::proto::k1s0::system::common::v1::Timestamp {
                seconds: t.seconds,
                nanos: t.nanos,
            }
        }),
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
            namespaces: vec!["system.auth".to_string()],
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
            namespaces: vec!["system.auth".to_string()],
        });
        let result = tonic_svc.watch_config(req);

        assert!(result.is_ok());
    }
}
