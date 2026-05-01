// 本ファイルは productionTransit の単体テスト。
//
// 目的:
//   - bao.Logical() narrow（transitWriter）に fake を注入し、Encrypt / Decrypt /
//     RotateKey が正しい path / payload で呼ぶことを確認する。
//   - OpenBao 応答の "vault:v<N>:..." ciphertext / base64 plaintext を本 adapter が
//     正しくパースしてレスポンス型に詰めることを確認する。
//   - 通信エラーが透過することを確認する（handler 段で Internal に翻訳される）。

package openbao

import (
	"context"
	"encoding/base64"
	"errors"
	"testing"

	bao "github.com/openbao/openbao/api/v2"
)

// fakeTransitWriter は transitWriter narrow を満たす test 用 fake。
// 受信した path と body を保持し、応答 *bao.Secret を任意に差し替えられる。
type fakeTransitWriter struct {
	// 直前の Write 呼出で受け取った path。
	gotPath string
	// 直前の Write 呼出で受け取った body。
	gotBody map[string]interface{}
	// 応答 *bao.Secret（nil なら "data 無し" を意味し、production 側は err を返す）。
	respSecret *bao.Secret
	// 応答 error。
	respErr error
}

func (f *fakeTransitWriter) WriteWithContext(_ context.Context, path string, data map[string]interface{}) (*bao.Secret, error) {
	f.gotPath = path
	f.gotBody = data
	return f.respSecret, f.respErr
}

func TestProductionTransit_EncryptCallsTransitEncryptPath(t *testing.T) {
	t.Parallel()
	w := &fakeTransitWriter{
		respSecret: &bao.Secret{
			Data: map[string]interface{}{
				// OpenBao Transit が返す典型的な ciphertext 形式。
				"ciphertext":  "vault:v1:abcdef",
				"key_version": float64(1),
			},
		},
	}
	tr := NewProductionTransitFromWriter(w, "transit")
	resp, err := tr.Encrypt(TransitEncryptRequest{
		KeyName:   "tenant-A.payments",
		Plaintext: []byte("hello"),
		AAD:       []byte("aad-bytes"),
	})
	if err != nil {
		t.Fatalf("Encrypt unexpected error: %v", err)
	}
	if string(resp.Ciphertext) != "vault:v1:abcdef" {
		t.Fatalf("ciphertext mismatch: got %q", resp.Ciphertext)
	}
	if resp.KeyVersion != 1 {
		t.Fatalf("KeyVersion mismatch: got %d", resp.KeyVersion)
	}
	if w.gotPath != "transit/encrypt/tenant-A.payments" {
		t.Fatalf("path mismatch: got %q", w.gotPath)
	}
	// plaintext / context は base64 エンコードされていること。
	if got, want := w.gotBody["plaintext"], base64.StdEncoding.EncodeToString([]byte("hello")); got != want {
		t.Fatalf("plaintext base64 mismatch: got %v want %v", got, want)
	}
	if got, want := w.gotBody["context"], base64.StdEncoding.EncodeToString([]byte("aad-bytes")); got != want {
		t.Fatalf("context base64 mismatch: got %v want %v", got, want)
	}
}

func TestProductionTransit_DecryptDecodesBase64Plaintext(t *testing.T) {
	t.Parallel()
	w := &fakeTransitWriter{
		respSecret: &bao.Secret{
			Data: map[string]interface{}{
				// OpenBao は plaintext を base64 で返す。
				"plaintext":   base64.StdEncoding.EncodeToString([]byte("hello")),
				"key_version": "1",
			},
		},
	}
	tr := NewProductionTransitFromWriter(w, "transit")
	resp, err := tr.Decrypt(TransitDecryptRequest{
		KeyName:    "tenant-A.payments",
		Ciphertext: []byte("vault:v1:abcdef"),
		AAD:        []byte("aad"),
	})
	if err != nil {
		t.Fatalf("Decrypt unexpected error: %v", err)
	}
	if string(resp.Plaintext) != "hello" {
		t.Fatalf("plaintext mismatch: got %q", resp.Plaintext)
	}
	if resp.KeyVersion != 1 {
		t.Fatalf("KeyVersion mismatch: got %d", resp.KeyVersion)
	}
	if w.gotPath != "transit/decrypt/tenant-A.payments" {
		t.Fatalf("path mismatch: got %q", w.gotPath)
	}
	if got := w.gotBody["ciphertext"]; got != "vault:v1:abcdef" {
		t.Fatalf("ciphertext payload mismatch: got %v", got)
	}
}

func TestProductionTransit_DecryptRejectsEmptyCiphertext(t *testing.T) {
	t.Parallel()
	tr := NewProductionTransitFromWriter(&fakeTransitWriter{}, "transit")
	_, err := tr.Decrypt(TransitDecryptRequest{
		KeyName:    "tenant-A.payments",
		Ciphertext: nil,
	})
	if !errors.Is(err, ErrTransitCiphertextMalformed) {
		t.Fatalf("expected ErrTransitCiphertextMalformed, got %v", err)
	}
}

