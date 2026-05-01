// 本ファイルは OpenBao Transit（暗号化サービス）の adapter。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/04_Secrets_API.md
//     - FR-T1-SECRETS-003（Transit 暗号化）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//
// 役割:
//   tier1 Secrets API の Encrypt / Decrypt / RotateKey を OpenBao Transit Engine 越しに
//   提供する narrow interface を定義する。production の OpenBao は AES-256-GCM の
//   convergent encryption を提供するが、本リポジトリの dev / CI 経路では
//   in-memory backend（InMemoryTransit）を使用する。
//
// 受け入れ基準（FR-T1-SECRETS-003）:
//   - 暗号化アルゴリズムは AES-256-GCM 固定
//   - 鍵名は <tenant_id>.<key_label> で tier1 が自動プレフィックス（呼出側で TenantPrefix が責務）
//   - 鍵バージョン管理が自動（RotateKey で新版生成、旧版でも Decrypt 可能）
//   - 復号時は ciphertext 中のバージョン番号から適切な鍵を選択
//
// Ciphertext フォーマット（in-memory / production 共通）:
//   [4 byte BE: key_version][12 byte: GCM nonce][N byte: AES-256-GCM(ciphertext+tag)]
//   - version は 1 始まり（RotateKey で +1）
//   - nonce は AES-GCM 推奨の 96 bit（12 byte）
//   - tag は 16 byte で ciphertext 末尾に含まれる（Go の cipher.AEAD 慣用）

package openbao

import (
	// 鍵管理の並行制御。
	"sync"
)

// TransitEncryptRequest は Transit.Encrypt の adapter 入力。
type TransitEncryptRequest struct {
	// tenant prefix 付与済の鍵名（"<tenant_id>.<key_label>" の形）。
	KeyName string
	// 平文。
	Plaintext []byte
	// AAD（Associated Authenticated Data）。GCM の追加認証データ。
	AAD []byte
}

// TransitEncryptResponse は Transit.Encrypt の adapter 応答。
type TransitEncryptResponse struct {
	// ciphertext（フォーマットは本ファイル冒頭を参照）。
	Ciphertext []byte
	// 暗号化に使用した鍵バージョン（observability 用）。
	KeyVersion int
}

// TransitDecryptRequest は Transit.Decrypt の adapter 入力。
type TransitDecryptRequest struct {
	// tenant prefix 付与済の鍵名。
	KeyName string
	// 暗号文。
	Ciphertext []byte
	// AAD（Encrypt 時と同じ値が必要）。
	AAD []byte
}

// TransitDecryptResponse は Transit.Decrypt の adapter 応答。
type TransitDecryptResponse struct {
	// 平文。
	Plaintext []byte
	// 復号に使用した鍵バージョン。
	KeyVersion int
}

// TransitRotateKeyRequest は Transit.RotateKey の adapter 入力。
type TransitRotateKeyRequest struct {
	// tenant prefix 付与済の鍵名。
	KeyName string
}

// TransitRotateKeyResponse は Transit.RotateKey の adapter 応答。
type TransitRotateKeyResponse struct {
	// 新バージョン番号。
	NewVersion int
	// 旧バージョン番号（new_version - 1）。
	PreviousVersion int
	// ローテーション時刻（Unix epoch ミリ秒）。
	RotatedAtMs int64
}

// TransitAdapter は Transit Engine 操作の interface。
//
// production: OpenBao Transit Engine の Encrypt / Decrypt / RotateKey API を呼ぶ実装。
// dev / CI: InMemoryTransit が AES-256-GCM の round-trip を成立させる。
type TransitAdapter interface {
	// 平文を暗号化する。
	Encrypt(req TransitEncryptRequest) (TransitEncryptResponse, error)
	// 暗号文を復号する。ciphertext から鍵バージョンを抽出し、対応する鍵で復号する。
	Decrypt(req TransitDecryptRequest) (TransitDecryptResponse, error)
	// 鍵をローテーションする。新バージョンが current になり、旧版は保持される（Decrypt 引き続き可）。
	RotateKey(req TransitRotateKeyRequest) (TransitRotateKeyResponse, error)
}

// transitKeyState は 1 鍵の全バージョンを保持する。
type transitKeyState struct {
	// versions[v] = 32 byte AES-256 鍵（v は 1 始まり）。
	versions map[int][]byte
	// current は現在の最新バージョン番号。
	current int
	// 排他制御。
	mu sync.Mutex
}

// InMemoryTransit は AES-256-GCM の in-memory Transit 実装。
//
// 用途:
//   - cmd/secret バイナリの dev / CI モード（OpenBao server 不在時の fallback）
//   - 本パッケージ内の round-trip 試験
//
// 制約:
//   - process 内でのみ永続化（再起動で全鍵消失）
//   - 鍵生成は crypto/rand、鍵長は AES-256 固定
//   - production OpenBao Transit との binary 互換は保証しない（dev/CI 専用）
type InMemoryTransit struct {
	// keys[key_name] = 鍵バージョン状態。
	mu   sync.Mutex
	keys map[string]*transitKeyState
}

// NewInMemoryTransit は空の InMemoryTransit を生成する。
func NewInMemoryTransit() *InMemoryTransit {
	return &InMemoryTransit{
		keys: map[string]*transitKeyState{},
	}
}
