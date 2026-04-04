package authlib

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
	// JWT クレームにロール情報を設定する（interface{} → any: Go 1.18+ 推奨エイリアスを使用する）
	require.NoError(t, token.Set("realm_access", map[string]any{
		"roles": []any{"user", "order_manager"},
	}))
	require.NoError(t, token.Set("resource_access", map[string]any{
		"task-server": map[string]any{
			"roles": []any{"read", "write"},
		},
	}))
	require.NoError(t, token.Set("tier_access", []any{"system", "business", "service"}))

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

// extractClaims が JWT トークンから全クレームフィールドを正しく抽出することを確認する。
func TestExtractClaims(t *testing.T) {
	privKey, _ := testKeyPair(t)
	tokenStr := generateTestToken(t, privKey)

	// Claims 抽出のためにパースのみ（検証なし）
	token, err := jwt.Parse([]byte(tokenStr), jwt.WithVerify(false), jwt.WithValidate(false))
	require.NoError(t, err)

	claims, err := extractClaims(token)
	require.NoError(t, err)

	assert.Equal(t, "user-uuid-1234", claims.Sub)
	// Deprecated フィールド（Iss, Aud, PreferredUsername）ではなく正式フィールドを参照する（ADR-0020）。
	assert.Equal(t, testIssuer, claims.Issuer)
	assert.Contains(t, claims.Audience, testAudience)
	assert.Equal(t, "Bearer", claims.Typ)
	assert.Equal(t, "react-spa", claims.Azp)
	assert.Equal(t, "openid profile email", claims.Scope)
	assert.Equal(t, "taro.yamada", claims.Username)
	assert.Equal(t, "taro.yamada@example.com", claims.Email)
	assert.Contains(t, claims.RealmAccess.Roles, "user")
	assert.Contains(t, claims.RealmAccess.Roles, "order_manager")
	assert.Contains(t, claims.ResourceAccess["task-server"].Roles, "read")
	assert.Contains(t, claims.ResourceAccess["task-server"].Roles, "write")
	assert.Equal(t, []string{"system", "business", "service"}, claims.TierAccess)
	// BSL-CRIT-003 監査対応: デフォルトトークンには tenant_id がないため空文字列となることを確認する。
	assert.Equal(t, "", claims.TenantID)
}

// BSL-CRIT-003 監査対応: extractClaims が tenant_id クレームを正しく抽出することを確認する。
// JWT に tenant_id がある場合は TenantID フィールドに格納され、
// JWT に tenant_id がない場合は空文字列となることを確認する。
func TestExtractClaims_TenantID(t *testing.T) {
	privKey, _ := testKeyPair(t)

	// 正常系: JWT に tenant_id が含まれる場合 → TenantID に格納されること
	t.Run("JWT に tenant_id がある場合は TenantID に格納される", func(t *testing.T) {
		tokenStr := generateTestToken(t, privKey, func(token jwt.Token) {
			// tenant_id カスタムクレームを設定する（Keycloak Protocol Mapper 相当）
			require.NoError(t, token.Set("tenant_id", "tenant-123"))
		})
		token, err := jwt.Parse([]byte(tokenStr), jwt.WithVerify(false), jwt.WithValidate(false))
		require.NoError(t, err)

		claims, err := extractClaims(token)
		require.NoError(t, err)

		// TenantID が正しく抽出されていること
		assert.Equal(t, "tenant-123", claims.TenantID)
	})

	// 正常系: JWT に tenant_id がない場合 → TenantID は空文字列となること
	t.Run("JWT に tenant_id がない場合は TenantID は空文字列", func(t *testing.T) {
		tokenStr := generateTestToken(t, privKey)
		token, err := jwt.Parse([]byte(tokenStr), jwt.WithVerify(false), jwt.WithValidate(false))
		require.NoError(t, err)

		claims, err := extractClaims(token)
		require.NoError(t, err)

		// TenantID は空文字列であること（"system" へのフォールバックは呼び出し元の責務）
		assert.Equal(t, "", claims.TenantID)
	})
}

// BSL-CRIT-003 監査対応: Claims の String メソッドが TenantID を含むことを確認する。
func TestClaims_String_IncludesTenantID(t *testing.T) {
	// TenantID が設定されている場合にデバッグ出力へ含まれることを確認する。
	claims := &Claims{
		Sub:      "user-1",
		Issuer:   testIssuer,
		Audience: []string{testAudience},
		Username: "taro",
		Email:    "taro@example.com",
		TenantID: "acme-corp",
	}
	s := claims.String()
	// tenant_id がデバッグ出力に含まれること
	assert.Contains(t, s, "acme-corp")
}

