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
	// ctx はアプリケーションレベルのコンテキスト。シャットダウン時にJWKSフェッチを含む
	// バックグラウンド操作がキャンセルされるようにするために保持する。
	ctx context.Context
	// mu はoidcConfigとverifierへの並行アクセスを保護するRWMutex。
	// Discover/ClearDiscoveryCacheは書き込みロック、読み取り系メソッドは読み取りロックを使用する。
	mu         sync.RWMutex
	oidcConfig *OIDCConfig
	// discovered はOIDC discoveryが正常に完了したかを示すフラグ（goroutine間で安全にアクセスするためatomic）
	discovered atomic.Bool
	// verifier はJWT署名検証器。初回のExtractSubject呼び出し時に生成してキャッシュする。
	verifier *oidc.IDTokenVerifier
}

// ClientOption は NewClient に渡せるオプション関数型。
type ClientOption func(*Client)

// WithHTTPTimeout は OAuth HTTP クライアントのタイムアウトを設定するオプション。
// デフォルト値は 10 秒。テストや低レイテンシ環境での上書きに使用する。
func WithHTTPTimeout(d time.Duration) ClientOption {
	return func(c *Client) {
		c.httpClient = &http.Client{Timeout: d}
	}
}

// NewClient creates an OIDC client for the BFF proxy.
// ctx はアプリケーションレベルのコンテキストで、シャットダウン時に JWKS フェッチを
// キャンセルできるようにするために必要。オプション関数を渡すことで HTTP タイムアウト等の
// 動作をカスタマイズできる。
func NewClient(ctx context.Context, discoveryURL, clientID, clientSecret, redirectURI string, scopes []string, opts ...ClientOption) *Client {
	c := &Client{
		ctx:          ctx,
		discoveryURL: discoveryURL,
		clientID:     clientID,
		clientSecret: clientSecret,
		redirectURI:  redirectURI,
		scopes:       scopes,
		httpClient:   &http.Client{Timeout: 10 * time.Second},
	}
	for _, opt := range opts {
		opt(c)
	}
	return c
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
		return nil, fmt.Errorf("discovery リクエストの作成に失敗しました: %w", err)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("OIDC discovery リクエストが失敗しました: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("OIDC discovery がステータス %d を返しました", resp.StatusCode)
	}

	// 1MB でボディサイズを制限（OOM 攻撃対策）（H-9 監査対応）
	body, err := io.ReadAll(io.LimitReader(resp.Body, 1<<20))
	if err != nil {
		return nil, fmt.Errorf("discovery レスポンスの読み込みに失敗しました: %w", err)
	}

	var cfg OIDCConfig
	if err := json.Unmarshal(body, &cfg); err != nil {
		return nil, fmt.Errorf("discovery レスポンスの解析に失敗しました: %w", err)
	}

	// M-3 対応: OIDC Discovery レスポンスの必須フィールドが欠落していないか検証する
	// 空フィールドのまま処理を続行すると後続の認証フローで不正な動作が起こる
	if cfg.Issuer == "" {
		return nil, fmt.Errorf("OIDC discovery レスポンスに issuer が含まれていません")
	}
	if cfg.AuthorizationEndpoint == "" {
		return nil, fmt.Errorf("OIDC discovery レスポンスに authorization_endpoint が含まれていません")
	}
	if cfg.TokenEndpoint == "" {
		return nil, fmt.Errorf("OIDC discovery レスポンスに token_endpoint が含まれていません")
	}
	if cfg.JwksURI == "" {
		return nil, fmt.Errorf("OIDC discovery レスポンスに jwks_uri が含まれていません")
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
		return nil, fmt.Errorf("トークンリクエストの作成に失敗しました: %w", err)
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("トークンリクエストが失敗しました: %w", err)
	}
	defer resp.Body.Close()

	// 1MB でボディサイズを制限（OOM 攻撃対策）（H-9 監査対応）
	body, err := io.ReadAll(io.LimitReader(resp.Body, 1<<20))
	if err != nil {
		return nil, fmt.Errorf("トークンレスポンスの読み込みに失敗しました: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		// HIGH-GO-003 監査対応: トークンエラーレスポンスボディにはクライアントシークレットや
		// エラー詳細が含まれる可能性があるため、ログやエラーメッセージからボディを除外する。
		// ステータスコードのみを返し、詳細はサーバーログで追跡可能にする。
		return nil, fmt.Errorf("トークンエンドポイントがステータス %d を返しました", resp.StatusCode)
	}

	var tokenResp TokenResponse
	if err := json.Unmarshal(body, &tokenResp); err != nil {
		return nil, fmt.Errorf("トークンレスポンスの解析に失敗しました: %w", err)
	}

	// M-3 対応: トークンレスポンスに access_token が含まれているか検証する
	// access_token が空の場合は認証フローが正常に完了していないと判断してエラーを返す
	if tokenResp.AccessToken == "" {
		return nil, fmt.Errorf("トークンレスポンスに access_token が含まれていません")
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
// ロール取得も必要な場合は ExtractClaims を使用すること。
func (c *Client) ExtractSubject(ctx context.Context, idToken string) (string, error) {
	subject, _, err := c.ExtractClaims(ctx, idToken)
	return subject, err
}

// keycloakIDTokenClaims は Keycloak ID トークンから subject と roles と tenant_id を取得するための claims 構造体。
// JWKS 署名検証済みのトークンに対して使用する。
type keycloakIDTokenClaims struct {
	// RealmAccess は Keycloak realm レベルのロール情報を保持する。
	RealmAccess struct {
		Roles []string `json:"roles"`
	} `json:"realm_access"`
	// TenantID は Keycloak カスタムクレームとして設定されるテナント識別子（MEDIUM-GO-001 監査対応）。
	// マルチテナント分離のため、セッション作成時にこの値をセッションデータに格納し、
	// 上流 API への X-Tenant-ID ヘッダー転送に使用する。
	TenantID string `json:"tenant_id"`
}

// IDTokenClaims は ExtractClaims が返す署名検証済みの ID トークンクレームを保持する。
// MEDIUM-GO-001 監査対応: TenantID フィールドを追加してテナント分離を実現する。
type IDTokenClaims struct {
	// Subject は OIDC sub クレーム（ユーザー識別子）。
	Subject string
	// Roles は Keycloak realm roles。
	Roles []string
	// TenantID は Keycloak カスタムクレームのテナント識別子。
	TenantID string
}

// ExtractClaims は ID トークンを JWKS で署名検証し、subject と realm roles と tenant_id を返す。
// アクセストークンの署名未検証によるロール改ざんリスクを排除するため、
// 必ず JWKS 検証済みの ID トークンから roles を取得する。
// 解析に失敗した場合は空スライスを返す（roles 取得失敗でログインを妨げない）。
func (c *Client) ExtractClaims(ctx context.Context, idToken string) (subject string, roles []string, err error) {
	claims, extractErr := c.ExtractFullClaims(ctx, idToken)
	if extractErr != nil {
		return "", []string{}, extractErr
	}
	return claims.Subject, claims.Roles, nil
}

// ExtractFullClaims は ID トークンを JWKS で署名検証し、IDTokenClaims 全体を返す。
// MEDIUM-GO-001 監査対応: tenant_id クレームを含む全クレームを返すことで
// テナント分離に必要な情報を一括取得できるようにする。
func (c *Client) ExtractFullClaims(ctx context.Context, idToken string) (*IDTokenClaims, error) {
	v, err := c.ensureVerifier()
	if err != nil {
		return nil, err
	}

	// JWKS による署名検証を行う
	token, err := v.Verify(ctx, idToken)
	if err != nil {
		return nil, fmt.Errorf("ID トークンの署名検証に失敗しました: %w", err)
	}

	// 署名検証済みトークンの claims から roles と tenant_id を取得する
	var rawClaims keycloakIDTokenClaims
	if claimsErr := token.Claims(&rawClaims); claimsErr != nil {
		// claims の取得に失敗しても subject は返す（roles/tenant_id は空）
		return &IDTokenClaims{Subject: token.Subject, Roles: []string{}}, nil
	}

	roles := rawClaims.RealmAccess.Roles
	if roles == nil {
		roles = []string{}
	}
	return &IDTokenClaims{
		Subject:  token.Subject,
		Roles:    roles,
		TenantID: rawClaims.TenantID,
	}, nil
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
		return nil, fmt.Errorf("OIDC discovery が未完了です。Discover() を先に呼び出してください")
	}

	cfg := c.oidcConfig
	if cfg.JwksURI == "" {
		return nil, fmt.Errorf("OIDC discovery ドキュメントに jwks_uri が含まれていません")
	}

	// JWKSエンドポイントから公開鍵を取得してトークン検証を行う。
	// c.ctx（アプリケーションレベルのコンテキスト）を使用することで、
	// シャットダウン時に JWKS のバックグラウンドフェッチがキャンセルされる。
	keySet := oidc.NewRemoteKeySet(c.ctx, cfg.JwksURI)
	// H-6 対応: go-oidc の IDTokenVerifier が iss（Issuer）と aud（Audience = ClientID）を自動検証している。
	// oidc.Config{ClientID: c.clientID} を渡すことで、Verify() 呼び出し時に
	// トークンの iss が cfg.Issuer と一致すること、かつ aud に c.clientID が含まれることを強制する。
	// これにより他の IdP や他クライアント向けのトークンを誤受け入れするリスクを排除する。
	c.verifier = oidc.NewVerifier(cfg.Issuer, keySet, &oidc.Config{ClientID: c.clientID})
	return c.verifier, nil
}
