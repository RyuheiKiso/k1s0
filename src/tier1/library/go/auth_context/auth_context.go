// auth_context.go — k1s0 tier1 Library Go 実装: auth_context スタンドアロンパッケージ
// keyhandle パッケージの AuthContext 実装を auth_context パッケージとして再エクスポートする。
// 04_認証適合仕様.md §v1 auth_class セット（5 class）および
// §AuthContext スキーマ（32 session_context の拡張・同型）に準拠する。
// 4 言語等価強度: Rust / C# / Go / TypeScript の AuthContext と同一インタフェースを提供する。

// パッケージ名: auth_context（tier1 Library の認証コンテキスト専用パッケージ）
package auth_context

import (
	// encoding/base64: JWT header の Base64URL デコードに使用する
	"encoding/base64"
	// encoding/json: JWT header の JSON unmarshal に使用する
	"encoding/json"
	// errors: エラー生成に使用する
	"errors"
	// fmt: SQL 文字列フォーマットに使用する
	"fmt"
	// strings: strings.Join で scopes を結合する
	"strings"
	// time: step_up_proven_at タイムスタンプ型に使用する
	"time"

	// uuid: session_id の UUID v4 生成に使用する（device attest / federated exchange）
	"github.com/google/uuid"
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
	attestationLevel *string
	// issuedVia: token 発行経路（v1_federated_exchange では "rfc8693_token_exchange"）
	// 空文字列 = 通常発行（human / workload / device / emergency）
	issuedVia string
	// stepUpProvenAt: 最終 step_up challenge 時刻（04_認証適合仕様.md §AuthContext スキーマ SoT 準拠）
	// nil = 未証明（workload / federated 等 step_up 不要クラス）、non-nil = challenge 完了時刻
	stepUpProvenAt *time.Time
	// isValid: token 検証が成功したかどうか（false の場合は GUC setter を空にする）
	isValid bool
}

// GetAuthClass は認証クラスを返す（読み取り専用アクセサ）
func (a *AuthContext) GetAuthClass() AuthClass {
	// authClass フィールドを返す
	return a.authClass
}

// GetSubjectID は canonical subject を返す（読み取り専用アクセサ）
func (a *AuthContext) GetSubjectID() string {
	// subjectID フィールドを返す
	return a.subjectID
}

// IsValid は token 検証結果を返す（読み取り専用アクセサ）
func (a *AuthContext) IsValid() bool {
	// isValid フィールドを返す
	return a.isValid
}

