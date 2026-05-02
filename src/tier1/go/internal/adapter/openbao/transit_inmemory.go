// 本ファイルは InMemoryTransit の操作実装（Encrypt / Decrypt / RotateKey）。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/04_Secrets_API.md（FR-T1-SECRETS-003）
//
// 役割:
//   AES-256-GCM の純粋暗号化ループを transit.go の TransitAdapter interface 越しに公開する。
//   key version は 1 始まり、ciphertext 先頭 4 byte に BE で埋め込む。
//
// セキュリティ注意:
//   - 鍵は crypto/rand から生成（OS RNG 経由）。AES-256 で 32 byte。
//   - GCM nonce は呼び出しごとに crypto/rand で 12 byte 生成（再利用すると破滅的なため）。
//   - AAD は呼出側責任（tenant_id / RPC 名等を JSON encoded で詰めることを推奨）。

package openbao

import (
	// AES ブロック暗号。
	"crypto/aes"
	// GCM AEAD wrapper。
	"crypto/cipher"
	// 鍵 / nonce 生成用 OS RNG。
	"crypto/rand"
	// バージョン番号 BE エンコード。
	"encoding/binary"
	// エラー定義。
	"errors"
	// "io" の Copy 経由で OS RNG から読む。
	"io"
	// ローテーション時刻記録。
	"time"
)

// ErrTransitKeyNotFound は鍵が未生成（Encrypt 前 / 不正 key_name）であることを示す。
// Decrypt 時に該当 key_name の状態が無い、または ciphertext 中の version が不正の場合に返す。
var ErrTransitKeyNotFound = errors.New("tier1/secrets/transit: key not found")

// ErrTransitCiphertextMalformed は ciphertext が短すぎる / 構造不正であることを示す。
var ErrTransitCiphertextMalformed = errors.New("tier1/secrets/transit: ciphertext malformed")

// AES-256 鍵長（byte）。
const transitKeyBytes = 32

// AES-GCM 推奨 nonce 長（byte）。
const transitNonceBytes = 12

// ciphertext 先頭の version field 長（4 byte BE）。
const transitVersionBytes = 4

// transitMinCiphertext は 暗号文として有効な最小サイズ（version + nonce + GCM tag）。
const transitMinCiphertext = transitVersionBytes + transitNonceBytes + 16 // GCM tag = 16 byte

// generateAESKey は OS RNG から 32 byte の AES-256 鍵を生成する。
func generateAESKey() ([]byte, error) {
	// 32 byte 配列を確保する。
	key := make([]byte, transitKeyBytes)
	// crypto/rand.Reader から full read（io.ReadFull で部分読みを排除）。
	if _, err := io.ReadFull(rand.Reader, key); err != nil {
		// 失敗は OS RNG 不足等の致命的エラー。
		return nil, err
	}
	return key, nil
}

// generateNonce は GCM 用の 12 byte ランダム nonce を生成する。
func generateNonce() ([]byte, error) {
	nonce := make([]byte, transitNonceBytes)
	if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
		return nil, err
	}
	return nonce, nil
}

// ensureKeyState は key_name の state を返す。未存在なら新規鍵を 1 件生成する。
// Encrypt 経路で鍵未存在時に lazy 生成する慣用（OpenBao Transit と同等の挙動）。
func (t *InMemoryTransit) ensureKeyState(name string) (*transitKeyState, error) {
	// 全 keys map の保護。
	t.mu.Lock()
	defer t.mu.Unlock()
	// 既存 state を返す。
	if s, ok := t.keys[name]; ok {
		return s, nil
	}
	// 新規生成: version 1 で初期化する。
	k, err := generateAESKey()
	if err != nil {
		return nil, err
	}
	state := &transitKeyState{
		versions: map[int][]byte{1: k},
		current:  1,
	}
	t.keys[name] = state
	return state, nil
}

