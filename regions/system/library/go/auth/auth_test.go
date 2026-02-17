package auth

import (
	"context"
	"crypto/rand"
	"crypto/rsa"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/lestrrat-go/jwx/v2/jwa"
	"github.com/lestrrat-go/jwx/v2/jwk"
	"github.com/lestrrat-go/jwx/v2/jwt"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// --- テストヘルパー ---

const (
	testIssuer   = "https://auth.k1s0.internal.example.com/realms/k1s0"
	testAudience = "k1s0-api"
	testKID      = "test-key-1"
)

// testKeyPair はテスト用の RSA 鍵ペアを生成する。
func testKeyPair(t *testing.T) (*rsa.PrivateKey, jwk.Set) {
	t.Helper()

	privKey, err := rsa.GenerateKey(rand.Reader, 2048)
	require.NoError(t, err)

	// JWK セットを構築
	jwkKey, err := jwk.FromRaw(privKey.PublicKey)
	require.NoError(t, err)
	require.NoError(t, jwkKey.Set(jwk.KeyIDKey, testKID))
	require.NoError(t, jwkKey.Set(jwk.AlgorithmKey, jwa.RS256))
	require.NoError(t, jwkKey.Set(jwk.KeyUsageKey, "sig"))

	keySet := jwk.NewSet()
	require.NoError(t, keySet.AddKey(jwkKey))

	return privKey, keySet
}

// mockFetcher はテスト用の JWKS フェッチャー。
type mockFetcher struct {
	keySet jwk.Set
	err    error
}

func (m *mockFetcher) FetchKeys(_ context.Context, _ string) (jwk.Set, error) {
	if m.err != nil {
		return nil, m.err
	}
	return m.keySet, nil
}

// generateTestToken はテスト用の JWT トークンを生成する。
func generateTestToken(t *testing.T, privKey *rsa.PrivateKey, opts ...func(jwt.Token)) string {
	t.Helper()

	token := jwt.New()
	require.NoError(t, token.Set(jwt.SubjectKey, "user-uuid-1234"))
	require.NoError(t, token.Set(jwt.IssuerKey, testIssuer))
	require.NoError(t, token.Set(jwt.AudienceKey, testAudience))
	require.NoError(t, token.Set(jwt.ExpirationKey, time.Now().Add(15*time.Minute)))
	require.NoError(t, token.Set(jwt.IssuedAtKey, time.Now()))
	require.NoError(t, token.Set(jwt.JwtIDKey, "token-uuid-5678"))
	require.NoError(t, token.Set("typ", "Bearer"))
	require.NoError(t, token.Set("azp", "react-spa"))
	require.NoError(t, token.Set("scope", "openid profile email"))
	require.NoError(t, token.Set("preferred_username", "taro.yamada"))
	require.NoError(t, token.Set("email", "taro.yamada@example.com"))
	require.NoError(t, token.Set("realm_access", map[string]interface{}{
		"roles": []interface{}{"user", "order_manager"},
	}))
	require.NoError(t, token.Set("resource_access", map[string]interface{}{
		"order-service": map[string]interface{}{
			"roles": []interface{}{"read", "write"},
		},
	}))
	require.NoError(t, token.Set("tier_access", []interface{}{"system", "business", "service"}))

	for _, opt := range opts {
		opt(token)
	}

	// JWK 秘密鍵を構築
	jwkPriv, err := jwk.FromRaw(privKey)
	require.NoError(t, err)
	require.NoError(t, jwkPriv.Set(jwk.KeyIDKey, testKID))
	require.NoError(t, jwkPriv.Set(jwk.AlgorithmKey, jwa.RS256))

	signed, err := jwt.Sign(token, jwt.WithKey(jwa.RS256, jwkPriv))
	require.NoError(t, err)

	return string(signed)
}

// --- Claims テスト ---

func TestExtractClaims(t *testing.T) {
	privKey, _ := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey)

	// Claims 抽出のためにパースのみ（検証なし）
	token, err := jwt.Parse([]byte(tokenStr), jwt.WithVerify(false), jwt.WithValidate(false))
	require.NoError(t, err)

	claims, err := extractClaims(token)
	require.NoError(t, err)

	assert.Equal(t, "user-uuid-1234", claims.Sub)
	assert.Equal(t, testIssuer, claims.Iss)
	assert.Equal(t, testAudience, claims.Aud)
	assert.Equal(t, "Bearer", claims.Typ)
	assert.Equal(t, "react-spa", claims.Azp)
	assert.Equal(t, "openid profile email", claims.Scope)
	assert.Equal(t, "taro.yamada", claims.PreferredUsername)
	assert.Equal(t, "taro.yamada@example.com", claims.Email)
	assert.Contains(t, claims.RealmAccess.Roles, "user")
	assert.Contains(t, claims.RealmAccess.Roles, "order_manager")
	assert.Contains(t, claims.ResourceAccess["order-service"].Roles, "read")
	assert.Contains(t, claims.ResourceAccess["order-service"].Roles, "write")
	assert.Equal(t, []string{"system", "business", "service"}, claims.TierAccess)
}

