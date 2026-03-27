package oauth

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestClient_Discover(t *testing.T) {
	// M-3 対応後: 必須フィールド（Issuer, AuthorizationEndpoint, TokenEndpoint, JwksURI）をすべて含めること
	oidcCfg := OIDCConfig{
		AuthorizationEndpoint: "https://idp.example.com/authorize",
		TokenEndpoint:         "https://idp.example.com/token",
		UserinfoEndpoint:      "https://idp.example.com/userinfo",
		EndSessionEndpoint:    "https://idp.example.com/logout",
		JwksURI:               "https://idp.example.com/jwks",
		Issuer:                "https://idp.example.com",
	}

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/.well-known/openid-configuration", r.URL.Path)
		w.Header().Set("Content-Type", "application/json")
		// errcheck: テストハンドラのエンコードエラーは無視する（§3.2 監査対応）
		_ = json.NewEncoder(w).Encode(oidcCfg)
	}))
	defer srv.Close()

	client := NewClient(context.Background(), srv.URL, "client-id", "client-secret", "http://localhost/callback", []string{"openid"})

	cfg, err := client.Discover(context.Background())
	require.NoError(t, err)
	assert.Equal(t, oidcCfg.AuthorizationEndpoint, cfg.AuthorizationEndpoint)
	assert.Equal(t, oidcCfg.TokenEndpoint, cfg.TokenEndpoint)
	assert.Equal(t, oidcCfg.Issuer, cfg.Issuer)
}

func TestClient_Discover_Cached(t *testing.T) {
	callCount := 0
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		callCount++
		w.Header().Set("Content-Type", "application/json")
		// M-3 対応後: キャッシュテストでも必須フィールドをすべて含める
		// errcheck: テストハンドラのエンコードエラーは無視する（§3.2 監査対応）
		_ = json.NewEncoder(w).Encode(OIDCConfig{
			Issuer:                "https://idp.example.com",
			AuthorizationEndpoint: "https://idp.example.com/authorize",
			TokenEndpoint:         "https://idp.example.com/token",
			JwksURI:               "https://idp.example.com/jwks",
		})
	}))
	defer srv.Close()

	client := NewClient(context.Background(), srv.URL, "client-id", "", "http://localhost/callback", nil)

	_, err := client.Discover(context.Background())
	require.NoError(t, err)

	_, err = client.Discover(context.Background())
	require.NoError(t, err)

	assert.Equal(t, 1, callCount, "discovery should be cached after first call")
}

func TestClient_AuthCodeURL(t *testing.T) {
	client := NewClient(context.Background(), "https://idp.example.com", "my-client", "", "http://localhost/callback", []string{"openid", "profile"})
	client.oidcConfig = &OIDCConfig{
		AuthorizationEndpoint: "https://idp.example.com/authorize",
	}

	url, err := client.AuthCodeURL("state123", "challenge456")
	require.NoError(t, err)

	assert.Contains(t, url, "response_type=code")
	assert.Contains(t, url, "client_id=my-client")
	assert.Contains(t, url, "state=state123")
	assert.Contains(t, url, "code_challenge=challenge456")
	assert.Contains(t, url, "code_challenge_method=S256")
	assert.Contains(t, url, "scope=openid+profile")
}

func TestClient_ExchangeCode(t *testing.T) {
	tokenResp := TokenResponse{
		AccessToken:  "access-123",
		RefreshToken: "refresh-456",
		IDToken:      "id-789",
		TokenType:    "Bearer",
		ExpiresIn:    300,
	}

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodPost, r.Method)
		assert.Equal(t, "application/x-www-form-urlencoded", r.Header.Get("Content-Type"))

		err := r.ParseForm()
		require.NoError(t, err)
		assert.Equal(t, "authorization_code", r.PostForm.Get("grant_type"))
		assert.Equal(t, "code-abc", r.PostForm.Get("code"))
		assert.Equal(t, "verifier-xyz", r.PostForm.Get("code_verifier"))

		w.Header().Set("Content-Type", "application/json")
		// errcheck: テストハンドラのエンコードエラーは無視する（§3.2 監査対応）
		_ = json.NewEncoder(w).Encode(tokenResp)
	}))
	defer srv.Close()

	client := NewClient(context.Background(), "https://idp.example.com", "my-client", "secret", "http://localhost/callback", nil)
	client.oidcConfig = &OIDCConfig{TokenEndpoint: srv.URL}

	resp, err := client.ExchangeCode(context.Background(), "code-abc", "verifier-xyz")
	require.NoError(t, err)
	assert.Equal(t, "access-123", resp.AccessToken)
	assert.Equal(t, "refresh-456", resp.RefreshToken)
	assert.Equal(t, 300, resp.ExpiresIn)
}

func TestClient_LogoutURL(t *testing.T) {
	client := NewClient(context.Background(), "https://idp.example.com", "my-client", "", "http://localhost/callback", nil)
	client.oidcConfig = &OIDCConfig{
		EndSessionEndpoint: "https://idp.example.com/logout",
	}

	url, err := client.LogoutURL("id-token-hint", "http://localhost:3000")
	require.NoError(t, err)
	assert.Contains(t, url, "id_token_hint=id-token-hint")
	assert.Contains(t, url, "post_logout_redirect_uri=")
}

// TestClient_IsDiscovered_Before はdiscovery実行前にfalseを返すことを確認する。
func TestClient_IsDiscovered_Before(t *testing.T) {
	client := NewClient(context.Background(), "https://idp.example.com", "client-id", "", "http://localhost/callback", nil)
	assert.False(t, client.IsDiscovered())
}

