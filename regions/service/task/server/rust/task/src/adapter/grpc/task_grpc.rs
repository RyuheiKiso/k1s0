// タスクサービス gRPC 実装。
// 各メソッドで proto Request をドメイン入力型に変換し、UseCase を呼び出して proto Response を返す。
use crate::domain::entity::task::{CreateChecklistItem, CreateTask, TaskFilter, TaskPriority, TaskStatus, UpdateTaskStatus};
use crate::proto::k1s0::service::task::v1::task_service_server::TaskService;
use crate::proto::k1s0::service::task::v1::{
    CreateTaskRequest, CreateTaskResponse, GetTaskRequest, GetTaskResponse,
    ListTasksRequest, ListTasksResponse, UpdateTaskStatusRequest, UpdateTaskStatusResponse,
    Task as ProtoTask,
};
use crate::proto::k1s0::system::common::v1::{PaginationResult, Timestamp};
use crate::usecase;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

// ドメイン Task をproto Task に変換するヘルパー
fn domain_task_to_proto(t: crate::domain::entity::task::Task) -> ProtoTask {
    ProtoTask {
        id: t.id.to_string(),
        project_id: t.project_id.to_string(),
        title: t.title,
        description: t.description,
        // ドメインステータス enum を proto i32 に変換する
        status: match t.status {
            TaskStatus::Open => 1,
            TaskStatus::InProgress => 2,
            TaskStatus::Review => 3,
            TaskStatus::Done => 4,
            TaskStatus::Cancelled => 5,
        },
        // ドメイン優先度 enum を proto i32 に変換する
        priority: match t.priority {
            TaskPriority::Low => 1,
            TaskPriority::Medium => 2,
            TaskPriority::High => 3,
            TaskPriority::Critical => 4,
        },
        assignee_id: t.assignee_id,
        // エンティティの reporter_id を proto フィールドに直接マップする
        reporter_id: t.reporter_id.unwrap_or_default(),
        due_date: t.due_date.map(datetime_to_timestamp),
        // エンティティの labels を proto フィールドに直接マップする
        labels: t.labels,
        created_by: t.created_by,
        updated_by: t.updated_by,
        version: t.version,
        checklist: vec![],
        created_at: Some(datetime_to_timestamp(t.created_at)),
        updated_at: Some(datetime_to_timestamp(t.updated_at)),
    }
}

// DateTime<Utc> を proto Timestamp に変換するヘルパー
fn datetime_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

// proto TaskStatus i32 をドメイン TaskStatus に変換する。
// 変換不可の場合は None を返す。
fn proto_status_to_domain(val: i32) -> Option<TaskStatus> {
    match val {
        1 => Some(TaskStatus::Open),
        2 => Some(TaskStatus::InProgress),
        3 => Some(TaskStatus::Review),
        4 => Some(TaskStatus::Done),
        5 => Some(TaskStatus::Cancelled),
        _ => None,
    }
}

