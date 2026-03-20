//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の SagaService トレイトを実装する。
//! 各メソッドで proto 型 ↔ 手動型の変換を行い、既存の SagaGrpcService に委譲する。

use std::collections::BTreeMap;
use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};
use crate::proto::k1s0::system::saga::v1::{
    saga_service_server::SagaService, CancelSagaRequest as ProtoCancelSagaRequest,
    CancelSagaResponse as ProtoCancelSagaResponse,
    CompensateSagaRequest as ProtoCompensateSagaRequest,
    CompensateSagaResponse as ProtoCompensateSagaResponse, GetSagaRequest as ProtoGetSagaRequest,
    GetSagaResponse as ProtoGetSagaResponse, ListSagasRequest as ProtoListSagasRequest,
    ListSagasResponse as ProtoListSagasResponse, ListWorkflowsRequest as ProtoListWorkflowsRequest,
    ListWorkflowsResponse as ProtoListWorkflowsResponse,
    RegisterWorkflowRequest as ProtoRegisterWorkflowRequest,
    RegisterWorkflowResponse as ProtoRegisterWorkflowResponse, SagaStateProto as ProtoSagaState,
    SagaStepLogProto as ProtoSagaStepLog, StartSagaRequest as ProtoStartSagaRequest,
    StartSagaResponse as ProtoStartSagaResponse, WorkflowSummary as ProtoWorkflowSummary,
};

use super::saga_grpc::{
    CancelSagaRequest, CompensateSagaRequest, GetSagaRequest, GrpcError, ListSagasRequest,
    ListWorkflowsRequest, RegisterWorkflowRequest, SagaGrpcService, StartSagaRequest,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::FailedPrecondition(msg) => Status::failed_precondition(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- 変換ヘルパー ---

/// RFC3339 文字列を proto Timestamp に変換する。
fn rfc3339_to_proto_timestamp(s: &str) -> Option<ProtoTimestamp> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| ProtoTimestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        })
}

fn json_to_prost_value(value: &serde_json::Value) -> prost_types::Value {
    let kind = match value {
        serde_json::Value::Null => prost_types::value::Kind::NullValue(0),
        serde_json::Value::Bool(v) => prost_types::value::Kind::BoolValue(*v),
        serde_json::Value::Number(v) => {
            prost_types::value::Kind::NumberValue(v.as_f64().unwrap_or(0.0))
        }
        serde_json::Value::String(v) => prost_types::value::Kind::StringValue(v.clone()),
        serde_json::Value::Array(values) => {
            let values = values.iter().map(json_to_prost_value).collect();
            prost_types::value::Kind::ListValue(prost_types::ListValue { values })
        }
        serde_json::Value::Object(map) => {
            let fields = map
                .iter()
                .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
                .collect();
            prost_types::value::Kind::StructValue(prost_types::Struct { fields })
        }
    };
    prost_types::Value { kind: Some(kind) }
}

fn json_to_prost_struct(value: &serde_json::Value) -> prost_types::Struct {
    match value {
        serde_json::Value::Object(map) => {
            let fields: BTreeMap<String, prost_types::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
                .collect();
            prost_types::Struct { fields }
        }
        _ => {
            let mut fields = BTreeMap::new();
            fields.insert("value".to_string(), json_to_prost_value(value));
            prost_types::Struct { fields }
        }
    }
}

fn prost_value_to_json(value: &prost_types::Value) -> serde_json::Value {
    match &value.kind {
        Some(prost_types::value::Kind::NullValue(_)) => serde_json::Value::Null,
        Some(prost_types::value::Kind::BoolValue(v)) => serde_json::Value::Bool(*v),
        Some(prost_types::value::Kind::NumberValue(v)) => serde_json::json!(*v),
        Some(prost_types::value::Kind::StringValue(v)) => serde_json::Value::String(v.clone()),
        Some(prost_types::value::Kind::ListValue(list)) => {
            serde_json::Value::Array(list.values.iter().map(prost_value_to_json).collect())
        }
        Some(prost_types::value::Kind::StructValue(v)) => prost_struct_to_json(v),
        None => serde_json::Value::Null,
    }
}

fn prost_struct_to_json(value: &prost_types::Struct) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = value
        .fields
        .iter()
        .map(|(k, v)| (k.clone(), prost_value_to_json(v)))
        .collect();
    serde_json::Value::Object(map)
}