// TestClient_IsDiscovered_After はdiscovery成功後にtrueを返すことを確認する。
func TestClient_IsDiscovered_After(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		// M-3 対応後: 必須フィールドをすべて含める
		// errcheck: テストハンドラのエンコードエラーは無視する（§3.2 監査対応）
		_ = json.NewEncoder(w).Encode(OIDCConfig{
			Issuer:                "https://idp.example.com",
			AuthorizationEndpoint: "https://idp.example.com/authorize",
			TokenEndpoint:         "https://idp.example.com/token",
			JwksURI:               "https://idp.example.com/jwks",
		})
	}))
	defer srv.Close()

	client := NewClient(context.Background(), srv.URL, "client-id", "", "http://localhost/callback", nil)

	_, err := client.Discover(context.Background())
	require.NoError(t, err)
	assert.True(t, client.IsDiscovered())
}

func TestClient_EnsureDiscovered_Error(t *testing.T) {
	client := NewClient(context.Background(), "https://idp.example.com", "my-client", "", "http://localhost/callback", nil)
	// No discovery performed.

	_, err := client.AuthCodeURL("state", "challenge")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "OIDC discovery not performed")
}

// TestDiscover_MissingJwksURI は jwks_uri が空の Discovery レスポンスでエラーになることを確認する。
// M-3 対応: 必須フィールドの欠落を早期検出することで、後続の署名検証失敗を防止する。
func TestDiscover_MissingJwksURI(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		// jwks_uri を意図的に省略した Discovery レスポンスを返す
		// errcheck: テストハンドラのエンコードエラーは無視する（§3.2 監査対応）
		_ = json.NewEncoder(w).Encode(OIDCConfig{
			Issuer:                "https://idp.example.com",
			AuthorizationEndpoint: "https://idp.example.com/authorize",
			TokenEndpoint:         "https://idp.example.com/token",
			// JwksURI: 空のまま（欠落を模擬）
		})
	}))
	defer srv.Close()

	client := NewClient(context.Background(), srv.URL, "client-id", "", "http://localhost/callback", nil)

	_, err := client.Discover(context.Background())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "jwks_uri")
}

// TestDiscover_MissingTokenEndpoint は token_endpoint が空の Discovery レスポンスでエラーになることを確認する。
// M-3 対応: token_endpoint が欠落するとコード交換フローが成立しないため早期エラーとする。
func TestDiscover_MissingTokenEndpoint(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		// token_endpoint を意図的に省略した Discovery レスポンスを返す
		// errcheck: テストハンドラのエンコードエラーは無視する（§3.2 監査対応）
		_ = json.NewEncoder(w).Encode(OIDCConfig{
			Issuer:                "https://idp.example.com",
			AuthorizationEndpoint: "https://idp.example.com/authorize",
			JwksURI:               "https://idp.example.com/jwks",
			// TokenEndpoint: 空のまま（欠落を模擬）
		})
	}))
	defer srv.Close()

	client := NewClient(context.Background(), srv.URL, "client-id", "", "http://localhost/callback", nil)

	_, err := client.Discover(context.Background())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "token_endpoint")
}

// TestExchangeCode_EmptyAccessToken は access_token が空のトークンレスポンスでエラーになることを確認する。
// M-3 対応: access_token が欠落したレスポンスを受け入れると後続の API 呼び出しがすべて失敗するため早期エラーとする。
func TestExchangeCode_EmptyAccessToken(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		// access_token が空のトークンレスポンスを返す（異常系を模擬）
		// errcheck: テストハンドラのエンコードエラーは無視する（§3.2 監査対応）
		_ = json.NewEncoder(w).Encode(TokenResponse{
			// AccessToken: 空のまま（欠落を模擬）
			RefreshToken: "refresh-456",
			TokenType:    "Bearer",
			ExpiresIn:    300,
		})
	}))
	defer srv.Close()

	client := NewClient(context.Background(), "https://idp.example.com", "my-client", "secret", "http://localhost/callback", nil)
	client.oidcConfig = &OIDCConfig{TokenEndpoint: srv.URL}

	_, err := client.ExchangeCode(context.Background(), "code-abc", "verifier-xyz")
	require.Error(t, err)
	assert.Contains(t, err.Error(), "access_token")
}

// TestClient_ClearDiscoveryCache はキャッシュクリア後に再discoveryが必要になることを確認する。
func TestClient_ClearDiscoveryCache(t *testing.T) {
	callCount := 0
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		callCount++
		w.Header().Set("Content-Type", "application/json")
		// M-3 対応後: 必須フィールドをすべて含める
		// errcheck: テストハンドラのエンコードエラーは無視する（§3.2 監査対応）
		_ = json.NewEncoder(w).Encode(OIDCConfig{
			Issuer:                "https://idp.example.com",
			AuthorizationEndpoint: "https://idp.example.com/authorize",
			TokenEndpoint:         "https://idp.example.com/token",
			JwksURI:               "https://idp.example.com/jwks",
		})
	}))
	defer srv.Close()

	client := NewClient(context.Background(), srv.URL, "client-id", "", "http://localhost/callback", nil)

	// 初回 discovery を実行する
	_, err := client.Discover(context.Background())
	require.NoError(t, err)
	assert.True(t, client.IsDiscovered())
	assert.Equal(t, 1, callCount)

	// キャッシュをクリアすると discovered が false に戻ること
	client.ClearDiscoveryCache()
	assert.False(t, client.IsDiscovered())

	// クリア後の AuthCodeURL はエラーを返すこと（再discovery が必要）
	_, err = client.AuthCodeURL("state", "challenge")
	assert.Error(t, err)

	// 再 discovery が成功すること（サーバーへの2回目のリクエスト）
	_, err = client.Discover(context.Background())
	require.NoError(t, err)
	assert.True(t, client.IsDiscovered())
	assert.Equal(t, 2, callCount, "キャッシュクリア後は再度 discovery リクエストが送信されること")
}
