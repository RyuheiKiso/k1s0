// 本ファイルは daprFeatureAdapter の単体テスト。
// Dapr SDK GetConfigurationItem を fake で差し替え、
// adapter が string → bool/number/object/string にパースして応答を組み立てることを検証する。

package dapr

import (
	"context"
	"errors"
	"testing"

	daprclient "github.com/dapr/go-sdk/client"
)

// fakeConfigClient は daprConfigClient の最小 fake 実装。
type fakeConfigClient struct {
	getFn func(ctx context.Context, store, key string, opts ...daprclient.ConfigurationOpt) (*daprclient.ConfigurationItem, error)
}

func (f *fakeConfigClient) GetConfigurationItem(ctx context.Context, store, key string, opts ...daprclient.ConfigurationOpt) (*daprclient.ConfigurationItem, error) {
	return f.getFn(ctx, store, key, opts...)
}

func newFeatureAdapterWithFake(t *testing.T, fake *fakeConfigClient) FeatureAdapter {
	t.Helper()
	return NewFeatureAdapter(NewWithConfigClient("test://noop", fake))
}

// EvaluateBoolean: "true" 文字列が bool true にパースされ、Variant/Reason が反映されることを検証。
func TestFeatureAdapter_EvaluateBoolean_True(t *testing.T) {
	fake := &fakeConfigClient{
		getFn: func(_ context.Context, store, key string, _ ...daprclient.ConfigurationOpt) (*daprclient.ConfigurationItem, error) {
			if store != "flagd-default" {
				t.Fatalf("store mismatch: %s", store)
			}
			if key != "tenant-A.checkout.fast-path" {
				t.Fatalf("key mismatch: %s", key)
			}
			return &daprclient.ConfigurationItem{
				Value:   "true",
				Version: "v2",
				Metadata: map[string]string{"reason": "TARGETING_MATCH"},
			}, nil
		},
	}
	a := newFeatureAdapterWithFake(t, fake)
	resp, err := a.EvaluateBoolean(context.Background(), FlagEvaluateRequest{
		FlagKey:  "tenant-A.checkout.fast-path",
		TenantID: "tenant-A",
	})
	if err != nil {
		t.Fatalf("EvaluateBoolean error: %v", err)
	}
	if !resp.Value {
		t.Fatalf("expected Value=true")
	}
	if resp.Variant != "v2" {
		t.Fatalf("variant mismatch: %s", resp.Variant)
	}
	if resp.Reason != "TARGETING_MATCH" {
		t.Fatalf("reason mismatch: %s", resp.Reason)
	}
}

// EvaluateBoolean: 値未存在時に Value=false / Reason=DEFAULT を返すことを検証。
func TestFeatureAdapter_EvaluateBoolean_NotFound(t *testing.T) {
	fake := &fakeConfigClient{
		getFn: func(_ context.Context, _, _ string, _ ...daprclient.ConfigurationOpt) (*daprclient.ConfigurationItem, error) {
			return nil, nil
		},
	}
	a := newFeatureAdapterWithFake(t, fake)
	resp, err := a.EvaluateBoolean(context.Background(), FlagEvaluateRequest{FlagKey: "x"})
	if err != nil {
		t.Fatalf("EvaluateBoolean error: %v", err)
	}
	if resp.Value {
		t.Fatalf("expected Value=false on NotFound")
	}
	if resp.Reason != "DEFAULT" {
		t.Fatalf("reason mismatch: %s", resp.Reason)
	}
	if resp.Variant != "default" {
		t.Fatalf("variant mismatch: %s", resp.Variant)
	}
}

