package tenantclient

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
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

// HttpTenantClient は HTTP REST 経由で tenant-server に接続するクライアント。
type HttpTenantClient struct {
	baseURL    string
	httpClient *http.Client
	config     TenantClientConfig
}

// NewHttpTenantClient は新しい HttpTenantClient を生成する。
// addr には "http://host:port" 形式のベース URL を指定する（例: "http://tenant-server:8080"）。
func NewHttpTenantClient(addr string, config TenantClientConfig) (*HttpTenantClient, error) {
	return &HttpTenantClient{
		baseURL:    addr,
		httpClient: &http.Client{},
		config:     config,
	}, nil
}

// NewHttpTenantClientWithHTTPClient はカスタム http.Client を使う HttpTenantClient を生成する（テスト用）。
func NewHttpTenantClientWithHTTPClient(addr string, config TenantClientConfig, httpClient *http.Client) (*HttpTenantClient, error) {
	return &HttpTenantClient{
		baseURL:    addr,
		httpClient: httpClient,
		config:     config,
	}, nil
}

// --- 内部 JSON 型 ---

type tenantAPIResponse struct {
	Tenant *tenantJSON `json:"tenant"`
}

type tenantsAPIResponse struct {
	Tenants []tenantJSON `json:"tenants"`
}

type tenantJSON struct {
	ID        string            `json:"id"`
	Name      string            `json:"name"`
	Status    string            `json:"status"`
	Plan      string            `json:"plan"`
	Settings  map[string]string `json:"settings,omitempty"`
	CreatedAt time.Time         `json:"created_at"`
}

type settingsAPIResponse struct {
	Settings *settingsValues `json:"settings"`
	// サーバーによっては settings ラッパーなしで直接 values を返すことがある
	Values map[string]string `json:"values,omitempty"`
}

type settingsValues struct {
	Values map[string]string `json:"values"`
}

type memberAPIResponse struct {
	Member *memberJSON `json:"member"`
}

type membersAPIResponse struct {
	Members []memberJSON `json:"members"`
}

type memberJSON struct {
	UserID   string    `json:"user_id"`
	Role     string    `json:"role"`
	JoinedAt time.Time `json:"joined_at"`
}

type provisioningStatusAPIResponse struct {
	Status string `json:"status"`
}

// --- 変換ヘルパー ---

func tenantFromJSON(t *tenantJSON) Tenant {
	return Tenant{
		ID:        t.ID,
		Name:      t.Name,
		Status:    TenantStatus(t.Status),
		Plan:      t.Plan,
		Settings:  t.Settings,
		CreatedAt: t.CreatedAt,
	}
}

func memberFromJSON(m *memberJSON) TenantMember {
	return TenantMember{
		UserID:   m.UserID,
		Role:     m.Role,
		JoinedAt: m.JoinedAt,
	}
}

// --- HTTP ヘルパー ---

func (c *HttpTenantClient) doRequest(ctx context.Context, method, path string, body interface{}) (*http.Response, error) {
	var reqBody io.Reader
	if body != nil {
		data, err := json.Marshal(body)
		if err != nil {
			return nil, fmt.Errorf("marshal request body: %w", err)
		}
		reqBody = bytes.NewReader(data)
	}

	req, err := http.NewRequestWithContext(ctx, method, c.baseURL+path, reqBody)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	return c.httpClient.Do(req)
}

// --- TenantClient 実装 ---

// GetTenant は指定したテナント ID のテナント情報を返す。
func (c *HttpTenantClient) GetTenant(ctx context.Context, tenantID string) (Tenant, error) {
	resp, err := c.doRequest(ctx, http.MethodGet, "/api/v1/tenants/"+tenantID, nil)
	if err != nil {
		return Tenant{}, fmt.Errorf("GetTenant request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return Tenant{}, fmt.Errorf("tenant not found: %s", tenantID)
	}
	if resp.StatusCode >= 300 {
		body, _ := io.ReadAll(resp.Body)
		return Tenant{}, fmt.Errorf("GetTenant server error (status %d): %s", resp.StatusCode, string(body))
	}

	var result tenantAPIResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return Tenant{}, fmt.Errorf("GetTenant decode error: %w", err)
	}
	if result.Tenant == nil {
		return Tenant{}, fmt.Errorf("GetTenant: empty tenant in response")
	}
	return tenantFromJSON(result.Tenant), nil
}

// ListTenants はフィルター条件に合うテナント一覧を返す。
func (c *HttpTenantClient) ListTenants(ctx context.Context, filter TenantFilter) ([]Tenant, error) {
	query := url.Values{}
	if filter.Status != nil {
		query.Set("status", string(*filter.Status))
	}
	if filter.Plan != nil {
		query.Set("plan", *filter.Plan)
	}

	path := "/api/v1/tenants"
	if len(query) > 0 {
		path += "?" + query.Encode()
	}

	resp, err := c.doRequest(ctx, http.MethodGet, path, nil)
	if err != nil {
		return nil, fmt.Errorf("ListTenants request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 300 {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("ListTenants server error (status %d): %s", resp.StatusCode, string(body))
	}

	var result tenantsAPIResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("ListTenants decode error: %w", err)
	}

	tenants := make([]Tenant, 0, len(result.Tenants))
	for i := range result.Tenants {
		tenants = append(tenants, tenantFromJSON(&result.Tenants[i]))
	}
	return tenants, nil
}

// IsActive は指定したテナントがアクティブかどうかを返す。
func (c *HttpTenantClient) IsActive(ctx context.Context, tenantID string) (bool, error) {
	t, err := c.GetTenant(ctx, tenantID)
	if err != nil {
		return false, err
	}
	return t.Status == TenantStatusActive, nil
}

