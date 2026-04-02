use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::DataLoader;
use async_graphql::futures_util::Stream;
use async_graphql::{Context, Data, ErrorExtensions, FieldResult, Object, Schema, Subscription};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    // H-15 監査対応: HeaderMap を追加して ip_address / user_agent をリクエストヘッダーから抽出する
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension,
    Json,
    Router,
};

use crate::adapter::middleware::auth_middleware::{AuthMiddlewareLayer, BearerToken, Claims};
use crate::domain::model::graphql_context::{
    ConfigLoader, FeatureFlagLoader, GraphqlContext, TenantLoader,
};
// ローダー構築時にポートトレイトオブジェクトへキャストするためインポートする
// H-02 監査対応: AuditEventType/AuditResult は GraphQL スキーマの enum 定義に必要だが
// graphql_handler.rs 内では直接参照されないためインポートを削除する
use crate::domain::model::{
    ApproveTaskPayload, AuditLogConnection, CancelInstancePayload, CatalogService,
    CatalogServiceConnection, ConfigEntry, CreateChannelPayload, CreateJobPayload,
    CreateSessionPayload, CreateTemplatePayload, CreateTenantPayload, CreateWorkflowPayload,
    DeleteChannelPayload, DeleteJobPayload, DeleteSecretPayload, DeleteServicePayload,
    DeleteTemplatePayload, DeleteWorkflowPayload, FeatureFlag, Job, JobExecution, Navigation,
    NotificationChannel, NotificationLog, NotificationTemplate, PauseJobPayload, PermissionCheck,
    ReassignTaskPayload, RecordAuditLogPayload, RefreshSessionPayload, RegisterServicePayload,
    RejectTaskPayload, ResumeJobPayload, RetryNotificationPayload, RevokeAllSessionsPayload,
    RevokeSessionPayload, Role, RotateSecretPayload, SecretMetadata, SendNotificationPayload,
    ServiceHealth, Session, SetFeatureFlagPayload, SetSecretPayload, StartInstancePayload, Tenant,
    TenantConnection, TenantStatus, TriggerJobPayload, UpdateChannelPayload, UpdateJobPayload,
    UpdateServicePayload, UpdateTemplatePayload, UpdateTenantPayload, UpdateWorkflowPayload, User,
    UserError, VaultAuditLogEntry, WorkflowDefinition, WorkflowInstance, WorkflowTask,
};
use crate::domain::port::{ConfigPort, FeatureFlagPort, TenantPort};
use crate::infrastructure::auth::JwksVerifier;
use crate::infrastructure::config::GraphQLConfig;
// gRPC クライアントと HTTP クライアントをインポートする。
// service-catalog は REST のみ提供するため ServiceCatalogHttpClient を使用する。
use crate::infrastructure::grpc::{
    AuthGrpcClient, ConfigGrpcClient, FeatureFlagGrpcClient, NavigationGrpcClient,
    NotificationGrpcClient, SchedulerGrpcClient, SessionGrpcClient, TenantGrpcClient,
    VaultGrpcClient, WorkflowGrpcClient,
};
use crate::infrastructure::http::ServiceCatalogHttpClient;
use crate::usecase::{
    AuthMutationResolver, AuthQueryResolver, ConfigQueryResolver, FeatureFlagQueryResolver,
    NavigationQueryResolver, NotificationMutationResolver, NotificationQueryResolver,
    SchedulerMutationResolver, SchedulerQueryResolver, ServiceCatalogMutationResolver,
    ServiceCatalogQueryResolver, SessionMutationResolver, SessionQueryResolver,
    SubscriptionResolver, TenantMutationResolver, TenantQueryResolver, VaultMutationResolver,
    VaultQueryResolver, WorkflowMutationResolver, WorkflowQueryResolver,
};

const CODE_FORBIDDEN: &str = "FORBIDDEN";
const CODE_VALIDATION: &str = "VALIDATION_ERROR";
const CODE_BACKEND: &str = "BACKEND_ERROR";

fn gql_error(code: &'static str, message: impl Into<String>) -> async_graphql::Error {
    async_graphql::Error::new(message.into()).extend_with(|_, e| e.set("code", code))
}

/// gRPC Status コードから GraphQL エラーコードを分類する（型安全版）。
/// M-15 監査対応: tonic::Status を直接受け取りステータスコードで分類する。
/// usecase 層が tonic::Status を直接返す箇所ではこちらを使用すること。
// H-02 監査対応: classify_domain_error と対になる関数。tonic::Status を直接受け取る usecase で使用予定
#[allow(dead_code)]
fn classify_from_grpc_status(status: &tonic::Status) -> &'static str {
    use crate::domain::error::GrpcErrorCategory;
    GrpcErrorCategory::from_tonic_code(status.code()).as_graphql_code()
}

/// エラーメッセージから GraphQL エラーコードを分類するフォールバック実装。
/// M-15 監査対応: tonic::Status のエラーコード文字列表現（"status: Unauthenticated, ..."）を
/// 考慮し、GrpcErrorCategory の型安全な分類を文字列解析の前段として適用する。
/// usecase 層が anyhow::Error に変換済みの場合に使用する。
fn classify_domain_error(message: &str) -> &'static str {
    let lower = message.to_ascii_lowercase();
    // tonic::Status の to_string() 表現: "status: Unauthenticated, message: ..."
    if lower.contains("status: unauthenticated")
        || lower.contains("unauthorized")
        || lower.contains("unauthenticated")
        || lower.contains("token expired")
        || lower.contains("認証")
    {
        return "UNAUTHENTICATED";
    }
    // tonic::Status の to_string() 表現: "status: PermissionDenied, ..."
    if lower.contains("status: permissiondenied")
        || lower.contains("forbidden")
        || lower.contains("permission denied")
        || lower.contains("access denied")
        || lower.contains("権限")
    {
        return CODE_FORBIDDEN;
    }
    // tonic::Status の to_string() 表現: "status: InvalidArgument, ..."
    if lower.contains("status: invalidargument")
        || lower.contains("status: failedprecondition")
        || lower.contains("validation")
        || lower.contains("invalid")
        || lower.contains("required")
    {
        return CODE_VALIDATION;
    }
    CODE_BACKEND
}

/// k1s0_server_common の RBAC ロジックを再利用する（P2-24）。
/// ローカル重複定義を廃止し、server-common の Tier と check_permission を使用する。
use k1s0_server_common::middleware::{check_permission, Tier};

/// 読み取り操作の認可チェック。sys_admin / sys_operator / sys_auditor ロールが必要。
fn ensure_read_permission(ctx: &Context<'_>) -> FieldResult<()> {
    let roles = if let Ok(gql_ctx) = ctx.data::<GraphqlContext>() {
        gql_ctx.roles.clone()
    } else if let Ok(claims) = ctx.data::<Claims>() {
        claims.roles()
    } else {
        vec![]
    };

    if check_permission(Tier::System, &roles, "read") {
        Ok(())
    } else {
        Err(gql_error(
            CODE_FORBIDDEN,
            "insufficient permissions for this operation",
        ))
    }
}

/// 書き込み操作の認可チェック。sys_admin / sys_operator ロールが必要。
fn ensure_write_permission(ctx: &Context<'_>) -> FieldResult<()> {
    let roles = if let Ok(gql_ctx) = ctx.data::<GraphqlContext>() {
        gql_ctx.roles.clone()
    } else if let Ok(claims) = ctx.data::<Claims>() {
        claims.roles()
    } else {
        vec![]
    };

    if check_permission(Tier::System, &roles, "write") {
        Ok(())
    } else {
        Err(gql_error(
            CODE_FORBIDDEN,
            "insufficient permissions for this operation",
        ))
    }
}

// --- Input types ---

/// テナント作成の入力型
#[derive(async_graphql::InputObject)]
pub struct CreateTenantInput {
    /// テナント名（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: String,
}

/// テナント更新の入力型
#[derive(async_graphql::InputObject)]
pub struct UpdateTenantInput {
    /// テナント名（1〜255文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: Option<String>,
    pub status: Option<TenantStatus>,
}

