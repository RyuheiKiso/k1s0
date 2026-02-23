package quotaclient

import (
	"context"
	"sync"
	"time"
)

// QuotaClient はクォータ操作のインターフェース。
type QuotaClient interface {
	Check(ctx context.Context, quotaID string, amount uint64) (*QuotaStatus, error)
	Increment(ctx context.Context, quotaID string, amount uint64) (*QuotaUsage, error)
	GetUsage(ctx context.Context, quotaID string) (*QuotaUsage, error)
	GetPolicy(ctx context.Context, quotaID string) (*QuotaPolicy, error)
}

// InMemoryQuotaClient はテスト用のインメモリ実装。
type InMemoryQuotaClient struct {
	mu       sync.Mutex
	usages   map[string]*QuotaUsage
	policies map[string]*QuotaPolicy
}

// NewInMemoryQuotaClient は新しい InMemoryQuotaClient を生成する。
func NewInMemoryQuotaClient() *InMemoryQuotaClient {
	return &InMemoryQuotaClient{
		usages:   make(map[string]*QuotaUsage),
		policies: make(map[string]*QuotaPolicy),
	}
}

// SetPolicy はテスト用ポリシーを登録する。
func (c *InMemoryQuotaClient) SetPolicy(quotaID string, policy *QuotaPolicy) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.policies[quotaID] = policy
}

func (c *InMemoryQuotaClient) getOrCreateUsage(quotaID string) *QuotaUsage {
	if u, ok := c.usages[quotaID]; ok {
		return u
	}
	p, ok := c.policies[quotaID]
	limit := uint64(1000)
	period := PeriodDaily
	if ok {
		limit = p.Limit
		period = p.Period
	}
	u := &QuotaUsage{
		QuotaID: quotaID,
		Used:    0,
		Limit:   limit,
		Period:  period,
		ResetAt: time.Now().Add(24 * time.Hour),
	}
	c.usages[quotaID] = u
	return u
}

// Check はクォータの残量を確認する。
func (c *InMemoryQuotaClient) Check(_ context.Context, quotaID string, amount uint64) (*QuotaStatus, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	usage := c.getOrCreateUsage(quotaID)
	remaining := usage.Limit - usage.Used
	allowed := amount <= remaining

	return &QuotaStatus{
		Allowed:   allowed,
		Remaining: remaining,
		Limit:     usage.Limit,
		ResetAt:   usage.ResetAt,
	}, nil
}

// Increment はクォータ使用量を加算する。
func (c *InMemoryQuotaClient) Increment(_ context.Context, quotaID string, amount uint64) (*QuotaUsage, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	usage := c.getOrCreateUsage(quotaID)
	usage.Used += amount

	result := *usage
	return &result, nil
}

// GetUsage はクォータ使用量を取得する。
func (c *InMemoryQuotaClient) GetUsage(_ context.Context, quotaID string) (*QuotaUsage, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	usage := c.getOrCreateUsage(quotaID)
	result := *usage
	return &result, nil
}

// GetPolicy はクォータポリシーを取得する。
func (c *InMemoryQuotaClient) GetPolicy(_ context.Context, quotaID string) (*QuotaPolicy, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if p, ok := c.policies[quotaID]; ok {
		result := *p
		return &result, nil
	}

	return &QuotaPolicy{
		QuotaID:       quotaID,
		Limit:         1000,
		Period:        PeriodDaily,
		ResetStrategy: "fixed",
	}, nil
}
