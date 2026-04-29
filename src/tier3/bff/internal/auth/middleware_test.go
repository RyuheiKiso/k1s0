// auth middleware の単体テスト。
//
// docs 正典:
//   docs/03_要件定義/30_非機能要件/E_セキュリティ.md NFR-E-AC-001 / 003

package auth

import (
	"context"
	"crypto/rand"
	"crypto/rsa"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/go-jose/go-jose/v4"
	"github.com/go-jose/go-jose/v4/jwt"
)

// helper: HS256 で signed JWT を作る。
func mintHS256(t *testing.T, secret []byte, claims AuthClaims) string {
	t.Helper()
	signer, err := jose.NewSigner(jose.SigningKey{Algorithm: jose.HS256, Key: secret}, (&jose.SignerOptions{}).WithType("JWT"))
	if err != nil {
		t.Fatalf("signer: %v", err)
	}
	tok, err := jwt.Signed(signer).Claims(claims).Serialize()
	if err != nil {
		t.Fatalf("sign: %v", err)
	}
	return tok
}

// recordedClaims は terminal handler が context から取り出すクレーム集合。
type recordedClaims struct {
	subject  string
	tenantID string
	roles    []string
	token    string
}

func recordingHandler(captured *recordedClaims) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		captured.subject = SubjectFromContext(r.Context())
		captured.tenantID = TenantIDFromContext(r.Context())
		captured.roles = RolesFromContext(r.Context())
		captured.token = TokenFromContext(r.Context())
		w.WriteHeader(http.StatusOK)
	})
}

// 既存テストのリグレッション保護: トークンなしは 401。
func TestRequired_NoToken_Returns401(t *testing.T) {
	called := false
	next := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { called = true })
	mw := Required("user")(next)
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/foo", nil)
	mw.ServeHTTP(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Errorf("expected 401, got %d", rec.Code)
	}
	if called {
		t.Error("next handler should not be called")
	}
}

// 既存テストのリグレッション保護: admin-token は admin role を取れる（off mode）。
// 既存実装の "admin-token" 後方互換は維持しているため、env 未設定（off mode 既定）で通る。
func TestRequired_AdminToken_Allowed(t *testing.T) {
	called := false
	next := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		called = true
		if SubjectFromContext(r.Context()) != "admin-user" {
			t.Errorf("subject = %q", SubjectFromContext(r.Context()))
		}
	})
	mw := Required("admin")(next)
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/foo", nil)
	req.Header.Set("Authorization", "Bearer admin-token")
	mw.ServeHTTP(rec, req)
	if !called {
		t.Error("next handler should be called")
	}
	if rec.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", rec.Code)
	}
}

// 既存テストのリグレッション保護: user role only の token は admin endpoint で 403。
func TestRequired_UserToken_AdminEndpoint_Returns403(t *testing.T) {
	next := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})
	mw := Required("admin")(next)
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/foo", nil)
	req.Header.Set("Authorization", "Bearer some-user-token-value")
	mw.ServeHTTP(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Errorf("expected 403, got %d", rec.Code)
	}
}

// hmac mode: 不正トークンは 401。
func TestRequired_HMAC_InvalidToken_Unauthorized(t *testing.T) {
	mw := requiredWithConfig(Config{Mode: AuthModeHMAC, HMACSecret: []byte("test-hmac-secret-32-bytes--------")}, "")
	w := httptest.NewRecorder()
	r := httptest.NewRequest("GET", "/x", nil)
	r.Header.Set("Authorization", "Bearer not-a-valid-jwt")
	mw(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { t.Fatal("should not call next") })).ServeHTTP(w, r)
	if w.Code != http.StatusUnauthorized {
		t.Fatalf("code = %d", w.Code)
	}
}

