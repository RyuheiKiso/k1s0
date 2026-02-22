//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の SagaService トレイトを実装する。
//! 各メソッドで proto 型 ↔ 手動型の変換を行い、既存の SagaGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::saga::v1::{
    saga_service_server::SagaService,
    CancelSagaRequest as ProtoCancelSagaRequest,
    CancelSagaResponse as ProtoCancelSagaResponse,
    GetSagaRequest as ProtoGetSagaRequest,
    GetSagaResponse as ProtoGetSagaResponse,
    ListSagasRequest as ProtoListSagasRequest,
    ListSagasResponse as ProtoListSagasResponse,
    ListWorkflowsRequest as ProtoListWorkflowsRequest,
    ListWorkflowsResponse as ProtoListWorkflowsResponse,
    RegisterWorkflowRequest as ProtoRegisterWorkflowRequest,
    RegisterWorkflowResponse as ProtoRegisterWorkflowResponse,
    SagaStateProto as ProtoSagaState,
    SagaStepLogProto as ProtoSagaStepLog,
    StartSagaRequest as ProtoStartSagaRequest,
    StartSagaResponse as ProtoStartSagaResponse,
    WorkflowSummary as ProtoWorkflowSummary,
};
use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};

use super::saga_grpc::{
    CancelSagaRequest, GetSagaRequest, GrpcError, ListSagasRequest, ListWorkflowsRequest,
    RegisterWorkflowRequest, SagaGrpcService, StartSagaRequest,
};

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

// --- 変換ヘルパー ---

/// RFC3339 文字列を proto Timestamp に変換する。
fn rfc3339_to_proto_timestamp(s: &str) -> Option<ProtoTimestamp> {
    chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| ProtoTimestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}

// --- SagaService tonic ラッパー ---

/// SagaServiceTonic は tonic の SagaService として SagaGrpcService をラップする。
pub struct SagaServiceTonic {
    inner: Arc<SagaGrpcService>,
}

impl SagaServiceTonic {
    pub fn new(inner: Arc<SagaGrpcService>) -> Self {
        Self { inner }
    }

    /// 内部サービスへの参照を返す。
    pub fn inner(&self) -> &SagaGrpcService {
        &self.inner
    }
}

