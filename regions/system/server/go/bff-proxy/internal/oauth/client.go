package oauth

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"
)

// OIDCConfig holds the OIDC provider endpoints discovered or configured.
type OIDCConfig struct {
	AuthorizationEndpoint string `json:"authorization_endpoint"`
	TokenEndpoint         string `json:"token_endpoint"`
	UserinfoEndpoint      string `json:"userinfo_endpoint"`
	EndSessionEndpoint    string `json:"end_session_endpoint"`
	Issuer                string `json:"issuer"`
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
	oidcConfig   *OIDCConfig
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
func (c *Client) Discover(ctx context.Context) (*OIDCConfig, error) {
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
	return &cfg, nil
}

// AuthCodeURL builds the authorization URL with PKCE parameters.
func (c *Client) AuthCodeURL(state, codeChallenge string) (string, error) {
	oidc, err := c.ensureDiscovered()
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

	return oidc.AuthorizationEndpoint + "?" + params.Encode(), nil
}

// ExchangeCode exchanges an authorization code for tokens using PKCE.
func (c *Client) ExchangeCode(ctx context.Context, code, codeVerifier string) (*TokenResponse, error) {
	oidc, err := c.ensureDiscovered()
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

	return c.tokenRequest(ctx, oidc.TokenEndpoint, data)
}

// RefreshToken exchanges a refresh token for new tokens.
func (c *Client) RefreshToken(ctx context.Context, refreshToken string) (*TokenResponse, error) {
	oidc, err := c.ensureDiscovered()
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

	return c.tokenRequest(ctx, oidc.TokenEndpoint, data)
}

// LogoutURL returns the OIDC end-session endpoint URL.
func (c *Client) LogoutURL(idTokenHint, postLogoutRedirectURI string) (string, error) {
	oidc, err := c.ensureDiscovered()
	if err != nil {
		return "", err
	}

	if oidc.EndSessionEndpoint == "" {
		return "", fmt.Errorf("end_session_endpoint not available")
	}

	params := url.Values{}
	if idTokenHint != "" {
		params.Set("id_token_hint", idTokenHint)
	}
	if postLogoutRedirectURI != "" {
		params.Set("post_logout_redirect_uri", postLogoutRedirectURI)
	}

	return oidc.EndSessionEndpoint + "?" + params.Encode(), nil
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

func (c *Client) ensureDiscovered() (*OIDCConfig, error) {
	if c.oidcConfig != nil {
		return c.oidcConfig, nil
	}
	return nil, fmt.Errorf("OIDC discovery not performed; call Discover() first")
}
