// 本ファイルは InMemoryTransit の単体テスト。
//
// 目的:
//   FR-T1-SECRETS-003 受け入れ基準の動作確認:
//   - AES-256-GCM round-trip
//   - 鍵バージョン管理（RotateKey 後も旧版で暗号化された ciphertext を Decrypt 可能）
//   - 復号時に ciphertext 中の version を自動解決
//   - AAD 不一致は復号失敗（GCM tag mismatch）
//   - tenant prefix で同 key_label が他テナントと衝突しない（呼出側責任）。

package openbao

import (
	"bytes"
	"errors"
	"testing"
)

// Encrypt → Decrypt の round-trip を検証する。
func TestInMemoryTransit_RoundTrip(t *testing.T) {
	tr := NewInMemoryTransit()
	plaintext := []byte("これは秘密のメッセージ")
	enc, err := tr.Encrypt(TransitEncryptRequest{
		KeyName:   "tenantA.payment-key",
		Plaintext: plaintext,
		AAD:       []byte("Secrets.Encrypt"),
	})
	if err != nil {
		t.Fatalf("encrypt: %v", err)
	}
	if enc.KeyVersion != 1 {
		t.Errorf("first encrypt should be version 1, got %d", enc.KeyVersion)
	}
	if len(enc.Ciphertext) < transitMinCiphertext {
		t.Errorf("ciphertext too short: %d", len(enc.Ciphertext))
	}
	dec, err := tr.Decrypt(TransitDecryptRequest{
		KeyName:    "tenantA.payment-key",
		Ciphertext: enc.Ciphertext,
		AAD:        []byte("Secrets.Encrypt"),
	})
	if err != nil {
		t.Fatalf("decrypt: %v", err)
	}
	if !bytes.Equal(dec.Plaintext, plaintext) {
		t.Errorf("plaintext mismatch: got %q, want %q", dec.Plaintext, plaintext)
	}
	if dec.KeyVersion != 1 {
		t.Errorf("decrypt version should be 1, got %d", dec.KeyVersion)
	}
}

// 同一 plaintext での Encrypt 結果は nonce 由来で毎回異なる（決定論的でない）ことを確認。
func TestInMemoryTransit_NondeterministicCiphertext(t *testing.T) {
	tr := NewInMemoryTransit()
	plaintext := []byte("payload")
	a, _ := tr.Encrypt(TransitEncryptRequest{KeyName: "T.K", Plaintext: plaintext})
	b, _ := tr.Encrypt(TransitEncryptRequest{KeyName: "T.K", Plaintext: plaintext})
	if bytes.Equal(a.Ciphertext, b.Ciphertext) {
		t.Errorf("two encryptions produced the same ciphertext (nonce reuse?)")
	}
}

// AAD 不一致は GCM tag mismatch で復号失敗。
func TestInMemoryTransit_AADMismatchFails(t *testing.T) {
	tr := NewInMemoryTransit()
	enc, _ := tr.Encrypt(TransitEncryptRequest{
		KeyName:   "T.K",
		Plaintext: []byte("p"),
		AAD:       []byte("AAD-A"),
	})
	_, err := tr.Decrypt(TransitDecryptRequest{
		KeyName:    "T.K",
		Ciphertext: enc.Ciphertext,
		AAD:        []byte("AAD-B"),
	})
	if err == nil {
		t.Errorf("decrypt with wrong AAD should fail")
	}
}

