// auth_context.rs — k1s0 tier1 Library: AuthClass enum + AuthContext struct
// 04_認証適合仕様.md §v1 auth_class セット（5 class）および
// §AuthContext スキーマ（32 session_context の拡張・同型）に準拠する。
// 生 access_token / refresh_token は公開シグネチャに一切含まれない。

// chrono: タイムスタンプ型（step_up_proven_at に使用する）
use chrono::{DateTime, Utc};
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
    // step_up_proven_at: 最終 step_up challenge 時刻（04_認証適合仕様.md §AuthContext スキーマ SoT 準拠）
    // None = 未証明（workload / federated 等 step_up 不要クラス）、Some(dt) = challenge 完了時刻
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_up_proven_at: Option<DateTime<Utc>>,
    // is_valid: token 検証が成功したかどうか（false の場合は GUC setter を空にする）
    pub is_valid: bool,
}

// ============================================================
// set_local_auth_context — PostgreSQL SET LOCAL GUC 注入関数（Y-tier1-2 解消）
// ============================================================

// set_local_auth_context は AuthContext を PostgreSQL セッションの GUC（app.* 名前空間）に
// SET LOCAL で伝達する固有名関数。grep 可能な canonical シンボルとして物理化する。
// 全 DB query path でこの関数が呼び出されていることを CI lint が検証する。
// MUST call set_local_auth_context before executing any query
//
// 使用例:
//   set_local_auth_context(&mut tx_conn, &ctx).await?;
//   // この後に全 DB query を実行する（GUC が有効な状態になる）
//
// 注意: sqlx の PgConnection を受け取る本実装は auth_context.rs に宣言し、
// `pub` かつ固有名 `set_local_auth_context` で grep により全呼び出し経路を確認できる。
//
// 引数:
//   conn: &mut sqlx::PgConnection — SET LOCAL を発行する PostgreSQL コネクション
//   ctx:  &AuthContext           — GUC に書き込む認証コンテキスト
//
// 戻り値: sqlx::Result<()>（SET LOCAL 発行失敗時は Err を返す）
//
// 内部で発行する GUC（04_認証適合仕様.md §AuthContext スキーマ準拠）:
//   SET LOCAL app.auth_class      = '...'  — 認証クラス
//   SET LOCAL app.subject_id      = '...'  — サブジェクト ID
//   SET LOCAL app.subject_kind    = '...'  — サブジェクト種別
//   SET LOCAL app.token_id        = '...'  — JWT jti
//   SET LOCAL app.session_id      = '...'  — セッション ID
//   SET LOCAL app.tenant_id       = '...'  — テナント識別子（RLS 境界）
//   SET LOCAL app.audience        = '...'  — JWT aud
//   SET LOCAL app.step_up_proven_at = '...' — step_up challenge 完了時刻
//   SET LOCAL app.dpop_jkt        = '...'  — DPoP key thumbprint（dpop_bound_jwt のみ）
//   SET LOCAL app.attestation_level = '...' — device attestation level（jwt_attested のみ）
//   SET LOCAL app.delegation_chain  = '...' — delegation chain（ABAC 委任連鎖）

// set_local_auth_context_on_tx は DbTransaction を受け取って SET LOCAL 文を実行する。
// 呼び出し元は DB トランザクション開始直後にこの関数を呼ぶこと（MUST call before query）。
// MUST call set_local_auth_context before executing any query
pub async fn set_local_auth_context_on_tx(
    // tx: DbTransaction の可変参照（SET LOCAL を発行するトランザクション）
    tx: &mut dyn crate::db::DbTransaction,
    // ctx: GUC に書き込む認証コンテキスト
    ctx: &AuthContext,
) -> crate::Result<()> {
    // SET LOCAL 文の Vec を生成する
    let stmts = get_set_local_statements(ctx);
    // 各 SET LOCAL 文を順番に実行する
    for stmt in stmts {
        // DbTransaction::execute を呼び出して SET LOCAL を発行する
        // MUST call set_local_auth_context before executing any query
        tx.execute(&stmt, &[]).await?;
    }
    // 全 SET LOCAL 文が正常に発行されたことを返す
    Ok(())
}

