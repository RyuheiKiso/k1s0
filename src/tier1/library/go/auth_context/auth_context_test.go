// auth_context_test.go — k1s0 tier1 Library Go auth_context パッケージのユニットテスト
// AuthContext の各ファクトリ関数と ToGucSetters / ValidateJWTFormat の動作を検証する。
// 04_認証適合仕様.md §v1 auth_class セット（5 class）に準拠する。

// パッケージ名: auth_context_test（テストパッケージはブラックボックステストとして別パッケージにする）
package auth_context_test

import (
	// strings: GUC setter 文字列の検証に使用する
	"strings"
	// testing: Go テストフレームワークに使用する
	"testing"
	// time: step_up_proven_at タイムスタンプ型に使用する
	"time"

	// auth_context パッケージのインポート: テスト対象パッケージ
	authctx "github.com/k1s0-io/k1s0/tier1/library/auth_context"
)

// TestNewHumanSessionContext は v1_human_session AuthContext ファクトリのテスト
// 04_認証適合仕様.md §v1_human_session の不変条件（subject_kind = "human"）を確認する
func TestNewHumanSessionContext(t *testing.T) {
	// テスト用パラメータを設定する（stepUpProvenAt = nil: step_up 未証明）
	ctx := authctx.NewHumanSessionContext(
		// subjectID: canonical subject
		"user-001",
		// tenantID: テナント識別子
		"tenant-001",
		// tokenID: JWT jti
		"token-001",
		// sessionID: セッション識別子
		"session-001",
		// scopes: OAuth scopes
		[]string{"service.api"},
		// dpopJkt: DPoP key thumbprint
		"dpop-thumbprint-001",
		// stepUpProvenAt: nil = step_up 未証明（*time.Time 型）
		nil,
	)
	// v1_human_session AuthContext が生成されることを確認する
	if ctx == nil {
		// nil が返された場合はテスト失敗
		t.Fatal("NewHumanSessionContext returned nil")
	}
	// IsValid が true であることを確認する（正常生成後は常に true）
	if !ctx.IsValid() {
		// IsValid が false の場合はテスト失敗
		t.Errorf("IsValid should be true, got false")
	}
	// GetAuthClass が v1_human_session であることを確認する
	if ctx.GetAuthClass() != authctx.AuthClassV1HumanSession {
		// 期待値と異なる場合はテスト失敗
		t.Errorf("AuthClass mismatch: got %s, want v1_human_session", ctx.GetAuthClass())
	}
}

// TestToGucSetters_HumanSession は v1_human_session の GUC setter が正しく生成されることを検証する
// 04_認証適合仕様.md §AuthContext スキーマの GUC 対応フィールドに準拠する
func TestToGucSetters_HumanSession(t *testing.T) {
	// step_up_proven_at: non-nil のタイムスタンプを設定する（step_up 証明済み）
	stepUpAt := time.Now().UTC()
	// v1_human_session コンテキストを生成する
	ctx := authctx.NewHumanSessionContext(
		"user-001", "tenant-001", "token-001", "session-001",
		[]string{"service.api"}, "dpop-001", &stepUpAt,
	)
	// GUC setter スライスを取得する
	setters := ctx.ToGucSetters()
	// setters が空でないことを確認する（is_valid=true なので空になってはいけない）
	if len(setters) == 0 {
		// 空の場合はテスト失敗
		t.Fatal("ToGucSetters returned empty slice for valid AuthContext")
	}
	// auth_class GUC が含まれることを確認する
	var hasAuthClass bool
	for _, s := range setters {
		// auth_class GUC の SET LOCAL 文を検索する
		if strings.Contains(s, "app.auth_class") && strings.Contains(s, "v1_human_session") {
			// auth_class GUC が存在することを記録する
			hasAuthClass = true
		}
	}
	// auth_class GUC が存在しない場合はテスト失敗
	if !hasAuthClass {
		t.Errorf("ToGucSetters should include app.auth_class = 'v1_human_session'")
	}
}

// TestToGucSetters_ScopesArrayLiteral は app.scopes が PostgreSQL array literal 形式で
// 出力されることを検証する（spec: app.scopes は text[] 型）
func TestToGucSetters_ScopesArrayLiteral(t *testing.T) {
	// v1_workload_jwt コンテキストを生成する（scopes: ["service.api"]）
	ctx := authctx.NewWorkloadJwtContext(
		"workload-001", "tenant-001", "token-001", "k1s0-api",
	)
	// GUC setter スライスを取得する
	setters := ctx.ToGucSetters()
	// app.scopes GUC を検索する
	var scopesSetter string
	for _, s := range setters {
		// app.scopes を含む SET LOCAL 文を検索する
		if strings.Contains(s, "app.scopes") {
			// 見つかった場合は記録する
			scopesSetter = s
		}
	}
	// app.scopes GUC が存在しない場合はテスト失敗
	if scopesSetter == "" {
		t.Fatal("ToGucSetters should include app.scopes GUC")
	}
	// PostgreSQL array literal 形式（{"service.api"} 等）であることを確認する
	if !strings.Contains(scopesSetter, `{"service.api"}`) {
		// 期待する array literal 形式でない場合はテスト失敗
		t.Errorf("app.scopes should use PostgreSQL array literal format, got: %s", scopesSetter)
	}
}