// RotateKey 後、旧版 ciphertext は引き続き Decrypt 可能（FR-T1-SECRETS-003 受け入れ基準）。
func TestInMemoryTransit_OldVersionDecryptableAfterRotation(t *testing.T) {
	tr := NewInMemoryTransit()
	enc1, _ := tr.Encrypt(TransitEncryptRequest{KeyName: "T.K", Plaintext: []byte("v1-plain")})
	if enc1.KeyVersion != 1 {
		t.Fatalf("expected v1, got %d", enc1.KeyVersion)
	}
	rot, err := tr.RotateKey(TransitRotateKeyRequest{KeyName: "T.K"})
	if err != nil {
		t.Fatalf("rotate: %v", err)
	}
	if rot.NewVersion != 2 || rot.PreviousVersion != 1 {
		t.Errorf("rotate version mismatch: new=%d prev=%d", rot.NewVersion, rot.PreviousVersion)
	}
	if rot.RotatedAtMs <= 0 {
		t.Errorf("rotated_at_ms should be positive")
	}
	// 旧版 ciphertext を新 current 鍵で復号する経路（version 自動解決）。
	dec1, err := tr.Decrypt(TransitDecryptRequest{
		KeyName:    "T.K",
		Ciphertext: enc1.Ciphertext,
	})
	if err != nil {
		t.Fatalf("decrypt v1 after rotate: %v", err)
	}
	if !bytes.Equal(dec1.Plaintext, []byte("v1-plain")) {
		t.Errorf("plaintext mismatch after rotate")
	}
	if dec1.KeyVersion != 1 {
		t.Errorf("decrypted v1 should report version 1, got %d", dec1.KeyVersion)
	}
	// 新 current 鍵で encrypt → version 2 が乗る。
	enc2, _ := tr.Encrypt(TransitEncryptRequest{KeyName: "T.K", Plaintext: []byte("v2-plain")})
	if enc2.KeyVersion != 2 {
		t.Errorf("encrypt after rotate should use v2, got %d", enc2.KeyVersion)
	}
	dec2, err := tr.Decrypt(TransitDecryptRequest{KeyName: "T.K", Ciphertext: enc2.Ciphertext})
	if err != nil {
		t.Fatalf("decrypt v2: %v", err)
	}
	if dec2.KeyVersion != 2 {
		t.Errorf("decrypt should report v2")
	}
}

// 鍵未存在で Decrypt（鍵が一度も Encrypt 経由で生成されず、ciphertext は別 source 由来）。
func TestInMemoryTransit_DecryptUnknownKey(t *testing.T) {
	tr := NewInMemoryTransit()
	// 形式上有効なダミー ciphertext（version=1, 12 byte nonce, 16 byte tag）。
	dummy := make([]byte, transitMinCiphertext)
	dummy[3] = 1 // version=1 BE
	_, err := tr.Decrypt(TransitDecryptRequest{
		KeyName:    "no-such-key",
		Ciphertext: dummy,
	})
	if !errors.Is(err, ErrTransitKeyNotFound) {
		t.Errorf("expected ErrTransitKeyNotFound, got %v", err)
	}
}

// ciphertext が短すぎる場合は ErrTransitCiphertextMalformed。
func TestInMemoryTransit_DecryptMalformed(t *testing.T) {
	tr := NewInMemoryTransit()
	_, err := tr.Decrypt(TransitDecryptRequest{
		KeyName:    "any",
		Ciphertext: []byte{0, 0, 0, 1, 0, 0, 0, 0}, // 8 byte で全然足りない
	})
	if !errors.Is(err, ErrTransitCiphertextMalformed) {
		t.Errorf("expected ErrTransitCiphertextMalformed, got %v", err)
	}
}

// テナント分離: 同 key_label でも tenant prefix で別鍵空間になる（呼出側 prefix 責務）。
func TestInMemoryTransit_TenantIsolation(t *testing.T) {
	tr := NewInMemoryTransit()
	// テナント A の鍵で encrypt。
	encA, _ := tr.Encrypt(TransitEncryptRequest{KeyName: "tenantA.shared-label", Plaintext: []byte("A-secret")})
	// テナント B の鍵（自動 lazy 生成）で同 ciphertext を decrypt しようとする → 別鍵なので失敗。
	_, err := tr.Decrypt(TransitDecryptRequest{
		KeyName:    "tenantB.shared-label",
		Ciphertext: encA.Ciphertext,
	})
	if err == nil {
		t.Errorf("decrypt with different tenant key must fail")
	}
}

// RotateKey 単独呼出（Encrypt 前）でも version 1 → 2 へ進める。
func TestInMemoryTransit_RotateKeyBeforeEncrypt(t *testing.T) {
	tr := NewInMemoryTransit()
	rot, err := tr.RotateKey(TransitRotateKeyRequest{KeyName: "fresh"})
	if err != nil {
		t.Fatalf("rotate: %v", err)
	}
	if rot.NewVersion != 2 {
		t.Errorf("first rotate (after lazy gen) should produce v2, got %d", rot.NewVersion)
	}
}
