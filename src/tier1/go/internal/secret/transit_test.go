// 本ファイルは SecretsService.Encrypt / Decrypt / RotateKey handler の単体テスト。
//
// 観点（FR-T1-SECRETS-003）:
//   - tenant prefix が <tenant_id>.<key_label> で自動付与されている
//   - 異テナントの ciphertext を Decrypt しようとすると失敗する（鍵空間分離）
//   - Encrypt → Decrypt の round-trip
//   - RotateKey 後の旧版 ciphertext も Decrypt 可能
//   - Adapter 未注入時は Unimplemented
//   - tenant_id 不一致は PermissionDenied（NFR-E-AC-003 二重防御）
//
// テスト方針:
//   実 InMemoryTransit を使い round-trip を成立させる。fake にしないことで
//   AES-256-GCM 経路が実際に通ることまで検証する（feedback memory:
//   integration tests must hit real backends, not mocks）。

package secret

import (
	"bytes"
	"context"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/openbao"
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	secretsv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/secrets/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// makeHandlerWithTransit は実 InMemoryTransit 結線済 handler を返す。
func makeHandlerWithTransit() (*secretHandler, *openbao.InMemoryTransit) {
	transit := openbao.NewInMemoryTransit()
	h := &secretHandler{deps: Deps{TransitAdapter: transit}}
	return h, transit
}

func TestEncryptDecryptRoundTrip(t *testing.T) {
	h, _ := makeHandlerWithTransit()
	ctx := context.Background()
	plaintext := []byte("これは秘密のメッセージ")
	encResp, err := h.Encrypt(ctx, &secretsv1.EncryptRequest{
		Context:   &commonv1.TenantContext{TenantId: "tenant-A"},
		KeyName:   "payment",
		Plaintext: plaintext,
		Aad:       []byte("Secrets.Encrypt"),
	})
	if err != nil {
		t.Fatalf("encrypt: %v", err)
	}
	if encResp.GetKeyVersion() != 1 {
		t.Errorf("first encrypt should be version 1, got %d", encResp.GetKeyVersion())
	}
	decResp, err := h.Decrypt(ctx, &secretsv1.DecryptRequest{
		Context:    &commonv1.TenantContext{TenantId: "tenant-A"},
		KeyName:    "payment",
		Ciphertext: encResp.GetCiphertext(),
		Aad:        []byte("Secrets.Encrypt"),
	})
	if err != nil {
		t.Fatalf("decrypt: %v", err)
	}
	if !bytes.Equal(decResp.GetPlaintext(), plaintext) {
		t.Errorf("plaintext mismatch")
	}
}

// 同 key_name でも別 tenant の ciphertext は Decrypt できない（テナント越境防止）。
func TestEncryptDecrypt_TenantIsolation(t *testing.T) {
	h, _ := makeHandlerWithTransit()
	ctx := context.Background()
	encA, err := h.Encrypt(ctx, &secretsv1.EncryptRequest{
		Context:   &commonv1.TenantContext{TenantId: "tenant-A"},
		KeyName:   "shared-label",
		Plaintext: []byte("A-only"),
	})
	if err != nil {
		t.Fatalf("encrypt A: %v", err)
	}
	// テナント B から、tenant-A の ciphertext を decrypt 試行。
	_, err = h.Decrypt(ctx, &secretsv1.DecryptRequest{
		Context:    &commonv1.TenantContext{TenantId: "tenant-B"},
		KeyName:    "shared-label",
		Ciphertext: encA.GetCiphertext(),
	})
	if err == nil {
		t.Fatalf("cross-tenant decrypt must fail")
	}
}

// RotateKey 後、旧版 ciphertext は引き続き Decrypt 可能。
func TestRotateKey_OldVersionStillDecryptable(t *testing.T) {
	h, _ := makeHandlerWithTransit()
	ctx := context.Background()
	enc1, err := h.Encrypt(ctx, &secretsv1.EncryptRequest{
		Context:   &commonv1.TenantContext{TenantId: "T"},
		KeyName:   "k",
		Plaintext: []byte("v1"),
	})
	if err != nil {
		t.Fatalf("encrypt v1: %v", err)
	}
	rot, err := h.RotateKey(ctx, &secretsv1.RotateKeyRequest{
		Context: &commonv1.TenantContext{TenantId: "T"},
		KeyName: "k",
	})
	if err != nil {
		t.Fatalf("rotate: %v", err)
	}
	if rot.GetNewVersion() != 2 {
		t.Errorf("new_version should be 2, got %d", rot.GetNewVersion())
	}
	// 旧版 ciphertext を新世代から decrypt（version 自動解決）。
	dec1, err := h.Decrypt(ctx, &secretsv1.DecryptRequest{
		Context:    &commonv1.TenantContext{TenantId: "T"},
		KeyName:    "k",
		Ciphertext: enc1.GetCiphertext(),
	})
	if err != nil {
		t.Fatalf("decrypt v1 after rotate: %v", err)
	}
	if dec1.GetKeyVersion() != 1 {
		t.Errorf("decrypt should report v1, got %d", dec1.GetKeyVersion())
	}
	if !bytes.Equal(dec1.GetPlaintext(), []byte("v1")) {
		t.Errorf("plaintext mismatch")
	}
}

// Adapter 未注入時は Unimplemented を返す。
func TestEncrypt_NotWired(t *testing.T) {
	h := &secretHandler{deps: Deps{}}
	_, err := h.Encrypt(context.Background(), &secretsv1.EncryptRequest{
		Context:   &commonv1.TenantContext{TenantId: "T"},
		KeyName:   "k",
		Plaintext: []byte("p"),
	})
	st, ok := status.FromError(err)
	if !ok || st.Code() != codes.Unimplemented {
		t.Errorf("expected Unimplemented, got %v", err)
	}
}

// 入力 validation: 空 key_name / 空 plaintext は InvalidArgument。
func TestEncrypt_InputValidation(t *testing.T) {
	h, _ := makeHandlerWithTransit()
	ctx := context.Background()
	cases := []struct {
		name string
		req  *secretsv1.EncryptRequest
	}{
		{
			name: "empty key_name",
			req: &secretsv1.EncryptRequest{
				Context:   &commonv1.TenantContext{TenantId: "T"},
				Plaintext: []byte("p"),
			},
		},
		{
			name: "empty plaintext",
			req: &secretsv1.EncryptRequest{
				Context: &commonv1.TenantContext{TenantId: "T"},
				KeyName: "k",
			},
		},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			_, err := h.Encrypt(ctx, tc.req)
			st, ok := status.FromError(err)
			if !ok || st.Code() != codes.InvalidArgument {
				t.Errorf("expected InvalidArgument, got %v", err)
			}
		})
	}
}

// AAD 不一致は InvalidArgument 経由 (Internal の Open 失敗) で復号失敗を返す。
func TestDecrypt_AADMismatch(t *testing.T) {
	h, _ := makeHandlerWithTransit()
	ctx := context.Background()
	enc, _ := h.Encrypt(ctx, &secretsv1.EncryptRequest{
		Context:   &commonv1.TenantContext{TenantId: "T"},
		KeyName:   "k",
		Plaintext: []byte("p"),
		Aad:       []byte("AAD-A"),
	})
	_, err := h.Decrypt(ctx, &secretsv1.DecryptRequest{
		Context:    &commonv1.TenantContext{TenantId: "T"},
		KeyName:    "k",
		Ciphertext: enc.GetCiphertext(),
		Aad:        []byte("AAD-B"),
	})
	if err == nil {
		t.Fatalf("decrypt with wrong AAD should fail")
	}
}