func TestClaims_IsExpired(t *testing.T) {
	claims := &Claims{Exp: time.Now().Add(-1 * time.Hour).Unix()}
	assert.True(t, claims.IsExpired())

	claims2 := &Claims{Exp: time.Now().Add(1 * time.Hour).Unix()}
	assert.False(t, claims2.IsExpired())
}

func TestClaims_String(t *testing.T) {
	claims := &Claims{
		Sub:              "user-1",
		Iss:              testIssuer,
		Aud:              testAudience,
		PreferredUsername: "taro",
		Email:            "taro@example.com",
	}
	s := claims.String()
	assert.Contains(t, s, "user-1")
	assert.Contains(t, s, "taro")
}

// --- JWKS Verifier テスト ---

func TestJWKSVerifier_VerifyToken_Success(t *testing.T) {
	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey)

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	claims, err := verifier.VerifyToken(context.Background(), tokenStr)
	require.NoError(t, err)
	assert.Equal(t, "user-uuid-1234", claims.Sub)
	assert.Equal(t, testIssuer, claims.Iss)
	assert.Equal(t, testAudience, claims.Aud)
	assert.Equal(t, "taro.yamada", claims.PreferredUsername)
}

func TestJWKSVerifier_VerifyToken_ExpiredToken(t *testing.T) {
	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey, func(token jwt.Token) {
		_ = token.Set(jwt.ExpirationKey, time.Now().Add(-1*time.Hour))
	})

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	_, err := verifier.VerifyToken(context.Background(), tokenStr)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "token validation failed")
}

func TestJWKSVerifier_VerifyToken_WrongIssuer(t *testing.T) {
	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey, func(token jwt.Token) {
		_ = token.Set(jwt.IssuerKey, "https://evil.example.com/realms/bad")
	})

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	_, err := verifier.VerifyToken(context.Background(), tokenStr)
	assert.Error(t, err)
}

func TestJWKSVerifier_VerifyToken_WrongAudience(t *testing.T) {
	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey, func(token jwt.Token) {
		_ = token.Set(jwt.AudienceKey, "wrong-audience")
	})

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	_, err := verifier.VerifyToken(context.Background(), tokenStr)
	assert.Error(t, err)
}

func TestJWKSVerifier_CacheTTL(t *testing.T) {
	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey)

	fetchCount := 0
	fetcher := &mockFetcher{keySet: keySet}
	countingFetcher := &countingFetcherWrapper{inner: fetcher, count: &fetchCount}

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		countingFetcher,
	)

	// 1回目: フェッチが発生
	_, err := verifier.VerifyToken(context.Background(), tokenStr)
	require.NoError(t, err)
	assert.Equal(t, 1, fetchCount)

	// 2回目: キャッシュから取得
	_, err = verifier.VerifyToken(context.Background(), tokenStr)
	require.NoError(t, err)
	assert.Equal(t, 1, fetchCount) // フェッチ回数は増えない
}

func TestJWKSVerifier_InvalidateCache(t *testing.T) {
	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey)

	fetchCount := 0
	fetcher := &mockFetcher{keySet: keySet}
	countingFetcher := &countingFetcherWrapper{inner: fetcher, count: &fetchCount}

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		countingFetcher,
	)

	_, err := verifier.VerifyToken(context.Background(), tokenStr)
	require.NoError(t, err)
	assert.Equal(t, 1, fetchCount)

	// キャッシュを無効化
	verifier.InvalidateCache()

	_, err = verifier.VerifyToken(context.Background(), tokenStr)
	require.NoError(t, err)
	assert.Equal(t, 2, fetchCount) // 再フェッチが発生
}

// countingFetcherWrapper はフェッチ回数を記録するラッパー。
type countingFetcherWrapper struct {
	inner JWKSFetcher
	count *int
}

func (c *countingFetcherWrapper) FetchKeys(ctx context.Context, jwksURL string) (jwk.Set, error) {
	*c.count++
	return c.inner.FetchKeys(ctx, jwksURL)
}

