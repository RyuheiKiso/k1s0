// auth_context.go — k1s0 tier1 Library Go 実装: AuthClass type + AuthContext struct
// 04_認証適合仕様.md §v1 auth_class セット（5 class）および
// §AuthContext スキーマ（32 session_context の拡張・同型）に準拠する。
// 生 access_token / refresh_token は公開シグネチャに一切含まれない。

// パッケージ名: keyhandle（tier1 Library の認証コンテキスト公開 API を提供する）
package keyhandle

import (
	// errors: エラー生成に使用する
	"errors"
	// fmt: SQL 文字列フォーマットに使用する
	"fmt"
	// strings: strings.Join で scopes を結合する
	"strings"
)

// AuthClass は 04_認証適合仕様.md §v1 auth_class セット（5 class）を宣言する型。
// class 1 値が subject_kind / token_type / lifetime_class / refresh_policy / step_up_required を
// 一意に導出する（dimension override 禁止）。
type AuthClass string

// AuthClass の定数定義: spec の class 名（snake_case）と 1:1 対応する
const (
	// AuthClassV1HumanSession: 業務担当者 SPA セッション（OIDC + DPoP + rotating refresh）
	AuthClassV1HumanSession AuthClass = "v1_human_session"
	// AuthClassV1WorkloadJwt: K8s ServiceAccount / SPIFFE SVID（短 TTL JWT、自動 renew）
	AuthClassV1WorkloadJwt AuthClass = "v1_workload_jwt"
	// AuthClassV1DeviceAttest: 工場端末（device cert、長 TTL、one_shot refresh）
	AuthClassV1DeviceAttest AuthClass = "v1_device_attest"
	// AuthClassV1FederatedExchange: 外部 IdP からの RFC 8693 token exchange
	AuthClassV1FederatedExchange AuthClass = "v1_federated_exchange"
	// AuthClassV1EmergencyStepUp: break-glass（always step_up、TTL<10m、no refresh）
	AuthClassV1EmergencyStepUp AuthClass = "v1_emergency_step_up"
)

// ErrInvalidAuthContext: 無効な AuthContext のエラー（is_valid=false 時に GUC setter 呼出をブロックする）
var ErrInvalidAuthContext = errors.New("invalid auth context: is_valid is false")

// AuthContext は tier1 Library が transaction 開始時に PostgreSQL GUC に SET LOCAL する
// 認証拡張フィールドを保持する opaque 型。
// 04_認証適合仕様.md §AuthContext スキーマ準拠。
// 生 access_token / refresh_token はフィールドに含まない。
// 小文字フィールドで unexported: 外部からの直接操作を禁止する。
type AuthContext struct {
	// authClass: v1_human_session 等（5 class のいずれか）
	authClass AuthClass
	// subjectID: canonical subject（Keycloak sub など）
	subjectID string
	// subjectKind: human / workload / device / external_subject
	subjectKind string
	// tokenID: JWT jti（revocation tracking 用）
	tokenID string
	// sessionID: human / device のセッション識別子（workload は空文字列）
	sessionID string
	// tenantID: リクエストのテナント識別子（tier2 TenantContext と同期する）
	tenantID string
	// audience: token aud（v1_federated_exchange で必須）
	audience string
	// scopes: OAuth scopes（service.api / emergency.break_glass 等）
	scopes []string
	// dpopJkt: DPoP key thumbprint（dpop_bound_jwt のみ設定される）
	dpopJkt string
	// attestationLevel: device attestation level（jwt_attested のみ設定される）
	attestationLevel string
	// stepUpProven: 最終 step_up challenge 済みフラグ
	stepUpProven bool
	// isValid: token 検証が成功したかどうか（false の場合は GUC setter を空にする）
	isValid bool
}

// AuthClass は認証クラスを返す（読み取り専用アクセサ）
func (a *AuthContext) AuthClass() AuthClass {
	// authClass フィールドを返す
	return a.authClass
}

// SubjectID は canonical subject を返す（読み取り専用アクセサ）
func (a *AuthContext) SubjectID() string {
	// subjectID フィールドを返す
	return a.subjectID
}

