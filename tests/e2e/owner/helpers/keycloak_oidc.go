// tests/e2e/owner/helpers/keycloak_oidc.go
//
// Keycloak OIDC handshake helper。tier3-web / smoke / examples 等で
// 認証済 access token を取得する経路。owner suite の tier3-web 検証 +
// SDK round-trip で必要。
//
// 設計正典:
//   ADR-SEC-001（Keycloak 採用）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/02_ディレクトリ構造.md
package helpers

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

// KeycloakClient は Keycloak OIDC token endpoint への薄い client
type KeycloakClient struct {
	// BaseURL は Keycloak の base URL（例: https://keycloak.localhost）
	BaseURL string
	// Realm は対象 realm（k1s0 owner suite では k1s0 を想定）
	Realm string
	// HTTPClient は token endpoint への request 用
	HTTPClient *http.Client
}

// NewKeycloakClient は base URL + realm から client を生成
func NewKeycloakClient(baseURL, realm string) *KeycloakClient {
	return &KeycloakClient{
		BaseURL:    baseURL,
		Realm:      realm,
		HTTPClient: &http.Client{Timeout: 30 * time.Second},
	}
}

// tokenResponse は token endpoint の response 構造
type tokenResponse struct {
	AccessToken  string `json:"access_token"`
	RefreshToken string `json:"refresh_token"`
	TokenType    string `json:"token_type"`
	ExpiresIn    int    `json:"expires_in"`
}

// PasswordGrantToken は Keycloak password grant flow で access token を取得する。
// owner suite の test-only 用途で、production では使わない（admin password 直接送信のため）。
func (c *KeycloakClient) PasswordGrantToken(
	ctx context.Context,
	clientID, username, password string,
) (string, error) {
	// token endpoint の URL を組み立て
	tokenURL := fmt.Sprintf("%s/realms/%s/protocol/openid-connect/token", c.BaseURL, c.Realm)
	// form-encoded body（password grant の標準形式）
	form := url.Values{}
	form.Set("grant_type", "password")
	form.Set("client_id", clientID)
	form.Set("username", username)
	form.Set("password", password)
	// HTTP POST request 構築
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, tokenURL, strings.NewReader(form.Encode()))
	if err != nil {
		return "", fmt.Errorf("Keycloak request 構築失敗: %w", err)
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	// 実行
	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return "", fmt.Errorf("Keycloak HTTP 失敗: %w", err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return "", fmt.Errorf("Keycloak unexpected status: %d, body: %s", resp.StatusCode, string(body))
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}
	var tr tokenResponse
	if err := json.Unmarshal(body, &tr); err != nil {
		return "", fmt.Errorf("Keycloak response decode 失敗: %w", err)
	}
	return tr.AccessToken, nil
}
