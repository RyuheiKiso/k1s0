package ratelimitclient_test

import (
	"context"
	"testing"

	ratelimitclient "github.com/k1s0-platform/system-library-go-ratelimit-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestCheck_Allowed(t *testing.T) {
	c := ratelimitclient.NewInMemoryClient()
	status, err := c.Check(context.Background(), "test-key", 1)
	require.NoError(t, err)
	assert.True(t, status.Allowed)
	assert.Equal(t, uint32(99), status.Remaining)
	assert.Nil(t, status.RetryAfterSecs)
}

func TestCheck_Denied(t *testing.T) {
	c := ratelimitclient.NewInMemoryClient()
	c.SetPolicy("limited-key", ratelimitclient.RateLimitPolicy{
		Key:        "limited-key",
		Limit:      2,
		WindowSecs: 60,
		Algorithm:  "fixed_window",
	})

	// 2回消費して制限超過にする
	_, err := c.Consume(context.Background(), "limited-key", 2)
	require.NoError(t, err)

	status, err := c.Check(context.Background(), "limited-key", 1)
	require.NoError(t, err)
	assert.False(t, status.Allowed)
	assert.Equal(t, uint32(0), status.Remaining)
	assert.NotNil(t, status.RetryAfterSecs)
}

func TestConsume_Success(t *testing.T) {
	c := ratelimitclient.NewInMemoryClient()
	result, err := c.Consume(context.Background(), "test-key", 1)
	require.NoError(t, err)
	assert.Equal(t, uint32(99), result.Remaining)
	assert.Equal(t, uint32(1), c.UsedCount("test-key"))
}

func TestConsume_ExceedsLimit(t *testing.T) {
	c := ratelimitclient.NewInMemoryClient()
	c.SetPolicy("small-key", ratelimitclient.RateLimitPolicy{
		Key:        "small-key",
		Limit:      1,
		WindowSecs: 60,
		Algorithm:  "token_bucket",
	})

	_, err := c.Consume(context.Background(), "small-key", 1)
	require.NoError(t, err)

	_, err = c.Consume(context.Background(), "small-key", 1)
	assert.Error(t, err)
}

func TestGetLimit_DefaultPolicy(t *testing.T) {
	c := ratelimitclient.NewInMemoryClient()
	policy, err := c.GetLimit(context.Background(), "unknown-key")
	require.NoError(t, err)
	assert.Equal(t, uint32(100), policy.Limit)
	assert.Equal(t, uint64(3600), policy.WindowSecs)
	assert.Equal(t, "token_bucket", policy.Algorithm)
}

func TestGetLimit_CustomPolicy(t *testing.T) {
	c := ratelimitclient.NewInMemoryClient()
	c.SetPolicy("tenant:T1", ratelimitclient.RateLimitPolicy{
		Key:        "tenant:T1",
		Limit:      50,
		WindowSecs: 1800,
		Algorithm:  "sliding_window",
	})

	policy, err := c.GetLimit(context.Background(), "tenant:T1")
	require.NoError(t, err)
	assert.Equal(t, "tenant:T1", policy.Key)
	assert.Equal(t, uint32(50), policy.Limit)
	assert.Equal(t, uint64(1800), policy.WindowSecs)
	assert.Equal(t, "sliding_window", policy.Algorithm)
}

func TestCheck_MultipleCosts(t *testing.T) {
	c := ratelimitclient.NewInMemoryClient()
	c.SetPolicy("cost-key", ratelimitclient.RateLimitPolicy{
		Key:        "cost-key",
		Limit:      10,
		WindowSecs: 60,
		Algorithm:  "fixed_window",
	})

	status, err := c.Check(context.Background(), "cost-key", 5)
	require.NoError(t, err)
	assert.True(t, status.Allowed)
	assert.Equal(t, uint32(5), status.Remaining)

	status, err = c.Check(context.Background(), "cost-key", 11)
	require.NoError(t, err)
	assert.False(t, status.Allowed)
}