// IsValid は token 検証結果を返す（読み取り専用アクセサ）
func (a *AuthContext) IsValid() bool {
	// isValid フィールドを返す
	return a.isValid
}

// NewHumanSessionContext は v1_human_session AuthContext を構築するファクトリ関数。
// OIDC code flow + DPoP 鍵束縛（RFC 9449）+ rotating refresh に対応する。
func NewHumanSessionContext(
	subjectID string,
	tenantID string,
	tokenID string,
	sessionID string,
	scopes []string,
	dpopJkt string,
	stepUpProven bool,
) *AuthContext {
	// v1_human_session の固定属性を適用する（dimension override 禁止）
	return &AuthContext{
		authClass: AuthClassV1HumanSession,
		subjectID: subjectID,
		// human session の subject_kind は常に "human"（spec §各 class の不変条件）
		subjectKind: "human",
		tokenID:      tokenID,
		sessionID:    sessionID,
		tenantID:     tenantID,
		// audience は human session では空文字列
		audience:     "",
		scopes:       scopes,
		dpopJkt:      dpopJkt,
		// human session には attestation_level は不要
		attestationLevel: "",
		stepUpProven: stepUpProven,
		isValid:      true,
	}
}

// NewWorkloadJwtContext は v1_workload_jwt AuthContext を構築するファクトリ関数。
// K8s ServiceAccount projection / SPIFFE SVID（短 TTL JWT、自動 renew）に対応する。
func NewWorkloadJwtContext(
	subjectID string,
	tenantID string,
	tokenID string,
	audience string,
) *AuthContext {
	// v1_workload_jwt の固定属性を適用する（dimension override 禁止）
	return &AuthContext{
		authClass: AuthClassV1WorkloadJwt,
		subjectID: subjectID,
		// workload JWT の subject_kind は常に "workload"
		subjectKind: "workload",
		tokenID:  tokenID,
		// workload は session を持たない（K8s pod lifecycle で管理する）
		sessionID: "",
		tenantID:  tenantID,
		audience:  audience,
		// workload のデフォルトスコープ（service.api のみ）
		scopes:           []string{"service.api"},
		dpopJkt:          "",
		attestationLevel: "",
		// workload は step_up が不要（never ポリシー）
		stepUpProven: false,
		isValid:      true,
	}
}

// NewDeviceAttestContext は v1_device_attest AuthContext を構築するファクトリ関数。
// 工場端末・KIOSK・現場ハンドヘルド（device cert、長 TTL、one_shot refresh）に対応する。
func NewDeviceAttestContext(
	subjectID string,
	tenantID string,
	tokenID string,
	sessionID string,
	attestationLevel string,
	stepUpProven bool,
) *AuthContext {
	// v1_device_attest の固定属性を適用する（dimension override 禁止）
	return &AuthContext{
		authClass: AuthClassV1DeviceAttest,
		subjectID: subjectID,
		// device attest の subject_kind は常に "device"
		subjectKind:      "device",
		tokenID:          tokenID,
		sessionID:        sessionID,
		tenantID:         tenantID,
		// device attest では audience は空文字列
		audience:         "",
		// device のデフォルトスコープ
		scopes:           []string{"device.api"},
		dpopJkt:          "",
		// attestationLevel: TPM / HSM / WebAuthn platform authenticator の種別
		attestationLevel: attestationLevel,
		stepUpProven:     stepUpProven,
		isValid:          true,
	}
}

// NewFederatedExchangeContext は v1_federated_exchange AuthContext を構築するファクトリ関数。
// 外部 IdP からの RFC 8693 token exchange（audience-restricted JWT）に対応する。
func NewFederatedExchangeContext(
	subjectID string,
	tenantID string,
	tokenID string,
	audience string,
) *AuthContext {
	// v1_federated_exchange の固定属性を適用する（dimension override 禁止）
	return &AuthContext{
		authClass: AuthClassV1FederatedExchange,
		subjectID: subjectID,
		// federated exchange の subject_kind は "external_subject"
		subjectKind: "external_subject",
		tokenID:     tokenID,
		// federated exchange は session を持たない
		sessionID: "",
		tenantID:  tenantID,
		// audience は federated exchange で必須（resource server を audience claim で絞る）
		audience:         audience,
		scopes:           []string{"business_op"},
		dpopJkt:          "",
		attestationLevel: "",
		// federated exchange は step_up 不要（never ポリシー）
		stepUpProven: false,
		isValid:      true,
	}
}

