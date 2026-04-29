// 本ファイルは tier2 共通 auth middleware の単体テスト。
//
// docs 正典:
//   docs/03_要件定義/00_共通規約.md §「認証認可」
//
// テスト観点:
//   - off mode: token 内容を見ず demo-tenant を context に積む
//   - hmac mode: 有効な HS256 token は通過、無効な署名は 401、tenant_id 欠如は 401
//   - missing/empty Authorization は 401

package auth

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/go-jose/go-jose/v4"
	"github.com/go-jose/go-jose/v4/jwt"
)

// passthroughHandler は middleware のテストで「通過したら何が context に入っているか」を
// 確認するためのシンプルな後段 handler。
func passthroughHandler(w http.ResponseWriter, r *http.Request) {
	subject := SubjectFromContext(r.Context())
	tenant := TenantIDFromContext(r.Context())
	token := TokenFromContext(r.Context())
	w.Header().Set("X-Subject", subject)
	w.Header().Set("X-Tenant", tenant)
	w.Header().Set("X-Token-Len", time.Now().UTC().Format(time.RFC3339)+"-"+lenStr(len(token)))
	w.WriteHeader(http.StatusOK)
}

// lenStr は int を文字列化する小ヘルパ（テスト目視用）。
func lenStr(n int) string {
	s := ""
	if n == 0 {
		return "0"
	}
	for n > 0 {
		s = string(rune('0'+n%10)) + s
		n /= 10
	}
	return s
}

func TestOffMode_AcceptsAnyTokenAndAttachesDemoTenant(t *testing.T) {
	cfg := Config{Mode: AuthModeOff}
	mw := RequiredWithConfig(cfg)
	req := httptest.NewRequest(http.MethodGet, "/x", nil)
	req.Header.Set("Authorization", "Bearer anything")
	rec := httptest.NewRecorder()
	mw(http.HandlerFunc(passthroughHandler)).ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("off mode should accept; got %d", rec.Code)
	}
	if got := rec.Header().Get("X-Tenant"); got != "demo-tenant" {
		t.Fatalf("off mode should set demo-tenant; got %q", got)
	}
	if got := rec.Header().Get("X-Subject"); got != "dev" {
		t.Fatalf("off mode subject should be dev; got %q", got)
	}
}

func TestRejects_MissingAuthorizationHeader(t *testing.T) {
	mw := RequiredWithConfig(Config{Mode: AuthModeOff})
	req := httptest.NewRequest(http.MethodGet, "/x", nil)
	rec := httptest.NewRecorder()
	mw(http.HandlerFunc(passthroughHandler)).ServeHTTP(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("missing auth should be 401; got %d", rec.Code)
	}
}

func TestRejects_NonBearerScheme(t *testing.T) {
	mw := RequiredWithConfig(Config{Mode: AuthModeOff})
	req := httptest.NewRequest(http.MethodGet, "/x", nil)
	req.Header.Set("Authorization", "Basic dXNlcjpwYXNz")
	rec := httptest.NewRecorder()
	mw(http.HandlerFunc(passthroughHandler)).ServeHTTP(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("non-bearer should be 401; got %d", rec.Code)
	}
}

func TestHmacMode_AcceptsValidToken(t *testing.T) {
	secret := []byte("test-secret-32bytes-long-aaaaaaaa")
	signer, err := jose.NewSigner(
		jose.SigningKey{Algorithm: jose.HS256, Key: secret},
		(&jose.SignerOptions{}).WithType("JWT"),
	)
	if err != nil {
		t.Fatalf("signer: %v", err)
	}
	claims := struct {
		TenantID string `json:"tenant_id"`
		jwt.Claims
	}{
		TenantID: "T-PROD",
		Claims: jwt.Claims{
			Subject:  "alice",
			Expiry:   jwt.NewNumericDate(time.Now().Add(60 * time.Second)),
			IssuedAt: jwt.NewNumericDate(time.Now()),
		},
	}
	tok, err := jwt.Signed(signer).Claims(claims).Serialize()
	if err != nil {
		t.Fatalf("sign: %v", err)
	}
	mw := RequiredWithConfig(Config{Mode: AuthModeHMAC, HMACSecret: secret})
	req := httptest.NewRequest(http.MethodGet, "/x", nil).WithContext(context.Background())
	req.Header.Set("Authorization", "Bearer "+tok)
	rec := httptest.NewRecorder()
	mw(http.HandlerFunc(passthroughHandler)).ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("valid hmac should be 200; got %d body=%s", rec.Code, rec.Body.String())
	}
	if rec.Header().Get("X-Tenant") != "T-PROD" {
		t.Fatalf("expected tenant T-PROD; got %q", rec.Header().Get("X-Tenant"))
	}
	if rec.Header().Get("X-Subject") != "alice" {
		t.Fatalf("expected subject alice; got %q", rec.Header().Get("X-Subject"))
	}
}

func TestHmacMode_RejectsInvalidSignature(t *testing.T) {
	correct := []byte("correct-secret-32bytes-long-aaaaa")
	wrong := []byte("wrong-secret-32bytes-long-aaaaaaaa")
	signer, _ := jose.NewSigner(jose.SigningKey{Algorithm: jose.HS256, Key: wrong},
		(&jose.SignerOptions{}).WithType("JWT"))
	claims := struct {
		TenantID string `json:"tenant_id"`
		jwt.Claims
	}{TenantID: "T1", Claims: jwt.Claims{
		Subject: "bob",
		Expiry:  jwt.NewNumericDate(time.Now().Add(60 * time.Second)),
	}}
	tok, _ := jwt.Signed(signer).Claims(claims).Serialize()
	mw := RequiredWithConfig(Config{Mode: AuthModeHMAC, HMACSecret: correct})
	req := httptest.NewRequest(http.MethodGet, "/x", nil)
	req.Header.Set("Authorization", "Bearer "+tok)
	rec := httptest.NewRecorder()
	mw(http.HandlerFunc(passthroughHandler)).ServeHTTP(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("wrong signature should be 401; got %d", rec.Code)
	}
}

func TestHmacMode_RejectsMissingTenantClaim(t *testing.T) {
	secret := []byte("test-secret-32bytes-long-aaaaaaaa")
	signer, _ := jose.NewSigner(jose.SigningKey{Algorithm: jose.HS256, Key: secret},
		(&jose.SignerOptions{}).WithType("JWT"))
	// tenant_id 欠落の token を作る。
	claims := struct {
		jwt.Claims
	}{Claims: jwt.Claims{
		Subject: "carol",
		Expiry:  jwt.NewNumericDate(time.Now().Add(60 * time.Second)),
	}}
	tok, _ := jwt.Signed(signer).Claims(claims).Serialize()
	mw := RequiredWithConfig(Config{Mode: AuthModeHMAC, HMACSecret: secret})
	req := httptest.NewRequest(http.MethodGet, "/x", nil)
	req.Header.Set("Authorization", "Bearer "+tok)
	rec := httptest.NewRecorder()
	mw(http.HandlerFunc(passthroughHandler)).ServeHTTP(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("missing tenant_id should be 401; got %d", rec.Code)
	}
}
