// タスクサービス gRPC 実装。
use crate::proto::k1s0::service::task::v1::task_service_server::TaskService;
use crate::proto::k1s0::service::task::v1::{
    CreateTaskRequest, CreateTaskResponse, GetTaskRequest, GetTaskResponse,
    ListTasksRequest, ListTasksResponse, UpdateTaskStatusRequest, UpdateTaskStatusResponse,
};
use crate::usecase;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct TaskGrpcService {
    pub create_task_uc: Arc<usecase::create_task::CreateTaskUseCase>,
    pub get_task_uc: Arc<usecase::get_task::GetTaskUseCase>,
    pub list_tasks_uc: Arc<usecase::list_tasks::ListTasksUseCase>,
    pub update_task_status_uc: Arc<usecase::update_task_status::UpdateTaskStatusUseCase>,
}

impl TaskGrpcService {
    pub fn new(
        create_task_uc: Arc<usecase::create_task::CreateTaskUseCase>,
        get_task_uc: Arc<usecase::get_task::GetTaskUseCase>,
        list_tasks_uc: Arc<usecase::list_tasks::ListTasksUseCase>,
        update_task_status_uc: Arc<usecase::update_task_status::UpdateTaskStatusUseCase>,
    ) -> Self {
        Self { create_task_uc, get_task_uc, list_tasks_uc, update_task_status_uc }
    }
}

#[tonic::async_trait]
impl TaskService for TaskGrpcService {
    async fn create_task(
        &self,
        _request: Request<CreateTaskRequest>,
    ) -> Result<Response<CreateTaskResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn get_task(
        &self,
        _request: Request<GetTaskRequest>,
    ) -> Result<Response<GetTaskResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn list_tasks(
        &self,
        _request: Request<ListTasksRequest>,
    ) -> Result<Response<ListTasksResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn update_task_status(
        &self,
        _request: Request<UpdateTaskStatusRequest>,
    ) -> Result<Response<UpdateTaskStatusResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
}
