package quotaclient_test

import (
	"context"
	"testing"
	"time"

	quotaclient "github.com/k1s0-platform/system-library-go-quota-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestCheck_Allowed(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	status, err := c.Check(context.Background(), "storage:tenant-1", 100)
	require.NoError(t, err)
	assert.True(t, status.Allowed)
	assert.Equal(t, uint64(1000), status.Remaining)
	assert.Equal(t, uint64(1000), status.Limit)
}

func TestCheck_Exceeded(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	ctx := context.Background()
	_, _ = c.Increment(ctx, "q1", 900)
	status, err := c.Check(ctx, "q1", 200)
	require.NoError(t, err)
	assert.False(t, status.Allowed)
	assert.Equal(t, uint64(100), status.Remaining)
}

func TestIncrement(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	usage, err := c.Increment(context.Background(), "q1", 500)
	require.NoError(t, err)
	assert.Equal(t, "q1", usage.QuotaID)
	assert.Equal(t, uint64(500), usage.Used)
	assert.Equal(t, uint64(1000), usage.Limit)
}

func TestIncrement_Accumulates(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	ctx := context.Background()
	_, _ = c.Increment(ctx, "q1", 300)
	usage, err := c.Increment(ctx, "q1", 200)
	require.NoError(t, err)
	assert.Equal(t, uint64(500), usage.Used)
}

func TestGetUsage(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	ctx := context.Background()
	_, _ = c.Increment(ctx, "q1", 100)
	usage, err := c.GetUsage(ctx, "q1")
	require.NoError(t, err)
	assert.Equal(t, uint64(100), usage.Used)
}

func TestGetPolicy_Default(t *testing.T) {
	c := quotaclient.NewInMemoryQuotaClient()
	policy, err := c.GetPolicy(context.Background(), "q1")
	require.NoError(t, err)
	assert.Equal(t, "q1", policy.QuotaID)
	assert.Equal(t, uint64(1000), policy.Limit)
	assert.Equal(t, quotaclient.PeriodDaily, policy.Period)
}

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

func TestCachedClient_DelegatesCheck(t *testing.T) {
	inner := quotaclient.NewInMemoryQuotaClient()
	cached := quotaclient.NewCachedQuotaClient(inner, time.Minute)
	status, err := cached.Check(context.Background(), "q1", 100)
	require.NoError(t, err)
	assert.True(t, status.Allowed)
}

func TestCachedClient_DelegatesIncrement(t *testing.T) {
	inner := quotaclient.NewInMemoryQuotaClient()
	cached := quotaclient.NewCachedQuotaClient(inner, time.Minute)
	usage, err := cached.Increment(context.Background(), "q1", 100)
	require.NoError(t, err)
	assert.Equal(t, uint64(100), usage.Used)
}
