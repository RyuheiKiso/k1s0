// auth_context.rs — k1s0 tier1 Library: AuthClass enum + AuthContext struct
// 04_認証適合仕様.md §v1 auth_class セット（5 class）および
// §AuthContext スキーマ（32 session_context の拡張・同型）に準拠する。
// 生 access_token / refresh_token は公開シグネチャに一切含まれない。

// serde: シリアライズ/デシリアライズ（derive feature を使用する）
use serde::{Deserialize, Serialize};
// uuid: UUID v4 生成（session_id のランダム生成に使用する）
use uuid::Uuid;

// AuthClass は 04_認証適合仕様.md §v1 auth_class セット（5 class）を宣言する。
// class 1 値が subject_kind / token_type / lifetime_class / refresh_policy / step_up_required を
// 一意に導出する（dimension override 禁止）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// serde: spec の class 名（snake_case）と 1:1 対応するシリアライズ形式
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

// AuthClass の文字列表現を返す（spec の class 名と 1:1 対応する）
impl std::fmt::Display for AuthClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 各 class を spec 定義の snake_case 文字列にマッピングする
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
// 04_認証適合仕様.md §AuthContext スキーマ準拠。
// 生 access_token / refresh_token はフィールドに含まない。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    // auth_class: v1_human_session 等（5 class のいずれか）
    pub auth_class: AuthClass,
    // subject_id: canonical subject（Keycloak sub など）
    pub subject_id: String,
    // subject_kind: human / workload / device / external_subject
    pub subject_kind: String,
    // token_id: JWT jti（revocation tracking 用）
    pub token_id: String,
    // session_id: human / device のセッション識別子（workload は空文字列）
    pub session_id: String,
    // tenant_id: リクエストのテナント識別子（tier2 TenantContext と同期する）
    pub tenant_id: String,
    // audience: token aud（v1_federated_exchange で必須）
    pub audience: String,
    // scopes: OAuth scopes（service.api / emergency.break_glass 等）
    pub scopes: Vec<String>,
    // dpop_jkt: DPoP key thumbprint（dpop_bound_jwt のみ設定される）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dpop_jkt: Option<String>,
    // attestation_level: device attestation level（jwt_attested のみ設定される）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation_level: Option<String>,
    // step_up_proven: 最終 step_up challenge 済みフラグ（always / on_high_risk の場合は true が必須）
    pub step_up_proven: bool,
    // is_valid: token 検証が成功したかどうか（false の場合は GUC setter を空にする）
    pub is_valid: bool,
}

// AuthContext のファクトリメソッド群
impl AuthContext {
    // new_human_session は v1_human_session AuthContext を構築する。
    // OIDC code flow + DPoP 鍵束縛（RFC 9449）+ rotating refresh に対応する。
    pub fn new_human_session(
        subject_id: String,
        tenant_id: String,
        token_id: String,
        scopes: Vec<String>,
        dpop_jkt: Option<String>,
        step_up_proven: bool,
    ) -> Self {
        // v1_human_session の固定属性を適用する（dimension override 禁止）
        Self {
            auth_class: AuthClass::V1HumanSession,
            subject_id,
            // human session の subject_kind は常に "human"（spec §各 class の不変条件）
            subject_kind: "human".to_string(),
            token_id,
            // session_id は UUID v4 で生成する（JWT セッション管理に使用する）
            session_id: Uuid::new_v4().to_string(),
            tenant_id,
            // audience は human session では空文字列（federated のみ必須）
            audience: String::new(),
            scopes,
            dpop_jkt,
            // human session には attestation_level は不要
            attestation_level: None,
            step_up_proven,
            is_valid: true,
        }
    }

    // new_workload_jwt は v1_workload_jwt AuthContext を構築する。
    // K8s ServiceAccount projection / SPIFFE SVID（短 TTL JWT、自動 renew）に対応する。
    pub fn new_workload_jwt(
        subject_id: String,
        tenant_id: String,
        token_id: String,
        audience: String,
    ) -> Self {
        // v1_workload_jwt の固定属性を適用する（dimension override 禁止）
        Self {
            auth_class: AuthClass::V1WorkloadJwt,
            subject_id,
            // workload JWT の subject_kind は常に "workload"
            subject_kind: "workload".to_string(),
            token_id,
            // workload は session を持たない（K8s pod lifecycle で管理する）
            session_id: String::new(),
            tenant_id,
            audience,
            // workload のデフォルトスコープ（service.api のみ）
            scopes: vec!["service.api".to_string()],
            // workload は DPoP 不要（JWT 短命で proof-of-possession 不要）
            dpop_jkt: None,
            attestation_level: None,
            // workload は step_up が不要（never ポリシー）
            step_up_proven: false,
            is_valid: true,
        }
    }