// Claims の IsExpired メソッドが期限切れと有効なトークンを正しく判定することを確認する。
// IsExpired は ExpiresAt (time.Time) を参照するため、Exp (int64) ではなく ExpiresAt を設定する。
func TestClaims_IsExpired(t *testing.T) {
	claims := &Claims{ExpiresAt: time.Now().Add(-1 * time.Hour)}
	assert.True(t, claims.IsExpired())

	claims2 := &Claims{ExpiresAt: time.Now().Add(1 * time.Hour)}
	assert.False(t, claims2.IsExpired())
}

// Claims の String メソッドが Subject やユーザー名を含む文字列を返し、
// email は PII マスキング（先頭1文字 + "***" + "@以降"）されることを確認する。
func TestClaims_String(t *testing.T) {
	claims := &Claims{
		Sub:      "user-1",
		Issuer:   testIssuer,
		Audience: []string{testAudience},
		Username: "taro",
		Email:    "taro@example.com",
	}
	s := claims.String()
	assert.Contains(t, s, "user-1")
	assert.Contains(t, s, "taro")
	// email はマスキングされ "t***@example.com" となること
	assert.Contains(t, s, "t***@example.com")
	// 元のメールアドレスがそのまま含まれていないこと
	assert.NotContains(t, s, "taro@example.com")
}

// --- JWKS Verifier テスト ---

// JWKSVerifier が有効なトークンを検証してクレームを正しく返すことを確認する。
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
	// Deprecated フィールド（Iss, Aud, PreferredUsername）ではなく正式フィールドを参照する（ADR-0020）。
	assert.Equal(t, testIssuer, claims.Issuer)
	assert.Contains(t, claims.Audience, testAudience)
	assert.Equal(t, "taro.yamada", claims.Username)
}

// JWKSVerifier が期限切れトークンの検証でエラーを返すことを確認する。
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

// JWKSVerifier が不正な Issuer を持つトークンの検証でエラーを返すことを確認する。
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

// JWKSVerifier が不正な Audience を持つトークンの検証でエラーを返すことを確認する。
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

// JWKSVerifier が TTL 内で JWKS キーをキャッシュして再フェッチを行わないことを確認する。
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

// JWKSVerifier の InvalidateCache がキャッシュを無効化し次回の検証で再フェッチが発生することを確認する。
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

// HasRole がクレームの RealmAccess ロールに指定ロールが含まれるか正しく判定することを確認する。
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

// HasResourceRole が特定リソースの ResourceAccess ロールに指定ロールが含まれるか正しく判定することを確認する。
func TestHasResourceRole(t *testing.T) {
	claims := &Claims{
		ResourceAccess: map[string]Access{
			"task-server": {Roles: []string{"read", "write"}},
		},
	}

	assert.True(t, HasResourceRole(claims, "task-server", "read"))
	assert.True(t, HasResourceRole(claims, "task-server", "write"))
	assert.False(t, HasResourceRole(claims, "task-server", "delete"))
	assert.False(t, HasResourceRole(claims, "user-service", "read"))
}

// HasPermission がリソースへのアクセス権を持つクレームに対して true を返すことを確認する。
func TestHasPermission(t *testing.T) {
	claims := &Claims{
		RealmAccess: RealmAccess{
			Roles: []string{"user"},
		},
		ResourceAccess: map[string]Access{
			"task-server": {Roles: []string{"read", "write"}},
		},
	}

	assert.True(t, HasPermission(claims, "task-server", "read"))
	assert.True(t, HasPermission(claims, "task-server", "write"))
	assert.False(t, HasPermission(claims, "task-server", "delete"))
}

// HasPermission が sys_admin ロールを持つクレームに対して全リソース・全操作で true を返すことを確認する。
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

// HasPermission が realm_access の admin ロールを持つクレームに対して全権限を付与しないことを確認する。
// 最小権限原則により、realm_access の admin は通常ロールとして扱い、sys_admin のみ全権限を持つ。
func TestHasPermission_RealmAdminNotAllPowerful(t *testing.T) {
	claims := &Claims{
		RealmAccess: RealmAccess{
			Roles: []string{"admin"},
		},
	}

	// realm_access の admin は全権限を持たない（最小権限原則）
	assert.False(t, HasPermission(claims, "any-resource", "read"))
	assert.False(t, HasPermission(claims, "any-resource", "write"))
	assert.False(t, HasPermission(claims, "any-resource", "delete"))
}