func TestProductionTransit_RotateKeyCallsRotatePath(t *testing.T) {
	t.Parallel()
	w := &fakeTransitWriter{
		// rotate API の応答は省略可（OpenBao は data なしの 204 系応答もあり得る）。
		respSecret: &bao.Secret{Data: map[string]interface{}{}},
	}
	tr := NewProductionTransitFromWriter(w, "transit")
	resp, err := tr.RotateKey(TransitRotateKeyRequest{KeyName: "tenant-A.payments"})
	if err != nil {
		t.Fatalf("RotateKey unexpected error: %v", err)
	}
	if w.gotPath != "transit/keys/tenant-A.payments/rotate" {
		t.Fatalf("rotate path mismatch: got %q", w.gotPath)
	}
	if resp.RotatedAtMs == 0 {
		t.Fatalf("RotatedAtMs should be set to wall clock at handler time")
	}
}

func TestProductionTransit_PropagatesWriteError(t *testing.T) {
	t.Parallel()
	wantErr := errors.New("network down")
	w := &fakeTransitWriter{respErr: wantErr}
	tr := NewProductionTransitFromWriter(w, "transit")
	_, err := tr.Encrypt(TransitEncryptRequest{KeyName: "k", Plaintext: []byte("x")})
	if !errors.Is(err, wantErr) {
		t.Fatalf("Encrypt should propagate writer error: got %v", err)
	}
}

func TestProductionTransit_ResolvesEmptyMountToDefault(t *testing.T) {
	t.Parallel()
	w := &fakeTransitWriter{
		respSecret: &bao.Secret{Data: map[string]interface{}{
			"ciphertext":  "vault:v1:abc",
			"key_version": float64(1),
		}},
	}
	// 空 mount は "transit" にフォールバック。
	tr := NewProductionTransitFromWriter(w, "")
	_, err := tr.Encrypt(TransitEncryptRequest{KeyName: "k", Plaintext: []byte("x")})
	if err != nil {
		t.Fatalf("Encrypt unexpected error: %v", err)
	}
	if w.gotPath != "transit/encrypt/k" {
		t.Fatalf("default mount fallback failed: path=%q", w.gotPath)
	}
}

func TestProductionTransit_TrimsSlashesInMount(t *testing.T) {
	t.Parallel()
	w := &fakeTransitWriter{
		respSecret: &bao.Secret{Data: map[string]interface{}{
			"ciphertext":  "vault:v1:abc",
			"key_version": float64(1),
		}},
	}
	tr := NewProductionTransitFromWriter(w, "/secrets/transit/")
	_, err := tr.Encrypt(TransitEncryptRequest{KeyName: "k", Plaintext: []byte("x")})
	if err != nil {
		t.Fatalf("Encrypt unexpected error: %v", err)
	}
	if w.gotPath != "secrets/transit/encrypt/k" {
		t.Fatalf("slash trim failed: path=%q", w.gotPath)
	}
}

func TestProductionTransit_NewProductionTransit_FallsBackToInMemoryWhenClientNil(t *testing.T) {
	t.Parallel()
	tr := NewProductionTransit(nil, "transit")
	// in-memory にフォールバックしているため、Encrypt → Decrypt の round-trip が成立する。
	enc, err := tr.Encrypt(TransitEncryptRequest{KeyName: "k", Plaintext: []byte("hello")})
	if err != nil {
		t.Fatalf("inmemory fallback Encrypt: %v", err)
	}
	dec, err := tr.Decrypt(TransitDecryptRequest{KeyName: "k", Ciphertext: enc.Ciphertext})
	if err != nil {
		t.Fatalf("inmemory fallback Decrypt: %v", err)
	}
	if string(dec.Plaintext) != "hello" {
		t.Fatalf("inmemory round-trip failed: got %q", dec.Plaintext)
	}
}

func TestProductionTransit_RejectsEmptyKeyName(t *testing.T) {
	t.Parallel()
	tr := NewProductionTransitFromWriter(&fakeTransitWriter{}, "transit")
	_, err := tr.Encrypt(TransitEncryptRequest{KeyName: ""})
	if err == nil {
		t.Fatalf("Encrypt with empty KeyName should fail")
	}
	_, err = tr.Decrypt(TransitDecryptRequest{KeyName: "", Ciphertext: []byte("x")})
	if err == nil {
		t.Fatalf("Decrypt with empty KeyName should fail")
	}
	_, err = tr.RotateKey(TransitRotateKeyRequest{KeyName: ""})
	if err == nil {
		t.Fatalf("RotateKey with empty KeyName should fail")
	}
}

func TestExtractKeyVersion_HandlesAllNumericForms(t *testing.T) {
	t.Parallel()
	// SDK は JSON 数値を float64 か json.Number で返す。string も string parse でケアする。
	cases := []struct {
		name string
		v    interface{}
		want int
	}{
		{"float64", float64(7), 7},
		{"int", int(3), 3},
		{"int64", int64(11), 11},
		{"string", "5", 5},
		{"missing", nil, 0},
		{"non-numeric string", "abc", 0},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			data := map[string]interface{}{}
			if c.v != nil {
				data["key_version"] = c.v
			}
			if got := extractKeyVersion(data, "key_version"); got != c.want {
				t.Fatalf("got %d want %d", got, c.want)
			}
		})
	}
}