// hmac mode: 期限切れ JWT は 401。
func TestRequired_HMAC_Expired_Unauthorized(t *testing.T) {
	secret := []byte("test-hmac-secret-32-bytes--------")
	tok := mintHS256(t, secret, AuthClaims{
		TenantID: "T",
		Claims: jwt.Claims{
			Subject:  "u",
			IssuedAt: jwt.NewNumericDate(time.Now().Add(-2 * time.Hour)),
			Expiry:   jwt.NewNumericDate(time.Now().Add(-1 * time.Hour)),
		},
	})
	mw := requiredWithConfig(Config{Mode: AuthModeHMAC, HMACSecret: secret}, "")
	w := httptest.NewRecorder()
	r := httptest.NewRequest("GET", "/x", nil)
	r.Header.Set("Authorization", "Bearer "+tok)
	mw(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { t.Fatal("should not call next") })).ServeHTTP(w, r)
	if w.Code != http.StatusUnauthorized {
		t.Fatalf("code = %d", w.Code)
	}
}

// hmac mode: tenant_id クレーム不在は 401。
func TestRequired_HMAC_MissingTenantClaim_Unauthorized(t *testing.T) {
	secret := []byte("test-hmac-secret-32-bytes--------")
	tok := mintHS256(t, secret, AuthClaims{
		Claims: jwt.Claims{
			Subject:  "u",
			IssuedAt: jwt.NewNumericDate(time.Now()),
			Expiry:   jwt.NewNumericDate(time.Now().Add(5 * time.Minute)),
		},
	})
	mw := requiredWithConfig(Config{Mode: AuthModeHMAC, HMACSecret: secret}, "")
	w := httptest.NewRecorder()
	r := httptest.NewRequest("GET", "/x", nil)
	r.Header.Set("Authorization", "Bearer "+tok)
	mw(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { t.Fatal("should not call next") })).ServeHTTP(w, r)
	if w.Code != http.StatusUnauthorized {
		t.Fatalf("code = %d", w.Code)
	}
}

// hmac mode 正常系: claims が context に展開される。
func TestRequired_HMAC_Valid_AttachClaims(t *testing.T) {
	secret := []byte("test-hmac-secret-32-bytes--------")
	tok := mintHS256(t, secret, AuthClaims{
		TenantID: "tenant-A",
		RealmAccess: &struct {
			Roles []string `json:"roles"`
		}{Roles: []string{"admin", "user"}},
		Claims: jwt.Claims{
			Subject:  "alice",
			IssuedAt: jwt.NewNumericDate(time.Now()),
			Expiry:   jwt.NewNumericDate(time.Now().Add(5 * time.Minute)),
		},
	})
	mw := requiredWithConfig(Config{Mode: AuthModeHMAC, HMACSecret: secret}, "admin")
	captured := &recordedClaims{}
	w := httptest.NewRecorder()
	r := httptest.NewRequest("GET", "/x", nil)
	r.Header.Set("Authorization", "Bearer "+tok)
	mw(recordingHandler(captured)).ServeHTTP(w, r)
	if w.Code != http.StatusOK {
		t.Fatalf("code = %d body=%s", w.Code, w.Body.String())
	}
	if captured.subject != "alice" {
		t.Errorf("subject = %q", captured.subject)
	}
	if captured.tenantID != "tenant-A" {
		t.Errorf("tenant_id = %q", captured.tenantID)
	}
	if len(captured.roles) != 2 {
		t.Errorf("roles = %v", captured.roles)
	}
	if captured.token != tok {
		t.Errorf("token not propagated to context")
	}
}

