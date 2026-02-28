package tenantclient

import (
	"context"
	"fmt"
	"sync"
	"time"
)

// TenantStatus はテナントのステータス。
type TenantStatus string

const (
	TenantStatusActive    TenantStatus = "active"
	TenantStatusSuspended TenantStatus = "suspended"
	TenantStatusDeleted   TenantStatus = "deleted"
)

// Tenant はテナント情報。
type Tenant struct {
	ID        string            `json:"id"`
	Name      string            `json:"name"`
	Status    TenantStatus      `json:"status"`
	Plan      string            `json:"plan"`
	Settings  map[string]string `json:"settings"`
	CreatedAt time.Time         `json:"created_at"`
}

// TenantFilter はテナント一覧取得フィルター。
type TenantFilter struct {
	Status *TenantStatus
	Plan   *string
}

// TenantSettings はテナント設定値ラッパー。
type TenantSettings struct {
	Values map[string]string
}

// Get は指定キーの設定値を返す。
func (s TenantSettings) Get(key string) (string, bool) {
	v, ok := s.Values[key]
	return v, ok
}

// TenantClientConfig はクライアント設定。
type TenantClientConfig struct {
	ServerURL        string
	CacheTTL         time.Duration
	CacheMaxCapacity int
}

// TenantClient はテナント操作インターフェース。
type TenantClient interface {
	GetTenant(ctx context.Context, tenantID string) (Tenant, error)
	ListTenants(ctx context.Context, filter TenantFilter) ([]Tenant, error)
	IsActive(ctx context.Context, tenantID string) (bool, error)
	GetSettings(ctx context.Context, tenantID string) (TenantSettings, error)
}

// InMemoryTenantClient はメモリ内のテナントクライアント。
type InMemoryTenantClient struct {
	mu      sync.RWMutex
	tenants []Tenant
}

// NewInMemoryTenantClient は新しい InMemoryTenantClient を生成する。
func NewInMemoryTenantClient() *InMemoryTenantClient {
	return &InMemoryTenantClient{}
}

// NewInMemoryTenantClientWithTenants はテナント一覧付きで生成する。
func NewInMemoryTenantClientWithTenants(tenants []Tenant) *InMemoryTenantClient {
	return &InMemoryTenantClient{tenants: tenants}
}

// AddTenant はテナントを追加する。
func (c *InMemoryTenantClient) AddTenant(t Tenant) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.tenants = append(c.tenants, t)
}

func (c *InMemoryTenantClient) GetTenant(_ context.Context, tenantID string) (Tenant, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	for _, t := range c.tenants {
		if t.ID == tenantID {
			return t, nil
		}
	}
	return Tenant{}, fmt.Errorf("tenant not found: %s", tenantID)
}

func (c *InMemoryTenantClient) ListTenants(_ context.Context, filter TenantFilter) ([]Tenant, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	var result []Tenant
	for _, t := range c.tenants {
		if filter.Status != nil && t.Status != *filter.Status {
			continue
		}
		if filter.Plan != nil && t.Plan != *filter.Plan {
			continue
		}
		result = append(result, t)
	}
	return result, nil
}

func (c *InMemoryTenantClient) IsActive(ctx context.Context, tenantID string) (bool, error) {
	t, err := c.GetTenant(ctx, tenantID)
	if err != nil {
		return false, err
	}
	return t.Status == TenantStatusActive, nil
}

func (c *InMemoryTenantClient) GetSettings(ctx context.Context, tenantID string) (TenantSettings, error) {
	t, err := c.GetTenant(ctx, tenantID)
	if err != nil {
		return TenantSettings{}, err
	}
	return TenantSettings{Values: t.Settings}, nil
}

// GrpcTenantClient は gRPC 経由で tenant-server に接続するクライアント。
type GrpcTenantClient struct {
	serverAddr string
	config     TenantClientConfig
}

// NewGrpcTenantClient は新しい GrpcTenantClient を生成する。
// addr には "host:port" 形式のサーバーアドレスを指定する（例: "tenant-server:8080"）。
func NewGrpcTenantClient(addr string, config TenantClientConfig) (*GrpcTenantClient, error) {
	return &GrpcTenantClient{
		serverAddr: addr,
		config:     config,
	}, nil
}

// GetTenant は指定したテナント ID のテナント情報を返す。
func (c *GrpcTenantClient) GetTenant(_ context.Context, tenantID string) (Tenant, error) {
	return Tenant{}, fmt.Errorf("gRPC client not yet connected: GetTenant(%s)", tenantID)
}

// ListTenants はフィルター条件に合うテナント一覧を返す。
func (c *GrpcTenantClient) ListTenants(_ context.Context, _ TenantFilter) ([]Tenant, error) {
	return nil, fmt.Errorf("gRPC client not yet connected: ListTenants")
}

// IsActive は指定したテナントがアクティブかどうかを返す。
func (c *GrpcTenantClient) IsActive(_ context.Context, tenantID string) (bool, error) {
	return false, fmt.Errorf("gRPC client not yet connected: IsActive(%s)", tenantID)
}

// GetSettings は指定したテナントの設定値を返す。
func (c *GrpcTenantClient) GetSettings(_ context.Context, tenantID string) (TenantSettings, error) {
	return TenantSettings{}, fmt.Errorf("gRPC client not yet connected: GetSettings(%s)", tenantID)
}