// NewEmergencyStepUpContext は v1_emergency_step_up AuthContext を構築するファクトリ関数。
// break-glass（always step_up、TTL<10m、no refresh、purpose=emergency 強制）に対応する。
func NewEmergencyStepUpContext(
	subjectID string,
	tenantID string,
	tokenID string,
	sessionID string,
	dpopJkt string,
) *AuthContext {
	// v1_emergency_step_up の固定属性を適用する（always step_up + purpose=emergency 強制）
	return &AuthContext{
		authClass: AuthClassV1EmergencyStepUp,
		subjectID: subjectID,
		// emergency の subject_kind は常に "human"（workload による break-glass 禁止）
		subjectKind: "human",
		tokenID:     tokenID,
		// emergency は session を持つ（break-glass セッションを追跡する）
		sessionID: sessionID,
		tenantID:  tenantID,
		// emergency では audience は空文字列
		audience:         "",
		scopes:           []string{"emergency.break_glass"},
		dpopJkt:          dpopJkt,
		attestationLevel: "",
		// v1_emergency_step_up は常に step_up 済みとして発行される（always ポリシー）
		stepUpProven: true,
		isValid:      true,
	}
}

// ToGucSetters は PostgreSQL GUC の SET LOCAL 文スライスを返す。
// is_valid が false の場合は空スライスを返す（無効な AuthContext で GUC を設定しない）。
// tier2 TenantContext の 4 GUC と組み合わせて session_context を構成する。
func (a *AuthContext) ToGucSetters() []string {
	// is_valid が false の場合は GUC setter を空にする（spec §AuthContext スキーマ準拠）
	if !a.isValid {
		return []string{}
	}
	// 04_認証適合仕様.md §AuthContext スキーマの全 GUC 対応フィールドを SET LOCAL 文にする
	setters := []string{
		// auth_class GUC を設定する
		fmt.Sprintf("SET LOCAL app.auth_class = '%s';", string(a.authClass)),
		// subject_id GUC を設定する（single quote をエスケープする）
		fmt.Sprintf("SET LOCAL app.subject_id = '%s';", strings.ReplaceAll(a.subjectID, "'", "''")),
		// subject_kind GUC を設定する
		fmt.Sprintf("SET LOCAL app.subject_kind = '%s';", a.subjectKind),
		// token_id GUC を設定する
		fmt.Sprintf("SET LOCAL app.token_id = '%s';", strings.ReplaceAll(a.tokenID, "'", "''")),
		// session_id GUC を設定する
		fmt.Sprintf("SET LOCAL app.session_id = '%s';", a.sessionID),
		// audience GUC を設定する
		fmt.Sprintf("SET LOCAL app.audience = '%s';", strings.ReplaceAll(a.audience, "'", "''")),
		// step_up_proven GUC を設定する（bool を文字列に変換する）
		fmt.Sprintf("SET LOCAL app.step_up_proven = '%v';", a.stepUpProven),
	}
	// dpop_jkt が空でない場合のみ GUC を設定する（dpop_bound_jwt のみ）
	if a.dpopJkt != "" {
		setters = append(setters, fmt.Sprintf("SET LOCAL app.dpop_jkt = '%s';", strings.ReplaceAll(a.dpopJkt, "'", "''")))
	}
	// attestation_level が空でない場合のみ GUC を設定する（jwt_attested のみ）
	if a.attestationLevel != "" {
		setters = append(setters, fmt.Sprintf("SET LOCAL app.attestation_level = '%s';", strings.ReplaceAll(a.attestationLevel, "'", "''")))
	}
	// 生成した GUC setter スライスを返す
	return setters
}
