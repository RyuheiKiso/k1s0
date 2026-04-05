use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::evaluation::EvaluationContext;
use crate::domain::entity::feature_flag::{FlagRule, FlagVariant};
use crate::usecase::create_flag::{CreateFlagError, CreateFlagInput, CreateFlagUseCase};
use crate::usecase::delete_flag::{DeleteFlagError, DeleteFlagUseCase};
use crate::usecase::evaluate_flag::{EvaluateFlagError, EvaluateFlagInput, EvaluateFlagUseCase};
use crate::usecase::get_flag::{GetFlagError, GetFlagUseCase};
use crate::usecase::list_flags::{ListFlagsError, ListFlagsUseCase};
use crate::usecase::update_flag::{UpdateFlagError, UpdateFlagInput, UpdateFlagUseCase};
use crate::usecase::watch_feature_flag::FeatureFlagChangeEvent;

use super::watch_stream::WatchFeatureFlagStreamHandler;

/// gRPC リクエストの tenant_id 文字列を UUID フォーマットで検証して String として返すヘルパー。
/// ADR-0028 Phase 1: gRPC メタデータ x-tenant-id から取得した文字列を検証する。
/// HIGH-005 対応: 検証後は String 型を返す（ドメイン層は TEXT 型で保持するため）。
fn parse_tenant_id(tenant_id_str: &str) -> Result<String, GrpcError> {
    Uuid::parse_str(tenant_id_str)
        .map(|_| tenant_id_str.to_string())
        .map_err(|_| GrpcError::InvalidArgument("tenant_id の形式が不正です".to_string()))
}

/// gRPC メタデータから tenant_id を取得するヘルパー。
/// HIGH-012 監査対応: "system" テナントへのフォールバックを廃止し、x-tenant-id が未設定の場合は
/// UNAUTHENTICATED エラーを返す。x-tenant-id は Kong の JWT 検証後に認証ミドルウェアがセットする。
/// フォールバックが存在すると認証バイパスや別テナントへの不正アクセスが可能になるため廃止した。
pub fn tenant_id_from_metadata(metadata: &tonic::metadata::MetadataMap) -> Result<String, GrpcError> {
    let tenant_id_str = metadata
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            tracing::error!(
                "x-tenant-id metadata missing or empty. \
                Must be set by auth middleware after Kong JWT validation."
            );
            GrpcError::Unauthenticated("x-tenant-id header is required".to_string())
        })?;
    parse_tenant_id(tenant_id_str)
}