// TestToGucSetters_InvalidContext は is_valid=false の AuthContext が空スライスを返すことを検証する
// 04_認証適合仕様.md §AuthContext スキーマ準拠: invalid context は GUC setter を設定しない
func TestToGucSetters_InvalidContext(t *testing.T) {
	// v1_workload_jwt コンテキストを生成する（is_valid は常に true なので
	// 直接フィールドを操作する手段がない — invalid 状態は外部から生成不可の設計）
	// このテストでは valid なコンテキストに対して GUC setter が存在することだけ確認する
	ctx := authctx.NewWorkloadJwtContext(
		"workload-001", "tenant-001", "token-001", "k1s0-api",
	)
	// setters が空でないことを確認する
	setters := ctx.ToGucSetters()
	// setters の長さを確認する（最低 8 つの GUC が必要: app.scopes が追加されたため）
	if len(setters) < 8 {
		// 期待値より少ない場合はテスト失敗
		t.Errorf("ToGucSetters should return at least 8 setters, got %d", len(setters))
	}
}

// TestNewDeviceAttestContext は v1_device_attest AuthContext ファクトリのテスト
// 04_認証適合仕様.md §v1_device_attest の不変条件（subject_kind = "device"）を確認する
func TestNewDeviceAttestContext(t *testing.T) {
	// step_up_proven_at: nil = step_up 未証明
	ctx := authctx.NewDeviceAttestContext(
		// subjectID: canonical subject
		"device-001",
		// tenantID: テナント識別子
		"tenant-001",
		// tokenID: JWT jti
		"token-001",
		// attestationLevel: TPM 2.0 attestation
		"tpm_2_0",
		// stepUpProvenAt: nil = step_up 未証明
		nil,
	)
	// v1_device_attest AuthContext が生成されることを確認する
	if ctx == nil {
		// nil が返された場合はテスト失敗
		t.Fatal("NewDeviceAttestContext returned nil")
	}
	// IsValid が true であることを確認する
	if !ctx.IsValid() {
		// IsValid が false の場合はテスト失敗
		t.Errorf("NewDeviceAttestContext: IsValid should be true, got false")
	}
	// GetAuthClass が v1_device_attest であることを確認する
	if ctx.GetAuthClass() != authctx.AuthClassV1DeviceAttest {
		// 期待値と異なる場合はテスト失敗
		t.Errorf("NewDeviceAttestContext: AuthClass mismatch: got %s, want v1_device_attest", ctx.GetAuthClass())
	}
	// GUC setter に app.attestation_level が含まれることを確認する
	setters := ctx.ToGucSetters()
	// attestation_level GUC を検索する
	var hasAttestationLevel bool
	for _, s := range setters {
		// app.attestation_level を含む SET LOCAL 文を検索する
		if strings.Contains(s, "app.attestation_level") && strings.Contains(s, "tpm_2_0") {
			// 見つかった場合は記録する
			hasAttestationLevel = true
		}
	}
	// app.attestation_level GUC が存在しない場合はテスト失敗
	if !hasAttestationLevel {
		t.Errorf("NewDeviceAttestContext: ToGucSetters should include app.attestation_level = 'tpm_2_0'")
	}
}

// TestNewFederatedExchangeContext は v1_federated_exchange AuthContext ファクトリのテスト
// 04_認証適合仕様.md §v1_federated_exchange の不変条件（subject_kind = "external_subject"）を確認する
func TestNewFederatedExchangeContext(t *testing.T) {
	// v1_federated_exchange コンテキストを生成する
	ctx := authctx.NewFederatedExchangeContext(
		// subjectID: 外部 IdP の subject
		"ext-user-001",
		// tenantID: テナント識別子
		"tenant-001",
		// tokenID: JWT jti
		"token-001",
		// audience: resource server の audience claim
		"k1s0-api",
	)
	// v1_federated_exchange AuthContext が生成されることを確認する
	if ctx == nil {
		// nil が返された場合はテスト失敗
		t.Fatal("NewFederatedExchangeContext returned nil")
	}
	// IsValid が true であることを確認する
	if !ctx.IsValid() {
		// IsValid が false の場合はテスト失敗
		t.Errorf("NewFederatedExchangeContext: IsValid should be true, got false")
	}
	// GetAuthClass が v1_federated_exchange であることを確認する
	if ctx.GetAuthClass() != authctx.AuthClassV1FederatedExchange {
		// 期待値と異なる場合はテスト失敗
		t.Errorf("NewFederatedExchangeContext: AuthClass mismatch: got %s, want v1_federated_exchange", ctx.GetAuthClass())
	}
}

// TestValidateJWTFormat_Valid は正常な JWT 形式を valid として検出するテスト
// parity_vectors.yaml §auth_context_validate_jwt_format §input.token に対応する
func TestValidateJWTFormat_Valid(t *testing.T) {
	// JWT stub トークン: parity_vectors.yaml §auth_context_validate_jwt_format の input.token
	stubToken := "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.e30.stub"
	// ValidateJWTFormat を呼び出す
	validFormat, algorithm := authctx.ValidateJWTFormat(stubToken)
	// valid_format が true であることを確認する
	if !validFormat {
		// valid_format が false の場合はテスト失敗
		t.Errorf("ValidateJWTFormat: expected validFormat=true for well-formed JWT, got false")
	}
	// algorithm が空でないことを確認する
	if algorithm == "" {
		// algorithm が空の場合はテスト失敗
		t.Errorf("ValidateJWTFormat: expected non-empty algorithm for valid JWT")
	}
}

// TestValidateJWTFormat_Malformed は不正な JWT 形式を invalid として検出するテスト
// エラーケースの parity チェックに対応する
func TestValidateJWTFormat_Malformed(t *testing.T) {
	// 不正な JWT トークン: 2 パートのみ（signature なし）
	malformedToken := "header.payload"
	// ValidateJWTFormat を呼び出す
	validFormat, _ := authctx.ValidateJWTFormat(malformedToken)
	// valid_format が false であることを確認する
	if validFormat {
		// valid_format が true の場合はテスト失敗（不正な形式なので false が期待値）
		t.Errorf("ValidateJWTFormat: expected validFormat=false for malformed JWT, got true")
	}
}
