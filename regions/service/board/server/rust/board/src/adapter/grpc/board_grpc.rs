// ボードサービス gRPC 実装。
use crate::proto::k1s0::service::board::v1::board_service_server::BoardService;
use crate::proto::k1s0::service::board::v1::{
    DecrementColumnRequest, DecrementColumnResponse,
    GetBoardColumnRequest, GetBoardColumnResponse,
    IncrementColumnRequest, IncrementColumnResponse,
    ListBoardColumnsRequest, ListBoardColumnsResponse,
    UpdateWipLimitRequest, UpdateWipLimitResponse,
};
use crate::usecase;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct BoardGrpcService {
    pub increment_column_uc: Arc<usecase::increment_column::IncrementColumnUseCase>,
    pub decrement_column_uc: Arc<usecase::decrement_column::DecrementColumnUseCase>,
    pub get_board_column_uc: Arc<usecase::get_board_column::GetBoardColumnUseCase>,
    pub list_board_columns_uc: Arc<usecase::list_board_columns::ListBoardColumnsUseCase>,
    pub update_wip_limit_uc: Arc<usecase::update_wip_limit::UpdateWipLimitUseCase>,
}

impl BoardGrpcService {
    pub fn new(
        increment_column_uc: Arc<usecase::increment_column::IncrementColumnUseCase>,
        decrement_column_uc: Arc<usecase::decrement_column::DecrementColumnUseCase>,
        get_board_column_uc: Arc<usecase::get_board_column::GetBoardColumnUseCase>,
        list_board_columns_uc: Arc<usecase::list_board_columns::ListBoardColumnsUseCase>,
        update_wip_limit_uc: Arc<usecase::update_wip_limit::UpdateWipLimitUseCase>,
    ) -> Self {
        Self { increment_column_uc, decrement_column_uc, get_board_column_uc, list_board_columns_uc, update_wip_limit_uc }
    }
}

#[tonic::async_trait]
impl BoardService for BoardGrpcService {
    async fn increment_column(&self, _req: Request<IncrementColumnRequest>) -> Result<Response<IncrementColumnResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
    async fn decrement_column(&self, _req: Request<DecrementColumnRequest>) -> Result<Response<DecrementColumnResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
    async fn get_board_column(&self, _req: Request<GetBoardColumnRequest>) -> Result<Response<GetBoardColumnResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
    async fn list_board_columns(&self, _req: Request<ListBoardColumnsRequest>) -> Result<Response<ListBoardColumnsResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
    async fn update_wip_limit(&self, _req: Request<UpdateWipLimitRequest>) -> Result<Response<UpdateWipLimitResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
}
