package vaultclient

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"sync"
	"time"
)

// Secret はシークレットの情報を表す。
type Secret struct {
	Path      string            `json:"path"`
	Data      map[string]string `json:"data"`
	Version   int64             `json:"version"`
	CreatedAt time.Time         `json:"created_at"`
}

// SecretRotatedEvent はシークレットローテーション通知。
type SecretRotatedEvent struct {
	Path    string `json:"path"`
	Version int64  `json:"version"`
}

// VaultClientConfig はクライアント設定。
type VaultClientConfig struct {
	ServerURL        string
	CacheTTL         time.Duration
	CacheMaxCapacity int
}

// VaultError はエラー種別を表す。
type VaultError struct {
	Code    string
	Message string
}

func (e *VaultError) Error() string {
	return fmt.Sprintf("%s: %s", e.Code, e.Message)
}

// NewNotFoundError は NotFound エラーを生成する。
func NewNotFoundError(path string) *VaultError {
	return &VaultError{Code: "NOT_FOUND", Message: path}
}

// NewPermissionDeniedError は PermissionDenied エラーを生成する。
func NewPermissionDeniedError(path string) *VaultError {
	return &VaultError{Code: "PERMISSION_DENIED", Message: path}
}

// VaultClient はシークレット操作のインターフェース。
type VaultClient interface {
	GetSecret(ctx context.Context, path string) (Secret, error)
	GetSecretValue(ctx context.Context, path, key string) (string, error)
	ListSecrets(ctx context.Context, pathPrefix string) ([]string, error)
	WatchSecret(ctx context.Context, path string) (<-chan SecretRotatedEvent, error)
}

// InMemoryVaultClient はメモリ内のシークレット管理クライアント。
type InMemoryVaultClient struct {
	mu     sync.RWMutex
	store  map[string]Secret
	config VaultClientConfig
}

// NewInMemoryVaultClient は新しい InMemoryVaultClient を生成する。
func NewInMemoryVaultClient(config VaultClientConfig) *InMemoryVaultClient {
	return &InMemoryVaultClient{
		store:  make(map[string]Secret),
		config: config,
	}
}

// PutSecret はシークレットを格納する。
func (c *InMemoryVaultClient) PutSecret(secret Secret) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.store[secret.Path] = secret
}

// GetSecret はシークレットを取得する。
func (c *InMemoryVaultClient) GetSecret(_ context.Context, path string) (Secret, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	s, ok := c.store[path]
	if !ok {
		return Secret{}, NewNotFoundError(path)
	}
	return s, nil
}

// GetSecretValue は指定キーの値を取得する。
func (c *InMemoryVaultClient) GetSecretValue(ctx context.Context, path, key string) (string, error) {
	s, err := c.GetSecret(ctx, path)
	if err != nil {
		return "", err
	}
	v, ok := s.Data[key]
	if !ok {
		return "", NewNotFoundError(fmt.Sprintf("%s/%s", path, key))
	}
	return v, nil
}

// ListSecrets はプレフィックスに一致するパス一覧を返す。
func (c *InMemoryVaultClient) ListSecrets(_ context.Context, pathPrefix string) ([]string, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	var paths []string
	for k := range c.store {
		if len(k) >= len(pathPrefix) && k[:len(pathPrefix)] == pathPrefix {
			paths = append(paths, k)
		}
	}
	return paths, nil
}

// WatchSecret はシークレットのローテーション通知チャンネルを返す。
func (c *InMemoryVaultClient) WatchSecret(_ context.Context, _ string) (<-chan SecretRotatedEvent, error) {
	ch := make(chan SecretRotatedEvent, 16)
	return ch, nil
}

// httpSecretResponse は vault-server のレスポンス形式。
type httpSecretResponse struct {
	Path      string            `json:"path"`
	Data      map[string]string `json:"data"`
	Version   int64             `json:"version"`
	CreatedAt time.Time         `json:"created_at"`
}

// httpCacheEntry はキャッシュエントリ。
type httpCacheEntry struct {
	secret    Secret
	fetchedAt time.Time
}

// HttpVaultClient は vault-server の REST API をHTTPで呼び出すクライアント。
type HttpVaultClient struct {
	config     VaultClientConfig
	httpClient *http.Client
	mu         sync.Mutex
	cache      map[string]httpCacheEntry
}