// ValidateJWTFormat は JWT トークン文字列の形式を検証する。
// parity_vectors.yaml §auth_context_validate_jwt_format に対応する。
// token: 検証対象の JWT 文字列（header.payload.signature の 3 パート構造）
// returns: valid_format（bool）, algorithm（string）
func ValidateJWTFormat(token string) (validFormat bool, algorithm string) {
	// JWT トークンをドット区切りで分割する
	parts := strings.SplitN(token, ".", 3)
	// 3 パートでなければ形式不正として false を返す
	if len(parts) != 3 {
		return false, ""
	}
	// 各パートが空でないことを確認する（空パートは不正な JWT 形式）
	for _, part := range parts {
		if part == "" {
			return false, ""
		}
	}
	// header パートを Base64URL デコードして alg フィールドを取り出す
	// RFC 7515 §2: JWT header は Base64URL エンコードされた JSON オブジェクト
	decoded, err := base64.RawURLEncoding.DecodeString(parts[0])
	// Base64URL デコードに失敗した場合は形式不正として false を返す
	if err != nil {
		return false, ""
	}
	// JSON 形式の JWT header を unmarshal して alg フィールドを取得する
	var header struct {
		// Alg: JWT header の "alg" フィールド（署名アルゴリズム名）
		Alg string `json:"alg"`
	}
	// JSON unmarshal に失敗した場合は形式不正として false を返す
	if err := json.Unmarshal(decoded, &header); err != nil {
		return false, ""
	}
	// alg フィールドが空の場合は形式不正（RFC 7515 §4.1.1: "alg" は必須）
	if header.Alg == "" {
		return false, ""
	}
	// header.Alg を algorithm として返す（実際のアルゴリズム名）
	algorithm = header.Alg
	// 3 パート構造 + Base64URL header + alg 非空 → 形式正常
	return true, algorithm
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
	// stepUpProvenAt: step_up challenge 完了時刻（未証明の場合は nil）
	stepUpProvenAt *time.Time,
) *AuthContext {
	// v1_human_session の固定属性を適用する（dimension override 禁止）
	return &AuthContext{
		authClass: AuthClassV1HumanSession,
		subjectID: subjectID,
		// human session の subject_kind は常に "human"（spec §各 class の不変条件）
		subjectKind: "human",
		tokenID:     tokenID,
		sessionID:   sessionID,
		tenantID:    tenantID,
		audience:    "",
		scopes:      scopes,
		dpopJkt:     dpopJkt,
		// human session には attestationLevel は不要
		attestationLevel: nil,
		issuedVia:        "",
		stepUpProvenAt:   stepUpProvenAt,
		isValid:          true,
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
		tokenID:     tokenID,
		sessionID:   "",
		tenantID:    tenantID,
		audience:    audience,
		scopes:      []string{"service.api"},
		dpopJkt:     "",
		// workload には attestationLevel は不要
		attestationLevel: nil,
		issuedVia:        "",
		// workload は step_up が不要（never ポリシー）。stepUpProvenAt は nil
		stepUpProvenAt: nil,
		isValid:        true,
	}
}

// NewDeviceAttestContext は v1_device_attest AuthContext を構築するファクトリ関数。
// 工場端末・KIOSK・現場ハンドヘルド（device cert、長 TTL、one_shot refresh）に対応する。
// Rust の new_device_attest と等価な 4 言語等価強度実装。
func NewDeviceAttestContext(
	subjectID string,
	tenantID string,
	tokenID string,
	attestationLevel string,
	// stepUpProvenAt: step_up challenge 完了時刻（未証明の場合は nil）
	stepUpProvenAt *time.Time,
) *AuthContext {
	// v1_device_attest の固定属性を適用する（dimension override 禁止）
	return &AuthContext{
		authClass: AuthClassV1DeviceAttest,
		subjectID: subjectID,
		// device attest の subject_kind は常に "device"（spec §各 class の不変条件）
		subjectKind: "device",
		tokenID:     tokenID,
		// device は session を持つ（device registration session ID を UUID v4 で生成する）
		sessionID: uuid.New().String(),
		tenantID:  tenantID,
		// device attest では audience は空文字列（resource server は token_id で管理する）
		audience: "",
		// device のデフォルトスコープ（device.api のみ）
		scopes:  []string{"device.api"},
		dpopJkt: "",
		// attestationLevel: TPM / HSM / WebAuthn platform authenticator の種別
		attestationLevel: &attestationLevel,
		issuedVia:        "",
		stepUpProvenAt:   stepUpProvenAt,
		isValid:          true,
	}
}

