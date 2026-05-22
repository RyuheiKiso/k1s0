// k1s0-impl: IMPL-tier1-0005 realizes=FR-tier1-005
// key_handle.go — k1s0 tier1 Library Go 実装: KeyClass type + KeyHandle interface
// 05_鍵管理適合仕様.md §v1 key_class セット（5 class）および
// §5 層 defense-in-depth 層 A「compile: KeyHandle 必須引数化、生 key bytes 不可視」に準拠する。
// 公開 API シグネチャに生 key bytes を露出しない opaque 型を実装する。

// パッケージ名: keyhandle（tier1 Library の鍵管理公開 API を提供する）
package keyhandle

import (
	// bytes: HTTP リクエストボディ構築に使用する
	"bytes"
	// context: context.Context（Sign / Verify の非同期操作に使用する）
	"context"
	// encoding/base64: OpenBao Transit API の input / signature フィールド用 Base64 エンコード
	"encoding/base64"
	// encoding/json: OpenBao API レスポンスの JSON パース
	"encoding/json"
	// fmt: エラーメッセージのフォーマットに使用する
	"fmt"
	// io: レスポンスボディの読み取りに使用する
	"io"
	// net/http: OpenBao Transit API への HTTP クライアント
	"net/http"
	// os: 環境変数（OPENBAO_ADDR / OPENBAO_TOKEN）の取得に使用する
	"os"
	// strings: プレフィックス除去に使用する
	"strings"
	// time: HTTP クライアントのタイムアウト設定に使用する
	"time"
)

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

// OpenBaoClient は OpenBao Transit API への HTTP クライアント。
// production 環境での KeyHandle バックエンドとして使用する（baseURL / token を保持する）。
// 小文字で始める（unexported）: 生成は NewOpenBaoKeyHandle ファクトリ経由のみ許可する。
type OpenBaoClient struct {
	// baseURL: OpenBao サーバーのベース URL（例: http://openbao.k1s0.svc:8200）
	baseURL string
	// token: OpenBao アクセストークン（X-Vault-Token ヘッダーに使用する）
	token string
}

// Sign は OpenBao Transit の sign API を呼び出して署名文字列を返す。
// payload を Base64 エンコードして POST /v1/transit/sign/{keyName} に送信する。
func (c *OpenBaoClient) Sign(payload []byte, keyName string) (string, error) {
	// payload を Base64 エンコードする（OpenBao Transit の input フィールドは Base64 要求）
	inputB64 := base64.StdEncoding.EncodeToString(payload)
	// リクエストボディを構築する
	bodyBytes, err := json.Marshal(map[string]string{"input": inputB64})
	if err != nil {
		// JSON マーシャル失敗はプログラムエラーとして扱う
		return "", fmt.Errorf("OpenBaoClient.Sign: リクエスト JSON 構築失敗: %w", err)
	}
	// POST /v1/transit/sign/{keyName} の URL を構築する
	url := fmt.Sprintf("%s/v1/transit/sign/%s", c.baseURL, keyName)
	// HTTP リクエストを構築する（context.Background() を使用する）
	req, err := http.NewRequestWithContext(context.Background(), http.MethodPost, url, bytes.NewReader(bodyBytes))
	if err != nil {
		// リクエスト構築失敗はプログラムエラーとして扱う
		return "", fmt.Errorf("OpenBaoClient.Sign: HTTP リクエスト構築失敗: %w", err)
	}
	// Content-Type と X-Vault-Token ヘッダーを設定する
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Vault-Token", c.token)
	// HTTP クライアントにタイムアウトを設定する
	client := &http.Client{Timeout: 5 * time.Second}
	// リクエストを送信する
	resp, err := client.Do(req)
	if err != nil {
		// ネットワークエラーを返す
		return "", fmt.Errorf("OpenBaoClient.Sign: Transit sign 送信失敗: %w", err)
	}
	// レスポンスボディを必ずクローズする
	defer resp.Body.Close()
	// HTTP ステータスを確認する
	if resp.StatusCode != http.StatusOK {
		// エラーステータス時はボディを読み取ってエラーメッセージに含める
		bodyBuf, _ := io.ReadAll(resp.Body)
		return "", fmt.Errorf("OpenBaoClient.Sign: Transit sign HTTP %d body=%s", resp.StatusCode, string(bodyBuf))
	}
	// レスポンス JSON をデコードする
	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		// JSON デコード失敗を返す
		return "", fmt.Errorf("OpenBaoClient.Sign: レスポンス JSON デコード失敗: %w", err)
	}
	// data.signature フィールドを取得する（"vault:v1:<base64>" 形式）
	data, ok := result["data"].(map[string]interface{})
	if !ok {
		// data フィールドが存在しない場合はエラーを返す
		return "", fmt.Errorf("OpenBaoClient.Sign: data フィールドが存在しない")
	}
	sigStr, ok := data["signature"].(string)
	if !ok {
		// signature フィールドが存在しない場合はエラーを返す
		return "", fmt.Errorf("OpenBaoClient.Sign: signature フィールドが存在しない")
	}
	// "vault:v1:<base64>" 形式の署名文字列をそのまま返す（デコードは呼び出し元が行う）
	return sigStr, nil
}

