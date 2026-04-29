// 本ファイルは AuthInterceptor の単体テスト。
//
// 検証観点:
//   - mode=off は完全 pass-through
//   - mode=hmac で JWT 不在は Unauthenticated
//   - mode=hmac で JWT 検証成功時は AuthInfo が context に attach される
//   - mode=hmac で JWT 検証失敗（署名不正 / 期限切れ / tenant_id 不在）は Unauthenticated
//   - tenant_id が JWT と TenantContext で不一致は PermissionDenied
//   - SkipMethods は認証不要で素通り（health / reflection）
//   - mode=jwks は JWKS から RSA 公開鍵で検証

package common

import (
	"context"
	"crypto/rand"
	"crypto/rsa"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/go-jose/go-jose/v4"
	"github.com/go-jose/go-jose/v4/jwt"
	"google.golang.org/grpc"
	grpccodes "google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// helper: HMAC で signed JWT を作る。
func makeHS256JWT(t *testing.T, secret []byte, claims AuthClaims) string {
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

// helper: ctx に "authorization: Bearer <jwt>" を詰める。
func ctxWithBearer(token string) context.Context {
	md := metadata.Pairs("authorization", "Bearer "+token)
	return metadata.NewIncomingContext(context.Background(), md)
}

// mode=off は既存挙動と等価で pass-through。
func TestAuthInterceptor_OffMode_PassThrough(t *testing.T) {
	icpt := AuthInterceptor(AuthInterceptorConfig{Mode: AuthModeOff})
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	resp, err := icpt(context.Background(), &fakeRequest{ctx: &fakeTenantContext{tenantID: "T1"}}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		// off mode では AuthInfo は付与されない（既存 handler 互換）。
		if _, ok := AuthFromContext(ctx); ok {
			t.Errorf("expected no AuthInfo in off mode")
		}
		return "ok", nil
	})
	if err != nil || resp != "ok" {
		t.Fatalf("pass-through failed: resp=%v err=%v", resp, err)
	}
}

// HMAC mode で JWT が無ければ Unauthenticated。
func TestAuthInterceptor_HMAC_MissingToken_Unauthenticated(t *testing.T) {
	cfg := AuthInterceptorConfig{Mode: AuthModeHMAC, HMACSecret: []byte("secret-32-bytes-min---------------")}
	icpt := AuthInterceptor(cfg)
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	_, err := icpt(context.Background(), &fakeRequest{}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return "ok", nil
	})
	if status.Code(err) != grpccodes.Unauthenticated {
		t.Fatalf("expected Unauthenticated, got %v", err)
	}
}

// HMAC mode で JWT 検証成功時は AuthInfo が context に attach される。
func TestAuthInterceptor_HMAC_Valid_AttachAuthInfo(t *testing.T) {
	secret := []byte("test-hmac-secret-32-bytes--------")
	cfg := AuthInterceptorConfig{Mode: AuthModeHMAC, HMACSecret: secret}
	tok := makeHS256JWT(t, secret, AuthClaims{
		TenantID: "T1",
		Subject:  "user-1",
		Claims: jwt.Claims{
			IssuedAt: jwt.NewNumericDate(time.Now()),
			Expiry:   jwt.NewNumericDate(time.Now().Add(5 * time.Minute)),
		},
	})
	icpt := AuthInterceptor(cfg)
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	called := false
	_, err := icpt(ctxWithBearer(tok), &fakeRequest{ctx: &fakeTenantContext{tenantID: "T1"}}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		called = true
		ai, ok := AuthFromContext(ctx)
		if !ok || ai.TenantID != "T1" || ai.Subject != "user-1" {
			t.Errorf("AuthInfo mismatch: %+v ok=%v", ai, ok)
		}
		return "ok", nil
	})
	if err != nil || !called {
		t.Fatalf("handler not called: err=%v called=%v", err, called)
	}
}

// JWT.tenant_id と TenantContext.tenant_id 不一致は PermissionDenied。
func TestAuthInterceptor_TenantMismatch_PermissionDenied(t *testing.T) {
	secret := []byte("test-hmac-secret-32-bytes--------")
	cfg := AuthInterceptorConfig{Mode: AuthModeHMAC, HMACSecret: secret}
	tok := makeHS256JWT(t, secret, AuthClaims{
		TenantID: "tenant-A",
		Subject:  "user-1",
		Claims: jwt.Claims{
			IssuedAt: jwt.NewNumericDate(time.Now()),
			Expiry:   jwt.NewNumericDate(time.Now().Add(5 * time.Minute)),
		},
	})
	icpt := AuthInterceptor(cfg)
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	// TenantContext に "tenant-B" を入れて不一致を起こす。
	_, err := icpt(ctxWithBearer(tok), &fakeRequest{ctx: &fakeTenantContext{tenantID: "tenant-B"}}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return "ok", nil
	})
	if status.Code(err) != grpccodes.PermissionDenied {
		t.Fatalf("expected PermissionDenied, got %v", err)
	}
}

