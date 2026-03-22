// ボードサービス gRPC 実装。
// 各メソッドで proto Request をドメイン入力型に変換し、UseCase を呼び出して proto Response を返す。
use crate::domain::entity::board_column::{
    BoardColumnFilter, DecrementColumnRequest as DomainDecrementReq,
    IncrementColumnRequest as DomainIncrementReq,
    UpdateWipLimitRequest as DomainUpdateWipReq,
};
use crate::proto::k1s0::service::board::v1::board_service_server::BoardService;
use crate::proto::k1s0::service::board::v1::{
    DecrementColumnRequest, DecrementColumnResponse,
    GetBoardColumnRequest, GetBoardColumnResponse,
    IncrementColumnRequest, IncrementColumnResponse,
    ListBoardColumnsRequest, ListBoardColumnsResponse,
    UpdateWipLimitRequest, UpdateWipLimitResponse,
    BoardColumn as ProtoBoardColumn,
};
use crate::proto::k1s0::system::common::v1::{PaginationResult, Timestamp};
use crate::usecase;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

// DateTime<Utc> を proto Timestamp に変換するヘルパー
fn datetime_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

// ドメイン BoardColumn を proto BoardColumn に変換するヘルパー
fn domain_column_to_proto(c: crate::domain::entity::board_column::BoardColumn) -> ProtoBoardColumn {
    ProtoBoardColumn {
        id: c.id.to_string(),
        project_id: c.project_id.to_string(),
        status_code: c.status_code,
        wip_limit: c.wip_limit,
        task_count: c.task_count,
        version: c.version,
        created_at: Some(datetime_to_timestamp(c.created_at)),
        updated_at: Some(datetime_to_timestamp(c.updated_at)),
    }
}

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
    // カラムタスク数増加: proto Request をドメイン IncrementColumnRequest に変換して UseCase を実行する
    async fn increment_column(&self, req: Request<IncrementColumnRequest>) -> Result<Response<IncrementColumnResponse>, Status> {
        let r = req.into_inner();
        let task_id = Uuid::parse_str(&r.task_id)
            .map_err(|_| Status::invalid_argument("invalid task_id"))?;
        let project_id = Uuid::parse_str(&r.project_id)
            .map_err(|_| Status::invalid_argument("invalid project_id"))?;

        let domain_req = DomainIncrementReq {
            task_id,
            project_id,
            status_code: r.status_code,
        };

        match self.increment_column_uc.execute(&domain_req).await {
            Ok(col) => Ok(Response::new(IncrementColumnResponse {
                column: Some(domain_column_to_proto(col)),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // カラムタスク数減少: proto Request をドメイン DecrementColumnRequest に変換して UseCase を実行する
    async fn decrement_column(&self, req: Request<DecrementColumnRequest>) -> Result<Response<DecrementColumnResponse>, Status> {
        let r = req.into_inner();
        let task_id = Uuid::parse_str(&r.task_id)
            .map_err(|_| Status::invalid_argument("invalid task_id"))?;
        let project_id = Uuid::parse_str(&r.project_id)
            .map_err(|_| Status::invalid_argument("invalid project_id"))?;

        let domain_req = DomainDecrementReq {
            task_id,
            project_id,
            status_code: r.status_code,
            // proto の reason は空文字の場合は None として扱う
            reason: if r.reason.is_empty() { None } else { Some(r.reason) },
        };

        match self.decrement_column_uc.execute(&domain_req).await {
            Ok(col) => Ok(Response::new(DecrementColumnResponse {
                column: Some(domain_column_to_proto(col)),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // ボードカラム取得: column_id を UUID に変換して UseCase を実行する
    async fn get_board_column(&self, req: Request<GetBoardColumnRequest>) -> Result<Response<GetBoardColumnResponse>, Status> {
        let r = req.into_inner();
        let id = Uuid::parse_str(&r.column_id)
            .map_err(|_| Status::invalid_argument("invalid column_id"))?;

        match self.get_board_column_uc.execute(id).await {
            Ok(Some(col)) => Ok(Response::new(GetBoardColumnResponse {
                column: Some(domain_column_to_proto(col)),
            })),
            // カラムが存在しない場合は NOT_FOUND を返す
            Ok(None) => Err(Status::not_found(format!("board column '{}' not found", id))),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // ボードカラム一覧: ページネーションパラメータを BoardColumnFilter に変換して UseCase を実行する
    async fn list_board_columns(&self, req: Request<ListBoardColumnsRequest>) -> Result<Response<ListBoardColumnsResponse>, Status> {
        let r = req.into_inner();

        // project_id が指定された場合は UUID に変換する
        let project_id = if let Some(ref pid) = r.project_id {
            Some(Uuid::parse_str(pid)
                .map_err(|_| Status::invalid_argument("invalid project_id"))?)
        } else {
            None
        };

        // ページネーション情報をオフセット・リミットに変換する
        let (limit, offset) = if let Some(pagination) = r.pagination {
            let page_size = pagination.page_size as i64;
            let page = (pagination.page as i64).max(1);
            (Some(page_size), Some((page - 1) * page_size))
        } else {
            (None, None)
        };

        let filter = BoardColumnFilter {
            project_id,
            status_code: r.status_code,
            limit,
            offset,
        };

        match self.list_board_columns_uc.execute(&filter).await {
            Ok((cols, total)) => {
                let proto_cols: Vec<_> = cols.into_iter().map(domain_column_to_proto).collect();
                let page_size = limit.unwrap_or(proto_cols.len() as i64) as i32;
                let page = if let Some(off) = offset {
                    (off / limit.unwrap_or(1).max(1) + 1) as i32
                } else {
                    1
                };
                Ok(Response::new(ListBoardColumnsResponse {
                    columns: proto_cols,
                    pagination: Some(PaginationResult {
                        total_count: total,
                        page,
                        page_size,
                        has_next: offset.unwrap_or(0) + limit.unwrap_or(total) < total,
                    }),
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // WIP 制限更新: proto Request をドメイン UpdateWipLimitRequest に変換して UseCase を実行する
    async fn update_wip_limit(&self, req: Request<UpdateWipLimitRequest>) -> Result<Response<UpdateWipLimitResponse>, Status> {
        let r = req.into_inner();
        let column_id = Uuid::parse_str(&r.column_id)
            .map_err(|_| Status::invalid_argument("invalid column_id"))?;

        let domain_req = DomainUpdateWipReq {
            column_id,
            wip_limit: r.wip_limit,
            expected_version: r.expected_version,
        };

        match self.update_wip_limit_uc.execute(&domain_req).await {
            Ok(col) => Ok(Response::new(UpdateWipLimitResponse {
                column: Some(domain_column_to_proto(col)),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