// EvaluateBoolean: パース不可な値で Reason=ERROR を返すことを検証。
func TestFeatureAdapter_EvaluateBoolean_ParseError(t *testing.T) {
	fake := &fakeConfigClient{
		getFn: func(_ context.Context, _, _ string, _ ...daprclient.ConfigurationOpt) (*daprclient.ConfigurationItem, error) {
			return &daprclient.ConfigurationItem{Value: "yes", Version: "v1"}, nil
		},
	}
	a := newFeatureAdapterWithFake(t, fake)
	resp, err := a.EvaluateBoolean(context.Background(), FlagEvaluateRequest{FlagKey: "x"})
	if err != nil {
		t.Fatalf("EvaluateBoolean error: %v", err)
	}
	if resp.Reason != "ERROR" {
		t.Fatalf("expected Reason=ERROR on parse fail, got %s", resp.Reason)
	}
}

// EvaluateString: 値がそのまま返ることを検証。
func TestFeatureAdapter_EvaluateString(t *testing.T) {
	fake := &fakeConfigClient{
		getFn: func(_ context.Context, _, _ string, _ ...daprclient.ConfigurationOpt) (*daprclient.ConfigurationItem, error) {
			return &daprclient.ConfigurationItem{Value: "premium", Version: "v3", Metadata: map[string]string{"reason": "SPLIT"}}, nil
		},
	}
	a := newFeatureAdapterWithFake(t, fake)
	resp, err := a.EvaluateString(context.Background(), FlagEvaluateRequest{FlagKey: "tier"})
	if err != nil {
		t.Fatalf("EvaluateString error: %v", err)
	}
	if resp.Value != "premium" {
		t.Fatalf("value mismatch: %s", resp.Value)
	}
	if resp.Reason != "SPLIT" {
		t.Fatalf("reason mismatch: %s", resp.Reason)
	}
}

// EvaluateNumber: 数値文字列が float64 にパースされることを検証。
func TestFeatureAdapter_EvaluateNumber(t *testing.T) {
	fake := &fakeConfigClient{
		getFn: func(_ context.Context, _, _ string, _ ...daprclient.ConfigurationOpt) (*daprclient.ConfigurationItem, error) {
			return &daprclient.ConfigurationItem{Value: "0.05", Version: "v1"}, nil
		},
	}
	a := newFeatureAdapterWithFake(t, fake)
	resp, err := a.EvaluateNumber(context.Background(), FlagEvaluateRequest{FlagKey: "rate"})
	if err != nil {
		t.Fatalf("EvaluateNumber error: %v", err)
	}
	if resp.Value != 0.05 {
		t.Fatalf("value mismatch: %v", resp.Value)
	}
}

// EvaluateObject: JSON 文字列がそのまま bytes として返ることを検証。
func TestFeatureAdapter_EvaluateObject(t *testing.T) {
	jsonValue := `{"limit":100,"unit":"req/min"}`
	fake := &fakeConfigClient{
		getFn: func(_ context.Context, _, _ string, _ ...daprclient.ConfigurationOpt) (*daprclient.ConfigurationItem, error) {
			return &daprclient.ConfigurationItem{Value: jsonValue, Version: "v1"}, nil
		},
	}
	a := newFeatureAdapterWithFake(t, fake)
	resp, err := a.EvaluateObject(context.Background(), FlagEvaluateRequest{FlagKey: "rate-limit"})
	if err != nil {
		t.Fatalf("EvaluateObject error: %v", err)
	}
	if string(resp.ValueJSON) != jsonValue {
		t.Fatalf("json bytes mismatch: %s", resp.ValueJSON)
	}
}

// SDK エラーが透過されることを検証。
func TestFeatureAdapter_SDKError(t *testing.T) {
	want := errors.New("config store unavailable")
	fake := &fakeConfigClient{
		getFn: func(_ context.Context, _, _ string, _ ...daprclient.ConfigurationOpt) (*daprclient.ConfigurationItem, error) {
			return nil, want
		},
	}
	a := newFeatureAdapterWithFake(t, fake)
	_, err := a.EvaluateBoolean(context.Background(), FlagEvaluateRequest{FlagKey: "x"})
	if !errors.Is(err, want) {
		t.Fatalf("error not transparent: %v", err)
	}
}