    // new_device_attest は v1_device_attest AuthContext を構築する。
    // 工場端末・KIOSK・現場ハンドヘルド（device cert、長 TTL、one_shot refresh）に対応する。
    pub fn new_device_attest(
        subject_id: String,
        tenant_id: String,
        token_id: String,
        attestation_level: String,
        step_up_proven: bool,
    ) -> Self {
        // v1_device_attest の固定属性を適用する（dimension override 禁止）
        Self {
            auth_class: AuthClass::V1DeviceAttest,
            subject_id,
            // device attest の subject_kind は常に "device"
            subject_kind: "device".to_string(),
            token_id,
            // device は session を持つ（device registration session ID を設定する）
            session_id: Uuid::new_v4().to_string(),
            tenant_id,
            // device attest では audience は空文字列（resource server は token_id で管理する）
            audience: String::new(),
            // device のデフォルトスコープ
            scopes: vec!["device.api".to_string()],
            // device attest は DPoP 不要（device cert で proof-of-possession を担保する）
            dpop_jkt: None,
            // attestation_level: TPM / HSM / WebAuthn platform authenticator の種別
            attestation_level: Some(attestation_level),
            step_up_proven,
            is_valid: true,
        }
    }

    // new_federated_exchange は v1_federated_exchange AuthContext を構築する。
    // 外部 IdP からの RFC 8693 token exchange（audience-restricted JWT）に対応する。
    pub fn new_federated_exchange(
        subject_id: String,
        tenant_id: String,
        token_id: String,
        audience: String,
    ) -> Self {
        // v1_federated_exchange の固定属性を適用する（dimension override 禁止）
        Self {
            auth_class: AuthClass::V1FederatedExchange,
            subject_id,
            // federated exchange の subject_kind は "external_subject"
            subject_kind: "external_subject".to_string(),
            token_id,
            // federated exchange は session を持たない（短命 JWT で session 管理不要）
            session_id: String::new(),
            tenant_id,
            // audience は federated exchange で必須（resource server を audience claim で絞る）
            audience,
            // federated exchange のデフォルトスコープ（business_op のみ）
            scopes: vec!["business_op".to_string()],
            // federated は DPoP 不要（短命 JWT で replay 防止）
            dpop_jkt: None,
            attestation_level: None,
            // federated exchange は step_up 不要（never ポリシー）
            step_up_proven: false,
            is_valid: true,
        }
    }

    // new_emergency_step_up は v1_emergency_step_up AuthContext を構築する。
    // break-glass（always step_up、TTL<10m、no refresh、purpose=emergency 強制）に対応する。
    pub fn new_emergency_step_up(
        subject_id: String,
        tenant_id: String,
        token_id: String,
        dpop_jkt: Option<String>,
    ) -> Self {
        // v1_emergency_step_up の固定属性を適用する（always step_up + purpose=emergency 強制）
        Self {
            auth_class: AuthClass::V1EmergencyStepUp,
            subject_id,
            // emergency の subject_kind は常に "human"（workload による break-glass 禁止）
            subject_kind: "human".to_string(),
            token_id,
            // emergency は session を持つ（break-glass セッションを追跡する）
            session_id: Uuid::new_v4().to_string(),
            tenant_id,
            // emergency では audience は空文字列（break-glass の範囲は purpose で管理する）
            audience: String::new(),
            // emergency のスコープ（break_glass を明示する）
            scopes: vec!["emergency.break_glass".to_string()],
            dpop_jkt,
            attestation_level: None,
            // v1_emergency_step_up は常に step_up 済みとして発行される（always ポリシー）
            step_up_proven: true,
            is_valid: true,
        }
    }

    // to_guc_setters は PostgreSQL GUC の SET LOCAL 文を生成する。
    // is_valid が false の場合は空 Vec を返す（無効な AuthContext で GUC を設定しない）。
    // tier2 TenantContext の 4 GUC と組み合わせて session_context を構成する。
    pub fn to_guc_setters(&self) -> Vec<String> {
        // is_valid が false の場合は GUC setter を空にする（spec §AuthContext スキーマ準拠）
        if !self.is_valid {
            return vec![];
        }
        // 04_認証適合仕様.md §AuthContext スキーマの全 GUC 対応フィールドを SET LOCAL 文にする
        let mut setters = vec![
            format!("SET LOCAL app.auth_class = '{}'", self.auth_class),
            format!("SET LOCAL app.subject_id = '{}'", self.subject_id),
            format!("SET LOCAL app.subject_kind = '{}'", self.subject_kind),
            format!("SET LOCAL app.token_id = '{}'", self.token_id),
            format!("SET LOCAL app.session_id = '{}'", self.session_id),
            format!("SET LOCAL app.audience = '{}'", self.audience),
            format!("SET LOCAL app.step_up_proven = '{}'", self.step_up_proven),
        ];
        // dpop_jkt が Some の場合のみ GUC を設定する（dpop_bound_jwt のみ）
        if let Some(ref jkt) = self.dpop_jkt {
            setters.push(format!("SET LOCAL app.dpop_jkt = '{}'", jkt));
        }
        // attestation_level が Some の場合のみ GUC を設定する（jwt_attested のみ）
        if let Some(ref level) = self.attestation_level {
            setters.push(format!("SET LOCAL app.attestation_level = '{}'", level));
        }
        setters
    }
}
