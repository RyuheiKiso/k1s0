package quotaclient

import (
	"context"
	"sync"
	"time"
)

type policyCacheEntry struct {
	policy    *QuotaPolicy
	expiresAt time.Time
}

// CachedQuotaClient はポリシーキャッシュ付きラッパー。
type CachedQuotaClient struct {
	inner     QuotaClient
	policyTTL time.Duration
	mu        sync.Mutex
	cache     map[string]policyCacheEntry
}

// NewCachedQuotaClient は新しい CachedQuotaClient を生成する。
func NewCachedQuotaClient(inner QuotaClient, policyTTL time.Duration) *CachedQuotaClient {
	return &CachedQuotaClient{
		inner:     inner,
		policyTTL: policyTTL,
		cache:     make(map[string]policyCacheEntry),
	}
}

func (c *CachedQuotaClient) Check(ctx context.Context, quotaID string, amount uint64) (*QuotaStatus, error) {
	return c.inner.Check(ctx, quotaID, amount)
}

func (c *CachedQuotaClient) Increment(ctx context.Context, quotaID string, amount uint64) (*QuotaUsage, error) {
	return c.inner.Increment(ctx, quotaID, amount)
}

func (c *CachedQuotaClient) GetUsage(ctx context.Context, quotaID string) (*QuotaUsage, error) {
	return c.inner.GetUsage(ctx, quotaID)
}

func (c *CachedQuotaClient) GetPolicy(ctx context.Context, quotaID string) (*QuotaPolicy, error) {
	c.mu.Lock()
	if entry, ok := c.cache[quotaID]; ok && time.Now().Before(entry.expiresAt) {
		c.mu.Unlock()
		return entry.policy, nil
	}
	c.mu.Unlock()

	policy, err := c.inner.GetPolicy(ctx, quotaID)
	if err != nil {
		return nil, err
	}

	c.mu.Lock()
	c.cache[quotaID] = policyCacheEntry{
		policy:    policy,
		expiresAt: time.Now().Add(c.policyTTL),
	}
	c.mu.Unlock()

	return policy, nil
}
