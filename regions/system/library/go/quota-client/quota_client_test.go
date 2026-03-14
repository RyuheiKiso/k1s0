package quotaclient_test

import (
	"context"
	"testing"
	"time"

	quotaclient "github.com/k1s0-platform/system-library-go-quota-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// クォータが上限以下の場合にCheckがAllowedを返すことを確認する。
func TestCheck_Allowed(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	status, err := c.Check(context.Background(), "storage:tenant-1", 100)
	require.NoError(t, err)
	assert.True(t, status.Allowed)
	assert.Equal(t, uint64(1000), status.Remaining)
	assert.Equal(t, uint64(1000), status.Limit)
}

// クォータが上限を超えた場合にCheckがDeniedを返すことを確認する。
func TestCheck_Exceeded(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	ctx := context.Background()
	_, _ = c.Increment(ctx, "q1", 900)
	status, err := c.Check(ctx, "q1", 200)
	require.NoError(t, err)
	assert.False(t, status.Allowed)
	assert.Equal(t, uint64(100), status.Remaining)
}

// Incrementが使用量を正しく加算して返すことを確認する。
func TestIncrement(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	usage, err := c.Increment(context.Background(), "q1", 500)
	require.NoError(t, err)
	assert.Equal(t, "q1", usage.QuotaID)
	assert.Equal(t, uint64(500), usage.Used)
	assert.Equal(t, uint64(1000), usage.Limit)
}

// 複数回のIncrementが使用量を累積加算することを確認する。
func TestIncrement_Accumulates(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	ctx := context.Background()
	_, _ = c.Increment(ctx, "q1", 300)
	usage, err := c.Increment(ctx, "q1", 200)
	require.NoError(t, err)
	assert.Equal(t, uint64(500), usage.Used)
}

// GetUsageがインクリメント後の正確な使用量を返すことを確認する。
func TestGetUsage(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	ctx := context.Background()
	_, _ = c.Increment(ctx, "q1", 100)
	usage, err := c.GetUsage(ctx, "q1")
	require.NoError(t, err)
	assert.Equal(t, uint64(100), usage.Used)
}

// GetPolicyがデフォルトポリシー（上限1000、日次リセット）を返すことを確認する。
func TestGetPolicy_Default(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	policy, err := c.GetPolicy(context.Background(), "q1")
	require.NoError(t, err)
	assert.Equal(t, "q1", policy.QuotaID)
	assert.Equal(t, uint64(1000), policy.Limit)
	assert.Equal(t, quotaclient.PeriodDaily, policy.Period)
}

// SetPolicyで登録したカスタムポリシーをGetPolicyが正しく返すことを確認する。
func TestGetPolicy_Custom(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	c.SetPolicy("q1", &quotaclient.QuotaPolicy{
		QuotaID:       "q1",
		Limit:         5000,
		Period:        quotaclient.PeriodMonthly,
		ResetStrategy: "sliding",
	})
	policy, err := c.GetPolicy(context.Background(), "q1")
	require.NoError(t, err)
	assert.Equal(t, uint64(5000), policy.Limit)
	assert.Equal(t, quotaclient.PeriodMonthly, policy.Period)
}

// CachedClientがGetPolicyの結果をキャッシュして同一データを返すことを確認する。
func TestCachedClient_CachesPolicy(t *testing.T) {
	inner := quotaclient.NewInMemoryQuotaClient()
	cached := quotaclient.NewCachedQuotaClient(inner, time.Minute)

	p1, err := cached.GetPolicy(context.Background(), "q1")
	require.NoError(t, err)
	p2, err := cached.GetPolicy(context.Background(), "q1")
	require.NoError(t, err)
	assert.Equal(t, p1.QuotaID, p2.QuotaID)
	assert.Equal(t, p1.Limit, p2.Limit)
}

// CachedClientがCheckを内部クライアントに委譲してAllowedを返すことを確認する。
func TestCachedClient_DelegatesCheck(t *testing.T) {
	inner := quotaclient.NewInMemoryQuotaClient()
	cached := quotaclient.NewCachedQuotaClient(inner, time.Minute)
	status, err := cached.Check(context.Background(), "q1", 100)
	require.NoError(t, err)
	assert.True(t, status.Allowed)
}

// CachedClientがIncrementを内部クライアントに委譲して使用量を返すことを確認する。
func TestCachedClient_DelegatesIncrement(t *testing.T) {
	inner := quotaclient.NewInMemoryQuotaClient()
	cached := quotaclient.NewCachedQuotaClient(inner, time.Minute)
	usage, err := cached.Increment(context.Background(), "q1", 100)
	require.NoError(t, err)
	assert.Equal(t, uint64(100), usage.Used)
}