// --- RBAC テスト ---

func TestHasRole(t *testing.T) {
	claims := &Claims{
		RealmAccess: RealmAccess{
			Roles: []string{"user", "order_manager"},
		},
	}

	assert.True(t, HasRole(claims, "user"))
	assert.True(t, HasRole(claims, "order_manager"))
	assert.False(t, HasRole(claims, "admin"))
	assert.False(t, HasRole(claims, "sys_admin"))
}

func TestHasResourceRole(t *testing.T) {
	claims := &Claims{
		ResourceAccess: map[string]Access{
			"order-service": {Roles: []string{"read", "write"}},
		},
	}

	assert.True(t, HasResourceRole(claims, "order-service", "read"))
	assert.True(t, HasResourceRole(claims, "order-service", "write"))
	assert.False(t, HasResourceRole(claims, "order-service", "delete"))
	assert.False(t, HasResourceRole(claims, "user-service", "read"))
}

func TestHasPermission(t *testing.T) {
	claims := &Claims{
		RealmAccess: RealmAccess{
			Roles: []string{"user"},
		},
		ResourceAccess: map[string]Access{
			"order-service": {Roles: []string{"read", "write"}},
		},
	}

	assert.True(t, HasPermission(claims, "order-service", "read"))
	assert.True(t, HasPermission(claims, "order-service", "write"))
	assert.False(t, HasPermission(claims, "order-service", "delete"))
}

func TestHasPermission_SysAdmin(t *testing.T) {
	claims := &Claims{
		RealmAccess: RealmAccess{
			Roles: []string{"sys_admin"},
		},
	}

	// sys_admin は全権限
	assert.True(t, HasPermission(claims, "any-resource", "read"))
	assert.True(t, HasPermission(claims, "any-resource", "write"))
	assert.True(t, HasPermission(claims, "any-resource", "delete"))
}

func TestHasPermission_ResourceAdmin(t *testing.T) {
	claims := &Claims{
		RealmAccess: RealmAccess{
			Roles: []string{"user"},
		},
		ResourceAccess: map[string]Access{
			"order-service": {Roles: []string{"admin"}},
		},
	}

	// リソースに admin ロールがある場合
	assert.True(t, HasPermission(claims, "order-service", "read"))
	assert.True(t, HasPermission(claims, "order-service", "write"))
	assert.True(t, HasPermission(claims, "order-service", "delete"))
}

func TestHasTierAccess(t *testing.T) {
	claims := &Claims{
		TierAccess: []string{"system", "business"},
	}

	assert.True(t, HasTierAccess(claims, "system"))
	assert.True(t, HasTierAccess(claims, "business"))
	assert.True(t, HasTierAccess(claims, "System")) // case insensitive
	assert.False(t, HasTierAccess(claims, "service"))
}

func TestHasTierAccess_Empty(t *testing.T) {
	claims := &Claims{
		TierAccess: nil,
	}

	assert.False(t, HasTierAccess(claims, "system"))
}

// --- Middleware テスト ---

func TestAuthMiddleware_NoAuthHeader(t *testing.T) {
	gin.SetMode(gin.TestMode)

	privKey, keySet := testKeyPair(t)
	_ = privKey

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	w := httptest.NewRecorder()
	c, r := gin.CreateTestContext(w)

	r.Use(AuthMiddleware(verifier))
	r.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})

	c.Request = httptest.NewRequest("GET", "/test", nil)
	r.ServeHTTP(w, c.Request)

	assert.Equal(t, http.StatusUnauthorized, w.Code)
}

func TestAuthMiddleware_ValidToken(t *testing.T) {
	gin.SetMode(gin.TestMode)

	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey)

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	w := httptest.NewRecorder()
	_, r := gin.CreateTestContext(w)

	r.Use(AuthMiddleware(verifier))
	r.GET("/test", func(c *gin.Context) {
		claims, ok := GetClaims(c)
		assert.True(t, ok)
		assert.Equal(t, "user-uuid-1234", claims.Sub)
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("Authorization", "Bearer "+tokenStr)
	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}

func TestAuthMiddleware_InvalidToken(t *testing.T) {
	gin.SetMode(gin.TestMode)

	_, keySet := testKeyPair(t)

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	w := httptest.NewRecorder()
	_, r := gin.CreateTestContext(w)

	r.Use(AuthMiddleware(verifier))
	r.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("Authorization", "Bearer invalid-token")
	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusUnauthorized, w.Code)
}

