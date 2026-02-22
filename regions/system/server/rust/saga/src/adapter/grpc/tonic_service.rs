use std::sync::Arc;

use tonic::{Request, Response, Status};

use super::saga_grpc::{
    CancelSagaRequest, CancelSagaResponse, GetSagaRequest, GetSagaResponse, GrpcError,
    ListSagasRequest, ListSagasResponse, ListWorkflowsRequest, ListWorkflowsResponse,
    RegisterWorkflowRequest, RegisterWorkflowResponse, SagaGrpcService, StartSagaRequest,
    StartSagaResponse,
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

// --- SagaService tonic ラッパー ---

/// SagaServiceTonic は tonic の gRPC サービスとして SagaGrpcService をラップする。
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

    /// Saga開始。
    pub async fn start_saga(
        &self,
        request: Request<StartSagaRequest>,
    ) -> Result<Response<StartSagaResponse>, Status> {
        let resp = self.inner.start_saga(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    /// Saga取得。
    pub async fn get_saga(
        &self,
        request: Request<GetSagaRequest>,
    ) -> Result<Response<GetSagaResponse>, Status> {
        let resp = self.inner.get_saga(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    /// Saga一覧取得。
    pub async fn list_sagas(
        &self,
        request: Request<ListSagasRequest>,
    ) -> Result<Response<ListSagasResponse>, Status> {
        let resp = self.inner.list_sagas(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    /// Sagaキャンセル。
    pub async fn cancel_saga(
        &self,
        request: Request<CancelSagaRequest>,
    ) -> Result<Response<CancelSagaResponse>, Status> {
        let resp = self.inner.cancel_saga(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    /// ワークフロー登録。
    pub async fn register_workflow(
        &self,
        request: Request<RegisterWorkflowRequest>,
    ) -> Result<Response<RegisterWorkflowResponse>, Status> {
        let resp = self.inner.register_workflow(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    /// ワークフロー一覧取得。
    pub async fn list_workflows(
        &self,
        request: Request<ListWorkflowsRequest>,
    ) -> Result<Response<ListWorkflowsResponse>, Status> {
        let resp = self.inner.list_workflows(request.into_inner()).await?;
        Ok(Response::new(resp))
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

        let req = Request::new(StartSagaRequest {
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

        let req = Request::new(ListSagasRequest {
            page: 1,
            page_size: 20,
            workflow_name: "".to_string(),
            status: "".to_string(),
            correlation_id: "".to_string(),
        });

        let resp = svc.list_sagas(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(inner.sagas.is_empty());
        assert_eq!(inner.total_count, 0);
        assert!(!inner.has_next);
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
        let req = Request::new(CancelSagaRequest {
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

        let req = Request::new(GetSagaRequest {
            saga_id: saga_id.to_string(),
        });

        let resp = svc.get_saga(req).await.unwrap();
        let inner = resp.into_inner();
        assert_eq!(inner.saga.id, saga_id.to_string());
        assert_eq!(inner.saga.workflow_name, "test-workflow");
        assert_eq!(inner.saga.status, "STARTED");
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

        let req = Request::new(RegisterWorkflowRequest {
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

        let req = Request::new(ListWorkflowsRequest {});
        let resp = svc.list_workflows(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(inner.workflows.is_empty());
    }
}
