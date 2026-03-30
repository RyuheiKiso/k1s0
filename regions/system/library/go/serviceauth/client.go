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

	// singleflight はトークン更新の重複リクエストを1つに集約し、スタンピード問題を防止する
	"golang.org/x/sync/singleflight"
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
	// sfGroup はトークン更新リクエストを singleflight で集約し、同時多発的なトークン更新（スタンピード）を防止する
	sfGroup singleflight.Group
	// mu はキャッシュへの読み書きを保護する RWMutex（読み取り専用パスのロック競合を最小化）
	mu     sync.RWMutex
	cached *ServiceToken
}

// NewClient は新しい ServiceAuthClient を生成する。
func NewClient(config *ServiceAuthConfig) ServiceAuthClient {
	return &httpServiceAuthClient{
		config:     config,
		// 無限待ちを防止するため30秒のタイムアウトを設定
		httpClient: &http.Client{Timeout: 30 * time.Second},
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

// GetCachedToken はキャッシュされたトークンを返す。
// キャッシュが有効な場合は RLock のみで高速に返す（高速パス）。
// キャッシュが無効な場合は singleflight で同時多発的なトークン更新を1リクエストに集約し、
// ロック外でネットワーク I/O を実行することでミューテックス保持中の HTTP ブロッキングを排除する。
func (c *httpServiceAuthClient) GetCachedToken(ctx context.Context) (string, error) {
	// まず読み取りロックでキャッシュを確認（高速パス）
	c.mu.RLock()
	if c.cached != nil && !c.cached.ShouldRefresh() {
		token := c.cached.BearerHeader()
		c.mu.RUnlock()
		return token, nil
	}
	c.mu.RUnlock()

	// singleflight で同一キーの同時リクエストを1つに集約する。
	// これにより、トークン期限切れ時に多数のゴルーチンが同時にトークン更新を試みる
	// 「キャッシュスタンピード」を防止し、認証サーバーへの負荷を抑制する。
	v, err, _ := c.sfGroup.Do("token", func() (interface{}, error) {
		// ダブルチェック: singleflight の待ち行列を抜けた際に、
		// 先行ゴルーチンがすでにキャッシュを更新している可能性があるため再確認する
		c.mu.RLock()
		if c.cached != nil && !c.cached.ShouldRefresh() {
			token := c.cached.BearerHeader()
			c.mu.RUnlock()
			return token, nil
		}
		c.mu.RUnlock()

		// ミューテックスを保持せずにネットワーク I/O を実行する（ロック外 HTTP）。
		// これにより他のゴルーチンがキャッシュ読み取りをブロックされない。
		token, err := c.GetToken(ctx)
		if err != nil {
			return "", err
		}

		// 書き込みロックを取得してキャッシュを更新する
		c.mu.Lock()
		c.cached = token
		c.mu.Unlock()

		return token.BearerHeader(), nil
	})
	if err != nil {
		return "", err
	}
	return v.(string), nil
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