// jwks mode end-to-end: httptest server で JWKS を提供し、RSA 鍵で signed した JWT を検証する。
func TestRequired_JWKS_Valid(t *testing.T) {
	priv, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		t.Fatalf("rsa keygen: %v", err)
	}
	jwks := jose.JSONWebKeySet{Keys: []jose.JSONWebKey{
		{Key: &priv.PublicKey, KeyID: "k1", Algorithm: string(jose.RS256), Use: "sig"},
	}}
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(jwks)
	}))
	defer srv.Close()

	signer, err := jose.NewSigner(
		jose.SigningKey{Algorithm: jose.RS256, Key: priv},
		(&jose.SignerOptions{}).WithType("JWT").WithHeader("kid", "k1"),
	)
	if err != nil {
		t.Fatalf("signer: %v", err)
	}
	tok, err := jwt.Signed(signer).Claims(AuthClaims{
		TenantID: "tenant-prod",
		RealmAccess: &struct {
			Roles []string `json:"roles"`
		}{Roles: []string{"user"}},
		Claims: jwt.Claims{
			Subject:  "bob",
			IssuedAt: jwt.NewNumericDate(time.Now()),
			Expiry:   jwt.NewNumericDate(time.Now().Add(5 * time.Minute)),
		},
	}).Serialize()
	if err != nil {
		t.Fatalf("sign: %v", err)
	}

	cfg := Config{
		Mode:       AuthModeJWKS,
		JWKSURL:    srv.URL,
		HTTPClient: srv.Client(),
	}
	mw := requiredWithConfig(cfg, "")
	captured := &recordedClaims{}
	w := httptest.NewRecorder()
	r := httptest.NewRequest("GET", "/x", nil)
	r.Header.Set("Authorization", "Bearer "+tok)
	mw(recordingHandler(captured)).ServeHTTP(w, r)
	if w.Code != http.StatusOK {
		t.Fatalf("code = %d body=%s", w.Code, w.Body.String())
	}
	if captured.subject != "bob" || captured.tenantID != "tenant-prod" {
		t.Errorf("claims not extracted: %+v", captured)
	}
}

// off mode: 任意 token は demo-tenant に集約される（本番投入時の越境を物理的に防ぐ）。
func TestRequired_OffMode_TenantIDClampedToDemo(t *testing.T) {
	mw := requiredWithConfig(Config{Mode: AuthModeOff}, "")
	captured := &recordedClaims{}
	w := httptest.NewRecorder()
	r := httptest.NewRequest("GET", "/x", nil)
	r.Header.Set("Authorization", "Bearer arbitrary-token-12345")
	mw(recordingHandler(captured)).ServeHTTP(w, r)
	if w.Code != http.StatusOK {
		t.Fatalf("code = %d", w.Code)
	}
	if captured.tenantID != "demo-tenant" {
		t.Errorf("tenant_id = %q want demo-tenant (off mode must clamp)", captured.tenantID)
	}
}

// バックドア リグレッション保護: hmac mode で偽装試行は 401 で弾かれる。
// 旧実装は token の最初 8 文字を subject に使う偽装を許容していた。
func TestRequired_HMAC_RegressionAgainstSubjectSpoofing(t *testing.T) {
	mw := requiredWithConfig(Config{Mode: AuthModeHMAC, HMACSecret: []byte("test-hmac-secret-32-bytes--------")}, "")
	w := httptest.NewRecorder()
	r := httptest.NewRequest("GET", "/x", nil)
	r.Header.Set("Authorization", "Bearer attacker12345")
	mw(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("subject spoofing must be blocked in hmac mode")
	})).ServeHTTP(w, r)
	if w.Code != http.StatusUnauthorized {
		t.Fatalf("code = %d (spoofing must yield 401 in hmac mode)", w.Code)
	}
	if !strings.Contains(w.Body.String(), "E-T3-BFF-AUTH-001") {
		t.Errorf("error response missing standard code: %s", w.Body.String())
	}
}

// context helpers の境界条件。
func TestContextHelpers_NoValue_ReturnsEmpty(t *testing.T) {
	ctx := context.Background()
	if SubjectFromContext(ctx) != "" {
		t.Errorf("subject should be empty")
	}
	if TenantIDFromContext(ctx) != "" {
		t.Errorf("tenant_id should be empty")
	}
	if TokenFromContext(ctx) != "" {
		t.Errorf("token should be empty")
	}
	if RolesFromContext(ctx) != nil {
		t.Errorf("roles should be nil")
	}
}
