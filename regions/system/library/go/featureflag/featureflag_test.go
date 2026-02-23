package featureflag_test

import (
	"context"
	"testing"

	ff "github.com/k1s0-platform/system-library-go-featureflag"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestEvaluate_EnabledFlag(t *testing.T) {
	client := ff.NewInMemoryFeatureFlagClient()
	client.SetFlag(&ff.FeatureFlag{
		ID:      "1",
		FlagKey: "dark-mode",
		Enabled: true,
		Variants: []ff.FlagVariant{
			{Name: "on", Value: "true", Weight: 100},
		},
	})

	result, err := client.Evaluate(context.Background(), "dark-mode", ff.NewEvaluationContext())
	require.NoError(t, err)
	assert.True(t, result.Enabled)
	assert.Equal(t, "FLAG_ENABLED", result.Reason)
	require.NotNil(t, result.Variant)
	assert.Equal(t, "on", *result.Variant)
}

func TestEvaluate_DisabledFlag(t *testing.T) {
	client := ff.NewInMemoryFeatureFlagClient()
	client.SetFlag(&ff.FeatureFlag{
		ID:      "2",
		FlagKey: "beta-feature",
		Enabled: false,
	})

	result, err := client.Evaluate(context.Background(), "beta-feature", ff.NewEvaluationContext())
	require.NoError(t, err)
	assert.False(t, result.Enabled)
	assert.Equal(t, "FLAG_DISABLED", result.Reason)
	assert.Nil(t, result.Variant)
}

func TestEvaluate_FlagNotFound(t *testing.T) {
	client := ff.NewInMemoryFeatureFlagClient()

	_, err := client.Evaluate(context.Background(), "missing", ff.NewEvaluationContext())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "FLAG_NOT_FOUND")
}

func TestGetFlag(t *testing.T) {
	client := ff.NewInMemoryFeatureFlagClient()
	client.SetFlag(&ff.FeatureFlag{
		ID:          "1",
		FlagKey:     "dark-mode",
		Description: "ダークモード機能",
		Enabled:     true,
	})

	flag, err := client.GetFlag(context.Background(), "dark-mode")
	require.NoError(t, err)
	assert.Equal(t, "dark-mode", flag.FlagKey)
	assert.Equal(t, "ダークモード機能", flag.Description)
}

func TestGetFlag_NotFound(t *testing.T) {
	client := ff.NewInMemoryFeatureFlagClient()

	_, err := client.GetFlag(context.Background(), "missing")
	require.Error(t, err)
	assert.Contains(t, err.Error(), "FLAG_NOT_FOUND")
}

func TestIsEnabled(t *testing.T) {
	client := ff.NewInMemoryFeatureFlagClient()
	client.SetFlag(&ff.FeatureFlag{
		ID:      "1",
		FlagKey: "dark-mode",
		Enabled: true,
	})
	client.SetFlag(&ff.FeatureFlag{
		ID:      "2",
		FlagKey: "beta",
		Enabled: false,
	})

	enabled, err := client.IsEnabled(context.Background(), "dark-mode", ff.NewEvaluationContext())
	require.NoError(t, err)
	assert.True(t, enabled)

	enabled, err = client.IsEnabled(context.Background(), "beta", ff.NewEvaluationContext())
	require.NoError(t, err)
	assert.False(t, enabled)
}

func TestEvaluationContext(t *testing.T) {
	ctx := ff.NewEvaluationContext().
		WithUserID("user-1").
		WithTenantID("tenant-1").
		WithAttribute("env", "production")

	assert.Equal(t, "user-1", *ctx.UserID)
	assert.Equal(t, "tenant-1", *ctx.TenantID)
	assert.Equal(t, "production", ctx.Attributes["env"])
}