// GetSettings は指定したテナントの設定値を返す。
func (c *HttpTenantClient) GetSettings(ctx context.Context, tenantID string) (TenantSettings, error) {
	resp, err := c.doRequest(ctx, http.MethodGet, "/api/v1/tenants/"+tenantID+"/settings", nil)
	if err != nil {
		return TenantSettings{}, fmt.Errorf("GetSettings request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return TenantSettings{}, fmt.Errorf("tenant not found: %s", tenantID)
	}
	if resp.StatusCode >= 300 {
		body, _ := io.ReadAll(resp.Body)
		return TenantSettings{}, fmt.Errorf("GetSettings server error (status %d): %s", resp.StatusCode, string(body))
	}

	var result settingsAPIResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return TenantSettings{}, fmt.Errorf("GetSettings decode error: %w", err)
	}

	// {"settings": {"values": {...}}} 形式
	if result.Settings != nil {
		return TenantSettings{Values: result.Settings.Values}, nil
	}
	// {"values": {...}} 形式（フラット）
	if result.Values != nil {
		return TenantSettings{Values: result.Values}, nil
	}
	return TenantSettings{Values: map[string]string{}}, nil
}

// CreateTenant は HTTP REST 経由でテナントを作成する。
func (c *HttpTenantClient) CreateTenant(ctx context.Context, req CreateTenantRequest) (Tenant, error) {
	resp, err := c.doRequest(ctx, http.MethodPost, "/api/v1/tenants", req)
	if err != nil {
		return Tenant{}, fmt.Errorf("CreateTenant request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 300 {
		body, _ := io.ReadAll(resp.Body)
		return Tenant{}, fmt.Errorf("CreateTenant server error (status %d): %s", resp.StatusCode, string(body))
	}

	var result tenantAPIResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return Tenant{}, fmt.Errorf("CreateTenant decode error: %w", err)
	}
	if result.Tenant == nil {
		return Tenant{}, fmt.Errorf("CreateTenant: empty tenant in response")
	}
	return tenantFromJSON(result.Tenant), nil
}

// AddMember は HTTP REST 経由でメンバーを追加する。
func (c *HttpTenantClient) AddMember(ctx context.Context, tenantID, userID, role string) (TenantMember, error) {
	reqBody := map[string]string{
		"user_id": userID,
		"role":    role,
	}
	resp, err := c.doRequest(ctx, http.MethodPost, "/api/v1/tenants/"+tenantID+"/members", reqBody)
	if err != nil {
		return TenantMember{}, fmt.Errorf("AddMember request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return TenantMember{}, fmt.Errorf("tenant not found: %s", tenantID)
	}
	if resp.StatusCode >= 300 {
		body, _ := io.ReadAll(resp.Body)
		return TenantMember{}, fmt.Errorf("AddMember server error (status %d): %s", resp.StatusCode, string(body))
	}

	var result memberAPIResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return TenantMember{}, fmt.Errorf("AddMember decode error: %w", err)
	}
	if result.Member == nil {
		return TenantMember{}, fmt.Errorf("AddMember: empty member in response")
	}
	return memberFromJSON(result.Member), nil
}

// RemoveMember は HTTP REST 経由でメンバーを削除する。
func (c *HttpTenantClient) RemoveMember(ctx context.Context, tenantID, userID string) error {
	resp, err := c.doRequest(ctx, http.MethodDelete, "/api/v1/tenants/"+tenantID+"/members/"+userID, nil)
	if err != nil {
		return fmt.Errorf("RemoveMember request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return fmt.Errorf("tenant or member not found: tenantID=%s userID=%s", tenantID, userID)
	}
	if resp.StatusCode >= 300 {
		body, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("RemoveMember server error (status %d): %s", resp.StatusCode, string(body))
	}
	return nil
}

// ListMembers は HTTP REST 経由でメンバー一覧を返す。
func (c *HttpTenantClient) ListMembers(ctx context.Context, tenantID string) ([]TenantMember, error) {
	resp, err := c.doRequest(ctx, http.MethodGet, "/api/v1/tenants/"+tenantID+"/members", nil)
	if err != nil {
		return nil, fmt.Errorf("ListMembers request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return nil, fmt.Errorf("tenant not found: %s", tenantID)
	}
	if resp.StatusCode >= 300 {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("ListMembers server error (status %d): %s", resp.StatusCode, string(body))
	}

	var result membersAPIResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("ListMembers decode error: %w", err)
	}

	members := make([]TenantMember, 0, len(result.Members))
	for i := range result.Members {
		members = append(members, memberFromJSON(&result.Members[i]))
	}
	return members, nil
}

// GetProvisioningStatus は HTTP REST 経由でプロビジョニング状態を返す。
// エンドポイントが 404 またはエラーの場合は ProvisioningStatusCompleted を返す。
func (c *HttpTenantClient) GetProvisioningStatus(ctx context.Context, tenantID string) (ProvisioningStatus, error) {
	resp, err := c.doRequest(ctx, http.MethodGet, "/api/v1/tenants/"+tenantID+"/provisioning-status", nil)
	if err != nil {
		// 接続エラー等は completed として扱う
		return ProvisioningStatusCompleted, nil
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return ProvisioningStatusCompleted, nil
	}
	if resp.StatusCode >= 300 {
		return ProvisioningStatusCompleted, nil
	}

	var result provisioningStatusAPIResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return ProvisioningStatusCompleted, nil
	}

	switch ProvisioningStatus(result.Status) {
	case ProvisioningStatusPending, ProvisioningStatusInProgress, ProvisioningStatusCompleted, ProvisioningStatusFailed:
		return ProvisioningStatus(result.Status), nil
	default:
		return ProvisioningStatusCompleted, nil
	}
}
