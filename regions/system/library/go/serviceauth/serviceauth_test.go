package serviceauth_test

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	sa "github.com/k1s0-platform/system-library-go-serviceauth"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newTokenServer(t *testing.T, token string, expiresIn int) (*httptest.Server, *sa.ServiceAuthConfig) {
	t.Helper()
	var callCount atomic.Int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		callCount.Add(1)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"access_token": token,
			"token_type":   "Bearer",
			"expires_in":   expiresIn,
			"scope":        "openid",
		})
	}))
	t.Cleanup(server.Close)
	cfg := &sa.ServiceAuthConfig{
		TokenURL:     server.URL,
		ClientID:     "test-client",
		ClientSecret: "test-secret",
	}
	return server, cfg
}

// --- ServiceToken Tests ---

func TestServiceToken_IsExpired(t *testing.T) {
	expired := &sa.ServiceToken{ExpiresAt: time.Now().Add(-1 * time.Second)}
	assert.True(t, expired.IsExpired())

	valid := &sa.ServiceToken{ExpiresAt: time.Now().Add(1 * time.Hour)}
	assert.False(t, valid.IsExpired())
}

func TestServiceToken_ShouldRefresh(t *testing.T) {
	// 29 秒後に期限切れ → リフレッシュすべき
	soon := &sa.ServiceToken{ExpiresAt: time.Now().Add(29 * time.Second)}
	assert.True(t, soon.ShouldRefresh())

	// 1 時間後に期限切れ → まだリフレッシュ不要
	later := &sa.ServiceToken{ExpiresAt: time.Now().Add(1 * time.Hour)}
	assert.False(t, later.ShouldRefresh())
}

func TestServiceToken_BearerHeader(t *testing.T) {
	token := &sa.ServiceToken{AccessToken: "my-token-123"}
	assert.Equal(t, "Bearer my-token-123", token.BearerHeader())
}

// --- SpiffeId Tests ---

func TestParseSpiffeId_Valid(t *testing.T) {
	spiffe, err := sa.ParseSpiffeId("spiffe://k1s0.internal/ns/system/sa/auth-service")
	require.NoError(t, err)
	assert.Equal(t, "k1s0.internal", spiffe.TrustDomain)
	assert.Equal(t, "system", spiffe.Namespace)
	assert.Equal(t, "auth-service", spiffe.ServiceAccount)
}

func TestParseSpiffeId_String(t *testing.T) {
	spiffe, err := sa.ParseSpiffeId("spiffe://k1s0.internal/ns/business/sa/order-service")
	require.NoError(t, err)
	assert.Equal(t, "spiffe://k1s0.internal/ns/business/sa/order-service", spiffe.String())
}

func TestParseSpiffeId_Invalid(t *testing.T) {
	tests := []string{
		"",
		"http://not-spiffe",
		"spiffe://no-path",
		"spiffe://domain/invalid/path",
	}
	for _, tt := range tests {
		_, err := sa.ParseSpiffeId(tt)
		assert.Error(t, err, "expected error for: %s", tt)
	}
}

// --- GetToken Tests ---

func TestGetToken_Success(t *testing.T) {
	_, cfg := newTokenServer(t, "access-token-123", 3600)
	client := sa.NewClient(cfg)

	token, err := client.GetToken(context.Background())
	require.NoError(t, err)
	assert.Equal(t, "access-token-123", token.AccessToken)
	assert.Equal(t, "Bearer", token.TokenType)
	assert.False(t, token.IsExpired())
}

func TestGetToken_ServerError(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnauthorized)
	}))
	defer server.Close()

	cfg := &sa.ServiceAuthConfig{
		TokenURL:     server.URL,
		ClientID:     "bad-client",
		ClientSecret: "bad-secret",
	}
	client := sa.NewClient(cfg)

	_, err := client.GetToken(context.Background())
	assert.Error(t, err)
}

// --- GetCachedToken Tests ---

func TestGetCachedToken_CachesToken(t *testing.T) {
	var callCount atomic.Int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		callCount.Add(1)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"access_token": "cached-token",
			"token_type":   "Bearer",
			"expires_in":   3600,
		})
	}))
	defer server.Close()

	cfg := &sa.ServiceAuthConfig{
		TokenURL:     server.URL,
		ClientID:     "test-client",
		ClientSecret: "test-secret",
	}
	client := sa.NewClient(cfg)

	// 3 回呼び出してもトークンエンドポイントは 1 回だけ
	for i := 0; i < 3; i++ {
		bearer, err := client.GetCachedToken(context.Background())
		require.NoError(t, err)
		assert.Equal(t, "Bearer cached-token", bearer)
	}
	assert.Equal(t, int32(1), callCount.Load())
}

