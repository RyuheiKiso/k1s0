package oauth

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/coreos/go-oidc/v3/oidc"
)

// OIDCConfig holds the OIDC provider endpoints discovered or configured.
type OIDCConfig struct {
	AuthorizationEndpoint string `json:"authorization_endpoint"`
	TokenEndpoint         string `json:"token_endpoint"`
	UserinfoEndpoint      string `json:"userinfo_endpoint"`
	EndSessionEndpoint    string `json:"end_session_endpoint"`
	// JwksURI はJWKSエンドポイントURL。JWT署名検証に使用する。
	JwksURI string `json:"jwks_uri"`
	Issuer  string `json:"issuer"`
}

// TokenResponse represents an OAuth2 token endpoint response.
type TokenResponse struct {
	AccessToken  string `json:"access_token"`
	RefreshToken string `json:"refresh_token,omitempty"`
	IDToken      string `json:"id_token,omitempty"`
	TokenType    string `json:"token_type"`
	ExpiresIn    int    `json:"expires_in"`
}

// Client handles OIDC provider interactions for the BFF.
type Client struct {
	discoveryURL string
	clientID     string
	clientSecret string
	redirectURI  string
	scopes       []string
	httpClient   *http.Client
	// mu はoidcConfigとverifierへの並行アクセスを保護するRWMutex。
	// Discover/ClearDiscoveryCacheは書き込みロック、読み取り系メソッドは読み取りロックを使用する。
	mu         sync.RWMutex
	oidcConfig *OIDCConfig
	// discovered はOIDC discoveryが正常に完了したかを示すフラグ（goroutine間で安全にアクセスするためatomic）
	discovered atomic.Bool
	// verifier はJWT署名検証器。初回のExtractSubject呼び出し時に生成してキャッシュする。
	verifier *oidc.IDTokenVerifier
}

// NewClient creates an OIDC client for the BFF proxy.
func NewClient(discoveryURL, clientID, clientSecret, redirectURI string, scopes []string) *Client {
	return &Client{
		discoveryURL: discoveryURL,
		clientID:     clientID,
		clientSecret: clientSecret,
		redirectURI:  redirectURI,
		scopes:       scopes,
		httpClient:   &http.Client{Timeout: 10 * time.Second},
	}
}

// Discover fetches the OIDC discovery document and caches the endpoints.
// 複数goroutineから同時に呼ばれてもデータ競合が起きないようRWMutexで保護する。
func (c *Client) Discover(ctx context.Context) (*OIDCConfig, error) {
	// まず読み取りロックでキャッシュを確認する（高速パス）
	c.mu.RLock()
	if c.oidcConfig != nil {
		cfg := c.oidcConfig
		c.mu.RUnlock()
		return cfg, nil
	}
	c.mu.RUnlock()

	// キャッシュミス時は書き込みロックを取得してdiscoveryを実行する
	c.mu.Lock()
	defer c.mu.Unlock()

	// ダブルチェック: 他のgoroutineが先にdiscoveryを完了した可能性がある
	if c.oidcConfig != nil {
		return c.oidcConfig, nil
	}

	wellKnown := strings.TrimSuffix(c.discoveryURL, "/") + "/.well-known/openid-configuration"

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, wellKnown, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create discovery request: %w", err)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("OIDC discovery request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("OIDC discovery returned status %d", resp.StatusCode)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read discovery response: %w", err)
	}

	var cfg OIDCConfig
	if err := json.Unmarshal(body, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse discovery response: %w", err)
	}

	c.oidcConfig = &cfg
	// discoveryが正常に完了したことを記録する（atomic: goroutine安全）
	c.discovered.Store(true)
	return &cfg, nil
}

// IsDiscovered はOIDC discoveryが正常に完了しているかを返す。
// readinessチェックでプロバイダとの接続状態を確認するために使用する。
func (c *Client) IsDiscovered() bool {
	return c.discovered.Load()
}

// AuthCodeURL builds the authorization URL with PKCE parameters.
func (c *Client) AuthCodeURL(state, codeChallenge string) (string, error) {
	cfg, err := c.ensureDiscovered()
	if err != nil {
		return "", err
	}

	params := url.Values{
		"response_type":         {"code"},
		"client_id":             {c.clientID},
		"redirect_uri":          {c.redirectURI},
		"scope":                 {strings.Join(c.scopes, " ")},
		"state":                 {state},
		"code_challenge":        {codeChallenge},
		"code_challenge_method": {"S256"},
	}

	return cfg.AuthorizationEndpoint + "?" + params.Encode(), nil
}

// ExchangeCode exchanges an authorization code for tokens using PKCE.
func (c *Client) ExchangeCode(ctx context.Context, code, codeVerifier string) (*TokenResponse, error) {
	cfg, err := c.ensureDiscovered()
	if err != nil {
		return nil, err
	}

	data := url.Values{
		"grant_type":    {"authorization_code"},
		"code":          {code},
		"redirect_uri":  {c.redirectURI},
		"client_id":     {c.clientID},
		"code_verifier": {codeVerifier},
	}

	if c.clientSecret != "" {
		data.Set("client_secret", c.clientSecret)
	}

	return c.tokenRequest(ctx, cfg.TokenEndpoint, data)
}

