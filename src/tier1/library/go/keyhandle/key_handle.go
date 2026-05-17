// key_handle.go — k1s0 tier1 Library Go 実装: KeyClass type + KeyHandle interface
// 05_鍵管理適合仕様.md §v1 key_class セット（5 class）および
// §5 層 defense-in-depth 層 A「compile: KeyHandle 必須引数化、生 key bytes 不可視」に準拠する。
// 公開 API シグネチャに生 key bytes を露出しない opaque 型を実装する。

// パッケージ名: keyhandle（tier1 Library の鍵管理公開 API を提供する）
package keyhandle

// context: context.Context（Sign / Verify の非同期操作に使用する）
import "context"

// KeyClass は 05_鍵管理適合仕様.md §v1 key_class セット（5 class）を宣言する型。
// class 1 値が purpose / rotation_cadence / scope / backend / destruction_method を一意に導出する
//（dimension override 禁止）。
type KeyClass string

// KeyClass の定数定義: spec の class 名（snake_case）と 1:1 対応する
const (
	// KeyClassV1DataDek: データ暗号化鍵（DEK）— per-tenant / software_kms_wrapped / crypto_shred
	KeyClassV1DataDek KeyClass = "v1_data_dek"
	// KeyClassV1DataKek: 鍵暗号化鍵（KEK）— per-tenant / hsm_pkcs11_shamir_distributed / hsm_zeroize_all_shares
	KeyClassV1DataKek KeyClass = "v1_data_kek"
	// KeyClassV1TokenSigning: JWT / DPoP 署名鍵 — platform / hsm_pkcs11 / jwks_revoke
	KeyClassV1TokenSigning KeyClass = "v1_token_signing"
	// KeyClassV1AuditRootSigning: audit hash chain root 署名鍵 — per-tenant / hsm_pkcs11 / external_notary_attest
	KeyClassV1AuditRootSigning KeyClass = "v1_audit_root_signing"
	// KeyClassV1MtlsWorkload: workload mTLS 鍵 — per_workload / spire / spire_revoke
	KeyClassV1MtlsWorkload KeyClass = "v1_mtls_workload"
)

// KeyHandle は生 key bytes を公開しない opaque 鍵抽象 interface。
// 05_鍵管理適合仕様.md §KeyHandle / KeyMaterial の言語横断型 に準拠する。
// Sign / Verify は OpenBao Transit への委譲として実装し、key bytes は tier1 境界を越えない。
type KeyHandle interface {
	// KeyID は OpenBao Transit のキー版数識別子（UUID v7 形式）を返す。
	KeyID() string
	// KeyClass は 5 class のいずれかを返す（purpose bundle の代表値）。
	KeyClass() KeyClass
	// IsValid は OpenBao による鍵の有効性確認結果を返す（revoke / rotate 後 false になる）。
	IsValid() bool
	// Sign は payload を鍵で署名し、署名バイト列を返す。
	// 生 key bytes は引数・戻り値のいずれにも含まれない（OpenBao Transit 委譲）。
	Sign(ctx context.Context, payload []byte) ([]byte, error)
	// Verify は payload と signature の一致を検証し、真偽値を返す。
	// 生 key bytes は引数・戻り値のいずれにも含まれない（OpenBao Transit 委譲）。
	Verify(ctx context.Context, payload []byte, signature []byte) (bool, error)
}

// stubKeyHandle は OpenBao Transit 呼出なしに動作する stub 実装。
// テスト・ドライラン用途（production では OpenBao 経由の KeyHandle 実装を使う）。
// 小文字で始める（unexported）: 生成は NewStubKeyHandle ファクトリ経由のみ許可する。
type stubKeyHandle struct {
	// keyID: OpenBao Transit のキー版数識別子（unexported: 外部からの直接アクセスを禁止する）
	keyID string
	// keyClass: 鍵の用途クラス（unexported: 外部からの直接アクセスを禁止する）
	keyClass KeyClass
	// isValid: OpenBao による有効性確認結果（unexported: 外部からの直接アクセスを禁止する）
	isValid bool
}

// NewStubKeyHandle は stub の KeyHandle を生成するファクトリ関数。
// 生 key bytes は受け取らない設計（spec §5 層 defense-in-depth 層 A に準拠する）。
func NewStubKeyHandle(keyID string, keyClass KeyClass, isValid bool) KeyHandle {
	// stubKeyHandle を生成して KeyHandle interface として返す
	return &stubKeyHandle{
		// keyID を設定する
		keyID: keyID,
		// keyClass を設定する
		keyClass: keyClass,
		// isValid を設定する
		isValid: isValid,
	}
}

// KeyID は OpenBao Transit のキー版数識別子を返す
func (h *stubKeyHandle) KeyID() string {
	// keyID を返す
	return h.keyID
}

// KeyClass は 5 class のいずれかを返す
func (h *stubKeyHandle) KeyClass() KeyClass {
	// keyClass を返す
	return h.keyClass
}

// IsValid は鍵の有効性を返す
func (h *stubKeyHandle) IsValid() bool {
	// isValid を返す
	return h.isValid
}

// Sign は stub 実装として空バイトスライスを返す。
// production 実装では OpenBao Transit /v1/transit/sign/:name を呼び出す。
func (h *stubKeyHandle) Sign(_ context.Context, _ []byte) ([]byte, error) {
	// stub: OpenBao Transit への委譲先は bfl/openbao.go を参照する
	return []byte{}, nil
}

// Verify は stub 実装として常に true を返す。
// production 実装では OpenBao Transit /v1/transit/verify/:name を呼び出す。
func (h *stubKeyHandle) Verify(_ context.Context, _ []byte, _ []byte) (bool, error) {
	// stub: OpenBao Transit への委譲先は bfl/openbao.go を参照する
	return true, nil
}