/// フィーチャーフラグ設定の入力型
#[derive(async_graphql::InputObject)]
pub struct SetFeatureFlagInput {
    pub enabled: bool,
    /// ロールアウト割合（0〜100）
    #[graphql(validator(minimum = 0, maximum = 100))]
    pub rollout_percentage: Option<i32>,
    /// 対象環境リスト
    pub target_environments: Option<Vec<String>>,
}

/// サービス登録の入力型
#[derive(async_graphql::InputObject)]
pub struct RegisterServiceInput {
    /// サービス名（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: String,
    /// 表示名（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub display_name: String,
    /// 説明（1〜2000文字）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub description: String,
    /// ティア（1〜50文字）
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub tier: String,
    /// バージョン（1〜50文字）
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub version: String,
    /// ベースURL（1〜2000文字）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub base_url: String,
    /// gRPCエンドポイント（1〜2000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub grpc_endpoint: Option<String>,
    /// ヘルスチェックURL（1〜2000文字）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub health_url: String,
}

/// サービス更新の入力型
#[derive(async_graphql::InputObject)]
pub struct UpdateServiceInput {
    /// 表示名（1〜255文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub display_name: Option<String>,
    /// 説明（1〜2000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub description: Option<String>,
    /// バージョン（1〜50文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub version: Option<String>,
    /// ベースURL（1〜2000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub base_url: Option<String>,
    /// gRPCエンドポイント（1〜2000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub grpc_endpoint: Option<String>,
    /// ヘルスチェックURL（1〜2000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub health_url: Option<String>,
}

// --- Auth / Session Input types ---

/// 監査ログ記録の入力型
/// H-15 監査対応: クライアントが userId/ipAddress/userAgent を送信できないよう入力型から除去する。
/// - userId: JWT claims の sub フィールドから取得（なりすまし防止）
/// - ipAddress: リクエストの X-Forwarded-For / RemoteAddr ヘッダから取得
/// - userAgent: リクエストの User-Agent ヘッダから取得
#[derive(async_graphql::InputObject)]
pub struct RecordAuditLogInput {
    /// イベント種別（1〜100文字）
    #[graphql(validator(min_length = 1, max_length = 100))]
    pub event_type: String,
    /// リソース名（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub resource: String,
    /// アクション名（1〜100文字）
    #[graphql(validator(min_length = 1, max_length = 100))]
    pub action: String,
    /// 結果（1〜100文字）
    #[graphql(validator(min_length = 1, max_length = 100))]
    pub result: String,
    /// リソースID（1〜255文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub resource_id: Option<String>,
    /// トレースID（1〜255文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub trace_id: Option<String>,
}

/// セッション作成の入力型
#[derive(async_graphql::InputObject)]
pub struct CreateSessionInput {
    /// ユーザーID（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub user_id: String,
    /// デバイスID（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub device_id: String,
    /// デバイス名（1〜255文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub device_name: Option<String>,
    /// デバイス種別（1〜50文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub device_type: Option<String>,
    /// ユーザーエージェント（1〜500文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 500))]
    pub user_agent: Option<String>,
    /// IPアドレス（1〜45文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 45))]
    pub ip_address: Option<String>,
    /// TTL秒数（省略可）
    pub ttl_seconds: Option<i32>,
}

/// セッションリフレッシュの入力型
#[derive(async_graphql::InputObject)]
pub struct RefreshSessionInput {
    /// セッションID（1文字以上）
    #[graphql(validator(min_length = 1))]
    pub session_id: String,
    /// TTL秒数（省略可）
    pub ttl_seconds: Option<i32>,
}

// --- Vault Input types ---

/// シークレット設定の入力型
#[derive(async_graphql::InputObject)]
pub struct SetSecretInput {
    /// シークレットパス（1〜500文字）
    #[graphql(validator(min_length = 1, max_length = 500))]
    pub path: String,
    pub data: Vec<SecretKeyValue>,
}

/// シークレットのキー・バリューペア
#[derive(async_graphql::InputObject)]
pub struct SecretKeyValue {
    /// キー名（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub key: String,
    /// 値（1〜10000文字）
    #[graphql(validator(min_length = 1, max_length = 10000))]
    pub value: String,
}

/// シークレットローテーションの入力型
#[derive(async_graphql::InputObject)]
pub struct RotateSecretInput {
    /// シークレットパス（1〜500文字）
    #[graphql(validator(min_length = 1, max_length = 500))]
    pub path: String,
    pub data: Vec<SecretKeyValue>,
}

// --- Scheduler Input types ---

/// ジョブ作成の入力型
#[derive(async_graphql::InputObject)]
pub struct CreateJobInput {
    /// ジョブ名（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: String,
    /// 説明（1〜2000文字）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub description: String,
    /// cron式（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub cron_expression: String,
    /// タイムゾーン（1〜50文字）
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub timezone: String,
    /// ターゲット種別（1〜50文字）
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub target_type: String,
    /// ターゲット（1〜2000文字）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub target: String,
}

/// ジョブ更新の入力型
#[derive(async_graphql::InputObject)]
pub struct UpdateJobInput {
    /// ジョブ名（1〜255文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: Option<String>,
    /// 説明（1〜2000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub description: Option<String>,
    /// cron式（1〜255文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub cron_expression: Option<String>,
    /// タイムゾーン（1〜50文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub timezone: Option<String>,
    /// ターゲット種別（1〜50文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub target_type: Option<String>,
    /// ターゲット（1〜2000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub target: Option<String>,
}

// --- Notification Input types ---

/// 通知送信の入力型
#[derive(async_graphql::InputObject)]
pub struct SendNotificationInput {
    /// チャネルID（1文字以上）
    #[graphql(validator(min_length = 1))]
    pub channel_id: String,
    /// テンプレートID（1文字以上、省略可）
    #[graphql(validator(min_length = 1))]
    pub template_id: Option<String>,
    /// 受信者（1〜500文字）
    #[graphql(validator(min_length = 1, max_length = 500))]
    pub recipient: String,
    /// 件名（1〜500文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 500))]
    pub subject: Option<String>,
    /// 本文（1〜50000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 50000))]
    pub body: Option<String>,
    /// テンプレート変数（省略可）
    pub template_variables: Option<Vec<TemplateVariableInput>>,
}

/// テンプレート変数のキー・バリューペア
#[derive(async_graphql::InputObject)]
pub struct TemplateVariableInput {
    /// キー名（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub key: String,
    /// 値（1〜10000文字）
    #[graphql(validator(min_length = 1, max_length = 10000))]
    pub value: String,
}

/// 通知チャネル作成の入力型
#[derive(async_graphql::InputObject)]
pub struct CreateChannelInput {
    /// チャネル名（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: String,
    /// チャネル種別（1〜50文字）
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub channel_type: String,
    /// 設定JSON（1〜10000文字）
    #[graphql(validator(min_length = 1, max_length = 10000))]
    pub config_json: String,
    pub enabled: bool,
}

/// 通知チャネル更新の入力型
#[derive(async_graphql::InputObject)]
pub struct UpdateChannelInput {
    /// チャネル名（1〜255文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: Option<String>,
    pub enabled: Option<bool>,
    /// 設定JSON（1〜10000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 10000))]
    pub config_json: Option<String>,
}

/// 通知テンプレート作成の入力型
#[derive(async_graphql::InputObject)]
pub struct CreateTemplateInput {
    /// テンプレート名（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: String,
    /// チャネル種別（1〜50文字）
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub channel_type: String,
    /// 件名テンプレート（1〜500文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 500))]
    pub subject_template: Option<String>,
    /// 本文テンプレート（1〜50000文字）
    #[graphql(validator(min_length = 1, max_length = 50000))]
    pub body_template: String,
}

/// 通知テンプレート更新の入力型
#[derive(async_graphql::InputObject)]
pub struct UpdateTemplateInput {
    /// テンプレート名（1〜255文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: Option<String>,
    /// 件名テンプレート（1〜500文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 500))]
    pub subject_template: Option<String>,
    /// 本文テンプレート（1〜50000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 50000))]
    pub body_template: Option<String>,
}

// --- Workflow Input types ---