// SkipMethods（health / reflection）は認証不要で素通り。
func TestAuthInterceptor_SkipMethods_PassThrough(t *testing.T) {
	cfg := AuthInterceptorConfig{
		Mode:        AuthModeHMAC,
		HMACSecret:  []byte("x"),
		SkipMethods: map[string]bool{"/grpc.health.v1.Health/Check": true},
	}
	icpt := AuthInterceptor(cfg)
	info := &grpc.UnaryServerInfo{FullMethod: "/grpc.health.v1.Health/Check"}
	called := false
	_, err := icpt(context.Background(), nil, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		called = true
		return "served", nil
	})
	if err != nil || !called {
		t.Fatalf("skip method failed: err=%v called=%v", err, called)
	}
}

// 期限切れ JWT は Unauthenticated。
func TestAuthInterceptor_HMAC_Expired_Unauthenticated(t *testing.T) {
	secret := []byte("test-hmac-secret-32-bytes--------")
	cfg := AuthInterceptorConfig{Mode: AuthModeHMAC, HMACSecret: secret}
	tok := makeHS256JWT(t, secret, AuthClaims{
		TenantID: "T1",
		Claims: jwt.Claims{
			IssuedAt: jwt.NewNumericDate(time.Now().Add(-2 * time.Hour)),
			Expiry:   jwt.NewNumericDate(time.Now().Add(-1 * time.Hour)),
		},
	})
	icpt := AuthInterceptor(cfg)
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	_, err := icpt(ctxWithBearer(tok), &fakeRequest{}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return "ok", nil
	})
	if status.Code(err) != grpccodes.Unauthenticated {
		t.Fatalf("expected Unauthenticated for expired, got %v", err)
	}
}

// tenant_id クレーム不在は Unauthenticated。
func TestAuthInterceptor_HMAC_MissingTenantClaim_Unauthenticated(t *testing.T) {
	secret := []byte("test-hmac-secret-32-bytes--------")
	cfg := AuthInterceptorConfig{Mode: AuthModeHMAC, HMACSecret: secret}
	tok := makeHS256JWT(t, secret, AuthClaims{
		TenantID: "",
		Claims:   jwt.Claims{IssuedAt: jwt.NewNumericDate(time.Now()), Expiry: jwt.NewNumericDate(time.Now().Add(5 * time.Minute))},
	})
	icpt := AuthInterceptor(cfg)
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	_, err := icpt(ctxWithBearer(tok), &fakeRequest{}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return "ok", nil
	})
	if status.Code(err) != grpccodes.Unauthenticated {
		t.Fatalf("expected Unauthenticated for missing tenant claim, got %v", err)
	}
}

// JWKS mode の end-to-end: httptest server で JWKS を提供し、RSA で signed した JWT を検証する。
func TestAuthInterceptor_JWKS_Valid(t *testing.T) {
	// RSA key pair を生成する。
	priv, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		t.Fatalf("rsa keygen: %v", err)
	}
	jwks := jose.JSONWebKeySet{Keys: []jose.JSONWebKey{
		{Key: &priv.PublicKey, KeyID: "k1", Algorithm: string(jose.RS256), Use: "sig"},
	}}
	// JWKS server。
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(jwks)
	}))
	defer srv.Close()

	// JWT を signing する。
	signer, err := jose.NewSigner(
		jose.SigningKey{Algorithm: jose.RS256, Key: priv},
		(&jose.SignerOptions{}).WithType("JWT").WithHeader("kid", "k1"),
	)
	if err != nil {
		t.Fatalf("signer: %v", err)
	}
	tok, err := jwt.Signed(signer).Claims(AuthClaims{
		TenantID: "T1",
		Subject:  "user-1",
		Claims: jwt.Claims{
			IssuedAt: jwt.NewNumericDate(time.Now()),
			Expiry:   jwt.NewNumericDate(time.Now().Add(5 * time.Minute)),
		},
	}).Serialize()
	if err != nil {
		t.Fatalf("sign: %v", err)
	}

	cfg := AuthInterceptorConfig{
		Mode:         AuthModeJWKS,
		JWKSURL:      srv.URL,
		JWKSCacheTTL: time.Minute,
		HTTPClient:   srv.Client(),
	}
	icpt := AuthInterceptor(cfg)
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	called := false
	_, err = icpt(ctxWithBearer(tok), &fakeRequest{ctx: &fakeTenantContext{tenantID: "T1"}}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		called = true
		ai, ok := AuthFromContext(ctx)
		if !ok || ai.TenantID != "T1" {
			t.Errorf("auth info mismatch: %+v ok=%v", ai, ok)
		}
		return "ok", nil
	})
	if err != nil || !called {
		t.Fatalf("JWKS verify failed: err=%v called=%v", err, called)
	}
}

// stripBearerPrefix の境界条件。
func TestStripBearerPrefix(t *testing.T) {
	cases := []struct {
		in, want string
	}{
		{"Bearer abc", "abc"},
		{"bearer abc", "abc"},
		{"BEARER abc", "abc"},
		{"Basic abc", ""},
		{"Bearer", ""},
		{"", ""},
	}
	for _, c := range cases {
		if got := stripBearerPrefix(c.in); got != c.want {
			t.Errorf("stripBearerPrefix(%q) = %q; want %q", c.in, got, c.want)
		}
	}
}