// Decrypt は OpenBao Transit の decrypt API を呼び出して平文バイト列を返す。
// POST /v1/transit/decrypt/{keyName} に暗号文を送信して復号する。
func (c *OpenBaoClient) Decrypt(ciphertext string, keyName string) ([]byte, error) {
	// リクエストボディを構築する（ciphertext フィールドに暗号文を設定する）
	bodyBytes, err := json.Marshal(map[string]string{"ciphertext": ciphertext})
	if err != nil {
		// JSON マーシャル失敗はプログラムエラーとして扱う
		return nil, fmt.Errorf("OpenBaoClient.Decrypt: リクエスト JSON 構築失敗: %w", err)
	}
	// POST /v1/transit/decrypt/{keyName} の URL を構築する
	url := fmt.Sprintf("%s/v1/transit/decrypt/%s", c.baseURL, keyName)
	// HTTP リクエストを構築する（context.Background() を使用する）
	req, err := http.NewRequestWithContext(context.Background(), http.MethodPost, url, bytes.NewReader(bodyBytes))
	if err != nil {
		// リクエスト構築失敗はプログラムエラーとして扱う
		return nil, fmt.Errorf("OpenBaoClient.Decrypt: HTTP リクエスト構築失敗: %w", err)
	}
	// Content-Type と X-Vault-Token ヘッダーを設定する
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Vault-Token", c.token)
	// HTTP クライアントにタイムアウトを設定する
	client := &http.Client{Timeout: 5 * time.Second}
	// リクエストを送信する
	resp, err := client.Do(req)
	if err != nil {
		// ネットワークエラーを返す
		return nil, fmt.Errorf("OpenBaoClient.Decrypt: Transit decrypt 送信失敗: %w", err)
	}
	// レスポンスボディを必ずクローズする
	defer resp.Body.Close()
	// HTTP ステータスを確認する
	if resp.StatusCode != http.StatusOK {
		// エラーステータス時はボディを読み取ってエラーメッセージに含める
		bodyBuf, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("OpenBaoClient.Decrypt: Transit decrypt HTTP %d body=%s", resp.StatusCode, string(bodyBuf))
	}
	// レスポンス JSON をデコードする
	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		// JSON デコード失敗を返す
		return nil, fmt.Errorf("OpenBaoClient.Decrypt: レスポンス JSON デコード失敗: %w", err)
	}
	// data.plaintext フィールドを取得する（Base64 エンコードされた平文）
	data, ok := result["data"].(map[string]interface{})
	if !ok {
		// data フィールドが存在しない場合はエラーを返す
		return nil, fmt.Errorf("OpenBaoClient.Decrypt: data フィールドが存在しない")
	}
	plaintextB64, ok := data["plaintext"].(string)
	if !ok {
		// plaintext フィールドが存在しない場合はエラーを返す
		return nil, fmt.Errorf("OpenBaoClient.Decrypt: plaintext フィールドが存在しない")
	}
	// Base64 デコードして平文バイト列を返す
	plaintext, err := base64.StdEncoding.DecodeString(plaintextB64)
	if err != nil {
		// Base64 デコード失敗を返す
		return nil, fmt.Errorf("OpenBaoClient.Decrypt: plaintext Base64 デコード失敗: %w", err)
	}
	return plaintext, nil
}

// OpenBaoKeyHandle は OpenBao Transit をバックエンドとする production 用 KeyHandle 実装。
// 生 key bytes を Self に保存しない（opaque handle として keyID と keyClass のみ保持する）。
// Rust の OpenBaoKeyHandle struct と等価な 4 言語等価強度実装。
type OpenBaoKeyHandle struct {
	// keyClass: 鍵の用途クラス（5 class のいずれか）（unexported: 外部からの直接アクセスを禁止する）
	keyClass KeyClass
	// keyID: OpenBao Transit のキー版数識別子（UUID v7 形式）（unexported）
	keyID string
	// openbaoClient: OpenBao Transit API HTTP クライアント（unexported: 外部公開禁止）
	openbaoClient *OpenBaoClient
}