// proto TaskPriority i32 をドメイン TaskPriority に変換する。
// 変換不可の場合は Medium をデフォルトとする。
fn proto_priority_to_domain(val: i32) -> TaskPriority {
    match val {
        1 => TaskPriority::Low,
        2 => TaskPriority::Medium,
        3 => TaskPriority::High,
        4 => TaskPriority::Critical,
        _ => TaskPriority::Medium,
    }
}

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
    // タスク作成: proto Request をドメイン CreateTask に変換して UseCase を実行する
    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> Result<Response<CreateTaskResponse>, Status> {
        let req = request.into_inner();

        // project_id を UUID に変換する
        let project_id = Uuid::parse_str(&req.project_id)
            .map_err(|_| Status::invalid_argument("invalid project_id"))?;

        // チェックリスト項目をドメイン型に変換する
        let checklist = req.checklist.into_iter().map(|item| CreateChecklistItem {
            title: item.title,
            sort_order: item.sort_order.unwrap_or(0),
        }).collect();

        let input = CreateTask {
            project_id,
            title: req.title,
            description: req.description,
            priority: proto_priority_to_domain(req.priority.unwrap_or(0)),
            assignee_id: req.assignee_id,
            // gRPC 呼び出し元の reporter_id は取得不可のため None とし、UseCase 側で created_by を使用する
            reporter_id: None,
            due_date: None,
            // proto の labels をドメイン入力に渡す
            labels: req.labels,
            checklist,
        };

        // gRPC メタデータから呼び出し元ユーザーとテナントIDを取得できないため "system" を使用する
        // TODO: gRPC メタデータから tenant_id を取得する機能を実装すること
        match self.create_task_uc.execute("system", &input, "grpc").await {
            Ok(task) => Ok(Response::new(CreateTaskResponse {
                task: Some(domain_task_to_proto(task)),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // タスク取得: task_id を UUID に変換して UseCase を実行する
    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> Result<Response<GetTaskResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::parse_str(&req.task_id)
            .map_err(|_| Status::invalid_argument("invalid task_id"))?;

        // gRPC メタデータから tenant_id を取得できないため "system" を使用する
        // TODO: gRPC メタデータから tenant_id を取得する機能を実装すること
        match self.get_task_uc.execute("system", id).await {
            Ok(Some(task)) => Ok(Response::new(GetTaskResponse {
                task: Some(domain_task_to_proto(task)),
            })),
            // タスクが存在しない場合は NOT_FOUND を返す
            Ok(None) => Err(Status::not_found(format!("task '{}' not found", id))),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // タスク一覧: ページネーションパラメータを TaskFilter に変換して UseCase を実行する
    async fn list_tasks(
        &self,
        request: Request<ListTasksRequest>,
    ) -> Result<Response<ListTasksResponse>, Status> {
        let req = request.into_inner();

        // project_id が指定された場合は UUID に変換する
        let project_id = if let Some(ref pid) = req.project_id {
            Some(Uuid::parse_str(pid)
                .map_err(|_| Status::invalid_argument("invalid project_id"))?)
        } else {
            None
        };

        // ページネーション情報をオフセット・リミットに変換する
        let (limit, offset) = if let Some(pagination) = req.pagination {
            let page_size = pagination.page_size as i64;
            let page = (pagination.page as i64).max(1);
            (Some(page_size), Some((page - 1) * page_size))
        } else {
            (None, None)
        };

        let filter = TaskFilter {
            project_id,
            assignee_id: req.assignee_id,
            status: req.status.and_then(proto_status_to_domain),
            limit,
            offset,
        };

        // gRPC メタデータから tenant_id を取得できないため "system" を使用する
        // TODO: gRPC メタデータから tenant_id を取得する機能を実装すること
        match self.list_tasks_uc.execute("system", &filter).await {
            Ok((tasks, total)) => {
                let proto_tasks: Vec<_> = tasks.into_iter().map(domain_task_to_proto).collect();
                let page_size = limit.unwrap_or(proto_tasks.len() as i64) as i32;
                let page = if let Some(off) = offset {
                    (off / limit.unwrap_or(1).max(1) + 1) as i32
                } else {
                    1
                };
                Ok(Response::new(ListTasksResponse {
                    tasks: proto_tasks,
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

    // タスクステータス更新: proto 入力をドメイン UpdateTaskStatus に変換して UseCase を実行する
    async fn update_task_status(
        &self,
        request: Request<UpdateTaskStatusRequest>,
    ) -> Result<Response<UpdateTaskStatusResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::parse_str(&req.task_id)
            .map_err(|_| Status::invalid_argument("invalid task_id"))?;

        let status = proto_status_to_domain(req.status)
            .ok_or_else(|| Status::invalid_argument("invalid status value"))?;

        // proto の expected_version を楽観的ロックに使用する
        let input = UpdateTaskStatus {
            status,
            expected_version: req.expected_version,
        };

        // gRPC メタデータから tenant_id を取得できないため "system" を使用する
        // TODO: gRPC メタデータから tenant_id を取得する機能を実装すること
        match self.update_task_status_uc.execute("system", id, &input, "grpc").await {
            Ok(task) => Ok(Response::new(UpdateTaskStatusResponse {
                task: Some(domain_task_to_proto(task)),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