// HasPermission がリソースに admin ロールを持つクレームに対してそのリソースの全操作で true を返すことを確認する。
func TestHasPermission_ResourceAdmin(t *testing.T) {
	claims := &Claims{
		RealmAccess: RealmAccess{
			Roles: []string{"user"},
		},
		ResourceAccess: map[string]Access{
			"task-server": {Roles: []string{"admin"}},
		},
	}

	// リソースに admin ロールがある場合
	assert.True(t, HasPermission(claims, "task-server", "read"))
	assert.True(t, HasPermission(claims, "task-server", "write"))
	assert.True(t, HasPermission(claims, "task-server", "delete"))
}

// HasTierAccess がティア階層ルールに従ってアクセス判定を行うことを確認する。
// system(0) > business(1) > service(2) の階層で、上位ティアは下位ティアにもアクセス可能。
func TestHasTierAccess(t *testing.T) {
	claims := &Claims{
		TierAccess: []string{"system", "business"},
	}

	assert.True(t, HasTierAccess(claims, "system"))
	assert.True(t, HasTierAccess(claims, "business"))
	assert.True(t, HasTierAccess(claims, "System")) // 大文字小文字を区別しない
	assert.True(t, HasTierAccess(claims, "service")) // system/business ティアは service にもアクセス可能
}

// HasTierAccess が TierAccess が nil のクレームに対して false を返すことを確認する。
func TestHasTierAccess_Empty(t *testing.T) {
	claims := &Claims{
		TierAccess: nil,
	}

	assert.False(t, HasTierAccess(claims, "system"))
}

// HasTierAccess_Hierarchy がティア階層に基づくアクセス制御を網羅的にテストする。
// system ティアは system, business, service の全てにアクセス可能。
// business ティアは business と service にアクセス可能。
// service ティアは service のみアクセス可能。
// 不明なティアは常に false を返す。
func TestHasTierAccess_Hierarchy(t *testing.T) {
	tests := []struct {
		name         string
		userTiers    []string
		requiredTier string
		want         bool
	}{
		// system ティアユーザーのアクセス判定
		{
			name:         "system ユーザーが system にアクセス → 許可",
			userTiers:    []string{"system"},
			requiredTier: "system",
			want:         true,
		},
		{
			name:         "system ユーザーが business にアクセス → 許可",
			userTiers:    []string{"system"},
			requiredTier: "business",
			want:         true,
		},
		{
			name:         "system ユーザーが service にアクセス → 許可",
			userTiers:    []string{"system"},
			requiredTier: "service",
			want:         true,
		},
		// business ティアユーザーのアクセス判定
		{
			name:         "business ユーザーが system にアクセス → 拒否",
			userTiers:    []string{"business"},
			requiredTier: "system",
			want:         false,
		},
		{
			name:         "business ユーザーが business にアクセス → 許可",
			userTiers:    []string{"business"},
			requiredTier: "business",
			want:         true,
		},
		{
			name:         "business ユーザーが service にアクセス → 許可",
			userTiers:    []string{"business"},
			requiredTier: "service",
			want:         true,
		},
		// service ティアユーザーのアクセス判定
		{
			name:         "service ユーザーが service にアクセス → 許可",
			userTiers:    []string{"service"},
			requiredTier: "service",
			want:         true,
		},
		{
			name:         "service ユーザーが business にアクセス → 拒否",
			userTiers:    []string{"service"},
			requiredTier: "business",
			want:         false,
		},
		// 不明なティアの判定
		{
			name:         "不明なティアが要求された場合 → 拒否",
			userTiers:    []string{"system"},
			requiredTier: "unknown",
			want:         false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			claims := &Claims{
				TierAccess: tt.userTiers,
			}
			got := HasTierAccess(claims, tt.requiredTier)
			assert.Equal(t, tt.want, got)
		})
	}
}

// --- Middleware テスト ---

// AuthMiddleware が Authorization ヘッダーのないリクエストに 401 を返すことを確認する。
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

// AuthMiddleware が有効なトークンを持つリクエストを通過させクレームをコンテキストに設定することを確認する。
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