#[derive(Debug, Clone)]
pub struct EvaluateFlagRequest {
    /// HIGH-005 対応: tenant_id は String 型（migration 006 で DB の TEXT 型に変更済み）。
    pub tenant_id: String,
    pub flag_key: String,
    pub user_id: String,
    pub context_tenant_id: String,
    pub attributes: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EvaluateFlagResponse {
    pub flag_key: String,
    pub enabled: bool,
    pub variant: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct GetFlagRequest {
    /// HIGH-005 対応: tenant_id は String 型（migration 006 で DB の TEXT 型に変更済み）。
    pub tenant_id: String,
    pub flag_key: String,
}

#[derive(Debug, Clone)]
pub struct GetFlagResponse {
    pub id: String,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<PbFlagVariant>,
    pub rules: Vec<PbFlagRule>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct PbFlagVariant {
    pub name: String,
    pub value: String,
    pub weight: i32,
}

#[derive(Debug, Clone)]
pub struct PbFlagRule {
    pub attribute: String,
    pub operator: String,
    pub value: String,
    pub variant: String,
}

#[derive(Debug, Clone)]
pub struct ListFlagsRequest {
    /// HIGH-005 対応: tenant_id は String 型（migration 006 で DB の TEXT 型に変更済み）。
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct ListFlagsResponse {
    pub flags: Vec<GetFlagResponse>,
}

#[derive(Debug, Clone)]
pub struct CreateFlagRequest {
    /// HIGH-005 対応: tenant_id は String 型（migration 006 で DB の TEXT 型に変更済み）。
    pub tenant_id: String,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<PbFlagVariant>,
}

#[derive(Debug, Clone)]
pub struct CreateFlagResponse {
    pub id: String,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<PbFlagVariant>,
    pub rules: Vec<PbFlagRule>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct UpdateFlagRequest {
    /// HIGH-005 対応: tenant_id は String 型（migration 006 で DB の TEXT 型に変更済み）。
    pub tenant_id: String,
    pub flag_key: String,
    pub enabled: Option<bool>,
    pub description: Option<String>,
    pub variants: Vec<PbFlagVariant>,
    pub rules: Vec<PbFlagRule>,
}

#[derive(Debug, Clone)]
pub struct UpdateFlagResponse {
    pub id: String,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<PbFlagVariant>,
    pub rules: Vec<PbFlagRule>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct DeleteFlagRequest {
    /// HIGH-005 対応: tenant_id は String 型（migration 006 で DB の TEXT 型に変更済み）。
    pub tenant_id: String,
    pub flag_key: String,
}

#[derive(Debug, Clone)]
pub struct DeleteFlagResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),
}

pub struct FeatureFlagGrpcService {
    list_flags_uc: Arc<ListFlagsUseCase>,
    evaluate_flag_uc: Arc<EvaluateFlagUseCase>,
    get_flag_uc: Arc<GetFlagUseCase>,
    create_flag_uc: Arc<CreateFlagUseCase>,
    update_flag_uc: Arc<UpdateFlagUseCase>,
    delete_flag_uc: Arc<DeleteFlagUseCase>,
    watch_sender: Option<tokio::sync::broadcast::Sender<FeatureFlagChangeEvent>>,
}

impl FeatureFlagGrpcService {
    #[allow(dead_code)]
    pub fn new(
        list_flags_uc: Arc<ListFlagsUseCase>,
        evaluate_flag_uc: Arc<EvaluateFlagUseCase>,
        get_flag_uc: Arc<GetFlagUseCase>,
        create_flag_uc: Arc<CreateFlagUseCase>,
        update_flag_uc: Arc<UpdateFlagUseCase>,
        delete_flag_uc: Arc<DeleteFlagUseCase>,
    ) -> Self {
        Self {
            list_flags_uc,
            evaluate_flag_uc,
            get_flag_uc,
            create_flag_uc,
            update_flag_uc,
            delete_flag_uc,
            watch_sender: None,
        }
    }

    pub fn new_with_watch(
        list_flags_uc: Arc<ListFlagsUseCase>,
        evaluate_flag_uc: Arc<EvaluateFlagUseCase>,
        get_flag_uc: Arc<GetFlagUseCase>,
        create_flag_uc: Arc<CreateFlagUseCase>,
        update_flag_uc: Arc<UpdateFlagUseCase>,
        delete_flag_uc: Arc<DeleteFlagUseCase>,
        watch_sender: tokio::sync::broadcast::Sender<FeatureFlagChangeEvent>,
    ) -> Self {
        Self {
            list_flags_uc,
            evaluate_flag_uc,
            get_flag_uc,
            create_flag_uc,
            update_flag_uc,
            delete_flag_uc,
            watch_sender: Some(watch_sender),
        }
    }

    pub fn watch_feature_flag(
        &self,
        flag_key: Option<String>,
    ) -> Result<WatchFeatureFlagStreamHandler, GrpcError> {
        match &self.watch_sender {
            Some(sender) => {
                let receiver = sender.subscribe();
                let filter = flag_key.filter(|k| !k.is_empty());
                Ok(WatchFeatureFlagStreamHandler::new(receiver, filter))
            }
            None => Err(GrpcError::Internal(
                "watch_feature_flag is not enabled on this server".to_string(),
            )),
        }
    }

    /// STATIC-CRITICAL-001 監査対応: テナントスコープでフィーチャーフラグを評価する。
    /// EvaluateFlagInput に tenant_id を含めてユースケースに渡す。
    pub async fn evaluate_flag(
        &self,
        req: EvaluateFlagRequest,
    ) -> Result<EvaluateFlagResponse, GrpcError> {
        let input = EvaluateFlagInput {
            tenant_id: req.tenant_id.clone(),
            flag_key: req.flag_key,
            context: EvaluationContext {
                user_id: if req.user_id.is_empty() {
                    None
                } else {
                    Some(req.user_id)
                },
                tenant_id: if req.context_tenant_id.is_empty() {
                    None
                } else {
                    Some(req.context_tenant_id)
                },
                attributes: req.attributes,
            },
        };

        match self.evaluate_flag_uc.execute(&input).await {
            Ok(result) => Ok(EvaluateFlagResponse {
                flag_key: result.flag_key,
                enabled: result.enabled,
                variant: result.variant,
                reason: result.reason,
            }),
            Err(EvaluateFlagError::FlagNotFound(key)) => {
                Err(GrpcError::NotFound(format!("flag not found: {}", key)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// STATIC-CRITICAL-001 監査対応: テナントスコープでフィーチャーフラグを取得する。
    pub async fn get_flag(&self, req: GetFlagRequest) -> Result<GetFlagResponse, GrpcError> {
        match self.get_flag_uc.execute(&req.tenant_id, &req.flag_key).await {
            Ok(flag) => Ok(GetFlagResponse {
                id: flag.id.to_string(),
                flag_key: flag.flag_key,
                description: flag.description,
                enabled: flag.enabled,
                variants: to_pb_variants(&flag.variants),
                rules: to_pb_rules(&flag.rules),
                created_at: flag.created_at,
                updated_at: flag.updated_at,
            }),
            Err(GetFlagError::NotFound(key)) => {
                Err(GrpcError::NotFound(format!("flag not found: {}", key)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// STATIC-CRITICAL-001 監査対応: テナントスコープのフィーチャーフラグ一覧を取得する。
    pub async fn list_flags(&self, req: ListFlagsRequest) -> Result<ListFlagsResponse, GrpcError> {
        let flags = self.list_flags_uc.execute(&req.tenant_id).await.map_err(|e| match e {
            ListFlagsError::Internal(msg) => GrpcError::Internal(msg),
        })?;

        Ok(ListFlagsResponse {
            flags: flags
                .into_iter()
                .map(|flag| GetFlagResponse {
                    id: flag.id.to_string(),
                    flag_key: flag.flag_key,
                    description: flag.description,
                    enabled: flag.enabled,
                    variants: to_pb_variants(&flag.variants),
                    rules: to_pb_rules(&flag.rules),
                    created_at: flag.created_at,
                    updated_at: flag.updated_at,
                })
                .collect(),
        })
    }

    /// STATIC-CRITICAL-001 監査対応: テナントスコープでフィーチャーフラグを作成する。
    pub async fn create_flag(
        &self,
        req: CreateFlagRequest,
    ) -> Result<CreateFlagResponse, GrpcError> {
        let input = CreateFlagInput {
            tenant_id: req.tenant_id.clone(),
            flag_key: req.flag_key,
            description: req.description,
            enabled: req.enabled,
            variants: req
                .variants
                .into_iter()
                .map(|v| FlagVariant {
                    name: v.name,
                    value: v.value,
                    weight: v.weight,
                })
                .collect(),
        };

        match self.create_flag_uc.execute(&input).await {
            Ok(flag) => Ok(CreateFlagResponse {
                id: flag.id.to_string(),
                flag_key: flag.flag_key,
                description: flag.description,
                enabled: flag.enabled,
                variants: to_pb_variants(&flag.variants),
                rules: to_pb_rules(&flag.rules),
                created_at: flag.created_at,
                updated_at: flag.updated_at,
            }),
            Err(CreateFlagError::AlreadyExists(key)) => Err(GrpcError::AlreadyExists(format!(
                "flag already exists: {}",
                key
            ))),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// STATIC-CRITICAL-001 監査対応: テナントスコープでフィーチャーフラグを更新する。
    pub async fn update_flag(
        &self,
        req: UpdateFlagRequest,
    ) -> Result<UpdateFlagResponse, GrpcError> {
        let input = UpdateFlagInput {
            tenant_id: req.tenant_id.clone(),
            flag_key: req.flag_key,
            enabled: req.enabled,
            description: req.description,
            variants: if req.variants.is_empty() {
                None
            } else {
                Some(
                    req.variants
                        .into_iter()
                        .map(|v| FlagVariant {
                            name: v.name,
                            value: v.value,
                            weight: v.weight,
                        })
                        .collect(),
                )
            },
            rules: if req.rules.is_empty() {
                None
            } else {
                Some(
                    req.rules
                        .into_iter()
                        .map(|r| FlagRule {
                            attribute: r.attribute,
                            operator: r.operator,
                            value: r.value,
                            variant: r.variant,
                        })
                        .collect(),
                )
            },
        };

        match self.update_flag_uc.execute(&input).await {
            Ok(flag) => Ok(UpdateFlagResponse {
                id: flag.id.to_string(),
                flag_key: flag.flag_key,
                description: flag.description,
                enabled: flag.enabled,
                variants: to_pb_variants(&flag.variants),
                rules: to_pb_rules(&flag.rules),
                created_at: flag.created_at,
                updated_at: flag.updated_at,
            }),
            Err(UpdateFlagError::NotFound(key)) => {
                Err(GrpcError::NotFound(format!("flag not found: {}", key)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// STATIC-CRITICAL-001 監査対応: テナントスコープでフィーチャーフラグを削除する。
    /// get_flag でフラグの存在確認後、delete_flag_uc で削除を実行する。
    pub async fn delete_flag(
        &self,
        req: DeleteFlagRequest,
    ) -> Result<DeleteFlagResponse, GrpcError> {
        let flag = self
            .get_flag_uc
            .execute(&req.tenant_id, &req.flag_key)
            .await
            .map_err(|e| match e {
                GetFlagError::NotFound(key) => {
                    GrpcError::NotFound(format!("flag not found: {}", key))
                }
                _ => GrpcError::Internal(e.to_string()),
            })?;

        self.delete_flag_uc
            .execute(&req.tenant_id, &flag.id)
            .await
            .map_err(|e| match e {
                DeleteFlagError::NotFound(id) => {
                    GrpcError::NotFound(format!("flag not found: {}", id))
                }
                DeleteFlagError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(DeleteFlagResponse {
            success: true,
            message: format!("flag {} deleted", req.flag_key),
        })
    }
}

fn to_pb_variants(variants: &[FlagVariant]) -> Vec<PbFlagVariant> {
    variants
        .iter()
        .map(|v| PbFlagVariant {
            name: v.name.clone(),
            value: v.value.clone(),
            weight: v.weight,
        })
        .collect()
}

fn to_pb_rules(rules: &[FlagRule]) -> Vec<PbFlagRule> {
    rules
        .iter()
        .map(|r| PbFlagRule {
            attribute: r.attribute.clone(),
            operator: r.operator.clone(),
            value: r.value.clone(),
            variant: r.variant.clone(),
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::feature_flag::FeatureFlag;
    use crate::domain::entity::flag_audit_log::FlagAuditLog;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;
    use crate::domain::repository::FlagAuditLogRepository;
    use crate::infrastructure::kafka_producer::NoopFlagEventPublisher;
    use std::collections::HashMap;

    /// システムテナント文字列: テスト共通（HIGH-005 対応: TEXT 型）
    fn system_tenant() -> String {
        "00000000-0000-0000-0000-000000000001".to_string()
    }

    struct NoopAuditRepo;

    #[async_trait::async_trait]
    impl FlagAuditLogRepository for NoopAuditRepo {
        async fn create(&self, _log: &FlagAuditLog) -> anyhow::Result<()> {
            Ok(())
        }

        async fn list_by_flag_id(
            &self,
            _flag_id: &uuid::Uuid,
            _limit: i64,
            _offset: i64,
        ) -> anyhow::Result<Vec<FlagAuditLog>> {
            Ok(vec![])
        }
    }

    fn make_service(mock: MockFeatureFlagRepository) -> FeatureFlagGrpcService {
        let repo = Arc::new(mock);
        let audit_repo = Arc::new(NoopAuditRepo);
        FeatureFlagGrpcService::new(
            Arc::new(ListFlagsUseCase::new(repo.clone())),
            Arc::new(EvaluateFlagUseCase::new(repo.clone())),
            Arc::new(GetFlagUseCase::new(repo.clone())),
            Arc::new(CreateFlagUseCase::new(
                repo.clone(),
                Arc::new(NoopFlagEventPublisher),
                audit_repo.clone(),
            )),
            Arc::new(UpdateFlagUseCase::new(
                repo.clone(),
                Arc::new(NoopFlagEventPublisher),
                audit_repo.clone(),
            )),
            Arc::new(DeleteFlagUseCase::new(
                repo,
                Arc::new(NoopFlagEventPublisher),
                audit_repo,
            )),
        )
    }

    /// FeatureFlag エンティティをテスト用に生成するヘルパー。
    fn make_flag(flag_key: &str, enabled: bool) -> FeatureFlag {
        FeatureFlag::new(
            system_tenant(),
            flag_key.to_string(),
            format!("{} description", flag_key),
            enabled,
        )
    }

    #[tokio::test]
    async fn test_evaluate_flag_success() {
        let mut mock = MockFeatureFlagRepository::new();
        let mut flag = make_flag("dark-mode", true);
        flag.variants
            .push(crate::domain::entity::feature_flag::FlagVariant {
                name: "on".to_string(),
                value: "true".to_string(),
                weight: 100,
            });
        let return_flag = flag.clone();

        // STATIC-CRITICAL-001: tenant_id を含む2引数シグネチャ
        mock.expect_find_by_key()
            .withf(|_tid, key| key == "dark-mode")
            .returning(move |_, _| Ok(return_flag.clone()));

        let svc = make_service(mock);
        let req = EvaluateFlagRequest {
            tenant_id: system_tenant(),
            flag_key: "dark-mode".to_string(),
            user_id: "user-1".to_string(),
            context_tenant_id: String::new(),
            attributes: HashMap::new(),
        };
        let resp = svc.evaluate_flag(req).await.unwrap();

        assert!(resp.enabled);
        assert_eq!(resp.variant, Some("on".to_string()));
        assert_eq!(resp.flag_key, "dark-mode");
    }

    #[tokio::test]
    async fn test_list_flags_success() {
        let mut mock = MockFeatureFlagRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む1引数シグネチャ
        mock.expect_find_all().returning(|_| Ok(vec![]));

        let svc = make_service(mock);
        let resp = svc.list_flags(ListFlagsRequest { tenant_id: system_tenant() }).await.unwrap();
        assert!(resp.flags.is_empty());
    }

    #[tokio::test]
    async fn test_create_flag_success() {
        let mut mock = MockFeatureFlagRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む2引数シグネチャ
        mock.expect_exists_by_key()
            .withf(|_tid, key| key == "new-flag")
            .returning(|_, _| Ok(false));
        mock.expect_create().returning(|_, _| Ok(()));

        let svc = make_service(mock);
        let req = CreateFlagRequest {
            tenant_id: system_tenant(),
            flag_key: "new-flag".to_string(),
            description: "New flag".to_string(),
            enabled: true,
            variants: vec![],
        };
        let resp = svc.create_flag(req).await.unwrap();

        assert_eq!(resp.flag_key, "new-flag");
        assert!(resp.enabled);
    }

    #[tokio::test]
    async fn test_update_flag_success() {
        let mut mock = MockFeatureFlagRepository::new();
        let flag = make_flag("dark-mode", true);
        let return_flag = flag.clone();

        // STATIC-CRITICAL-001: tenant_id を含む2引数シグネチャ
        mock.expect_find_by_key()
            .withf(|_tid, key| key == "dark-mode")
            .returning(move |_, _| Ok(return_flag.clone()));
        mock.expect_update().returning(|_, _| Ok(()));

        let svc = make_service(mock);
        let req = UpdateFlagRequest {
            tenant_id: system_tenant(),
            flag_key: "dark-mode".to_string(),
            enabled: Some(false),
            description: None,
            variants: vec![],
            rules: vec![],
        };
        let resp = svc.update_flag(req).await.unwrap();

        assert_eq!(resp.flag_key, "dark-mode");
        assert!(!resp.enabled);
    }

    #[tokio::test]
    async fn test_delete_flag_success() {
        let mut mock = MockFeatureFlagRepository::new();
        // 固定IDを使ってfind_by_keyとfind_allが同じフラグを指すようにする
        let fixed_id = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let flag = FeatureFlag::new(
            system_tenant(),
            "old-flag".to_string(),
            "old-flag description".to_string(),
            true,
        );
        // IDを固定値に差し替える
        let mut flag_with_id = flag.clone();
        flag_with_id.id = fixed_id;
        let return_flag = flag_with_id.clone();
        let return_flag2 = flag_with_id.clone();

        // STATIC-CRITICAL-001: get_flag（find_by_key）と delete（find_all + delete）の2引数シグネチャ
        mock.expect_find_by_key()
            .withf(|_tid, key| key == "old-flag")
            .returning(move |_, _| Ok(return_flag.clone()));
        mock.expect_find_all()
            .returning(move |_| Ok(vec![return_flag2.clone()]));
        mock.expect_delete()
            .withf(move |_tid, id| *id == fixed_id)
            .returning(|_, _| Ok(true));

        let svc = make_service(mock);
        let req = DeleteFlagRequest {
            tenant_id: system_tenant(),
            flag_key: "old-flag".to_string(),
        };
        let resp = svc.delete_flag(req).await.unwrap();

        assert!(resp.success);
        assert!(resp.message.contains("old-flag"));
    }

    #[tokio::test]
    async fn test_parse_tenant_id_valid() {
        let result = parse_tenant_id("00000000-0000-0000-0000-000000000001");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), system_tenant());
    }

    #[tokio::test]
    async fn test_parse_tenant_id_invalid() {
        let result = parse_tenant_id("not-a-uuid");
        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::InvalidArgument(msg) => assert!(msg.contains("tenant_id")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