// NewFederatedExchangeContext は v1_federated_exchange AuthContext を構築するファクトリ関数。
// 外部 IdP からの RFC 8693 token exchange（audience-restricted JWT）に対応する。
// Rust の new_federated_exchange と等価な 4 言語等価強度実装。
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
		// federated exchange は session を持たない（短命 JWT で session 管理不要）
		sessionID: "",
		tenantID:  tenantID,
		// audience は federated exchange で必須（resource server を audience claim で絞る）
		audience: audience,
		// federated exchange のデフォルトスコープ（federated.api のみ）
		scopes:  []string{"federated.api"},
		dpopJkt: "",
		// federated には attestationLevel は不要
		attestationLevel: nil,
		// RFC 8693 token exchange 経由で発行されたことを記録する
		issuedVia: "rfc8693_token_exchange",
		// federated exchange は step_up 不要（never ポリシー）。stepUpProvenAt は nil
		stepUpProvenAt: nil,
		isValid:        true,
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
	// stepUpProvenAt: emergency factory では必須（always step_up ポリシーのため non-nil 必須）
	stepUpProvenAt time.Time,
) *AuthContext {
	// v1_emergency_step_up の固定属性を適用する（always step_up + purpose=emergency 強制）
	return &AuthContext{
		authClass: AuthClassV1EmergencyStepUp,
		subjectID: subjectID,
		// emergency の subject_kind は常に "human"（workload による break-glass 禁止）
		subjectKind: "human",
		tokenID:     tokenID,
		sessionID:   sessionID,
		tenantID:    tenantID,
		audience:    "",
		scopes:      []string{"emergency.break_glass"},
		dpopJkt:     dpopJkt,
		// emergency には attestationLevel は不要
		attestationLevel: nil,
		issuedVia:        "",
		// v1_emergency_step_up は常に step_up 済みとして発行される（always ポリシー）。ポインタを設定する
		stepUpProvenAt: &stepUpProvenAt,
		isValid:        true,
	}
}

// ToGucSetters は PostgreSQL GUC の SET LOCAL 文スライスを返す。
// is_valid が false の場合は空スライスを返す（無効な AuthContext で GUC を設定しない）。
func (a *AuthContext) ToGucSetters() []string {
	// is_valid が false の場合は GUC setter を空にする（spec §AuthContext スキーマ準拠）
	if !a.isValid {
		return []string{}
	}
	// 04_認証適合仕様.md §AuthContext スキーマの全 GUC 対応フィールドを SET LOCAL 文にする
	// step_up_proven_at: nil の場合は空文字列、non-nil の場合は RFC 3339 形式で emit する
	stepUpProvenAtStr := ""
	// stepUpProvenAt が non-nil の場合は RFC 3339 形式にフォーマットする
	if a.stepUpProvenAt != nil {
		// RFC 3339 形式（ISO 8601）で emit する
		stepUpProvenAtStr = a.stepUpProvenAt.UTC().Format(time.RFC3339)
	}
	// scopes を PostgreSQL native array literal に変換する（spec: app.scopes は text[] 型）
	// 各 scope を二重引用符で囲み、{...} でラップする（例: {"service.api","device.api"}）
	quotedScopes := make([]string, len(a.scopes))
	// 各 scope を二重引用符でクォートして配列要素にする
	for i, s := range a.scopes {
		// PostgreSQL array literal の要素は二重引用符でクォートする
		quotedScopes[i] = `"` + strings.ReplaceAll(s, `"`, `\"`) + `"`
	}
	// {scope1,scope2,...} 形式の PostgreSQL array literal を生成する
	scopesArrayLiteral := fmt.Sprintf("{%s}", strings.Join(quotedScopes, ","))
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
		// scopes GUC を PostgreSQL array literal 形式で設定する（spec: text[] 型）
		fmt.Sprintf("SET LOCAL app.scopes = '%s';", scopesArrayLiteral),
		// step_up_proven_at GUC を設定する（GUC 名を app.step_up_proven_at に変更）
		fmt.Sprintf("SET LOCAL app.step_up_proven_at = '%s';", stepUpProvenAtStr),
	}
	// dpop_jkt が空でない場合のみ GUC を設定する（dpop_bound_jwt のみ）
	if a.dpopJkt != "" {
		setters = append(setters, fmt.Sprintf("SET LOCAL app.dpop_jkt = '%s';", strings.ReplaceAll(a.dpopJkt, "'", "''")))
	}
	// attestation_level が non-nil の場合のみ GUC を設定する（jwt_attested のみ）
	if a.attestationLevel != nil {
		setters = append(setters, fmt.Sprintf("SET LOCAL app.attestation_level = '%s';", strings.ReplaceAll(*a.attestationLevel, "'", "''")))
	}
	// 生成した GUC setter スライスを返す
	return setters
}