// AuthMiddleware が無効なトークンを持つリクエストに 401 を返すことを確認する。
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

// RequireRole が必要なロールを持つクレームのリクエストを正常に通過させることを確認する。
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

// RequireRole が必要なロールを持たないクレームのリクエストに 403 を返すことを確認する。
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

// RequirePermission が必要なリソース権限を持つクレームのリクエストを正常に通過させることを確認する。
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
	r.Use(RequirePermission("task-server", "read"))
	r.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("Authorization", "Bearer "+tokenStr)
	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}

// RequirePermission が必要なリソース権限を持たないクレームのリクエストに 403 を返すことを確認する。
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
	r.Use(RequirePermission("task-server", "delete"))
	r.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("Authorization", "Bearer "+tokenStr)
	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusForbidden, w.Code)
}

// RequireTierAccess が必要なティアアクセスを持つクレームのリクエストを正常に通過させることを確認する。
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

// RequireTierAccess が必要なティアアクセスを持たないクレームのリクエストに 403 を返すことを確認する。
// service ティアのみのユーザーが system ティアを要求するエンドポイントにアクセスした場合、拒否される。
func TestRequireTierAccess_Forbidden(t *testing.T) {
	gin.SetMode(gin.TestMode)

	privKey, keySet := testKeyPair(t)
	// service ティアのみを持つユーザーのトークンを生成
	tokenStr := generateTestToken(t, privKey, func(token jwt.Token) {
		// interface{} → any: Go 1.18+ 推奨エイリアスを使用する
		_ = token.Set("tier_access", []any{"service"})
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
	// service ティアユーザーは system ティアにアクセスできない
	r.Use(RequireTierAccess("system"))
	r.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})

	req := httptest.NewRequest("GET", "/test", nil)
	req.Header.Set("Authorization", "Bearer "+tokenStr)
	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusForbidden, w.Code)
}

// --- extractBearerToken テスト ---

// extractBearerToken が有効な Bearer トークンを Authorization ヘッダーから正しく抽出することを確認する。
func TestExtractBearerToken_Valid(t *testing.T) {
	req := httptest.NewRequest("GET", "/", nil)
	req.Header.Set("Authorization", "Bearer mytoken123")

	token, err := extractBearerToken(req)
	require.NoError(t, err)
	assert.Equal(t, "mytoken123", token)
}

// extractBearerToken が Authorization ヘッダーが存在しない場合に ErrMissingToken を返すことを確認する。
func TestExtractBearerToken_Missing(t *testing.T) {
	req := httptest.NewRequest("GET", "/", nil)

	_, err := extractBearerToken(req)
	assert.Error(t, err)
	assert.Equal(t, ErrMissingToken, err)
}

// extractBearerToken が Bearer 以外のスキームを持つ Authorization ヘッダーに ErrInvalidAuthHeader を返すことを確認する。
func TestExtractBearerToken_InvalidFormat(t *testing.T) {
	req := httptest.NewRequest("GET", "/", nil)
	req.Header.Set("Authorization", "Basic abc123")

	_, err := extractBearerToken(req)
	assert.Error(t, err)
	assert.Equal(t, ErrInvalidAuthHeader, err)
}

// extractBearerToken が Bearer の後にトークン値がない場合に ErrMissingToken を返すことを確認する。
func TestExtractBearerToken_EmptyToken(t *testing.T) {
	req := httptest.NewRequest("GET", "/", nil)
	req.Header.Set("Authorization", "Bearer ")

	_, err := extractBearerToken(req)
	assert.Error(t, err)
	assert.Equal(t, ErrMissingToken, err)
}

// --- GetClaimsFromContext テスト ---

// GetClaimsFromContext がコンテキストに格納されたクレームを正しく取得することを確認する。
func TestGetClaimsFromContext(t *testing.T) {
	claims := &Claims{Sub: "user-1"}
	ctx := context.WithValue(context.Background(), ClaimsContextKey, claims)

	got, ok := GetClaimsFromContext(ctx)
	assert.True(t, ok)
	assert.Equal(t, "user-1", got.Sub)
}

// GetClaimsFromContext がクレームを含まないコンテキストに対して false を返すことを確認する。
func TestGetClaimsFromContext_Empty(t *testing.T) {
	_, ok := GetClaimsFromContext(context.Background())
	assert.False(t, ok)
}