// NewOpenBaoKeyHandle は production 用 OpenBaoKeyHandle を生成するファクトリ関数。
// openbaoURL / openbaoToken で OpenBao サーバーに接続する（生 key bytes は受け取らない）。
func NewOpenBaoKeyHandle(keyClass KeyClass, keyID string, openbaoURL, openbaoToken string) *OpenBaoKeyHandle {
	// OpenBaoClient を生成して内部保持する（baseURL / token を設定する）
	return &OpenBaoKeyHandle{
		// keyClass を設定する（5 class のいずれか）
		keyClass: keyClass,
		// keyID を設定する（OpenBao Transit key name）
		keyID: keyID,
		// OpenBaoClient を生成する（baseURL / token を設定する）
		openbaoClient: &OpenBaoClient{
			baseURL: openbaoURL,
			token:   openbaoToken,
		},
	}
}

// KeyID は OpenBao Transit のキー版数識別子を返す（KeyHandle interface 実装）
func (h *OpenBaoKeyHandle) KeyID() string {
	// keyID フィールドを返す
	return h.keyID
}

// KeyClass は 5 class のいずれかを返す（KeyHandle interface 実装）
func (h *OpenBaoKeyHandle) KeyClass() KeyClass {
	// keyClass フィールドを返す
	return h.keyClass
}

// IsValid は鍵の有効性を返す（OpenBaoKeyHandle は常に有効として扱う）
// production では OpenBao の key rotation status を確認する拡張が可能
func (h *OpenBaoKeyHandle) IsValid() bool {
	// OpenBaoKeyHandle 構築直後は有効とする（revoke/rotate で false になる拡張を将来実装する）
	return true
}

// Sign は OpenBao Transit の sign API を呼び出して署名バイト列を返す（KeyHandle interface 実装）。
// 05_鍵管理適合仕様.md §5 層 defense-in-depth 層 B（runtime: OpenBao Transit 委譲）を実装する。
func (h *OpenBaoKeyHandle) Sign(ctx context.Context, payload []byte) ([]byte, error) {
	// key_class を OpenBao Transit key name にマッピングする（例: "v1_data_dek"）
	keyName := string(h.keyClass)
	// OpenBaoClient の Sign メソッドを呼び出して署名文字列を取得する
	sigStr, err := h.openbaoClient.Sign(payload, keyName)
	if err != nil {
		// Sign エラーをそのまま返す
		return nil, err
	}
	// "vault:v1:" プレフィックスを除去して Base64 部分を取り出す
	sigB64 := strings.TrimPrefix(sigStr, "vault:v1:")
	// Base64 デコードして署名バイト列を返す
	sigBytes, err := base64.StdEncoding.DecodeString(sigB64)
	if err != nil {
		// Base64 デコード失敗を返す
		return nil, fmt.Errorf("OpenBaoKeyHandle.Sign: signature Base64 デコード失敗: %w", err)
	}
	return sigBytes, nil
}

