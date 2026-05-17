// auth_context.rs — spec 04 認証適合仕様: AuthContext opaque 型
// 04_認証適合仕様.md §AuthContext スキーマに準拠する。
// tier2 の TenantContext（4 GUC）と組み合わせて 1 つの session_context を構成する。
// 生の access_token / refresh_token は公開シグネチャに含まれない（zeroize で隠蔽）。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// AuthClass は 04_認証適合仕様.md §v1 auth_class セット（5 class）を宣言する。
// class 1 値が subject_kind / token_type / lifetime_class / refresh_policy / step_up_required を
// 一意に導出する（dimension override 禁止）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthClass {
    // v1_human_session: 業務担当者 SPA セッション（OIDC + DPoP + rotating refresh）
    V1HumanSession,
    // v1_workload_jwt: K8s ServiceAccount / SPIFFE SVID（短 TTL JWT、自動 renew）
    V1WorkloadJwt,
    // v1_device_attest: 工場端末（device cert、長 TTL、one_shot refresh）
    V1DeviceAttest,
    // v1_federated_exchange: 外部 IdP からの RFC 8693 token exchange
    V1FederatedExchange,
    // v1_emergency_step_up: break-glass（always step_up、TTL<10m、no refresh）
    V1EmergencyStepUp,
}

impl std::fmt::Display for AuthClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // spec §v1 auth_class セット の文字列表現を返す
        match self {
            AuthClass::V1HumanSession => write!(f, "v1_human_session"),
            AuthClass::V1WorkloadJwt => write!(f, "v1_workload_jwt"),
            AuthClass::V1DeviceAttest => write!(f, "v1_device_attest"),
            AuthClass::V1FederatedExchange => write!(f, "v1_federated_exchange"),
            AuthClass::V1EmergencyStepUp => write!(f, "v1_emergency_step_up"),
        }
    }
}

// AuthContext は tier1 Library が transaction 開始時に PostgreSQL GUC に SET LOCAL する
// 認証拡張フィールドを保持する opaque 型。
// 生 access_token / refresh_token はフィールドに含まない。
// 04_認証適合仕様.md §AuthContext スキーマ準拠。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    // auth_class: v1_human_session 等
    pub auth_class: AuthClass,
    // subject_id: canonical subject（Keycloak sub など）
    pub subject_id: String,
    // subject_kind: human / workload / device / external_subject
    pub subject_kind: String,
    // token_id: JWT jti（revocation tracking）
    pub token_id: String,
    // session_id: human / device のセッション識別子
    pub session_id: String,
    // tenant_id: リクエストのテナント識別子（tier2 TenantContext と同期）
    pub tenant_id: String,
    // audience: token aud（federated に必須）
    pub audience: String,
    // scopes: OAuth scopes
    pub scopes: Vec<String>,
    // dpop_jkt: DPoP key thumbprint（dpop_bound_jwt のみ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dpop_jkt: Option<String>,
    // attestation_level: device attestation level（jwt_attested のみ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation_level: Option<String>,
    // step_up_proven: 最終 step_up challenge 済みフラグ
    pub step_up_proven: bool,
    // is_valid: 検証が成功したかどうか
    pub is_valid: bool,
}

impl AuthContext {
    // new_human_session は v1_human_session AuthContext を構築する。
    pub fn new_human_session(
        subject_id: String,
        tenant_id: String,
        token_id: String,
        scopes: Vec<String>,
        dpop_jkt: Option<String>,
        step_up_proven: bool,
    ) -> Self {
        // v1_human_session の固定属性を適用する
        Self {
            auth_class: AuthClass::V1HumanSession,
            subject_id,
            // human session の subject_kind は常に "human"
            subject_kind: "human".to_string(),
            token_id,
            // session_id は JWT セッション管理に使用する UUID を生成する
            session_id: Uuid::new_v4().to_string(),
            tenant_id,
            audience: String::new(),
            scopes,
            dpop_jkt,
            attestation_level: None,
            step_up_proven,
            is_valid: true,
        }
    }

    // new_workload_jwt は v1_workload_jwt AuthContext を構築する。
    pub fn new_workload_jwt(
        subject_id: String,
        tenant_id: String,
        token_id: String,
        audience: String,
    ) -> Self {
        // v1_workload_jwt の固定属性を適用する
        Self {
            auth_class: AuthClass::V1WorkloadJwt,
            subject_id,
            // workload JWT の subject_kind は常に "workload"
            subject_kind: "workload".to_string(),
            token_id,
            // workload は session を持たない（K8s pod の lifecycle で管理）
            session_id: String::new(),
            tenant_id,
            audience,
            scopes: vec!["service.api".to_string()],
            dpop_jkt: None,
            attestation_level: None,
            // workload は step_up が不要（never ポリシー）
            step_up_proven: false,
            is_valid: true,
        }
    }

    // new_emergency_step_up は v1_emergency_step_up AuthContext を構築する。
    pub fn new_emergency_step_up(
        subject_id: String,
        tenant_id: String,
        token_id: String,
        dpop_jkt: Option<String>,
    ) -> Self {
        // v1_emergency_step_up は always step_up かつ purpose=emergency 強制
        Self {
            auth_class: AuthClass::V1EmergencyStepUp,
            subject_id,
            subject_kind: "human".to_string(),
            token_id,
            session_id: Uuid::new_v4().to_string(),
            tenant_id,
            audience: String::new(),
            scopes: vec!["emergency.break_glass".to_string()],
            dpop_jkt,
            attestation_level: None,
            // v1_emergency_step_up は常に step_up 済みとして発行される
            step_up_proven: true,
            is_valid: true,
        }
    }

    // to_guc_setters は PostgreSQL GUC の SET LOCAL 文を生成する。
    // tier2 TenantContext の 4 GUC と組み合わせて session_context を構成する。
    pub fn to_guc_setters(&self) -> Vec<String> {
        // spec §AuthContext スキーマの GUC 対応フィールドを SET LOCAL 文にする
        vec![
            format!("SET LOCAL app.auth_class = '{}'", self.auth_class),
            format!("SET LOCAL app.subject_id = '{}'", self.subject_id),
            format!("SET LOCAL app.subject_kind = '{}'", self.subject_kind),
            format!("SET LOCAL app.token_id = '{}'", self.token_id),
            format!("SET LOCAL app.session_id = '{}'", self.session_id),
            format!("SET LOCAL app.step_up_proven = '{}'", self.step_up_proven),
        ]
    }
}
