package vaultclient

import (
	"context"
	"fmt"
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