// RefreshToken exchanges a refresh token for new tokens.
func (c *Client) RefreshToken(ctx context.Context, refreshToken string) (*TokenResponse, error) {
	cfg, err := c.ensureDiscovered()
	if err != nil {
		return nil, err
	}

	data := url.Values{
		"grant_type":    {"refresh_token"},
		"refresh_token": {refreshToken},
		"client_id":     {c.clientID},
	}

	if c.clientSecret != "" {
		data.Set("client_secret", c.clientSecret)
	}

	return c.tokenRequest(ctx, cfg.TokenEndpoint, data)
}

// LogoutURL returns the OIDC end-session endpoint URL.
func (c *Client) LogoutURL(idTokenHint, postLogoutRedirectURI string) (string, error) {
	cfg, err := c.ensureDiscovered()
	if err != nil {
		return "", err
	}

	if cfg.EndSessionEndpoint == "" {
		return "", fmt.Errorf("end_session_endpoint not available")
	}

	params := url.Values{}
	if idTokenHint != "" {
		params.Set("id_token_hint", idTokenHint)
	}
	if postLogoutRedirectURI != "" {
		params.Set("post_logout_redirect_uri", postLogoutRedirectURI)
	}

	return cfg.EndSessionEndpoint + "?" + params.Encode(), nil
}

func (c *Client) tokenRequest(ctx context.Context, endpoint string, data url.Values) (*TokenResponse, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, endpoint, strings.NewReader(data.Encode()))
	if err != nil {
		return nil, fmt.Errorf("failed to create token request: %w", err)
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("token request failed: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read token response: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("token endpoint returned status %d: %s", resp.StatusCode, string(body))
	}

	var tokenResp TokenResponse
	if err := json.Unmarshal(body, &tokenResp); err != nil {
		return nil, fmt.Errorf("failed to parse token response: %w", err)
	}

	return &tokenResp, nil
}

// ClearDiscoveryCache はキャッシュ済みの OIDC discovery 結果と verifier をクリアする。
// ログアウト時に呼び出し、次回ログインで最新のプロバイダ情報を再取得させる。
// 書き込みロックで保護し、読み取り中のgoroutineとのデータ競合を防止する。
func (c *Client) ClearDiscoveryCache() {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.oidcConfig = nil
	c.verifier = nil
	c.discovered.Store(false)
}

// ensureDiscovered はoidcConfigが存在することを読み取りロックで安全に確認する。
func (c *Client) ensureDiscovered() (*OIDCConfig, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	if c.oidcConfig != nil {
		return c.oidcConfig, nil
	}
	return nil, fmt.Errorf("OIDC discovery not performed; call Discover() first")
}

// ExtractSubject はIDトークンをJWKSで署名検証し、subクレームを返す。
// verifierはキャッシュし、JWKS公開鍵の再取得はライブラリ内部のキャッシュに委ねる。
func (c *Client) ExtractSubject(ctx context.Context, idToken string) (string, error) {
	v, err := c.ensureVerifier()
	if err != nil {
		return "", err
	}

	token, err := v.Verify(ctx, idToken)
	if err != nil {
		return "", fmt.Errorf("ID token signature verification failed: %w", err)
	}

	return token.Subject, nil
}

// ensureVerifier はverifierを初期化してキャッシュする。
// RemoteKeySetはcontext.Backgroundで生成し、JWKS鍵のHTTPフェッチを
// リクエストコンテキストのキャンセルから切り離す。
// 読み取りロックでキャッシュを確認し、未初期化の場合は書き込みロックで初期化する。
func (c *Client) ensureVerifier() (*oidc.IDTokenVerifier, error) {
	// まず読み取りロックでキャッシュを確認する（高速パス）
	c.mu.RLock()
	if c.verifier != nil {
		v := c.verifier
		c.mu.RUnlock()
		return v, nil
	}
	c.mu.RUnlock()

	// キャッシュミス時は書き込みロックを取得してverifierを初期化する
	c.mu.Lock()
	defer c.mu.Unlock()

	// ダブルチェック: 他のgoroutineが先に初期化を完了した可能性がある
	if c.verifier != nil {
		return c.verifier, nil
	}

	if c.oidcConfig == nil {
		return nil, fmt.Errorf("OIDC not discovered: OIDC discovery not performed; call Discover() first")
	}

	cfg := c.oidcConfig
	if cfg.JwksURI == "" {
		return nil, fmt.Errorf("jwks_uri not available in OIDC discovery document")
	}

	// JWKSエンドポイントから公開鍵を取得してトークン検証を行う
	keySet := oidc.NewRemoteKeySet(context.Background(), cfg.JwksURI)
	c.verifier = oidc.NewVerifier(cfg.Issuer, keySet, &oidc.Config{ClientID: c.clientID})
	return c.verifier, nil
}