func TestRequireRole_Authorized(t *testing.T) {
	gin.SetMode(gin.TestMode)

	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey)

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	w := httptest.NewRecorder()
	_, r := gin.CreateTestContext(w)

	r.Use(AuthMiddleware(verifier))
	r.Use(RequireRole("user"))
	r.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("Authorization", "Bearer "+tokenStr)
	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}

func TestRequireRole_Forbidden(t *testing.T) {
	gin.SetMode(gin.TestMode)

	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey)

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	w := httptest.NewRecorder()
	_, r := gin.CreateTestContext(w)

	r.Use(AuthMiddleware(verifier))
	r.Use(RequireRole("sys_admin"))
	r.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("Authorization", "Bearer "+tokenStr)
	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusForbidden, w.Code)
}

func TestRequirePermission_Authorized(t *testing.T) {
	gin.SetMode(gin.TestMode)

	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey)

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	w := httptest.NewRecorder()
	_, r := gin.CreateTestContext(w)

	r.Use(AuthMiddleware(verifier))
	r.Use(RequirePermission("order-service", "read"))
	r.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("Authorization", "Bearer "+tokenStr)
	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}

func TestRequirePermission_Forbidden(t *testing.T) {
	gin.SetMode(gin.TestMode)

	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey)

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	w := httptest.NewRecorder()
	_, r := gin.CreateTestContext(w)

	r.Use(AuthMiddleware(verifier))
	r.Use(RequirePermission("order-service", "delete"))
	r.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("Authorization", "Bearer "+tokenStr)
	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusForbidden, w.Code)
}

func TestRequireTierAccess_Authorized(t *testing.T) {
	gin.SetMode(gin.TestMode)

	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey)

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	w := httptest.NewRecorder()
	_, r := gin.CreateTestContext(w)

	r.Use(AuthMiddleware(verifier))
	r.Use(RequireTierAccess("business"))
	r.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("Authorization", "Bearer "+tokenStr)
	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}

func TestRequireTierAccess_Forbidden(t *testing.T) {
	gin.SetMode(gin.TestMode)

	privKey, keySet := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey, func(token jwt.Token) {
		_ = token.Set("tier_access", []interface{}{"system"})
	})

	verifier := NewJWKSVerifierWithFetcher(
		"https://auth.example.com/jwks",
		testIssuer,
		testAudience,
		10*time.Minute,
		&mockFetcher{keySet: keySet},
	)

	w := httptest.NewRecorder()
	_, r := gin.CreateTestContext(w)

	r.Use(AuthMiddleware(verifier))
	r.Use(RequireTierAccess("service"))
	r.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("Authorization", "Bearer "+tokenStr)
	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusForbidden, w.Code)
}

// --- extractBearerToken テスト ---

func TestExtractBearerToken_Valid(t *testing.T) {
	req := httptest.NewRequest("GET", "/", nil)
	req.Header.Set("Authorization", "Bearer mytoken123")

	token, err := extractBearerToken(req)
	require.NoError(t, err)
	assert.Equal(t, "mytoken123", token)
}

func TestExtractBearerToken_Missing(t *testing.T) {
	req := httptest.NewRequest("GET", "/", nil)

	_, err := extractBearerToken(req)
	assert.Error(t, err)
	assert.Equal(t, ErrMissingToken, err)
}

func TestExtractBearerToken_InvalidFormat(t *testing.T) {
	req := httptest.NewRequest("GET", "/", nil)
	req.Header.Set("Authorization", "Basic abc123")

	_, err := extractBearerToken(req)
	assert.Error(t, err)
	assert.Equal(t, ErrInvalidAuthHeader, err)
}

func TestExtractBearerToken_EmptyToken(t *testing.T) {
	req := httptest.NewRequest("GET", "/", nil)
	req.Header.Set("Authorization", "Bearer ")

	_, err := extractBearerToken(req)
	assert.Error(t, err)
	assert.Equal(t, ErrMissingToken, err)
}

// --- GetClaimsFromContext テスト ---

func TestGetClaimsFromContext(t *testing.T) {
	claims := &Claims{Sub: "user-1"}
	ctx := context.WithValue(context.Background(), ClaimsContextKey, claims)

	got, ok := GetClaimsFromContext(ctx)
	assert.True(t, ok)
	assert.Equal(t, "user-1", got.Sub)
}

func TestGetClaimsFromContext_Empty(t *testing.T) {
	_, ok := GetClaimsFromContext(context.Background())
	assert.False(t, ok)
}
