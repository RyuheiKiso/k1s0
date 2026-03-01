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

// CreateTenantRequest はテナント作成リクエスト。
type CreateTenantRequest struct {
	Name        string `json:"name"`
	Plan        string `json:"plan"`
	AdminUserID string `json:"admin_user_id,omitempty"`
}

// TenantMember はテナントメンバー情報。
type TenantMember struct {
	UserID   string    `json:"user_id"`
	Role     string    `json:"role"`
	JoinedAt time.Time `json:"joined_at"`
}

// ProvisioningStatus はプロビジョニング状態。
type ProvisioningStatus string

const (
	ProvisioningStatusPending    ProvisioningStatus = "pending"
	ProvisioningStatusInProgress ProvisioningStatus = "in_progress"
	ProvisioningStatusCompleted  ProvisioningStatus = "completed"
	ProvisioningStatusFailed     ProvisioningStatus = "failed"
)

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
	CreateTenant(ctx context.Context, req CreateTenantRequest) (Tenant, error)
	AddMember(ctx context.Context, tenantID, userID, role string) (TenantMember, error)
	RemoveMember(ctx context.Context, tenantID, userID string) error
	ListMembers(ctx context.Context, tenantID string) ([]TenantMember, error)
	GetProvisioningStatus(ctx context.Context, tenantID string) (ProvisioningStatus, error)
}

// InMemoryTenantClient はメモリ内のテナントクライアント。
type InMemoryTenantClient struct {
	tenants      map[string]Tenant
	members      map[string][]TenantMember
	provisioning map[string]ProvisioningStatus
	mu           sync.RWMutex
}

// NewInMemoryTenantClient は新しい InMemoryTenantClient を生成する。
func NewInMemoryTenantClient() *InMemoryTenantClient {
	return &InMemoryTenantClient{
		tenants:      make(map[string]Tenant),
		members:      make(map[string][]TenantMember),
		provisioning: make(map[string]ProvisioningStatus),
	}
}

// NewInMemoryTenantClientWithTenants はテナント一覧付きで生成する。
func NewInMemoryTenantClientWithTenants(tenants []Tenant) *InMemoryTenantClient {
	m := make(map[string]Tenant, len(tenants))
	for _, t := range tenants {
		m[t.ID] = t
	}
	return &InMemoryTenantClient{
		tenants:      m,
		members:      make(map[string][]TenantMember),
		provisioning: make(map[string]ProvisioningStatus),
	}
}

// AddTenant はテナントを追加する。
func (c *InMemoryTenantClient) AddTenant(t Tenant) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.tenants[t.ID] = t
}

func (c *InMemoryTenantClient) GetTenant(_ context.Context, tenantID string) (Tenant, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	t, ok := c.tenants[tenantID]
	if !ok {
		return Tenant{}, fmt.Errorf("tenant not found: %s", tenantID)
	}
	return t, nil
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

func (c *InMemoryTenantClient) CreateTenant(_ context.Context, req CreateTenantRequest) (Tenant, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	id := fmt.Sprintf("tenant-%d", len(c.tenants)+1)
	tenant := Tenant{
		ID:        id,
		Name:      req.Name,
		Status:    TenantStatusActive,
		Plan:      req.Plan,
		Settings:  map[string]string{},
		CreatedAt: time.Now(),
	}
	c.tenants[id] = tenant
	c.provisioning[id] = ProvisioningStatusPending
	return tenant, nil
}

func (c *InMemoryTenantClient) AddMember(_ context.Context, tenantID, userID, role string) (TenantMember, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if _, ok := c.tenants[tenantID]; !ok {
		return TenantMember{}, fmt.Errorf("tenant not found: %s", tenantID)
	}
	member := TenantMember{UserID: userID, Role: role, JoinedAt: time.Now()}
	c.members[tenantID] = append(c.members[tenantID], member)
	return member, nil
}

func (c *InMemoryTenantClient) RemoveMember(_ context.Context, tenantID, userID string) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	members := c.members[tenantID]
	filtered := members[:0]
	for _, m := range members {
		if m.UserID != userID {
			filtered = append(filtered, m)
		}
	}
	c.members[tenantID] = filtered
	return nil
}

func (c *InMemoryTenantClient) ListMembers(_ context.Context, tenantID string) ([]TenantMember, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return append([]TenantMember{}, c.members[tenantID]...), nil
}

func (c *InMemoryTenantClient) GetProvisioningStatus(_ context.Context, tenantID string) (ProvisioningStatus, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	status, ok := c.provisioning[tenantID]
	if !ok {
		return "", fmt.Errorf("provisioning status not found for tenant: %s", tenantID)
	}
	return status, nil
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

// CreateTenant は gRPC 経由でテナントを作成する（未実装）。
func (c *GrpcTenantClient) CreateTenant(_ context.Context, _ CreateTenantRequest) (Tenant, error) {
	panic("not yet implemented")
}

// AddMember は gRPC 経由でメンバーを追加する（未実装）。
func (c *GrpcTenantClient) AddMember(_ context.Context, _, _, _ string) (TenantMember, error) {
	panic("not yet implemented")
}

// RemoveMember は gRPC 経由でメンバーを削除する（未実装）。
func (c *GrpcTenantClient) RemoveMember(_ context.Context, _, _ string) error {
	panic("not yet implemented")
}

// ListMembers は gRPC 経由でメンバー一覧を返す（未実装）。
func (c *GrpcTenantClient) ListMembers(_ context.Context, _ string) ([]TenantMember, error) {
	panic("not yet implemented")
}

// GetProvisioningStatus は gRPC 経由でプロビジョニング状態を返す（未実装）。
func (c *GrpcTenantClient) GetProvisioningStatus(_ context.Context, _ string) (ProvisioningStatus, error) {
	panic("not yet implemented")
}