/// ワークフロー作成の入力型
#[derive(async_graphql::InputObject)]
pub struct CreateWorkflowInput {
    /// ワークフロー名（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: String,
    /// 説明（1〜2000文字）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub description: String,
    pub enabled: bool,
    pub steps: Vec<WorkflowStepInput>,
}

/// ワークフローステップの入力型
#[derive(Debug, async_graphql::InputObject)]
pub struct WorkflowStepInput {
    /// ステップID（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub step_id: String,
    /// ステップ名（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: String,
    /// ステップ種別（1〜50文字）
    #[graphql(validator(min_length = 1, max_length = 50))]
    pub step_type: String,
    /// 担当ロール（1〜100文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 100))]
    pub assignee_role: Option<String>,
    /// タイムアウト時間（省略可）
    pub timeout_hours: Option<i32>,
    /// 承認時の遷移先ステップ（1〜255文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub on_approve: Option<String>,
    /// 却下時の遷移先ステップ（1〜255文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub on_reject: Option<String>,
}

/// ワークフロー更新の入力型
#[derive(async_graphql::InputObject)]
pub struct UpdateWorkflowInput {
    /// ワークフロー名（1〜255文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: Option<String>,
    /// 説明（1〜2000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub steps: Option<Vec<WorkflowStepInput>>,
}

/// ワークフローインスタンス開始の入力型
#[derive(async_graphql::InputObject)]
pub struct StartInstanceInput {
    /// ワークフローID（1文字以上）
    #[graphql(validator(min_length = 1))]
    pub workflow_id: String,
    /// タイトル（1〜255文字）
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub title: String,
    /// 起票者ID（1文字以上）
    #[graphql(validator(min_length = 1))]
    pub initiator_id: String,
    /// コンテキストJSON（1〜50000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 50000))]
    pub context_json: Option<String>,
}

/// タスク再割り当ての入力型
#[derive(async_graphql::InputObject)]
pub struct ReassignTaskInput {
    /// タスクID（1文字以上）
    #[graphql(validator(min_length = 1))]
    pub task_id: String,
    /// 新しい担当者ID（1文字以上）
    #[graphql(validator(min_length = 1))]
    pub new_assignee_id: String,
    /// 理由（1〜2000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub reason: Option<String>,
    /// 操作者ID（1文字以上）
    #[graphql(validator(min_length = 1))]
    pub actor_id: String,
}

/// タスク判断（承認/却下）の入力型
#[derive(async_graphql::InputObject)]
pub struct TaskDecisionInput {
    /// タスクID（1文字以上）
    #[graphql(validator(min_length = 1))]
    pub task_id: String,
    /// 操作者ID（1文字以上）
    #[graphql(validator(min_length = 1))]
    pub actor_id: String,
    /// コメント（1〜2000文字、省略可）
    #[graphql(validator(min_length = 1, max_length = 2000))]
    pub comment: Option<String>,
}

// --- Gateway 構造体: router() の引数爆発を解消する ---

/// バックエンドクライアント群。readyz ヘルスチェックと DataLoader 生成に使用。
/// service_catalog は REST クライアント、それ以外は gRPC クライアントを使用する。
pub struct GatewayClients {
    pub tenant: Arc<TenantGrpcClient>,
    pub feature_flag: Arc<FeatureFlagGrpcClient>,
    pub config: Arc<ConfigGrpcClient>,
    pub navigation: Arc<NavigationGrpcClient>,
    // service-catalog は HTTP/axum で実装されており gRPC が存在しないため REST クライアントを使用する
    pub service_catalog: Arc<ServiceCatalogHttpClient>,
    pub auth: Arc<AuthGrpcClient>,
    pub session: Arc<SessionGrpcClient>,
    pub vault: Arc<VaultGrpcClient>,
    pub scheduler: Arc<SchedulerGrpcClient>,
    pub notification: Arc<NotificationGrpcClient>,
    pub workflow: Arc<WorkflowGrpcClient>,
}

/// GraphQL リゾルバ群。QueryRoot / MutationRoot / SubscriptionRoot を構成する。
pub struct GatewayResolvers {
    pub tenant_query: Arc<TenantQueryResolver>,
    pub feature_flag_query: Arc<FeatureFlagQueryResolver>,
    pub config_query: Arc<ConfigQueryResolver>,
    pub navigation_query: Arc<NavigationQueryResolver>,
    pub service_catalog_query: Arc<ServiceCatalogQueryResolver>,
    pub tenant_mutation: Arc<TenantMutationResolver>,
    pub service_catalog_mutation: Arc<ServiceCatalogMutationResolver>,
    pub subscription: Arc<SubscriptionResolver>,
    pub auth_query: Arc<AuthQueryResolver>,
    pub auth_mutation: Arc<AuthMutationResolver>,
    pub session_query: Arc<SessionQueryResolver>,
    pub session_mutation: Arc<SessionMutationResolver>,
    pub vault_query: Arc<VaultQueryResolver>,
    pub vault_mutation: Arc<VaultMutationResolver>,
    pub scheduler_query: Arc<SchedulerQueryResolver>,
    pub scheduler_mutation: Arc<SchedulerMutationResolver>,
    pub notification_query: Arc<NotificationQueryResolver>,
    pub notification_mutation: Arc<NotificationMutationResolver>,
    pub workflow_query: Arc<WorkflowQueryResolver>,
    pub workflow_mutation: Arc<WorkflowMutationResolver>,
}

// --- Query ---

pub struct QueryRoot {
    pub tenant_query: Arc<TenantQueryResolver>,
    pub feature_flag_query: Arc<FeatureFlagQueryResolver>,
    pub config_query: Arc<ConfigQueryResolver>,
    pub navigation_query: Arc<NavigationQueryResolver>,
    pub service_catalog_query: Arc<ServiceCatalogQueryResolver>,
    pub auth_query: Arc<AuthQueryResolver>,
    pub session_query: Arc<SessionQueryResolver>,
    pub vault_query: Arc<VaultQueryResolver>,
    pub scheduler_query: Arc<SchedulerQueryResolver>,
    pub notification_query: Arc<NotificationQueryResolver>,
    pub workflow_query: Arc<WorkflowQueryResolver>,
}

