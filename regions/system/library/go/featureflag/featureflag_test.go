package featureflag_test

import (
	"context"
	"testing"

	ff "github.com/k1s0-platform/system-library-go-featureflag"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Evaluateが有効なフラグを評価して有効状態とバリアント名を返すことを確認する。
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

// Evaluateが無効なフラグを評価してFLAG_DISABLEDを返しバリアントがnilであることを確認する。
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

// Evaluateが存在しないフラグキーを指定した場合にFLAG_NOT_FOUNDエラーを返すことを確認する。
func TestEvaluate_FlagNotFound(t *testing.T) {
	client := ff.NewInMemoryFeatureFlagClient()

	_, err := client.Evaluate(context.Background(), "missing", ff.NewEvaluationContext())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "FLAG_NOT_FOUND")
}

// GetFlagが指定したフラグキーのFeatureFlagオブジェクトを正常に取得することを確認する。
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

// GetFlagが存在しないフラグキーを指定した場合にFLAG_NOT_FOUNDエラーを返すことを確認する。
func TestGetFlag_NotFound(t *testing.T) {
	client := ff.NewInMemoryFeatureFlagClient()

	_, err := client.GetFlag(context.Background(), "missing")
	require.Error(t, err)
	assert.Contains(t, err.Error(), "FLAG_NOT_FOUND")
}

// IsEnabledがフラグの有効・無効状態を正しいブール値で返すことを確認する。
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

// EvaluationContextのビルダーメソッドがユーザーID・テナントID・カスタム属性を正しく設定することを確認する。
func TestEvaluationContext(t *testing.T) {
	ctx := ff.NewEvaluationContext().
		WithUserID("user-1").
		WithTenantID("tenant-1").
		WithAttribute("env", "production")

	assert.Equal(t, "user-1", *ctx.UserID)
	assert.Equal(t, "tenant-1", *ctx.TenantID)
	assert.Equal(t, "production", ctx.Attributes["env"])
}