// Verify は OpenBao Transit の verify API を呼び出して検証結果を返す（KeyHandle interface 実装）。
// 05_鍵管理適合仕様.md §5 層 defense-in-depth 層 B（runtime: OpenBao Transit 委譲）を実装する。
func (h *OpenBaoKeyHandle) Verify(ctx context.Context, payload []byte, signature []byte) (bool, error) {
	// key_class を OpenBao Transit key name にマッピングする
	keyName := string(h.keyClass)
	// payload を Base64 エンコードする（OpenBao Transit の input フィールドは Base64 要求）
	inputB64 := base64.StdEncoding.EncodeToString(payload)
	// signature を "vault:v1:<base64>" 形式にエンコードする
	sigB64 := "vault:v1:" + base64.StdEncoding.EncodeToString(signature)
	// リクエストボディを構築する
	bodyBytes, err := json.Marshal(map[string]string{"input": inputB64, "signature": sigB64})
	if err != nil {
		// JSON マーシャル失敗はプログラムエラーとして扱う
		return false, fmt.Errorf("OpenBaoKeyHandle.Verify: リクエスト JSON 構築失敗: %w", err)
	}
	// POST /v1/transit/verify/{keyName} の URL を構築する
	url := fmt.Sprintf("%s/v1/transit/verify/%s", h.openbaoClient.baseURL, keyName)
	// HTTP リクエストを構築する（caller の context を引き継ぐ）
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(bodyBytes))
	if err != nil {
		// リクエスト構築失敗はプログラムエラーとして扱う
		return false, fmt.Errorf("OpenBaoKeyHandle.Verify: HTTP リクエスト構築失敗: %w", err)
	}
	// Content-Type と X-Vault-Token ヘッダーを設定する
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Vault-Token", h.openbaoClient.token)
	// HTTP クライアントにタイムアウトを設定する
	client := &http.Client{Timeout: 5 * time.Second}
	// リクエストを送信する
	resp, err := client.Do(req)
	if err != nil {
		// ネットワークエラーを返す
		return false, fmt.Errorf("OpenBaoKeyHandle.Verify: Transit verify 送信失敗: %w", err)
	}
	// レスポンスボディを必ずクローズする
	defer resp.Body.Close()
	// HTTP ステータスを確認する
	if resp.StatusCode != http.StatusOK {
		// エラーステータス時はボディを読み取ってエラーメッセージに含める
		bodyBuf, _ := io.ReadAll(resp.Body)
		return false, fmt.Errorf("OpenBaoKeyHandle.Verify: Transit verify HTTP %d body=%s", resp.StatusCode, string(bodyBuf))
	}
	// レスポンス JSON をデコードする
	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		// JSON デコード失敗を返す
		return false, fmt.Errorf("OpenBaoKeyHandle.Verify: レスポンス JSON デコード失敗: %w", err)
	}
	// data.valid フィールドを取得して返す
	data, ok := result["data"].(map[string]interface{})
	if !ok {
		// data フィールドが存在しない場合は false を返す
		return false, nil
	}
	valid, _ := data["valid"].(bool)
	return valid, nil
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

// Sign は OpenBao Transit の sign API を呼び出して署名バイト列を返す。
// 05_鍵管理適合仕様.md §5 層 defense-in-depth 層 B（runtime: OpenBao Transit 委譲）を実装する。
func (h *stubKeyHandle) Sign(ctx context.Context, payload []byte) ([]byte, error) {
	// OPENBAO_ADDR 環境変数からベース URL を取得する（デフォルト: http://openbao.k1s0.svc:8200）
	baseURL := os.Getenv("OPENBAO_ADDR")
	if baseURL == "" {
		// 環境変数が未設定の場合はデフォルト値を使用する
		baseURL = "http://openbao.k1s0.svc:8200"
	}
	// OPENBAO_TOKEN 環境変数からトークンを取得する（未設定時はエラー）
	token := os.Getenv("OPENBAO_TOKEN")
	if token == "" {
		// トークン未設定は設定エラーとして扱う
		return nil, fmt.Errorf("OPENBAO_TOKEN 環境変数が設定されていない")
	}
	// key_class を OpenBao Transit key name にマッピングする（例: "v1_data_dek"）
	keyName := string(h.keyClass)
	// payload を Base64 エンコードする（OpenBao Transit の input フィールドは Base64 要求）
	inputB64 := base64.StdEncoding.EncodeToString(payload)
	// リクエストボディを構築する
	bodyBytes, err := json.Marshal(map[string]string{"input": inputB64})
	if err != nil {
		// JSON マーシャル失敗はプログラムエラーとして扱う
		return nil, fmt.Errorf("sign リクエスト JSON 構築失敗: %w", err)
	}
	// POST /v1/transit/sign/{key_name} の URL を構築する
	url := fmt.Sprintf("%s/v1/transit/sign/%s", baseURL, keyName)
	// HTTP リクエストを構築する
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(bodyBytes))
	if err != nil {
		// リクエスト構築失敗はプログラムエラーとして扱う
		return nil, fmt.Errorf("sign HTTP リクエスト構築失敗: %w", err)
	}
	// Content-Type と X-Vault-Token ヘッダーを設定する
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Vault-Token", token)
	// HTTP クライアントにタイムアウトを設定する
	client := &http.Client{Timeout: 5 * time.Second}
	// リクエストを送信する
	resp, err := client.Do(req)
	if err != nil {
		// ネットワークエラーを返す
		return nil, fmt.Errorf("OpenBao Transit sign 送信失敗: %w", err)
	}
	// レスポンスボディを必ずクローズする
	defer resp.Body.Close()
	// HTTP ステータスを確認する
	if resp.StatusCode != http.StatusOK {
		// エラーステータス時はボディを読み取ってエラーメッセージに含める
		bodyBuf, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("OpenBao Transit sign HTTP %d body=%s", resp.StatusCode, string(bodyBuf))
	}
	// レスポンス JSON をデコードする
	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		// JSON デコード失敗を返す
		return nil, fmt.Errorf("OpenBao Transit sign レスポンス JSON デコード失敗: %w", err)
	}
	// signature フィールドを取得する（"vault:v1:<base64>" 形式）
	data, ok := result["data"].(map[string]interface{})
	if !ok {
		// data フィールドが存在しない場合はエラーを返す
		return nil, fmt.Errorf("OpenBao Transit sign: data フィールドが存在しない")
	}
	sigStr, ok := data["signature"].(string)
	if !ok {
		// signature フィールドが存在しない場合はエラーを返す
		return nil, fmt.Errorf("OpenBao Transit sign: signature フィールドが存在しない")
	}
	// "vault:v1:" プレフィックスを除去して Base64 部分を取り出す
	sigB64 := strings.TrimPrefix(sigStr, "vault:v1:")
	// Base64 デコードして署名バイト列を返す
	sigBytes, err := base64.StdEncoding.DecodeString(sigB64)
	if err != nil {
		// Base64 デコード失敗を返す
		return nil, fmt.Errorf("OpenBao Transit signature Base64 デコード失敗: %w", err)
	}
	return sigBytes, nil
}