#[Object]
impl QueryRoot {
    async fn tenant(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<Option<Tenant>> {
        ensure_read_permission(ctx)?;
        self.tenant_query
            .get_tenant(id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn tenants(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
    ) -> FieldResult<TenantConnection> {
        ensure_read_permission(ctx)?;
        // ページネーション上限を 100 件にクランプしてサービス負荷を制限する（M-19 監査対応）
        let first = first.map(|n| n.clamp(1, 100));
        self.tenant_query
            .list_tenants(first, after)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn feature_flag(
        &self,
        ctx: &Context<'_>,
        key: String,
    ) -> FieldResult<Option<FeatureFlag>> {
        ensure_read_permission(ctx)?;
        self.feature_flag_query
            .get_feature_flag(&key)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn feature_flags(
        &self,
        ctx: &Context<'_>,
        environment: Option<String>,
    ) -> FieldResult<Vec<FeatureFlag>> {
        ensure_read_permission(ctx)?;
        self.feature_flag_query
            .list_feature_flags(environment.as_deref())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn config(&self, ctx: &Context<'_>, key: String) -> FieldResult<Option<ConfigEntry>> {
        ensure_read_permission(ctx)?;
        if let Ok(gql_ctx) = ctx.data::<GraphqlContext>() {
            return gql_ctx
                .config_loader
                .load_one(key.clone())
                .await
                .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()));
        }

        self.config_query
            .get_config(&key)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    /// M-3 監査対応: bearer_token GraphQL 引数を廃止。
    /// トークンをクエリ引数に含めるとアクセスログ・サーバーログに記録されるリスクがあるため、
    /// HTTP Authorization ヘッダー経由で受け取ったトークンをコンテキストから取得して転送する。
    async fn navigation(&self, ctx: &Context<'_>) -> FieldResult<Navigation> {
        ensure_read_permission(ctx)?;
        // GraphqlContext から検証済み raw トークンを取得する（引数経由のトークン漏洩防止）
        let token = ctx
            .data::<GraphqlContext>()
            .map(|c| c.bearer_token.as_str())
            .unwrap_or_default()
            .to_owned();
        self.navigation_query
            .get_navigation(&token)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn catalog_service(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<Option<CatalogService>> {
        ensure_read_permission(ctx)?;
        self.service_catalog_query
            .get_service(id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn catalog_services(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        tier: Option<String>,
        status: Option<String>,
        search: Option<String>,
    ) -> FieldResult<CatalogServiceConnection> {
        ensure_read_permission(ctx)?;
        // ページネーション上限を 100 件にクランプしてサービス負荷を制限する（M-19 監査対応）
        let first = first.map(|n| n.clamp(1, 100));
        self.service_catalog_query
            .list_services(first, tier.as_deref(), status.as_deref(), search.as_deref())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn service_health(
        &self,
        ctx: &Context<'_>,
        service_id: Option<String>,
    ) -> FieldResult<Vec<ServiceHealth>> {
        ensure_read_permission(ctx)?;
        self.service_catalog_query
            .health_check(service_id.as_deref())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    // --- Auth queries ---

    async fn user(
        &self,
        ctx: &Context<'_>,
        user_id: async_graphql::ID,
    ) -> FieldResult<Option<User>> {
        ensure_read_permission(ctx)?;
        self.auth_query
            .get_user(user_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn users(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<i32>,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> FieldResult<Vec<User>> {
        ensure_read_permission(ctx)?;
        // ページネーション上限を 100 件にクランプしてサービス負荷を制限する（M-19 監査対応）
        let first = first.map(|n| n.clamp(1, 100));
        self.auth_query
            .list_users(first, after, search.as_deref(), enabled)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn user_roles(
        &self,
        ctx: &Context<'_>,
        user_id: async_graphql::ID,
    ) -> FieldResult<Vec<Role>> {
        ensure_read_permission(ctx)?;
        self.auth_query
            .get_user_roles(user_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn check_permission(
        &self,
        ctx: &Context<'_>,
        permission: String,
        resource: String,
        roles: Vec<String>,
        user_id: Option<String>,
    ) -> FieldResult<PermissionCheck> {
        ensure_read_permission(ctx)?;
        self.auth_query
            .check_permission(user_id.as_deref(), &permission, &resource, &roles)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn search_audit_logs(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<i32>,
        user_id: Option<String>,
        event_type: Option<String>,
        result: Option<String>,
    ) -> FieldResult<AuditLogConnection> {
        ensure_read_permission(ctx)?;
        // ページネーション上限を 100 件にクランプしてサービス負荷を制限する（M-19 監査対応）
        let first = first.map(|n| n.clamp(1, 100));
        self.auth_query
            .search_audit_logs(
                first,
                after,
                user_id.as_deref(),
                event_type.as_deref(),
                result.as_deref(),
            )
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    // --- Session queries ---

    async fn session(
        &self,
        ctx: &Context<'_>,
        session_id: async_graphql::ID,
    ) -> FieldResult<Option<Session>> {
        ensure_read_permission(ctx)?;
        self.session_query
            .get_session(session_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn user_sessions(
        &self,
        ctx: &Context<'_>,
        user_id: async_graphql::ID,
    ) -> FieldResult<Vec<Session>> {
        ensure_read_permission(ctx)?;
        self.session_query
            .list_user_sessions(user_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    // --- Vault queries ---

    async fn secret_metadata(
        &self,
        ctx: &Context<'_>,
        path: String,
    ) -> FieldResult<Option<SecretMetadata>> {
        ensure_read_permission(ctx)?;
        self.vault_query
            .get_secret_metadata(&path)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn secrets(&self, ctx: &Context<'_>, prefix: Option<String>) -> FieldResult<Vec<String>> {
        ensure_read_permission(ctx)?;
        self.vault_query
            .list_secrets(prefix.as_deref())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn vault_audit_logs(
        &self,
        ctx: &Context<'_>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> FieldResult<Vec<VaultAuditLogEntry>> {
        ensure_read_permission(ctx)?;
        // ページネーション上限を 100 件にクランプしてサービス負荷を制限する（M-19 監査対応）
        let limit = limit.map(|n| n.clamp(1, 100));
        self.vault_query
            .list_audit_logs(offset, limit)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    // --- Scheduler queries ---

    async fn job(&self, ctx: &Context<'_>, job_id: async_graphql::ID) -> FieldResult<Option<Job>> {
        ensure_read_permission(ctx)?;
        self.scheduler_query
            .get_job(job_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn jobs(
        &self,
        ctx: &Context<'_>,
        status: Option<String>,
        first: Option<i32>,
        after: Option<i32>,
    ) -> FieldResult<Vec<Job>> {
        ensure_read_permission(ctx)?;
        self.scheduler_query
            .list_jobs(status.as_deref(), first, after)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn job_execution(
        &self,
        ctx: &Context<'_>,
        execution_id: async_graphql::ID,
    ) -> FieldResult<Option<JobExecution>> {
        ensure_read_permission(ctx)?;
        self.scheduler_query
            .get_job_execution(execution_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn job_executions(
        &self,
        ctx: &Context<'_>,
        job_id: async_graphql::ID,
        first: Option<i32>,
        after: Option<i32>,
        status: Option<String>,
    ) -> FieldResult<Vec<JobExecution>> {
        ensure_read_permission(ctx)?;
        self.scheduler_query
            .list_executions(job_id.as_str(), first, after, status.as_deref())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    // --- Notification queries ---

    async fn notification(
        &self,
        ctx: &Context<'_>,
        notification_id: async_graphql::ID,
    ) -> FieldResult<Option<NotificationLog>> {
        ensure_read_permission(ctx)?;
        self.notification_query
            .get_notification(notification_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn notifications(
        &self,
        ctx: &Context<'_>,
        channel_id: Option<String>,
        status: Option<String>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> FieldResult<Vec<NotificationLog>> {
        ensure_read_permission(ctx)?;
        self.notification_query
            .list_notifications(channel_id.as_deref(), status.as_deref(), page, page_size)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn notification_channel(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<Option<NotificationChannel>> {
        ensure_read_permission(ctx)?;
        self.notification_query
            .get_channel(id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn notification_channels(
        &self,
        ctx: &Context<'_>,
        channel_type: Option<String>,
        enabled_only: Option<bool>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> FieldResult<Vec<NotificationChannel>> {
        ensure_read_permission(ctx)?;
        self.notification_query
            .list_channels(
                channel_type.as_deref(),
                enabled_only.unwrap_or(false),
                page,
                page_size,
            )
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn notification_template(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<Option<NotificationTemplate>> {
        ensure_read_permission(ctx)?;
        self.notification_query
            .get_template(id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn notification_templates(
        &self,
        ctx: &Context<'_>,
        channel_type: Option<String>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> FieldResult<Vec<NotificationTemplate>> {
        ensure_read_permission(ctx)?;
        self.notification_query
            .list_templates(channel_type.as_deref(), page, page_size)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    // --- Workflow queries ---

    async fn workflow(
        &self,
        ctx: &Context<'_>,
        workflow_id: async_graphql::ID,
    ) -> FieldResult<Option<WorkflowDefinition>> {
        ensure_read_permission(ctx)?;
        self.workflow_query
            .get_workflow(workflow_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn workflows(
        &self,
        ctx: &Context<'_>,
        enabled_only: Option<bool>,
        first: Option<i32>,
        after: Option<i32>,
    ) -> FieldResult<Vec<WorkflowDefinition>> {
        ensure_read_permission(ctx)?;
        self.workflow_query
            .list_workflows(enabled_only.unwrap_or(false), first, after)
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn workflow_instance(
        &self,
        ctx: &Context<'_>,
        instance_id: async_graphql::ID,
    ) -> FieldResult<Option<WorkflowInstance>> {
        ensure_read_permission(ctx)?;
        self.workflow_query
            .get_instance(instance_id.as_str())
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    async fn workflow_instances(
        &self,
        ctx: &Context<'_>,
        status: Option<String>,
        workflow_id: Option<String>,
        initiator_id: Option<String>,
        first: Option<i32>,
        after: Option<i32>,
    ) -> FieldResult<Vec<WorkflowInstance>> {
        ensure_read_permission(ctx)?;
        self.workflow_query
            .list_instances(
                status.as_deref(),
                workflow_id.as_deref(),
                initiator_id.as_deref(),
                first,
                after,
            )
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }

    #[allow(clippy::too_many_arguments)]
    async fn workflow_tasks(
        &self,
        ctx: &Context<'_>,
        assignee_id: Option<String>,
        status: Option<String>,
        instance_id: Option<String>,
        overdue_only: Option<bool>,
        first: Option<i32>,
        after: Option<i32>,
    ) -> FieldResult<Vec<WorkflowTask>> {
        ensure_read_permission(ctx)?;
        self.workflow_query
            .list_tasks(
                assignee_id.as_deref(),
                status.as_deref(),
                instance_id.as_deref(),
                overdue_only.unwrap_or(false),
                first,
                after,
            )
            .await
            .map_err(|e| gql_error(classify_domain_error(&e.to_string()), e.to_string()))
    }
}

// --- Mutation ---

pub struct MutationRoot {
    pub tenant_mutation: Arc<TenantMutationResolver>,
    pub feature_flag_client: Arc<FeatureFlagGrpcClient>,
    pub service_catalog_mutation: Arc<ServiceCatalogMutationResolver>,
    pub auth_mutation: Arc<AuthMutationResolver>,
    pub session_mutation: Arc<SessionMutationResolver>,
    pub vault_mutation: Arc<VaultMutationResolver>,
    pub scheduler_mutation: Arc<SchedulerMutationResolver>,
    pub notification_mutation: Arc<NotificationMutationResolver>,
    pub workflow_mutation: Arc<WorkflowMutationResolver>,
}

#[Object]
impl MutationRoot {
    async fn create_tenant(
        &self,
        ctx: &Context<'_>,
        input: CreateTenantInput,
    ) -> FieldResult<CreateTenantPayload> {
        ensure_write_permission(ctx)?;
        let claims = ctx.data::<Claims>().ok();
        let owner_user_id = claims.map(|c| c.sub.as_str()).unwrap_or("unknown");
        Ok(self
            .tenant_mutation
            .create_tenant(&input.name, owner_user_id)
            .await)
    }

    async fn update_tenant(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
        input: UpdateTenantInput,
    ) -> FieldResult<UpdateTenantPayload> {
        ensure_write_permission(ctx)?;
        let status_str = input.status.map(|s| match s {
            TenantStatus::Active => "ACTIVE".to_string(),
            TenantStatus::Suspended => "SUSPENDED".to_string(),
            TenantStatus::Deleted => "DELETED".to_string(),
        });
        Ok(self
            .tenant_mutation
            .update_tenant(id.as_str(), input.name.as_deref(), status_str.as_deref())
            .await)
    }

    async fn set_feature_flag(
        &self,
        ctx: &Context<'_>,
        key: String,
        input: SetFeatureFlagInput,
    ) -> FieldResult<SetFeatureFlagPayload> {
        ensure_write_permission(ctx)?;
        match self
            .feature_flag_client
            .set_flag(
                &key,
                input.enabled,
                input.rollout_percentage,
                input.target_environments,
            )
            .await
        {
            Ok(flag) => Ok(SetFeatureFlagPayload {
                feature_flag: Some(flag),
                errors: vec![],
            }),
            Err(e) => {
                let msg = e.to_string();
                Ok(SetFeatureFlagPayload {
                    feature_flag: None,
                    errors: vec![UserError {
                        field: None,
                        message: msg,
                    }],
                })
            }
        }
    }

    async fn register_service(
        &self,
        ctx: &Context<'_>,
        input: RegisterServiceInput,
    ) -> FieldResult<RegisterServicePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .service_catalog_mutation
            .register_service(
                &input.name,
                &input.display_name,
                &input.description,
                &input.tier,
                &input.version,
                &input.base_url,
                input.grpc_endpoint.as_deref(),
                &input.health_url,
                HashMap::new(),
            )
            .await)
    }

    async fn update_service(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
        input: UpdateServiceInput,
    ) -> FieldResult<UpdateServicePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .service_catalog_mutation
            .update_service(
                id.as_str(),
                input.display_name.as_deref(),
                input.description.as_deref(),
                input.version.as_deref(),
                input.base_url.as_deref(),
                input.grpc_endpoint.as_deref(),
                input.health_url.as_deref(),
                HashMap::new(),
            )
            .await)
    }

    async fn delete_service(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<DeleteServicePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .service_catalog_mutation
            .delete_service(id.as_str())
            .await)
    }

    // --- Auth mutations ---

    async fn record_audit_log(
        &self,
        ctx: &Context<'_>,
        input: RecordAuditLogInput,
    ) -> FieldResult<RecordAuditLogPayload> {
        ensure_write_permission(ctx)?;
        // H-15 監査対応: userId/ipAddress/userAgent はクライアント入力ではなくサーバーサイドで取得する
        // GraphqlContext に JWT claims と HTTP ヘッダーから抽出した値が格納されている
        let gql_ctx = ctx
            .data::<GraphqlContext>()
            .map_err(|_| async_graphql::Error::new("コンテキストの取得に失敗しました"))?;
        Ok(self
            .auth_mutation
            .record_audit_log(
                &input.event_type,
                &gql_ctx.user_id,
                &gql_ctx.ip_address,
                &gql_ctx.user_agent,
                &input.resource,
                &input.action,
                &input.result,
                input.resource_id.as_deref(),
                input.trace_id.as_deref(),
            )
            .await)
    }

    // --- Session mutations ---

    async fn create_session(
        &self,
        ctx: &Context<'_>,
        input: CreateSessionInput,
    ) -> FieldResult<CreateSessionPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .session_mutation
            .create_session(
                &input.user_id,
                &input.device_id,
                input.device_name.as_deref(),
                input.device_type.as_deref(),
                input.user_agent.as_deref(),
                input.ip_address.as_deref(),
                input.ttl_seconds,
            )
            .await)
    }

    async fn refresh_session(
        &self,
        ctx: &Context<'_>,
        input: RefreshSessionInput,
    ) -> FieldResult<RefreshSessionPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .session_mutation
            .refresh_session(&input.session_id, input.ttl_seconds)
            .await)
    }

    async fn revoke_session(
        &self,
        ctx: &Context<'_>,
        session_id: async_graphql::ID,
    ) -> FieldResult<RevokeSessionPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .session_mutation
            .revoke_session(session_id.as_str())
            .await)
    }

    async fn revoke_all_sessions(
        &self,
        ctx: &Context<'_>,
        user_id: async_graphql::ID,
    ) -> FieldResult<RevokeAllSessionsPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .session_mutation
            .revoke_all_sessions(user_id.as_str())
            .await)
    }

    // --- Vault mutations ---

    async fn set_secret(
        &self,
        ctx: &Context<'_>,
        input: SetSecretInput,
    ) -> FieldResult<SetSecretPayload> {
        ensure_write_permission(ctx)?;
        let data: HashMap<String, String> = input
            .data
            .into_iter()
            .map(|kv| (kv.key, kv.value))
            .collect();
        Ok(self.vault_mutation.set_secret(&input.path, data).await)
    }

    async fn rotate_secret(
        &self,
        ctx: &Context<'_>,
        input: RotateSecretInput,
    ) -> FieldResult<RotateSecretPayload> {
        ensure_write_permission(ctx)?;
        let data: HashMap<String, String> = input
            .data
            .into_iter()
            .map(|kv| (kv.key, kv.value))
            .collect();
        Ok(self.vault_mutation.rotate_secret(&input.path, data).await)
    }

    async fn delete_secret(
        &self,
        ctx: &Context<'_>,
        path: String,
        versions: Option<Vec<i64>>,
    ) -> FieldResult<DeleteSecretPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .vault_mutation
            .delete_secret(&path, versions.unwrap_or_default())
            .await)
    }

    // --- Scheduler mutations ---

    async fn create_job(
        &self,
        ctx: &Context<'_>,
        input: CreateJobInput,
    ) -> FieldResult<CreateJobPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .scheduler_mutation
            .create_job(
                &input.name,
                &input.description,
                &input.cron_expression,
                &input.timezone,
                &input.target_type,
                &input.target,
            )
            .await)
    }

    async fn update_job(
        &self,
        ctx: &Context<'_>,
        job_id: async_graphql::ID,
        input: UpdateJobInput,
    ) -> FieldResult<UpdateJobPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .scheduler_mutation
            .update_job(
                job_id.as_str(),
                input.name.as_deref(),
                input.description.as_deref(),
                input.cron_expression.as_deref(),
                input.timezone.as_deref(),
                input.target_type.as_deref(),
                input.target.as_deref(),
            )
            .await)
    }

    async fn delete_job(
        &self,
        ctx: &Context<'_>,
        job_id: async_graphql::ID,
    ) -> FieldResult<DeleteJobPayload> {
        ensure_write_permission(ctx)?;
        Ok(self.scheduler_mutation.delete_job(job_id.as_str()).await)
    }

    async fn pause_job(
        &self,
        ctx: &Context<'_>,
        job_id: async_graphql::ID,
    ) -> FieldResult<PauseJobPayload> {
        ensure_write_permission(ctx)?;
        Ok(self.scheduler_mutation.pause_job(job_id.as_str()).await)
    }

    async fn resume_job(
        &self,
        ctx: &Context<'_>,
        job_id: async_graphql::ID,
    ) -> FieldResult<ResumeJobPayload> {
        ensure_write_permission(ctx)?;
        Ok(self.scheduler_mutation.resume_job(job_id.as_str()).await)
    }

    async fn trigger_job(
        &self,
        ctx: &Context<'_>,
        job_id: async_graphql::ID,
    ) -> FieldResult<TriggerJobPayload> {
        ensure_write_permission(ctx)?;
        Ok(self.scheduler_mutation.trigger_job(job_id.as_str()).await)
    }

    // --- Notification mutations ---

    async fn send_notification(
        &self,
        ctx: &Context<'_>,
        input: SendNotificationInput,
    ) -> FieldResult<SendNotificationPayload> {
        ensure_write_permission(ctx)?;
        let vars: HashMap<String, String> = input
            .template_variables
            .unwrap_or_default()
            .into_iter()
            .map(|kv| (kv.key, kv.value))
            .collect();
        Ok(self
            .notification_mutation
            .send_notification(
                &input.channel_id,
                input.template_id.as_deref(),
                &vars,
                &input.recipient,
                input.subject.as_deref(),
                input.body.as_deref(),
            )
            .await)
    }

    async fn retry_notification(
        &self,
        ctx: &Context<'_>,
        notification_id: async_graphql::ID,
    ) -> FieldResult<RetryNotificationPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .notification_mutation
            .retry_notification(notification_id.as_str())
            .await)
    }

    async fn create_channel(
        &self,
        ctx: &Context<'_>,
        input: CreateChannelInput,
    ) -> FieldResult<CreateChannelPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .notification_mutation
            .create_channel(
                &input.name,
                &input.channel_type,
                &input.config_json,
                input.enabled,
            )
            .await)
    }

    async fn update_channel(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
        input: UpdateChannelInput,
    ) -> FieldResult<UpdateChannelPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .notification_mutation
            .update_channel(
                id.as_str(),
                input.name.as_deref(),
                input.enabled,
                input.config_json.as_deref(),
            )
            .await)
    }

    async fn delete_channel(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<DeleteChannelPayload> {
        ensure_write_permission(ctx)?;
        Ok(self.notification_mutation.delete_channel(id.as_str()).await)
    }

    async fn create_template(
        &self,
        ctx: &Context<'_>,
        input: CreateTemplateInput,
    ) -> FieldResult<CreateTemplatePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .notification_mutation
            .create_template(
                &input.name,
                &input.channel_type,
                input.subject_template.as_deref(),
                &input.body_template,
            )
            .await)
    }

    async fn update_template(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
        input: UpdateTemplateInput,
    ) -> FieldResult<UpdateTemplatePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .notification_mutation
            .update_template(
                id.as_str(),
                input.name.as_deref(),
                input.subject_template.as_deref(),
                input.body_template.as_deref(),
            )
            .await)
    }

    async fn delete_template(
        &self,
        ctx: &Context<'_>,
        id: async_graphql::ID,
    ) -> FieldResult<DeleteTemplatePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .notification_mutation
            .delete_template(id.as_str())
            .await)
    }

    // --- Workflow mutations ---

    async fn create_workflow(
        &self,
        ctx: &Context<'_>,
        input: CreateWorkflowInput,
    ) -> FieldResult<CreateWorkflowPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .create_workflow(&input.name, &input.description, input.enabled, &input.steps)
            .await)
    }

    async fn update_workflow(
        &self,
        ctx: &Context<'_>,
        workflow_id: async_graphql::ID,
        input: UpdateWorkflowInput,
    ) -> FieldResult<UpdateWorkflowPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .update_workflow(
                workflow_id.as_str(),
                input.name.as_deref(),
                input.description.as_deref(),
                input.enabled,
                input.steps.as_deref(),
            )
            .await)
    }

    async fn delete_workflow(
        &self,
        ctx: &Context<'_>,
        workflow_id: async_graphql::ID,
    ) -> FieldResult<DeleteWorkflowPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .delete_workflow(workflow_id.as_str())
            .await)
    }

    async fn start_workflow_instance(
        &self,
        ctx: &Context<'_>,
        input: StartInstanceInput,
    ) -> FieldResult<StartInstancePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .start_instance(
                &input.workflow_id,
                &input.title,
                &input.initiator_id,
                input.context_json.as_deref(),
            )
            .await)
    }

    async fn cancel_workflow_instance(
        &self,
        ctx: &Context<'_>,
        instance_id: async_graphql::ID,
        reason: Option<String>,
    ) -> FieldResult<CancelInstancePayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .cancel_instance(instance_id.as_str(), reason.as_deref())
            .await)
    }

    async fn reassign_task(
        &self,
        ctx: &Context<'_>,
        input: ReassignTaskInput,
    ) -> FieldResult<ReassignTaskPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .reassign_task(
                &input.task_id,
                &input.new_assignee_id,
                input.reason.as_deref(),
                &input.actor_id,
            )
            .await)
    }

    async fn approve_task(
        &self,
        ctx: &Context<'_>,
        input: TaskDecisionInput,
    ) -> FieldResult<ApproveTaskPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .approve_task(&input.task_id, &input.actor_id, input.comment.as_deref())
            .await)
    }

    async fn reject_task(
        &self,
        ctx: &Context<'_>,
        input: TaskDecisionInput,
    ) -> FieldResult<RejectTaskPayload> {
        ensure_write_permission(ctx)?;
        Ok(self
            .workflow_mutation
            .reject_task(&input.task_id, &input.actor_id, input.comment.as_deref())
            .await)
    }
}

// --- Subscription ---

pub struct SubscriptionRoot {
    pub subscription: Arc<SubscriptionResolver>,
}

#[Subscription]
impl SubscriptionRoot {
    /// 設定変更イベントをストリームで購読する。
    /// 接続失敗時は GraphQL エラーとして返し、ストリーム中のエラーはアイテムレベルで伝播する（P2-26）。
    /// 購読には読み取り権限（sys_admin / sys_operator / sys_auditor）が必要。
    #[graphql(name = "configChanged")]
    async fn config_changed(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] namespaces: Vec<String>,
    ) -> async_graphql::Result<impl Stream<Item = async_graphql::Result<ConfigEntry>>> {
        // RBAC チェック: subscription にも read 権限を要求する（query と同等の保護）
        ensure_read_permission(ctx)?;
        let stream = self
            .subscription
            .watch_config(namespaces)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        // tonic::Status エラーを async_graphql::Error に変換してサブスクライバーに伝播する
        use async_graphql::futures_util::StreamExt;
        Ok(stream.map(|item| {
            item.map_err(|status| async_graphql::Error::new(format!("stream error: {status}")))
        }))
    }

    /// テナント更新イベントをストリームで購読する。gRPC 接続失敗時は GraphQL エラーとして返す。
    /// 購読には読み取り権限（sys_admin / sys_operator / sys_auditor）が必要。
    #[graphql(name = "tenantUpdated")]
    async fn tenant_updated(
        &self,
        ctx: &Context<'_>,
        tenant_id: async_graphql::ID,
    ) -> async_graphql::Result<impl Stream<Item = Tenant>> {
        // RBAC チェック: subscription にも read 権限を要求する（query と同等の保護）
        ensure_read_permission(ctx)?;
        self.subscription
            .watch_tenant_updated(tenant_id.to_string())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// フィーチャーフラグ変更イベントをストリームで購読する。gRPC 接続失敗時は GraphQL エラーとして返す。
    /// 購読には読み取り権限（sys_admin / sys_operator / sys_auditor）が必要。
    #[graphql(name = "featureFlagChanged")]
    async fn feature_flag_changed(
        &self,
        ctx: &Context<'_>,
        key: String,
    ) -> async_graphql::Result<impl Stream<Item = FeatureFlag>> {
        // RBAC チェック: subscription にも read 権限を要求する（query と同等の保護）
        ensure_read_permission(ctx)?;
        self.subscription
            .watch_feature_flag_changed(key)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

/// アプリケーション共有状態。GraphQL スキーマと Prometheus メトリクスを保持する。
#[derive(Clone)]
pub struct AppState {
    pub schema: AppSchema,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub query_timeout: std::time::Duration,
    pub jwks_verifier: Arc<JwksVerifier>,
    pub tenant_client: Arc<TenantGrpcClient>,
    pub feature_flag_client: Arc<FeatureFlagGrpcClient>,
    pub config_client: Arc<ConfigGrpcClient>,
    pub navigation_client: Arc<NavigationGrpcClient>,
    // service-catalog は REST クライアントを使用する
    pub service_catalog_client: Arc<ServiceCatalogHttpClient>,
    pub auth_client: Arc<AuthGrpcClient>,
    pub session_client: Arc<SessionGrpcClient>,
    pub vault_client: Arc<VaultGrpcClient>,
    pub scheduler_client: Arc<SchedulerGrpcClient>,
    pub notification_client: Arc<NotificationGrpcClient>,
    pub workflow_client: Arc<WorkflowGrpcClient>,
    pub tenant_loader: Arc<DataLoader<TenantLoader>>,
    pub flag_loader: Arc<DataLoader<FeatureFlagLoader>>,
    pub config_loader: Arc<DataLoader<ConfigLoader>>,
}

pub fn router(
    jwks_verifier: Arc<JwksVerifier>,
    clients: GatewayClients,
    resolvers: GatewayResolvers,
    graphql_cfg: GraphQLConfig,
    metrics: Arc<k1s0_telemetry::metrics::Metrics>,
) -> Router {
    let mut builder = Schema::build(
        QueryRoot {
            tenant_query: resolvers.tenant_query,
            feature_flag_query: resolvers.feature_flag_query,
            config_query: resolvers.config_query,
            navigation_query: resolvers.navigation_query,
            service_catalog_query: resolvers.service_catalog_query,
            auth_query: resolvers.auth_query,
            session_query: resolvers.session_query,
            vault_query: resolvers.vault_query,
            scheduler_query: resolvers.scheduler_query,
            notification_query: resolvers.notification_query,
            workflow_query: resolvers.workflow_query,
        },
        MutationRoot {
            tenant_mutation: resolvers.tenant_mutation,
            feature_flag_client: clients.feature_flag.clone(),
            service_catalog_mutation: resolvers.service_catalog_mutation,
            auth_mutation: resolvers.auth_mutation,
            session_mutation: resolvers.session_mutation,
            vault_mutation: resolvers.vault_mutation,
            scheduler_mutation: resolvers.scheduler_mutation,
            notification_mutation: resolvers.notification_mutation,
            workflow_mutation: resolvers.workflow_mutation,
        },
        SubscriptionRoot {
            subscription: resolvers.subscription,
        },
    )
    .limit_depth(graphql_cfg.max_depth as usize)
    .limit_complexity(graphql_cfg.max_complexity as usize);

    if !graphql_cfg.introspection {
        builder = builder.disable_introspection();
    }

    let schema = builder.finish();

    let query_timeout = std::time::Duration::from_secs(graphql_cfg.query_timeout_seconds as u64);
    // 各 gRPC クライアントをポートトレイトオブジェクトにキャストし、DataLoader に渡す。
    // これにより domain 層はインフラ層の具象型に依存せず抽象に依存できる。
    let tenant_port: Arc<dyn TenantPort> = clients.tenant.clone();
    let feature_flag_port: Arc<dyn FeatureFlagPort> = clients.feature_flag.clone();
    let config_port: Arc<dyn ConfigPort> = clients.config.clone();
    let tenant_loader = Arc::new(DataLoader::new(
        TenantLoader {
            client: tenant_port,
        },
        tokio::spawn,
    ));
    let flag_loader = Arc::new(DataLoader::new(
        FeatureFlagLoader {
            client: feature_flag_port,
        },
        tokio::spawn,
    ));
    let config_loader = Arc::new(DataLoader::new(
        ConfigLoader {
            client: config_port,
        },
        tokio::spawn,
    ));

    let app_state = AppState {
        schema: schema.clone(),
        metrics,
        query_timeout,
        jwks_verifier: jwks_verifier.clone(),
        tenant_client: clients.tenant,
        feature_flag_client: clients.feature_flag,
        config_client: clients.config,
        navigation_client: clients.navigation,
        service_catalog_client: clients.service_catalog,
        auth_client: clients.auth,
        session_client: clients.session,
        vault_client: clients.vault,
        scheduler_client: clients.scheduler,
        notification_client: clients.notification,
        workflow_client: clients.workflow,
        tenant_loader,
        flag_loader,
        config_loader,
    };

    let graphql_post = post(graphql_handler).layer(AuthMiddlewareLayer::new(jwks_verifier));

    let mut router = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics_handler))
        .route("/graphql", graphql_post)
        .route("/graphql/ws", get(graphql_ws_handler))
        .with_state(app_state);

    // 開発環境のみ Playground を有効化
    if graphql_cfg.playground {
        router = router.route("/graphql", get(graphql_playground));
    }

    router
}

async fn graphql_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    // M-3 監査対応: raw トークンをコンテキスト経由で取得し、navigation 等の下流サービスへ転送する
    Extension(bearer_token): Extension<BearerToken>,
    // H-15 監査対応: ip_address と user_agent をリクエストヘッダーから抽出してコンテキストに注入する
    // クライアントが偽装できないサーバーサイド情報として監査ログに使用する
    headers: HeaderMap,
    req: GraphQLRequest,
) -> impl IntoResponse {
    // H-15 監査対応: X-Forwarded-For が存在する場合はプロキシ経由の実際のクライアント IP を使用する
    // 存在しない場合は "unknown" をフォールバックとする（ConnectInfo は別途ミドルウェアで対応が必要なため省略）
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    // H-15 監査対応: User-Agent ヘッダーからクライアントエージェント文字列を取得する
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let request = req
        .into_inner()
        .data(GraphqlContext {
            user_id: claims.sub.clone(),
            roles: claims.roles(),
            request_id: uuid::Uuid::new_v4().to_string(),
            bearer_token: bearer_token.0,
            ip_address,
            user_agent,
            tenant_loader: state.tenant_loader.clone(),
            flag_loader: state.flag_loader.clone(),
            config_loader: state.config_loader.clone(),
        })
        .data(claims);
    match tokio::time::timeout(state.query_timeout, state.schema.execute(request)).await {
        Ok(resp) => GraphQLResponse::from(resp).into_response(),
        Err(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "data": null,
                "errors": [{
                    "message": "query execution timed out",
                    "extensions": { "code": "TIMEOUT" }
                }]
            })),
        )
            .into_response(),
    }
}

async fn graphql_playground() -> impl IntoResponse {
    Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql")
            .subscription_endpoint("/graphql/ws"),
    ))
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // HIGH-001 対応: 全バックエンドサービスを tokio::join! で並列チェックし、
    // 直列実行による応答時間の累積（約4.1秒）を解消する。
    // 各チェックには2秒のタイムアウトを設定し、5秒の healthcheck timeout に収まるようにする。
    // essential サービス（auth/session/tenant/config）の失敗のみ not_ready とし、
    // non-essential サービスの失敗は degraded として HTTP 200 で返す。
    // MED-005 対応: workflow など1サービスの障害で gateway 全体が not_ready になる問題を解消する。
    use std::time::Duration;