// NewHttpVaultClient は新しい HttpVaultClient を生成する。
func NewHttpVaultClient(config VaultClientConfig) *HttpVaultClient {
	return &HttpVaultClient{
		config:     config,
		httpClient: &http.Client{Timeout: 30 * time.Second},
		cache:      make(map[string]httpCacheEntry),
	}
}

// GetSecret はシークレットを取得する。キャッシュがあればそれを返す。
func (c *HttpVaultClient) GetSecret(ctx context.Context, path string) (Secret, error) {
	c.mu.Lock()
	if entry, ok := c.cache[path]; ok {
		ttl := c.config.CacheTTL
		if ttl == 0 {
			ttl = 600 * time.Second
		}
		if time.Since(entry.fetchedAt) < ttl {
			c.mu.Unlock()
			return entry.secret, nil
		}
	}
	c.mu.Unlock()

	reqURL := fmt.Sprintf("%s/api/v1/secrets/%s", c.config.ServerURL, path)
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, reqURL, nil)
	if err != nil {
		return Secret{}, &VaultError{Code: "SERVER_ERROR", Message: err.Error()}
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return Secret{}, &VaultError{Code: "SERVER_ERROR", Message: err.Error()}
	}
	defer resp.Body.Close()

	switch resp.StatusCode {
	case http.StatusOK:
		var body httpSecretResponse
		if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
			return Secret{}, &VaultError{Code: "SERVER_ERROR", Message: err.Error()}
		}
		secret := Secret{
			Path:      body.Path,
			Data:      body.Data,
			Version:   body.Version,
			CreatedAt: body.CreatedAt,
		}
		c.mu.Lock()
		c.cache[path] = httpCacheEntry{secret: secret, fetchedAt: time.Now()}
		c.mu.Unlock()
		return secret, nil
	case http.StatusNotFound:
		return Secret{}, NewNotFoundError(path)
	case http.StatusForbidden, http.StatusUnauthorized:
		return Secret{}, NewPermissionDeniedError(path)
	default:
		bodyBytes, _ := io.ReadAll(resp.Body)
		return Secret{}, &VaultError{Code: "SERVER_ERROR", Message: fmt.Sprintf("status %d: %s", resp.StatusCode, bodyBytes)}
	}
}

// GetSecretValue は指定キーの値を取得する。
func (c *HttpVaultClient) GetSecretValue(ctx context.Context, path, key string) (string, error) {
	s, err := c.GetSecret(ctx, path)
	if err != nil {
		return "", err
	}
	v, ok := s.Data[key]
	if !ok {
		return "", NewNotFoundError(fmt.Sprintf("%s/%s", path, key))
	}
	return v, nil
}

// ListSecrets はプレフィックスに一致するパス一覧を返す。
func (c *HttpVaultClient) ListSecrets(ctx context.Context, pathPrefix string) ([]string, error) {
	reqURL := fmt.Sprintf("%s/api/v1/secrets?prefix=%s", c.config.ServerURL, url.QueryEscape(pathPrefix))
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, reqURL, nil)
	if err != nil {
		return nil, &VaultError{Code: "SERVER_ERROR", Message: err.Error()}
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, &VaultError{Code: "SERVER_ERROR", Message: err.Error()}
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, &VaultError{Code: "SERVER_ERROR", Message: fmt.Sprintf("list_secrets failed: %d", resp.StatusCode)}
	}

	var paths []string
	if err := json.NewDecoder(resp.Body).Decode(&paths); err != nil {
		return nil, &VaultError{Code: "SERVER_ERROR", Message: err.Error()}
	}
	return paths, nil
}

// WatchSecret はシークレットのローテーション通知チャンネルを返す。
func (c *HttpVaultClient) WatchSecret(ctx context.Context, path string) (<-chan SecretRotatedEvent, error) {
	ch := make(chan SecretRotatedEvent, 16)
	ttl := c.config.CacheTTL
	if ttl == 0 {
		ttl = 600 * time.Second
	}

	go func() {
		defer close(ch)
		var lastVersion int64 = -1
		ticker := time.NewTicker(ttl)
		defer ticker.Stop()
		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				secret, err := c.GetSecret(ctx, path)
				if err != nil {
					continue
				}
				if lastVersion >= 0 && secret.Version != lastVersion {
					select {
					case ch <- SecretRotatedEvent{Path: path, Version: secret.Version}:
					case <-ctx.Done():
						return
					}
				}
				lastVersion = secret.Version
			}
		}
	}()

	return ch, nil
}