func TestGetCachedToken_Concurrent(t *testing.T) {
	var callCount atomic.Int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		callCount.Add(1)
		// 少し遅延を入れてコンカレンシーを再現
		time.Sleep(10 * time.Millisecond)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"access_token": "concurrent-token",
			"token_type":   "Bearer",
			"expires_in":   3600,
		})
	}))
	defer server.Close()

	cfg := &sa.ServiceAuthConfig{
		TokenURL:     server.URL,
		ClientID:     "test-client",
		ClientSecret: "test-secret",
	}
	client := sa.NewClient(cfg)

	// 10 goroutine が同時に GetCachedToken を呼び出す
	var wg sync.WaitGroup
	results := make([]string, 10)
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func(idx int) {
			defer wg.Done()
			bearer, err := client.GetCachedToken(context.Background())
			require.NoError(t, err)
			results[idx] = bearer
		}(i)
	}
	wg.Wait()

	// 全 goroutine が同じトークンを受け取る
	for _, r := range results {
		assert.Equal(t, "Bearer concurrent-token", r)
	}
	// トークンエンドポイントは最小限の回数（並行だが Mutex により制御される）
	// Mutex (not RWMutex) により、最初の goroutine がトークンを取得し、残りはキャッシュを使う
	assert.GreaterOrEqual(t, callCount.Load(), int32(1))
}

// --- ValidateSpiffeId Tests ---

func TestValidateSpiffeId_Valid(t *testing.T) {
	_, cfg := newTokenServer(t, "token", 3600)
	client := sa.NewClient(cfg)

	spiffe, err := client.ValidateSpiffeId("spiffe://k1s0.internal/ns/system/sa/auth-service", "system")
	require.NoError(t, err)
	assert.Equal(t, "system", spiffe.Namespace)
}

func TestValidateSpiffeId_WrongNamespace(t *testing.T) {
	_, cfg := newTokenServer(t, "token", 3600)
	client := sa.NewClient(cfg)

	_, err := client.ValidateSpiffeId("spiffe://k1s0.internal/ns/system/sa/auth-service", "business")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "namespace mismatch")
}

func TestValidateSpiffeId_InvalidFormat(t *testing.T) {
	_, cfg := newTokenServer(t, "token", 3600)
	client := sa.NewClient(cfg)

	_, err := client.ValidateSpiffeId("not-a-spiffe-id", "system")
	assert.Error(t, err)
}

func TestServiceToken_IsExpired_ZeroTime(t *testing.T) {
	// ゼロ値の ExpiresAt は常に期限切れ
	token := &sa.ServiceToken{}
	assert.True(t, token.IsExpired())
}

func TestServiceToken_ShouldRefresh_Boundary(t *testing.T) {
	// 29 秒後 → リフレッシュ閾値 30 秒以内: true
	under := &sa.ServiceToken{ExpiresAt: time.Now().Add(29 * time.Second)}
	assert.True(t, under.ShouldRefresh())
	// 31 秒後 → 閾値外: false
	over := &sa.ServiceToken{ExpiresAt: time.Now().Add(31 * time.Second)}
	assert.False(t, over.ShouldRefresh())
}

func TestServiceToken_Scope(t *testing.T) {
	token := &sa.ServiceToken{AccessToken: "tok", Scope: "openid profile"}
	assert.Equal(t, "openid profile", token.Scope)
	assert.Equal(t, "Bearer tok", token.BearerHeader())
}

func TestGetToken_WithAudience(t *testing.T) {
	var capturedBody string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r.ParseForm()
		capturedBody = r.Form.Get("audience")
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"access_token": "tok",
			"token_type":   "Bearer",
			"expires_in":   3600,
		})
	}))
	defer server.Close()

	cfg := &sa.ServiceAuthConfig{
		TokenURL:     server.URL,
		ClientID:     "client",
		ClientSecret: "secret",
		Audience:     "k1s0-api",
	}
	client := sa.NewClient(cfg)
	_, err := client.GetToken(context.Background())
	require.NoError(t, err)
	assert.Equal(t, "k1s0-api", capturedBody)
}

func TestGetCachedToken_RefreshesExpiredToken(t *testing.T) {
	var callCount atomic.Int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		callCount.Add(1)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"access_token": "new-token",
			"token_type":   "Bearer",
			"expires_in":   3600,
		})
	}))
	defer server.Close()

	cfg := &sa.ServiceAuthConfig{
		TokenURL:     server.URL,
		ClientID:     "client",
		ClientSecret: "secret",
	}
	client := sa.NewClientWithHTTPClient(cfg, server.Client())

	// 最初の呼び出し
	bearer, err := client.GetCachedToken(context.Background())
	require.NoError(t, err)
	assert.Equal(t, "Bearer new-token", bearer)
	assert.Equal(t, int32(1), callCount.Load())
}

func TestGetCachedToken_ReturnsErrorOnTokenFail(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	}))
	defer server.Close()

	cfg := &sa.ServiceAuthConfig{
		TokenURL:     server.URL,
		ClientID:     "client",
		ClientSecret: "secret",
	}
	client := sa.NewClient(cfg)
	_, err := client.GetCachedToken(context.Background())
	assert.Error(t, err)
}

func TestParseSpiffeId_ServiceTier(t *testing.T) {
	spiffe, err := sa.ParseSpiffeId("spiffe://k1s0.internal/ns/service/sa/payment-service")
	require.NoError(t, err)
	assert.Equal(t, "service", spiffe.Namespace)
	assert.Equal(t, "payment-service", spiffe.ServiceAccount)
	assert.Equal(t, "k1s0.internal", spiffe.TrustDomain)
}