    // 各バックエンドチェックのタイムアウト（2秒）
    let timeout = Duration::from_secs(2);

    // 全11サービスを同時に並列実行する
    let (
        auth_result,
        session_result,
        tenant_result,
        config_result,
        featureflag_result,
        navigation_result,
        service_catalog_result,
        vault_result,
        scheduler_result,
        notification_result,
        workflow_result,
    ) = tokio::join!(
        tokio::time::timeout(timeout, state.auth_client.health_check()),
        tokio::time::timeout(timeout, state.session_client.health_check()),
        tokio::time::timeout(timeout, state.tenant_client.list_tenants(1, 1)),
        tokio::time::timeout(
            timeout,
            state.config_client.get_config("__readyz__", "__readyz__")
        ),
        tokio::time::timeout(timeout, state.feature_flag_client.list_flags(None)),
        tokio::time::timeout(timeout, state.navigation_client.get_navigation("")),
        tokio::time::timeout(
            timeout,
            state.service_catalog_client.list_services(1, 1, None, None, None)
        ),
        tokio::time::timeout(timeout, state.vault_client.health_check()),
        tokio::time::timeout(timeout, state.scheduler_client.health_check()),
        tokio::time::timeout(timeout, state.notification_client.health_check()),
        tokio::time::timeout(timeout, state.workflow_client.health_check()),
    );

