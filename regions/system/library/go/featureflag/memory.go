package featureflag

import (
	"context"
	"sync"
)

// InMemoryFeatureFlagClient はメモリ内フィーチャーフラグクライアントの実装。
type InMemoryFeatureFlagClient struct {
	mu    sync.RWMutex
	flags map[string]*FeatureFlag
}

// NewInMemoryFeatureFlagClient は新しい InMemoryFeatureFlagClient を生成する。
func NewInMemoryFeatureFlagClient() *InMemoryFeatureFlagClient {
	return &InMemoryFeatureFlagClient{
		flags: make(map[string]*FeatureFlag),
	}
}

// SetFlag はフラグを設定する。
func (c *InMemoryFeatureFlagClient) SetFlag(flag *FeatureFlag) {
	c.mu.Lock()
	defer c.mu.Unlock()
	copy := *flag
	c.flags[flag.FlagKey] = &copy
}

func (c *InMemoryFeatureFlagClient) Evaluate(_ context.Context, flagKey string, _ *EvaluationContext) (*EvaluationResult, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()

	flag, ok := c.flags[flagKey]
	if !ok {
		return nil, NewFlagNotFoundError(flagKey)
	}

	result := &EvaluationResult{
		FlagKey: flagKey,
		Enabled: flag.Enabled,
	}
	if flag.Enabled {
		result.Reason = "FLAG_ENABLED"
	} else {
		result.Reason = "FLAG_DISABLED"
	}
	if len(flag.Variants) > 0 {
		v := flag.Variants[0].Name
		result.Variant = &v
	}
	return result, nil
}

func (c *InMemoryFeatureFlagClient) GetFlag(_ context.Context, flagKey string) (*FeatureFlag, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()

	flag, ok := c.flags[flagKey]
	if !ok {
		return nil, NewFlagNotFoundError(flagKey)
	}
	copy := *flag
	return &copy, nil
}

func (c *InMemoryFeatureFlagClient) IsEnabled(ctx context.Context, flagKey string, evalCtx *EvaluationContext) (bool, error) {
	result, err := c.Evaluate(ctx, flagKey, evalCtx)
	if err != nil {
		return false, err
	}
	return result.Enabled, nil
}
