// アクティビティサービス gRPC 実装。
// 各メソッドで proto Request をドメイン入力型に変換し、UseCase を呼び出して proto Response を返す。
// RLS テナント分離のため gRPC メタデータから x-tenant-id を取得する。存在しない場合は "system" を使用する。
use crate::domain::entity::activity::{
    ActivityFilter, ActivityStatus, ActivityType, CreateActivity,
};
use crate::proto::k1s0::service::activity::v1::activity_service_server::ActivityService;
use crate::proto::k1s0::service::activity::v1::{
    ApproveActivityRequest, ApproveActivityResponse,
    CreateActivityRequest, CreateActivityResponse,
    GetActivityRequest, GetActivityResponse,
    ListActivitiesRequest, ListActivitiesResponse,
    RejectActivityRequest, RejectActivityResponse,
    SubmitActivityRequest, SubmitActivityResponse,
    Activity as ProtoActivity,
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

// ドメイン ActivityType enum を proto i32 に変換する
fn activity_type_to_i32(t: &ActivityType) -> i32 {
    match t {
        ActivityType::Comment => 1,
        ActivityType::TimeEntry => 2,
        ActivityType::StatusChange => 3,
        ActivityType::Assignment => 4,
    }
}

// ドメイン ActivityStatus enum を proto i32 に変換する
fn activity_status_to_i32(s: &ActivityStatus) -> i32 {
    match s {
        ActivityStatus::Active => 1,
        ActivityStatus::Submitted => 2,
        ActivityStatus::Approved => 3,
        ActivityStatus::Rejected => 4,
    }
}

// proto ActivityType i32 をドメイン ActivityType に変換する。
// 変換不可の場合は Comment をデフォルトとする。
fn proto_activity_type_to_domain(val: i32) -> ActivityType {
    match val {
        1 => ActivityType::Comment,
        2 => ActivityType::TimeEntry,
        3 => ActivityType::StatusChange,
        4 => ActivityType::Assignment,
        _ => ActivityType::Comment,
    }
}

// proto ActivityStatus i32 をドメイン ActivityStatus に変換する。
// 変換不可の場合は None を返す。
fn proto_activity_status_to_domain(val: i32) -> Option<ActivityStatus> {
    match val {
        1 => Some(ActivityStatus::Active),
        2 => Some(ActivityStatus::Submitted),
        3 => Some(ActivityStatus::Approved),
        4 => Some(ActivityStatus::Rejected),
        _ => None,
    }
}

// ドメイン Activity をproto Activity に変換するヘルパー
fn domain_activity_to_proto(a: crate::domain::entity::activity::Activity) -> ProtoActivity {
    ProtoActivity {
        id: a.id.to_string(),
        task_id: a.task_id.to_string(),
        actor_id: a.actor_id,
        activity_type: activity_type_to_i32(&a.activity_type),
        content: a.content,
        duration_minutes: a.duration_minutes,
        status: activity_status_to_i32(&a.status),
        metadata: None,
        idempotency_key: a.idempotency_key,
        version: a.version,
        created_at: Some(datetime_to_timestamp(a.created_at)),
        updated_at: Some(datetime_to_timestamp(a.updated_at)),
    }
}

/// gRPC メタデータから x-tenant-id を取得する。
/// 存在しない場合または不正なバイト列の場合は "system" をフォールバックとして使用し、
/// 認証ミドルウェアの設定漏れを検知するために警告ログを出力する。
fn tenant_id_from_metadata<T>(req: &Request<T>) -> String {
    let tenant_id = req.metadata()
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    match tenant_id {
        Some(id) => id,
        None => {
            // x-tenant-id が設定されていない場合は認証ミドルウェアの設定漏れを示す
            tracing::warn!(
                "x-tenant-id metadata missing, falling back to 'system'. \
                This should be set by the auth middleware."
            );
            "system".to_string()
        }
    }
}

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
    // アクティビティ作成: proto Request をドメイン CreateActivity に変換して UseCase を実行する
    async fn create_activity(&self, req: Request<CreateActivityRequest>) -> Result<Response<CreateActivityResponse>, Status> {
        // メタデータから tenant_id を取得する
        let tenant_id = tenant_id_from_metadata(&req);
        let r = req.into_inner();
        let task_id = Uuid::parse_str(&r.task_id)
            .map_err(|_| Status::invalid_argument("invalid task_id"))?;

        let input = CreateActivity {
            task_id,
            activity_type: proto_activity_type_to_domain(r.activity_type),
            content: r.content,
            duration_minutes: r.duration_minutes,
            idempotency_key: r.idempotency_key,
        };

        match self.create_activity_uc.execute(&tenant_id, &input, "grpc").await {
            Ok(activity) => Ok(Response::new(CreateActivityResponse {
                activity: Some(domain_activity_to_proto(activity)),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // アクティビティ取得: activity_id を UUID に変換して UseCase を実行する
    async fn get_activity(&self, req: Request<GetActivityRequest>) -> Result<Response<GetActivityResponse>, Status> {
        // メタデータから tenant_id を取得する
        let tenant_id = tenant_id_from_metadata(&req);
        let r = req.into_inner();
        let id = Uuid::parse_str(&r.activity_id)
            .map_err(|_| Status::invalid_argument("invalid activity_id"))?;

        match self.get_activity_uc.execute(&tenant_id, id).await {
            Ok(Some(activity)) => Ok(Response::new(GetActivityResponse {
                activity: Some(domain_activity_to_proto(activity)),
            })),
            // アクティビティが存在しない場合は NOT_FOUND を返す
            Ok(None) => Err(Status::not_found(format!("activity '{}' not found", id))),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // アクティビティ一覧: ページネーションパラメータを ActivityFilter に変換して UseCase を実行する
    async fn list_activities(&self, req: Request<ListActivitiesRequest>) -> Result<Response<ListActivitiesResponse>, Status> {
        // メタデータから tenant_id を取得する
        let tenant_id = tenant_id_from_metadata(&req);
        let r = req.into_inner();

        // task_id が指定された場合は UUID に変換する
        let task_id = if let Some(ref tid) = r.task_id {
            Some(Uuid::parse_str(tid)
                .map_err(|_| Status::invalid_argument("invalid task_id"))?)
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

        // status_enum を優先し、なければ deprecated status フィールドを使用する
        let status = r.status_enum
            .and_then(proto_activity_status_to_domain)
            .or_else(|| {
                #[allow(deprecated)]
                r.status.as_deref().and_then(|s| s.parse().ok())
            });

        let filter = ActivityFilter {
            task_id,
            actor_id: r.actor_id,
            status,
            limit,
            offset,
        };

        match self.list_activities_uc.execute(&tenant_id, &filter).await {
            Ok((activities, total)) => {
                let proto_activities: Vec<_> = activities.into_iter().map(domain_activity_to_proto).collect();
                let page_size = limit.unwrap_or(proto_activities.len() as i64) as i32;
                let page = if let Some(off) = offset {
                    (off / limit.unwrap_or(1).max(1) + 1) as i32
                } else {
                    1
                };
                Ok(Response::new(ListActivitiesResponse {
                    activities: proto_activities,
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

    // アクティビティ提出: activity_id を UUID に変換して UseCase を実行する（Active → Submitted）
    async fn submit_activity(&self, req: Request<SubmitActivityRequest>) -> Result<Response<SubmitActivityResponse>, Status> {
        // メタデータから tenant_id を取得する
        let tenant_id = tenant_id_from_metadata(&req);
        let r = req.into_inner();
        let id = Uuid::parse_str(&r.activity_id)
            .map_err(|_| Status::invalid_argument("invalid activity_id"))?;

        match self.submit_activity_uc.execute(&tenant_id, id, "grpc").await {
            Ok(activity) => Ok(Response::new(SubmitActivityResponse {
                activity: Some(domain_activity_to_proto(activity)),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // アクティビティ承認: activity_id を UUID に変換して UseCase を実行する（Submitted → Approved）
    async fn approve_activity(&self, req: Request<ApproveActivityRequest>) -> Result<Response<ApproveActivityResponse>, Status> {
        // メタデータから tenant_id を取得する
        let tenant_id = tenant_id_from_metadata(&req);
        let r = req.into_inner();
        let id = Uuid::parse_str(&r.activity_id)
            .map_err(|_| Status::invalid_argument("invalid activity_id"))?;

        match self.approve_activity_uc.execute(&tenant_id, id, "grpc").await {
            Ok(activity) => Ok(Response::new(ApproveActivityResponse {
                activity: Some(domain_activity_to_proto(activity)),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // アクティビティ却下: activity_id を UUID に変換して UseCase を実行する（Submitted → Rejected）
    async fn reject_activity(&self, req: Request<RejectActivityRequest>) -> Result<Response<RejectActivityResponse>, Status> {
        // メタデータから tenant_id を取得する
        let tenant_id = tenant_id_from_metadata(&req);
        let r = req.into_inner();
        let id = Uuid::parse_str(&r.activity_id)
            .map_err(|_| Status::invalid_argument("invalid activity_id"))?;

        match self.reject_activity_uc.execute(&tenant_id, id, "grpc", &r.reason).await {
            Ok(activity) => Ok(Response::new(RejectActivityResponse {
                activity: Some(domain_activity_to_proto(activity)),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
