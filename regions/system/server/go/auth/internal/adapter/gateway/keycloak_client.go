package gateway

import (
	"context"
	"errors"
	"net/http"
	"sync"
	"time"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/domain/repository"
	"github.com/k1s0-platform/system-server-go-auth/internal/infra/config"
)

// ErrUserNotFound はユーザーが見つからない場合のエラー。
var ErrUserNotFound = errors.New("user not found")

// KeycloakClient は Keycloak Admin API と通信するゲートウェイ。
// UserRepository インターフェースを実装する。
type KeycloakClient struct {
	baseURL      string
	realm        string
	clientID     string
	clientSecret string
	httpClient   *http.Client

	mu          sync.RWMutex
	adminToken  string
	tokenExpiry time.Time
}

// NewKeycloakClient は新しい KeycloakClient を作成する。
func NewKeycloakClient(cfg config.OIDCConfig) *KeycloakClient {
	return &KeycloakClient{
		baseURL:      extractBaseURL(cfg.DiscoveryURL),
		realm:        "k1s0",
		clientID:     cfg.ClientID,
		clientSecret: cfg.ClientSecret,
		httpClient: &http.Client{
			Timeout: 10 * time.Second,
		},
	}
}

// GetUser は Keycloak Admin API からユーザー情報を取得する。
func (c *KeycloakClient) GetUser(ctx context.Context, userID string) (*model.User, error) {
	// 本番実装:
	// 1. Admin Token を取得
	// 2. GET /admin/realms/{realm}/users/{id}
	// 3. レスポンスを model.User にマッピング
	return nil, ErrUserNotFound
}

// ListUsers は Keycloak Admin API からユーザー一覧を取得する。
func (c *KeycloakClient) ListUsers(ctx context.Context, params repository.UserListParams) ([]*model.User, int, error) {
	// 本番実装:
	// 1. Admin Token を取得
	// 2. GET /admin/realms/{realm}/users?first=...&max=...&search=...
	// 3. レスポンスを []*model.User にマッピング
	return []*model.User{}, 0, nil
}

// GetUserRoles は Keycloak Admin API からユーザーのロール一覧を取得する。
func (c *KeycloakClient) GetUserRoles(ctx context.Context, userID string) ([]*model.Role, map[string][]*model.Role, error) {
	// 本番実装:
	// 1. Admin Token を取得
	// 2. GET /admin/realms/{realm}/users/{id}/role-mappings
	// 3. レスポンスを []*model.Role にマッピング
	return []*model.Role{}, map[string][]*model.Role{}, nil
}

// Healthy は Keycloak への接続を確認する。
func (c *KeycloakClient) Healthy(ctx context.Context) error {
	// 本番実装:
	// GET /realms/{realm} をリクエストして接続確認
	return nil
}

// extractBaseURL は Discovery URL からベース URL を抽出する。
func extractBaseURL(discoveryURL string) string {
	// https://auth.example.com/realms/k1s0/.well-known/openid-configuration
	// -> https://auth.example.com
	if len(discoveryURL) == 0 {
		return ""
	}
	// 簡易実装: /realms/ の前までを取得
	for i := 0; i < len(discoveryURL)-7; i++ {
		if discoveryURL[i:i+8] == "/realms/" {
			return discoveryURL[:i]
		}
	}
	return discoveryURL
}