fn json_bytes_to_prost_struct(bytes: &[u8]) -> Option<prost_types::Struct> {
    if bytes.is_empty() {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    Some(json_to_prost_struct(&value))
}

fn empty_string_to_none(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
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
    #[allow(dead_code)]
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
        let payload = inner
            .payload
            .as_ref()
            .map(prost_struct_to_json)
            .unwrap_or_else(|| serde_json::json!({}));
        let payload = serde_json::to_vec(&payload)
            .map_err(|e| Status::invalid_argument(format!("invalid payload: {}", e)))?;
        let req = StartSagaRequest {
            workflow_name: inner.workflow_name,
            payload,
            correlation_id: inner.correlation_id,
            initiated_by: inner.initiated_by,
        };
        let resp = self
            .inner
            .start_saga(req)
            .await
            .map_err(Into::<Status>::into)?;
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
        let resp = self
            .inner
            .get_saga(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_saga = ProtoSagaState {
            id: resp.saga.id,
            workflow_name: resp.saga.workflow_name,
            current_step: resp.saga.current_step,
            status: resp.saga.status,
            payload: json_bytes_to_prost_struct(&resp.saga.payload),
            correlation_id: empty_string_to_none(resp.saga.correlation_id),
            initiated_by: empty_string_to_none(resp.saga.initiated_by),
            error_message: empty_string_to_none(resp.saga.error_message),
            created_at: rfc3339_to_proto_timestamp(&resp.saga.created_at),
            updated_at: rfc3339_to_proto_timestamp(&resp.saga.updated_at),
            // 後方互換フィールド（0 = UNSPECIFIED）
            status_enum: 0,
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
                request_payload: json_bytes_to_prost_struct(&log.request_payload),
                response_payload: json_bytes_to_prost_struct(&log.response_payload),
                error_message: empty_string_to_none(log.error_message),
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
            workflow_name: inner.workflow_name.unwrap_or_default(),
            status: inner.status.unwrap_or_default(),
            correlation_id: inner.correlation_id.unwrap_or_default(),
        };
        let resp = self
            .inner
            .list_sagas(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_sagas = resp
            .sagas
            .into_iter()
            .map(|saga| ProtoSagaState {
                id: saga.id,
                workflow_name: saga.workflow_name,
                current_step: saga.current_step,
                status: saga.status,
                payload: json_bytes_to_prost_struct(&saga.payload),
                correlation_id: empty_string_to_none(saga.correlation_id),
                initiated_by: empty_string_to_none(saga.initiated_by),
                error_message: empty_string_to_none(saga.error_message),
                created_at: rfc3339_to_proto_timestamp(&saga.created_at),
                updated_at: rfc3339_to_proto_timestamp(&saga.updated_at),
                // 後方互換フィールド（0 = UNSPECIFIED）
                status_enum: 0,
            })
            .collect();

        let proto_pagination = Some(ProtoPaginationResult {
            total_count: resp.total_count,
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
        let resp = self
            .inner
            .cancel_saga(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoCancelSagaResponse {
            success: resp.success,
            message: resp.message,
        }))
    }

    async fn compensate_saga(
        &self,
        request: Request<ProtoCompensateSagaRequest>,
    ) -> Result<Response<ProtoCompensateSagaResponse>, Status> {
        let req = CompensateSagaRequest {
            saga_id: request.into_inner().saga_id,
        };
        let resp = self
            .inner
            .compensate_saga(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoCompensateSagaResponse {
            success: resp.success,
            status: resp.status,
            message: resp.message,
            saga_id: resp.saga_id,
        }))
    }

    async fn register_workflow(
        &self,
        request: Request<ProtoRegisterWorkflowRequest>,
    ) -> Result<Response<ProtoRegisterWorkflowResponse>, Status> {
        let req = RegisterWorkflowRequest {
            workflow_yaml: request.into_inner().workflow_yaml,
        };
        let resp = self
            .inner
            .register_workflow(req)
            .await
            .map_err(Into::<Status>::into)?;
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
        let resp = self
            .inner
            .list_workflows(req)
            .await
            .map_err(Into::<Status>::into)?;
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::saga_state::SagaState;
    use crate::domain::entity::workflow::WorkflowDefinition;
    use crate::domain::repository::saga_repository::MockSagaRepository;
    use crate::domain::repository::workflow_repository::MockWorkflowRepository;
    use crate::infrastructure::grpc_caller::MockGrpcStepCaller;
    use crate::usecase::ExecuteSagaUseCase;
    use crate::usecase::{
        CancelSagaUseCase, GetSagaUseCase, ListSagasUseCase, ListWorkflowsUseCase,
        RegisterWorkflowUseCase, StartSagaUseCase,
    };
    use tonic::Code;

    fn make_tonic_service(
        start_saga_uc: Arc<StartSagaUseCase>,
        get_saga_uc: Arc<GetSagaUseCase>,
        list_sagas_uc: Arc<ListSagasUseCase>,
        cancel_saga_uc: Arc<CancelSagaUseCase>,
        register_workflow_uc: Arc<RegisterWorkflowUseCase>,
        list_workflows_uc: Arc<ListWorkflowsUseCase>,
    ) -> SagaServiceTonic {
        let execute_saga_uc = make_dummy_execute_uc();
        let grpc_svc = Arc::new(SagaGrpcService::new(
            start_saga_uc,
            get_saga_uc,
            list_sagas_uc,
            cancel_saga_uc,
            execute_saga_uc,
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

    fn make_dummy_execute_uc() -> Arc<ExecuteSagaUseCase> {
        Arc::new(ExecuteSagaUseCase::new(
            Arc::new(MockSagaRepository::new()),
            Arc::new(MockGrpcStepCaller::new()),
            None,
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
            payload: Some(json_to_prost_struct(
                &serde_json::json!({"order_id": "123"}),
            )),
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
            workflow_name: None,
            status: None,
            correlation_id: None,
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