// Encrypt は AES-256-GCM で平文を暗号化する。
// 出力 ciphertext は [4 byte BE version][12 byte nonce][AES-GCM(ct+tag)] の連結。
func (t *InMemoryTransit) Encrypt(req TransitEncryptRequest) (TransitEncryptResponse, error) {
	// 鍵未存在なら lazy 生成する（OpenBao Transit と同等）。
	state, err := t.ensureKeyState(req.KeyName)
	if err != nil {
		return TransitEncryptResponse{}, err
	}
	// state 内の current バージョン鍵を読む（短時間 lock）。
	state.mu.Lock()
	version := state.current
	key := state.versions[version]
	state.mu.Unlock()
	// AES ブロック暗号を構築する。
	block, err := aes.NewCipher(key)
	if err != nil {
		return TransitEncryptResponse{}, err
	}
	// GCM AEAD wrapper を作る（NonceSize=12, TagSize=16）。
	aead, err := cipher.NewGCM(block)
	if err != nil {
		return TransitEncryptResponse{}, err
	}
	// nonce 生成（crypto/rand）。
	nonce, err := generateNonce()
	if err != nil {
		return TransitEncryptResponse{}, err
	}
	// AES-GCM 暗号化（tag が末尾に付与される）。
	sealed := aead.Seal(nil, nonce, req.Plaintext, req.AAD)
	// 出力は [version: 4 BE][nonce: 12][sealed]。
	out := make([]byte, transitVersionBytes+transitNonceBytes+len(sealed))
	binary.BigEndian.PutUint32(out[:transitVersionBytes], uint32(version))
	copy(out[transitVersionBytes:transitVersionBytes+transitNonceBytes], nonce)
	copy(out[transitVersionBytes+transitNonceBytes:], sealed)
	return TransitEncryptResponse{
		Ciphertext: out,
		KeyVersion: version,
	}, nil
}

// Decrypt は ciphertext を復号する。version は ciphertext 先頭から取り出し、
// 該当 version の鍵で復号する（旧版鍵で暗号化された ciphertext も復号できる）。
func (t *InMemoryTransit) Decrypt(req TransitDecryptRequest) (TransitDecryptResponse, error) {
	// ciphertext の最低長確認。
	if len(req.Ciphertext) < transitMinCiphertext {
		return TransitDecryptResponse{}, ErrTransitCiphertextMalformed
	}
	// version をデコードする。
	version := int(binary.BigEndian.Uint32(req.Ciphertext[:transitVersionBytes]))
	// nonce を取り出す。
	nonce := req.Ciphertext[transitVersionBytes : transitVersionBytes+transitNonceBytes]
	// 残り（sealed: ciphertext + tag）。
	sealed := req.Ciphertext[transitVersionBytes+transitNonceBytes:]
	// 該当 key state を取得する（未存在は鍵不在エラー）。
	t.mu.Lock()
	state, ok := t.keys[req.KeyName]
	t.mu.Unlock()
	if !ok {
		return TransitDecryptResponse{}, ErrTransitKeyNotFound
	}
	// state から指定 version の鍵を取り出す。
	state.mu.Lock()
	key, ok := state.versions[version]
	state.mu.Unlock()
	if !ok {
		// version は ciphertext 由来だが、対応する鍵が削除済の場合（将来の retention）。
		return TransitDecryptResponse{}, ErrTransitKeyNotFound
	}
	// AES-GCM で復号する。
	block, err := aes.NewCipher(key)
	if err != nil {
		return TransitDecryptResponse{}, err
	}
	aead, err := cipher.NewGCM(block)
	if err != nil {
		return TransitDecryptResponse{}, err
	}
	plain, err := aead.Open(nil, nonce, sealed, req.AAD)
	if err != nil {
		// GCM tag mismatch は AAD 違い / ciphertext 改竄を意味する。
		return TransitDecryptResponse{}, err
	}
	return TransitDecryptResponse{
		Plaintext:  plain,
		KeyVersion: version,
	}, nil
}

// RotateKey は鍵を新バージョンに上げる。旧バージョンは保持され、引き続き Decrypt 可能。
func (t *InMemoryTransit) RotateKey(req TransitRotateKeyRequest) (TransitRotateKeyResponse, error) {
	// 鍵未存在なら lazy 生成する。
	state, err := t.ensureKeyState(req.KeyName)
	if err != nil {
		return TransitRotateKeyResponse{}, err
	}
	// 新版鍵を生成する。
	newKey, err := generateAESKey()
	if err != nil {
		return TransitRotateKeyResponse{}, err
	}
	// state を更新する。
	state.mu.Lock()
	previous := state.current
	state.current = previous + 1
	state.versions[state.current] = newKey
	current := state.current
	state.mu.Unlock()
	return TransitRotateKeyResponse{
		NewVersion:      current,
		PreviousVersion: previous,
		RotatedAtMs:     time.Now().UnixMilli(),
	}, nil
}