    // Result<Result<T, E>, Elapsed> をステータス文字列に変換するローカルヘルパー関数
    fn to_status<T, E: std::fmt::Display>(
        r: Result<Result<T, E>, tokio::time::error::Elapsed>,
    ) -> String {
        match r {
            Ok(Ok(_)) => "ok".to_string(),
            Ok(Err(e)) => format!("error: {}", e),
            Err(_) => "error: timeout".to_string(),
        }
    }

    // 各サービスの結果をステータス文字列に変換する
    let auth_status = to_status(auth_result);
    let session_status = to_status(session_result);
    let tenant_status = to_status(tenant_result);
    let config_status = to_status(config_result);
    let featureflag_status = to_status(featureflag_result);
    let navigation_status = to_status(navigation_result);
    let service_catalog_status = to_status(service_catalog_result);
    let vault_status = to_status(vault_result);
    let scheduler_status = to_status(scheduler_result);
    let notification_status = to_status(notification_result);
    let workflow_status = to_status(workflow_result);

    // essential サービス（基幹機能）が全て ok の場合のみ ready または degraded とする
    let essential_ok = auth_status == "ok"
        && session_status == "ok"
        && tenant_status == "ok"
        && config_status == "ok";

    // non-essential を含む全サービスが ok かどうか確認する
    let all_ok = essential_ok
        && featureflag_status == "ok"
        && navigation_status == "ok"
        && service_catalog_status == "ok"
        && vault_status == "ok"
        && scheduler_status == "ok"
        && notification_status == "ok"
        && workflow_status == "ok";