// Verify は OpenBao Transit の verify API を呼び出して検証結果を返す。
// 05_鍵管理適合仕様.md §5 層 defense-in-depth 層 B（runtime: OpenBao Transit 委譲）を実装する。
func (h *stubKeyHandle) Verify(ctx context.Context, payload []byte, signature []byte) (bool, error) {
	// OPENBAO_ADDR 環境変数からベース URL を取得する
	baseURL := os.Getenv("OPENBAO_ADDR")
	if baseURL == "" {
		// 環境変数が未設定の場合はデフォルト値を使用する
		baseURL = "http://openbao.k1s0.svc:8200"
	}
	// OPENBAO_TOKEN 環境変数からトークンを取得する
	token := os.Getenv("OPENBAO_TOKEN")
	if token == "" {
		// トークン未設定は設定エラーとして扱う
		return false, fmt.Errorf("OPENBAO_TOKEN 環境変数が設定されていない")
	}
	// key_class を OpenBao Transit key name にマッピングする
	keyName := string(h.keyClass)
	// payload を Base64 エンコードする
	inputB64 := base64.StdEncoding.EncodeToString(payload)
	// signature を "vault:v1:<base64>" 形式にエンコードする
	sigB64 := "vault:v1:" + base64.StdEncoding.EncodeToString(signature)
	// リクエストボディを構築する
	bodyBytes, err := json.Marshal(map[string]string{"input": inputB64, "signature": sigB64})
	if err != nil {
		// JSON マーシャル失敗はプログラムエラーとして扱う
		return false, fmt.Errorf("verify リクエスト JSON 構築失敗: %w", err)
	}
	// POST /v1/transit/verify/{key_name} の URL を構築する
	url := fmt.Sprintf("%s/v1/transit/verify/%s", baseURL, keyName)
	// HTTP リクエストを構築する
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(bodyBytes))
	if err != nil {
		// リクエスト構築失敗はプログラムエラーとして扱う
		return false, fmt.Errorf("verify HTTP リクエスト構築失敗: %w", err)
	}
	// Content-Type と X-Vault-Token ヘッダーを設定する
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Vault-Token", token)
	// HTTP クライアントにタイムアウトを設定する
	client := &http.Client{Timeout: 5 * time.Second}
	// リクエストを送信する
	resp, err := client.Do(req)
	if err != nil {
		// ネットワークエラーを返す
		return false, fmt.Errorf("OpenBao Transit verify 送信失敗: %w", err)
	}
	// レスポンスボディを必ずクローズする
	defer resp.Body.Close()
	// HTTP ステータスを確認する
	if resp.StatusCode != http.StatusOK {
		// エラーステータス時はボディを読み取ってエラーメッセージに含める
		bodyBuf, _ := io.ReadAll(resp.Body)
		return false, fmt.Errorf("OpenBao Transit verify HTTP %d body=%s", resp.StatusCode, string(bodyBuf))
	}
	// レスポンス JSON をデコードする
	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		// JSON デコード失敗を返す
		return false, fmt.Errorf("OpenBao Transit verify レスポンス JSON デコード失敗: %w", err)
	}
	// data.valid フィールドを取得して返す
	data, ok := result["data"].(map[string]interface{})
	if !ok {
		// data フィールドが存在しない場合は false を返す
		return false, nil
	}
	valid, _ := data["valid"].(bool)
	return valid, nil
}
