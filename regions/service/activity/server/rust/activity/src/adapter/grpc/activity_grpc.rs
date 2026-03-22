use crate::proto::k1s0::service::activity::v1::activity_service_server::ActivityService;
use crate::proto::k1s0::service::activity::v1::{
    ApproveActivityRequest, ApproveActivityResponse,
    CreateActivityRequest, CreateActivityResponse,
    GetActivityRequest, GetActivityResponse,
    ListActivitiesRequest, ListActivitiesResponse,
    RejectActivityRequest, RejectActivityResponse,
    SubmitActivityRequest, SubmitActivityResponse,
};
use crate::usecase;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct ActivityGrpcService {
    pub create_activity_uc: Arc<usecase::create_activity::CreateActivityUseCase>,
    pub get_activity_uc: Arc<usecase::get_activity::GetActivityUseCase>,
    pub list_activities_uc: Arc<usecase::list_activities::ListActivitiesUseCase>,
    pub submit_activity_uc: Arc<usecase::submit_activity::SubmitActivityUseCase>,
    pub approve_activity_uc: Arc<usecase::approve_activity::ApproveActivityUseCase>,
    pub reject_activity_uc: Arc<usecase::reject_activity::RejectActivityUseCase>,
}

impl ActivityGrpcService {
    pub fn new(
        create_activity_uc: Arc<usecase::create_activity::CreateActivityUseCase>,
        get_activity_uc: Arc<usecase::get_activity::GetActivityUseCase>,
        list_activities_uc: Arc<usecase::list_activities::ListActivitiesUseCase>,
        submit_activity_uc: Arc<usecase::submit_activity::SubmitActivityUseCase>,
        approve_activity_uc: Arc<usecase::approve_activity::ApproveActivityUseCase>,
        reject_activity_uc: Arc<usecase::reject_activity::RejectActivityUseCase>,
    ) -> Self {
        Self { create_activity_uc, get_activity_uc, list_activities_uc, submit_activity_uc, approve_activity_uc, reject_activity_uc }
    }
}

#[tonic::async_trait]
impl ActivityService for ActivityGrpcService {
    async fn create_activity(&self, _req: Request<CreateActivityRequest>) -> Result<Response<CreateActivityResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
    async fn get_activity(&self, _req: Request<GetActivityRequest>) -> Result<Response<GetActivityResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
    async fn list_activities(&self, _req: Request<ListActivitiesRequest>) -> Result<Response<ListActivitiesResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
    async fn submit_activity(&self, _req: Request<SubmitActivityRequest>) -> Result<Response<SubmitActivityResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
    async fn approve_activity(&self, _req: Request<ApproveActivityRequest>) -> Result<Response<ApproveActivityResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
    async fn reject_activity(&self, _req: Request<RejectActivityRequest>) -> Result<Response<RejectActivityResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
}
