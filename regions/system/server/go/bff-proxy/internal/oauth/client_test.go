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
	oidcCfg := OIDCConfig{
		AuthorizationEndpoint: "https://idp.example.com/authorize",
		TokenEndpoint:         "https://idp.example.com/token",
		UserinfoEndpoint:      "https://idp.example.com/userinfo",
		EndSessionEndpoint:    "https://idp.example.com/logout",
		Issuer:                "https://idp.example.com",
	}

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/.well-known/openid-configuration", r.URL.Path)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(oidcCfg)
	}))
	defer srv.Close()

	client := NewClient(srv.URL, "client-id", "client-secret", "http://localhost/callback", []string{"openid"})

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
		json.NewEncoder(w).Encode(OIDCConfig{Issuer: "https://idp.example.com"})
	}))
	defer srv.Close()

	client := NewClient(srv.URL, "client-id", "", "http://localhost/callback", nil)

	_, err := client.Discover(context.Background())
	require.NoError(t, err)

	_, err = client.Discover(context.Background())
	require.NoError(t, err)

	assert.Equal(t, 1, callCount, "discovery should be cached after first call")
}

func TestClient_AuthCodeURL(t *testing.T) {
	client := NewClient("https://idp.example.com", "my-client", "", "http://localhost/callback", []string{"openid", "profile"})
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
		json.NewEncoder(w).Encode(tokenResp)
	}))
	defer srv.Close()

	client := NewClient("https://idp.example.com", "my-client", "secret", "http://localhost/callback", nil)
	client.oidcConfig = &OIDCConfig{TokenEndpoint: srv.URL}

	resp, err := client.ExchangeCode(context.Background(), "code-abc", "verifier-xyz")
	require.NoError(t, err)
	assert.Equal(t, "access-123", resp.AccessToken)
	assert.Equal(t, "refresh-456", resp.RefreshToken)
	assert.Equal(t, 300, resp.ExpiresIn)
}

func TestClient_LogoutURL(t *testing.T) {
	client := NewClient("https://idp.example.com", "my-client", "", "http://localhost/callback", nil)
	client.oidcConfig = &OIDCConfig{
		EndSessionEndpoint: "https://idp.example.com/logout",
	}

	url, err := client.LogoutURL("id-token-hint", "http://localhost:3000")
	require.NoError(t, err)
	assert.Contains(t, url, "id_token_hint=id-token-hint")
	assert.Contains(t, url, "post_logout_redirect_uri=")
}

func TestClient_EnsureDiscovered_Error(t *testing.T) {
	client := NewClient("https://idp.example.com", "my-client", "", "http://localhost/callback", nil)
	// No discovery performed.

	_, err := client.AuthCodeURL("state", "challenge")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "OIDC discovery not performed")
}