#[async_trait::async_trait]
impl SagaService for SagaServiceTonic {
    async fn start_saga(
        &self,
        request: Request<ProtoStartSagaRequest>,
    ) -> Result<Response<ProtoStartSagaResponse>, Status> {
        let inner = request.into_inner();
        let req = StartSagaRequest {
            workflow_name: inner.workflow_name,
            payload: inner.payload,
            correlation_id: inner.correlation_id,
            initiated_by: inner.initiated_by,
        };
        let resp = self.inner.start_saga(req).await.map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoStartSagaResponse {
            saga_id: resp.saga_id,
            status: resp.status,
        }))
    }

    async fn get_saga(
        &self,
        request: Request<ProtoGetSagaRequest>,
    ) -> Result<Response<ProtoGetSagaResponse>, Status> {
        let req = GetSagaRequest {
            saga_id: request.into_inner().saga_id,
        };
        let resp = self.inner.get_saga(req).await.map_err(Into::<Status>::into)?;

        let proto_saga = ProtoSagaState {
            id: resp.saga.id,
            workflow_name: resp.saga.workflow_name,
            current_step: resp.saga.current_step,
            status: resp.saga.status,
            payload: resp.saga.payload,
            correlation_id: resp.saga.correlation_id,
            initiated_by: resp.saga.initiated_by,
            error_message: resp.saga.error_message,
            created_at: rfc3339_to_proto_timestamp(&resp.saga.created_at),
            updated_at: rfc3339_to_proto_timestamp(&resp.saga.updated_at),
        };

        let proto_step_logs = resp
            .step_logs
            .into_iter()
            .map(|log| ProtoSagaStepLog {
                id: log.id,
                saga_id: log.saga_id,
                step_index: log.step_index,
                step_name: log.step_name,
                action: log.action,
                status: log.status,
                request_payload: log.request_payload,
                response_payload: log.response_payload,
                error_message: log.error_message,
                started_at: rfc3339_to_proto_timestamp(&log.started_at),
                completed_at: if log.completed_at.is_empty() {
                    None
                } else {
                    rfc3339_to_proto_timestamp(&log.completed_at)
                },
            })
            .collect();

        Ok(Response::new(ProtoGetSagaResponse {
            saga: Some(proto_saga),
            step_logs: proto_step_logs,
        }))
    }

    async fn list_sagas(
        &self,
        request: Request<ProtoListSagasRequest>,
    ) -> Result<Response<ProtoListSagasResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner
            .pagination
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, 20));
        let req = ListSagasRequest {
            page,
            page_size,
            workflow_name: inner.workflow_name,
            status: inner.status,
            correlation_id: inner.correlation_id,
        };
        let resp = self.inner.list_sagas(req).await.map_err(Into::<Status>::into)?;

        let proto_sagas = resp
            .sagas
            .into_iter()
            .map(|saga| ProtoSagaState {
                id: saga.id,
                workflow_name: saga.workflow_name,
                current_step: saga.current_step,
                status: saga.status,
                payload: saga.payload,
                correlation_id: saga.correlation_id,
                initiated_by: saga.initiated_by,
                error_message: saga.error_message,
                created_at: rfc3339_to_proto_timestamp(&saga.created_at),
                updated_at: rfc3339_to_proto_timestamp(&saga.updated_at),
            })
            .collect();

        let proto_pagination = Some(ProtoPaginationResult {
            total_count: resp.total_count as i32,
            page: resp.page,
            page_size: resp.page_size,
            has_next: resp.has_next,
        });

        Ok(Response::new(ProtoListSagasResponse {
            sagas: proto_sagas,
            pagination: proto_pagination,
        }))
    }

    async fn cancel_saga(
        &self,
        request: Request<ProtoCancelSagaRequest>,
    ) -> Result<Response<ProtoCancelSagaResponse>, Status> {
        let req = CancelSagaRequest {
            saga_id: request.into_inner().saga_id,
        };
        let resp = self.inner.cancel_saga(req).await.map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoCancelSagaResponse {
            success: resp.success,
            message: resp.message,
        }))
    }

    async fn register_workflow(
        &self,
        request: Request<ProtoRegisterWorkflowRequest>,
    ) -> Result<Response<ProtoRegisterWorkflowResponse>, Status> {
        let req = RegisterWorkflowRequest {
            workflow_yaml: request.into_inner().workflow_yaml,
        };
        let resp = self.inner.register_workflow(req).await.map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoRegisterWorkflowResponse {
            name: resp.name,
            step_count: resp.step_count,
        }))
    }

    async fn list_workflows(
        &self,
        _request: Request<ProtoListWorkflowsRequest>,
    ) -> Result<Response<ProtoListWorkflowsResponse>, Status> {
        let req = ListWorkflowsRequest {};
        let resp = self.inner.list_workflows(req).await.map_err(Into::<Status>::into)?;
        let workflows = resp
            .workflows
            .into_iter()
            .map(|wf| ProtoWorkflowSummary {
                name: wf.name,
                step_count: wf.step_count,
                step_names: wf.step_names,
            })
            .collect();
        Ok(Response::new(ProtoListWorkflowsResponse { workflows }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::saga_state::SagaState;
    use crate::domain::entity::workflow::WorkflowDefinition;
    use crate::domain::repository::saga_repository::MockSagaRepository;
    use crate::domain::repository::workflow_repository::MockWorkflowRepository;
    use crate::infrastructure::grpc_caller::MockGrpcStepCaller;
    use crate::usecase::{
        CancelSagaUseCase, GetSagaUseCase, ListSagasUseCase, ListWorkflowsUseCase,
        RegisterWorkflowUseCase, StartSagaUseCase,
    };
    use crate::usecase::ExecuteSagaUseCase;
    use tonic::Code;

    fn make_tonic_service(
        start_saga_uc: Arc<StartSagaUseCase>,
        get_saga_uc: Arc<GetSagaUseCase>,
        list_sagas_uc: Arc<ListSagasUseCase>,
        cancel_saga_uc: Arc<CancelSagaUseCase>,
        register_workflow_uc: Arc<RegisterWorkflowUseCase>,
        list_workflows_uc: Arc<ListWorkflowsUseCase>,
    ) -> SagaServiceTonic {
        let grpc_svc = Arc::new(SagaGrpcService::new(
            start_saga_uc,
            get_saga_uc,
            list_sagas_uc,
            cancel_saga_uc,
            register_workflow_uc,
            list_workflows_uc,
        ));
        SagaServiceTonic::new(grpc_svc)
    }

    fn make_dummy_start_uc() -> Arc<StartSagaUseCase> {
        let mut mock_wf_repo = MockWorkflowRepository::new();
        mock_wf_repo.expect_get().returning(|_| Ok(None));
        let mock_saga_repo = MockSagaRepository::new();
        let mock_caller = MockGrpcStepCaller::new();
        let execute_uc = Arc::new(ExecuteSagaUseCase::new(
            Arc::new(mock_saga_repo),
            Arc::new(mock_caller),
            None,
        ));
        let mock_saga_repo2 = MockSagaRepository::new();
        Arc::new(StartSagaUseCase::new(
            Arc::new(mock_saga_repo2),
            Arc::new(mock_wf_repo),
            execute_uc,
        ))
    }

    fn make_dummy_get_uc() -> Arc<GetSagaUseCase> {
        let mut mock = MockSagaRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));
        Arc::new(GetSagaUseCase::new(Arc::new(mock)))
    }

    fn make_dummy_list_sagas_uc() -> Arc<ListSagasUseCase> {
        let mut mock = MockSagaRepository::new();
        mock.expect_list().returning(|_| Ok((vec![], 0)));
        Arc::new(ListSagasUseCase::new(Arc::new(mock)))
    }

    fn make_dummy_cancel_uc() -> Arc<CancelSagaUseCase> {
        let mut mock = MockSagaRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));
        Arc::new(CancelSagaUseCase::new(Arc::new(mock)))
    }

    fn make_dummy_register_uc() -> Arc<RegisterWorkflowUseCase> {
        let mock = MockWorkflowRepository::new();
        Arc::new(RegisterWorkflowUseCase::new(Arc::new(mock)))
    }

    fn make_dummy_list_workflows_uc() -> Arc<ListWorkflowsUseCase> {
        let mut mock = MockWorkflowRepository::new();
        mock.expect_list().returning(|| Ok(vec![]));
        Arc::new(ListWorkflowsUseCase::new(Arc::new(mock)))
    }

    // --- テスト1: start_saga 成功 ---
    #[tokio::test]
    async fn test_start_saga_success() {
        let yaml = r#"
name: test-workflow
steps:
  - name: step1
    service: svc
    method: Svc.Do
"#;
        let workflow = WorkflowDefinition::from_yaml(yaml).unwrap();
        let workflow_clone = workflow.clone();

        let mut mock_wf_repo = MockWorkflowRepository::new();
        mock_wf_repo
            .expect_get()
            .returning(move |_| Ok(Some(workflow_clone.clone())));

        let mut mock_saga_repo = MockSagaRepository::new();
        mock_saga_repo.expect_create().returning(|_| Ok(()));
        mock_saga_repo.expect_find_by_id().returning(|_| Ok(None));

        let mut mock_caller = MockGrpcStepCaller::new();
        mock_caller
            .expect_call_step()
            .returning(|_, _, _| Ok(serde_json::json!({})));

        let execute_uc = Arc::new(ExecuteSagaUseCase::new(
            Arc::new(MockSagaRepository::new()),
            Arc::new(mock_caller),
            None,
        ));

        let start_uc = Arc::new(StartSagaUseCase::new(
            Arc::new(mock_saga_repo),
            Arc::new(mock_wf_repo),
            execute_uc,
        ));

        let svc = make_tonic_service(
            start_uc,
            make_dummy_get_uc(),
            make_dummy_list_sagas_uc(),
            make_dummy_cancel_uc(),
            make_dummy_register_uc(),
            make_dummy_list_workflows_uc(),
        );

        let req = Request::new(ProtoStartSagaRequest {
            workflow_name: "test-workflow".to_string(),
            payload: serde_json::to_vec(&serde_json::json!({"order_id": "123"})).unwrap(),
            correlation_id: "corr-001".to_string(),
            initiated_by: "user-1".to_string(),
        });

        let resp = svc.start_saga(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(!inner.saga_id.is_empty());
        assert_eq!(inner.status, "STARTED");
    }

    // --- テスト2: list_sagas → 空リスト ---
    #[tokio::test]
    async fn test_list_sagas_empty() {
        let mut mock_saga_repo = MockSagaRepository::new();
        mock_saga_repo.expect_list().returning(|_| Ok((vec![], 0)));

        let list_sagas_uc = Arc::new(ListSagasUseCase::new(Arc::new(mock_saga_repo)));

        let svc = make_tonic_service(
            make_dummy_start_uc(),
            make_dummy_get_uc(),
            list_sagas_uc,
            make_dummy_cancel_uc(),
            make_dummy_register_uc(),
            make_dummy_list_workflows_uc(),
        );

        let req = Request::new(ProtoListSagasRequest {
            pagination: Some(crate::proto::k1s0::system::common::v1::Pagination {
                page: 1,
                page_size: 20,
            }),
            workflow_name: "".to_string(),
            status: "".to_string(),
            correlation_id: "".to_string(),
        });

        let resp = svc.list_sagas(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(inner.sagas.is_empty());
        let pagination = inner.pagination.unwrap();
        assert_eq!(pagination.total_count, 0);
        assert!(!pagination.has_next);
    }

    // --- テスト3: cancel_saga → not found エラー ---
    #[tokio::test]
    async fn test_cancel_saga_not_found() {
        let mut mock_saga_repo = MockSagaRepository::new();
        mock_saga_repo.expect_find_by_id().returning(|_| Ok(None));

        let cancel_uc = Arc::new(CancelSagaUseCase::new(Arc::new(mock_saga_repo)));

        let svc = make_tonic_service(
            make_dummy_start_uc(),
            make_dummy_get_uc(),
            make_dummy_list_sagas_uc(),
            cancel_uc,
            make_dummy_register_uc(),
            make_dummy_list_workflows_uc(),
        );

        let saga_id = uuid::Uuid::new_v4().to_string();
        let req = Request::new(ProtoCancelSagaRequest {
            saga_id: saga_id.clone(),
        });

        let result = svc.cancel_saga(req).await;
        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), Code::NotFound);
        assert!(status.message().contains("not found"));
    }

    // --- テスト4: GrpcError → Status 変換 ---
    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("saga not found: abc".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::NotFound);
        assert!(status.message().contains("saga not found: abc"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("invalid saga_id: xyz".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::InvalidArgument);
        assert!(status.message().contains("invalid saga_id: xyz"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database connection failed".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("database connection failed"));
    }

    // --- テスト5: get_saga → 存在するSagaを取得 ---
    #[tokio::test]
    async fn test_get_saga_success() {
        let saga = SagaState::new(
            "test-workflow".to_string(),
            serde_json::json!({"key": "value"}),
            Some("corr-001".to_string()),
            Some("user-1".to_string()),
        );
        let saga_id = saga.saga_id;
        let saga_clone = saga.clone();

        let mut mock_saga_repo = MockSagaRepository::new();
        mock_saga_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(saga_clone.clone())));
        mock_saga_repo
            .expect_find_step_logs()
            .returning(|_| Ok(vec![]));

        let get_uc = Arc::new(GetSagaUseCase::new(Arc::new(mock_saga_repo)));

        let svc = make_tonic_service(
            make_dummy_start_uc(),
            get_uc,
            make_dummy_list_sagas_uc(),
            make_dummy_cancel_uc(),
            make_dummy_register_uc(),
            make_dummy_list_workflows_uc(),
        );

        let req = Request::new(ProtoGetSagaRequest {
            saga_id: saga_id.to_string(),
        });

        let resp = svc.get_saga(req).await.unwrap();
        let inner = resp.into_inner();
        let saga_proto = inner.saga.unwrap();
        assert_eq!(saga_proto.id, saga_id.to_string());
        assert_eq!(saga_proto.workflow_name, "test-workflow");
        assert_eq!(saga_proto.status, "STARTED");
        assert!(inner.step_logs.is_empty());
    }

    // --- テスト6: register_workflow → 成功 ---
    #[tokio::test]
    async fn test_register_workflow_success() {
        let mut mock_wf_repo = MockWorkflowRepository::new();
        mock_wf_repo.expect_register().returning(|_| Ok(()));

        let register_uc = Arc::new(RegisterWorkflowUseCase::new(Arc::new(mock_wf_repo)));

        let svc = make_tonic_service(
            make_dummy_start_uc(),
            make_dummy_get_uc(),
            make_dummy_list_sagas_uc(),
            make_dummy_cancel_uc(),
            register_uc,
            make_dummy_list_workflows_uc(),
        );

        let yaml = r#"
name: order-fulfillment
steps:
  - name: reserve-inventory
    service: inventory-service
    method: InventoryService.Reserve
  - name: process-payment
    service: payment-service
    method: PaymentService.Charge
"#;

        let req = Request::new(ProtoRegisterWorkflowRequest {
            workflow_yaml: yaml.to_string(),
        });

        let resp = svc.register_workflow(req).await.unwrap();
        let inner = resp.into_inner();
        assert_eq!(inner.name, "order-fulfillment");
        assert_eq!(inner.step_count, 2);
    }

    // --- テスト7: list_workflows → 空リスト ---
    #[tokio::test]
    async fn test_list_workflows_empty() {
        let mut mock_wf_repo = MockWorkflowRepository::new();
        mock_wf_repo.expect_list().returning(|| Ok(vec![]));

        let list_workflows_uc = Arc::new(ListWorkflowsUseCase::new(Arc::new(mock_wf_repo)));

        let svc = make_tonic_service(
            make_dummy_start_uc(),
            make_dummy_get_uc(),
            make_dummy_list_sagas_uc(),
            make_dummy_cancel_uc(),
            make_dummy_register_uc(),
            list_workflows_uc,
        );

        let req = Request::new(ProtoListWorkflowsRequest {});
        let resp = svc.list_workflows(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(inner.workflows.is_empty());
    }
}
