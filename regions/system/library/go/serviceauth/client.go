package serviceauth

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"sync"
	"time"
)

// ServiceAuthConfig は serviceauth クライアントの設定。
type ServiceAuthConfig struct {
	// TokenURL は OAuth2 トークンエンドポイント URL。
	TokenURL string
	// ClientID はクライアント ID。
	ClientID string
	// ClientSecret はクライアントシークレット。
	ClientSecret string
	// Audience はトークンの対象オーディエンス（省略可能）。
	Audience string
}

// httpServiceAuthClient は net/http を使った ServiceAuthClient 実装。
type httpServiceAuthClient struct {
	config     *ServiceAuthConfig
	httpClient *http.Client
	mu         sync.Mutex
	cached     *ServiceToken
}

// NewClient は新しい ServiceAuthClient を生成する。
func NewClient(config *ServiceAuthConfig) ServiceAuthClient {
	return &httpServiceAuthClient{
		config:     config,
		httpClient: &http.Client{},
	}
}

// NewClientWithHTTPClient はカスタム http.Client を使う ServiceAuthClient を生成する（テスト用）。
func NewClientWithHTTPClient(config *ServiceAuthConfig, httpClient *http.Client) ServiceAuthClient {
	return &httpServiceAuthClient{
		config:     config,
		httpClient: httpClient,
	}
}

// GetToken は OAuth2 Client Credentials フローで新しいトークンを取得する。
func (c *httpServiceAuthClient) GetToken(ctx context.Context) (*ServiceToken, error) {
	data := url.Values{}
	data.Set("grant_type", "client_credentials")
	data.Set("client_id", c.config.ClientID)
	data.Set("client_secret", c.config.ClientSecret)
	if c.config.Audience != "" {
		data.Set("audience", c.config.Audience)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.config.TokenURL,
		strings.NewReader(data.Encode()))
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("execute request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("token request failed (status %d): %s", resp.StatusCode, string(body))
	}

	var tokenResp struct {
		AccessToken string `json:"access_token"`
		TokenType   string `json:"token_type"`
		ExpiresIn   int    `json:"expires_in"`
		Scope       string `json:"scope"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&tokenResp); err != nil {
		return nil, fmt.Errorf("decode token response: %w", err)
	}

	return &ServiceToken{
		AccessToken: tokenResp.AccessToken,
		TokenType:   tokenResp.TokenType,
		ExpiresAt:   time.Now().Add(time.Duration(tokenResp.ExpiresIn) * time.Second),
		Scope:       tokenResp.Scope,
	}, nil
}

// GetCachedToken はキャッシュからトークンを返す（期限切れなら再取得）。
// sync.Mutex で保護し、double-check locking を使用する。
func (c *httpServiceAuthClient) GetCachedToken(ctx context.Context) (string, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	// キャッシュが有効なら返す
	if c.cached != nil && !c.cached.ShouldRefresh() {
		return c.cached.BearerHeader(), nil
	}

	// 新しいトークンを取得
	token, err := c.GetToken(ctx)
	if err != nil {
		return "", err
	}
	c.cached = token
	return token.BearerHeader(), nil
}

// ValidateSpiffeId は SPIFFE ID を検証し、期待ネームスペースと一致するか確認する。
func (c *httpServiceAuthClient) ValidateSpiffeId(spiffeId string, expectedNamespace string) (*SpiffeId, error) {
	parsed, err := ParseSpiffeId(spiffeId)
	if err != nil {
		return nil, err
	}
	if parsed.Namespace != expectedNamespace {
		return nil, fmt.Errorf("SPIFFE ID namespace mismatch: expected %s, got %s", expectedNamespace, parsed.Namespace)
	}
	return parsed, nil
}