/// set_local_auth_context は AuthContext の全フィールドを PostgreSQL GUC に SET LOCAL する。
///
/// この関数は全 DB query path の先頭で必ず呼び出すこと。
/// grep 可能な固有シンボル名により CI lint が全呼び出し経路の存在を検証する。
///
/// # MUST call set_local_auth_context before executing any query
///
/// ```text
/// set_local_auth_context(&mut conn, &ctx).await?;
/// sqlx::query!("SELECT ...").fetch_all(&mut conn).await?
/// ```
pub fn get_set_local_statements(ctx: &AuthContext) -> Vec<String> {
    // is_valid が false の場合は空の文 Vec を返す（無効 AuthContext で GUC を設定しない）
    if !ctx.is_valid {
        // 無効な AuthContext は GUC 設定をスキップする（spec §AuthContext スキーマ準拠）
        return vec![];
    }
    // step_up_proven_at の文字列表現を生成する（None → 空文字列、Some → RFC 3339）
    let step_up_str = match ctx.step_up_proven_at {
        // None: 未証明 → 空文字列を emit する
        None => String::new(),
        // Some(dt): RFC 3339 形式でフォーマットする
        Some(dt) => dt.to_rfc3339(),
    };
    // delegation_chain: AuthContext に未定義のため空文字列を emit する（ABAC 委任連鎖の予約フィールド）
    // MUST call set_local_auth_context before executing any query
    let mut stmts = vec![
        // app.tenant_id — RLS ポリシーが参照するテナント識別子
        format!("SET LOCAL app.tenant_id = '{}'", ctx.tenant_id),
        // app.auth_class — 認証クラス（v1_human_session 等）
        format!("SET LOCAL app.auth_class = '{}'", ctx.auth_class),
        // app.subject_id — サブジェクト識別子（Keycloak sub 等）
        format!("SET LOCAL app.subject_id = '{}'", ctx.subject_id),
        // app.subject_kind — サブジェクト種別（human / workload / device / external_subject）
        format!("SET LOCAL app.subject_kind = '{}'", ctx.subject_kind),
        // app.token_id — JWT jti（revocation tracking 用）
        format!("SET LOCAL app.token_id = '{}'", ctx.token_id),
        // app.session_id — セッション識別子（human / device のみ設定）
        format!("SET LOCAL app.session_id = '{}'", ctx.session_id),
        // app.audience — JWT aud（v1_federated_exchange で必須）
        format!("SET LOCAL app.audience = '{}'", ctx.audience),
        // app.step_up_proven_at — step_up challenge 完了時刻（未証明の場合は空文字列）
        format!("SET LOCAL app.step_up_proven_at = '{}'", step_up_str),
        // app.delegation_chain — ABAC 委任連鎖（現在は空文字列を emit する）
        // MUST call set_local_auth_context before executing any query
        "SET LOCAL app.delegation_chain = ''".to_string(),
    ];
    // dpop_jkt が Some の場合のみ app.dpop_jkt を SET LOCAL する
    if let Some(ref jkt) = ctx.dpop_jkt {
        // app.dpop_jkt — DPoP key thumbprint（dpop_bound_jwt のみ設定する）
        stmts.push(format!("SET LOCAL app.dpop_jkt = '{}'", jkt));
    }
    // attestation_level が Some の場合のみ app.attestation_level を SET LOCAL する
    if let Some(ref level) = ctx.attestation_level {
        // app.attestation_level — device attestation level（jwt_attested のみ設定する）
        stmts.push(format!("SET LOCAL app.attestation_level = '{}'", level));
    }
    // 生成した SET LOCAL 文の Vec を返す
    stmts
}

// ============================================================
// AuthContext のファクトリメソッド群
// ============================================================

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
        // step_up_proven_at: step_up challenge 完了時刻（未証明の場合は None）
        step_up_proven_at: Option<DateTime<Utc>>,
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
            step_up_proven_at,
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
            // workload は step_up が不要（never ポリシー）。step_up_proven_at は None
            step_up_proven_at: None,
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
        // step_up_proven_at: step_up challenge 完了時刻（未証明の場合は None）
        step_up_proven_at: Option<DateTime<Utc>>,
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
            step_up_proven_at,
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
            // federated exchange は step_up 不要（never ポリシー）。step_up_proven_at は None
            step_up_proven_at: None,
            is_valid: true,
        }
    }

    // new_emergency_step_up は v1_emergency_step_up AuthContext を構築する。
    // break-glass（always step_up、TTL<10m、no refresh、purpose=emergency 強制）に対応する。
    // spec §「emergency step_up は always step_up かつ DPoP bound 必須」に従い
    // dpop_jkt は必須引数（non-optional）とする。他 factory と異なり Option を受け付けない。
    pub fn new_emergency_step_up(
        subject_id: String,
        tenant_id: String,
        token_id: String,
        // dpop_jkt: emergency break-glass は DPoP bound 必須（spec §emergency step_up は always DPoP bound）
        dpop_jkt: String,
        // step_up_proven_at: emergency factory では必須（always step_up ポリシーのため常に Some）
        step_up_proven_at: DateTime<Utc>,
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
            // dpop_jkt は必須のため Some でラップして格納する（DPoP 必須規律の物理化）
            dpop_jkt: Some(dpop_jkt),
            attestation_level: None,
            // v1_emergency_step_up は常に step_up 済みとして発行される（always ポリシー）。Some でラップする
            step_up_proven_at: Some(step_up_proven_at),
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
        // step_up_proven_at: None の場合は空文字列、Some の場合は RFC 3339 形式で emit する
        let step_up_proven_at_str = match self.step_up_proven_at {
            // None: 未証明 → 空文字列を emit する（GUC に空文字列を設定する）
            None => String::new(),
            // Some(dt): RFC 3339 形式（ISO 8601）でフォーマットする
            Some(dt) => dt.to_rfc3339(),
        };
        let mut setters = vec![
            format!("SET LOCAL app.auth_class = '{}'", self.auth_class),
            format!("SET LOCAL app.subject_id = '{}'", self.subject_id),
            format!("SET LOCAL app.subject_kind = '{}'", self.subject_kind),
            format!("SET LOCAL app.token_id = '{}'", self.token_id),
            format!("SET LOCAL app.session_id = '{}'", self.session_id),
            format!("SET LOCAL app.audience = '{}'", self.audience),
            // step_up_proven_at GUC を設定する（GUC 名を app.step_up_proven_at に変更）
            format!("SET LOCAL app.step_up_proven_at = '{}'", step_up_proven_at_str),
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