    // ステータスと HTTP ステータスコードを決定する:
    // - 全サービス ok → "ready" / 200
    // - essential のみ ok → "degraded" / 200（non-essential の障害は許容）
    // - essential に失敗あり → "not_ready" / 503
    let (status_str, status_code) = if all_ok {
        ("ready", StatusCode::OK)
    } else if essential_ok {
        ("degraded", StatusCode::OK)
    } else {
        ("not_ready", StatusCode::SERVICE_UNAVAILABLE)
    };

    (
        status_code,
        Json(serde_json::json!({
            "status": status_str,
            "checks": {
                "auth_grpc": auth_status,
                "session_grpc": session_status,
                "tenant_grpc": tenant_status,
                "config_grpc": config_status,
                "featureflag_grpc": featureflag_status,
                "navigation_grpc": navigation_status,
                // service-catalog は REST 接続のため key 名を http に変更
                "service_catalog_http": service_catalog_status,
                "vault_grpc": vault_status,
                "scheduler_grpc": scheduler_status,
                "notification_grpc": notification_status,
                "workflow_grpc": workflow_status,
            }
        })),
    )
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

async fn graphql_ws_handler(
    State(state): State<AppState>,
    protocol: GraphQLProtocol,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let schema = state.schema.clone();
    let verifier = state.jwks_verifier.clone();
    let tenant_loader = state.tenant_loader.clone();
    let flag_loader = state.flag_loader.clone();
    let config_loader = state.config_loader.clone();

    ws.protocols(async_graphql::http::ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |socket| async move {
            GraphQLWebSocket::new(socket, schema, protocol)
                .on_connection_init(move |payload| {
                    let verifier = verifier.clone();
                    let tenant_loader = tenant_loader.clone();
                    let flag_loader = flag_loader.clone();
                    let config_loader = config_loader.clone();
                    async move {
                        let token = extract_bearer_token_from_connection_init(&payload)
                            .ok_or_else(|| {
                                gql_error(
                                    CODE_VALIDATION,
                                    "missing bearer token in connection_init payload",
                                )
                            })?;

                        let claims = verifier.verify_token(&token).await.map_err(|_| {
                            gql_error(CODE_FORBIDDEN, "invalid or expired JWT token")
                        })?;

                        let mut data = Data::default();
                        data.insert(GraphqlContext {
                            user_id: claims.sub.clone(),
                            roles: claims.roles(),
                            request_id: uuid::Uuid::new_v4().to_string(),
                            // WebSocket 接続の場合、token は connection_init ペイロードから取得済み
                            bearer_token: token.clone(),
                            // WebSocket 接続では HTTP ヘッダーが利用不可のためデフォルト値を使用する
                            ip_address: String::new(),
                            user_agent: String::new(),
                            tenant_loader,
                            flag_loader,
                            config_loader,
                        });
                        data.insert(claims);
                        Ok(data)
                    }
                })
                .serve()
                .await;
        })
}

fn extract_bearer_token_from_connection_init(payload: &serde_json::Value) -> Option<String> {
    fn normalize(v: &str) -> String {
        v.trim().to_ascii_lowercase()
    }

    fn pick_token(value: &serde_json::Value) -> Option<String> {
        let token = value.as_str()?.trim();
        if token.is_empty() {
            return None;
        }
        if let Some(bearer) = token
            .strip_prefix("Bearer ")
            .or_else(|| token.strip_prefix("bearer "))
        {
            let trimmed = bearer.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        } else {
            Some(token.to_string())
        }
    }

    let obj = payload.as_object()?;

    for (key, value) in obj {
        let key_norm = normalize(key);
        if matches!(
            key_norm.as_str(),
            "authorization" | "authtoken" | "token" | "bearer_token"
        ) {
            if let Some(token) = pick_token(value) {
                return Some(token);
            }
        }
    }

    if let Some(headers) = obj.get("headers").and_then(|v| v.as_object()) {
        for (key, value) in headers {
            let key_norm = normalize(key);
            if key_norm == "authorization" {
                if let Some(token) = pick_token(value) {
                    return Some(token);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::extract_bearer_token_from_connection_init;
    use serde_json::json;

    #[test]
    fn extracts_token_from_authorization_bearer() {
        let payload = json!({
            "authorization": "Bearer abc.def.ghi"
        });
        assert_eq!(
            extract_bearer_token_from_connection_init(&payload).as_deref(),
            Some("abc.def.ghi")
        );
    }

    #[test]
    fn extracts_token_from_headers_authorization() {
        let payload = json!({
            "headers": {
                "Authorization": "bearer token-123"
            }
        });
        assert_eq!(
            extract_bearer_token_from_connection_init(&payload).as_deref(),
            Some("token-123")
        );
    }

    #[test]
    fn returns_none_when_token_missing() {
        let payload = json!({
            "headers": {
                "x-request-id": "req-1"
            }
        });
        assert!(extract_bearer_token_from_connection_init(&payload).is_none());
    }
}
