// 本ファイルは flagdFeatureAdapter の単体テスト。
// OpenFeature SDK Client を fake で差し替え、adapter が EvaluationContext / Variant /
// Reason を正しく組み立て、SDK の boolean / string / float / object 応答を
// FlagXxxResponse に詰め直すことを検証する。

package dapr

import (
	"context"
	"errors"
	"testing"

	"github.com/open-feature/go-sdk/openfeature"
)

// fakeOFClient は openFeatureClient interface を満たす最小 fake。
// 各メソッドにフックを指定し、引数捕捉と応答カスタマイズを行う。
type fakeOFClient struct {
	boolFn   func(ctx context.Context, flag string, def bool, ec openfeature.EvaluationContext) (openfeature.BooleanEvaluationDetails, error)
	stringFn func(ctx context.Context, flag string, def string, ec openfeature.EvaluationContext) (openfeature.StringEvaluationDetails, error)
	floatFn  func(ctx context.Context, flag string, def float64, ec openfeature.EvaluationContext) (openfeature.FloatEvaluationDetails, error)
	objFn    func(ctx context.Context, flag string, def any, ec openfeature.EvaluationContext) (openfeature.InterfaceEvaluationDetails, error)
}

func (f *fakeOFClient) BooleanValueDetails(ctx context.Context, flag string, defaultValue bool, evalCtx openfeature.EvaluationContext, _ ...openfeature.Option) (openfeature.BooleanEvaluationDetails, error) {
	return f.boolFn(ctx, flag, defaultValue, evalCtx)
}
func (f *fakeOFClient) StringValueDetails(ctx context.Context, flag string, defaultValue string, evalCtx openfeature.EvaluationContext, _ ...openfeature.Option) (openfeature.StringEvaluationDetails, error) {
	return f.stringFn(ctx, flag, defaultValue, evalCtx)
}
func (f *fakeOFClient) FloatValueDetails(ctx context.Context, flag string, defaultValue float64, evalCtx openfeature.EvaluationContext, _ ...openfeature.Option) (openfeature.FloatEvaluationDetails, error) {
	return f.floatFn(ctx, flag, defaultValue, evalCtx)
}
func (f *fakeOFClient) ObjectValueDetails(ctx context.Context, flag string, defaultValue any, evalCtx openfeature.EvaluationContext, _ ...openfeature.Option) (openfeature.InterfaceEvaluationDetails, error) {
	return f.objFn(ctx, flag, defaultValue, evalCtx)
}

// EvaluateBoolean: SDK が true / variant / reason を返した場合に
// FlagBooleanResponse へ忠実に詰め直されることを検証。tenant_id が
// EvaluationContext.tenant 属性に詰まることも併せて確認する。
func TestFlagdFeatureAdapter_EvaluateBoolean_True(t *testing.T) {
	fake := &fakeOFClient{
		boolFn: func(_ context.Context, flag string, _ bool, ec openfeature.EvaluationContext) (openfeature.BooleanEvaluationDetails, error) {
			if flag != "tenant-A.checkout.fast-path" {
				t.Fatalf("flag mismatch: %s", flag)
			}
			if got := ec.Attributes()[flagdEvalCtxKeyTenant]; got != "tenant-A" {
				t.Fatalf("tenant attribute mismatch: %v", got)
			}
			return openfeature.BooleanEvaluationDetails{
				Value: true,
				EvaluationDetails: openfeature.EvaluationDetails{
					FlagKey:  flag,
					FlagType: openfeature.Boolean,
					ResolutionDetail: openfeature.ResolutionDetail{
						Variant: "v2",
						Reason:  openfeature.TargetingMatchReason,
					},
				},
			}, nil
		},
	}
	a := NewFlagdFeatureAdapter(fake)
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
	if resp.Reason != string(openfeature.TargetingMatchReason) {
		t.Fatalf("reason mismatch: %s", resp.Reason)
	}
}

// EvaluateBoolean: SDK が空 variant / 空 reason を返した場合に
// "default" / "DEFAULT" にフォールバックすることを検証。
func TestFlagdFeatureAdapter_EvaluateBoolean_DefaultsFilled(t *testing.T) {
	fake := &fakeOFClient{
		boolFn: func(_ context.Context, _ string, _ bool, _ openfeature.EvaluationContext) (openfeature.BooleanEvaluationDetails, error) {
			// SDK が variant / reason を埋めなかったケース。
			return openfeature.BooleanEvaluationDetails{Value: false}, nil
		},
	}
	a := NewFlagdFeatureAdapter(fake)
	resp, err := a.EvaluateBoolean(context.Background(), FlagEvaluateRequest{FlagKey: "x"})
	if err != nil {
		t.Fatalf("EvaluateBoolean error: %v", err)
	}
	if resp.Value {
		t.Fatalf("expected Value=false default")
	}
	if resp.Reason != "DEFAULT" {
		t.Fatalf("reason mismatch: %s", resp.Reason)
	}
	if resp.Variant != "default" {
		t.Fatalf("variant mismatch: %s", resp.Variant)
	}
}

// EvaluateString: value / variant / reason がそのまま流れることを検証。
func TestFlagdFeatureAdapter_EvaluateString(t *testing.T) {
	fake := &fakeOFClient{
		stringFn: func(_ context.Context, _ string, _ string, _ openfeature.EvaluationContext) (openfeature.StringEvaluationDetails, error) {
			return openfeature.StringEvaluationDetails{
				Value: "premium",
				EvaluationDetails: openfeature.EvaluationDetails{
					ResolutionDetail: openfeature.ResolutionDetail{
						Variant: "v3",
						Reason:  openfeature.SplitReason,
					},
				},
			}, nil
		},
	}
	a := NewFlagdFeatureAdapter(fake)
	resp, err := a.EvaluateString(context.Background(), FlagEvaluateRequest{FlagKey: "tier"})
	if err != nil {
		t.Fatalf("EvaluateString error: %v", err)
	}
	if resp.Value != "premium" {
		t.Fatalf("value mismatch: %s", resp.Value)
	}
	if resp.Reason != string(openfeature.SplitReason) {
		t.Fatalf("reason mismatch: %s", resp.Reason)
	}
}

// EvaluateNumber: float64 がそのまま流れることを検証。
func TestFlagdFeatureAdapter_EvaluateNumber(t *testing.T) {
	fake := &fakeOFClient{
		floatFn: func(_ context.Context, _ string, _ float64, _ openfeature.EvaluationContext) (openfeature.FloatEvaluationDetails, error) {
			return openfeature.FloatEvaluationDetails{
				Value: 0.05,
				EvaluationDetails: openfeature.EvaluationDetails{
					ResolutionDetail: openfeature.ResolutionDetail{Variant: "v1", Reason: openfeature.StaticReason},
				},
			}, nil
		},
	}
	a := NewFlagdFeatureAdapter(fake)
	resp, err := a.EvaluateNumber(context.Background(), FlagEvaluateRequest{FlagKey: "rate"})
	if err != nil {
		t.Fatalf("EvaluateNumber error: %v", err)
	}
	if resp.Value != 0.05 {
		t.Fatalf("value mismatch: %v", resp.Value)
	}
}

// EvaluateObject: 任意の Go 値が JSON bytes に marshal されることを検証。
func TestFlagdFeatureAdapter_EvaluateObject(t *testing.T) {
	fake := &fakeOFClient{
		objFn: func(_ context.Context, _ string, _ any, _ openfeature.EvaluationContext) (openfeature.InterfaceEvaluationDetails, error) {
			return openfeature.InterfaceEvaluationDetails{
				Value: map[string]any{"limit": 100, "unit": "req/min"},
				EvaluationDetails: openfeature.EvaluationDetails{
					ResolutionDetail: openfeature.ResolutionDetail{Variant: "v1", Reason: openfeature.StaticReason},
				},
			}, nil
		},
	}
	a := NewFlagdFeatureAdapter(fake)
	resp, err := a.EvaluateObject(context.Background(), FlagEvaluateRequest{FlagKey: "rate-limit"})
	if err != nil {
		t.Fatalf("EvaluateObject error: %v", err)
	}
	// JSON marshal 後の文字列に "limit":100 と "unit":"req/min" が含まれることを確認する。
	got := string(resp.ValueJSON)
	if !contains(got, `"limit":100`) || !contains(got, `"unit":"req/min"`) {
		t.Fatalf("ValueJSON mismatch: %s", got)
	}
}

// SDK エラーが透過されることを検証。
func TestFlagdFeatureAdapter_SDKError(t *testing.T) {
	want := errors.New("flagd unavailable")
	fake := &fakeOFClient{
		boolFn: func(_ context.Context, _ string, _ bool, _ openfeature.EvaluationContext) (openfeature.BooleanEvaluationDetails, error) {
			return openfeature.BooleanEvaluationDetails{}, want
		},
	}
	a := NewFlagdFeatureAdapter(fake)
	_, err := a.EvaluateBoolean(context.Background(), FlagEvaluateRequest{FlagKey: "x"})
	if !errors.Is(err, want) {
		t.Fatalf("error not transparent: %v", err)
	}
}

// makeEvalCtx: targetingKey が subject > targetingKey > tenant_id の優先順で解決される
// ことを検証。
func TestMakeEvalCtx_TargetingKeyPriority(t *testing.T) {
	cases := []struct {
		name string
		req  FlagEvaluateRequest
		want string
	}{
		{
			name: "subject 優先",
			req: FlagEvaluateRequest{
				EvaluationContext: map[string]string{"subject": "alice", "targetingKey": "ignored"},
				TenantID:          "t-1",
			},
			want: "alice",
		},
		{
			name: "subject 不在なら targetingKey",
			req: FlagEvaluateRequest{
				EvaluationContext: map[string]string{"targetingKey": "user-9"},
				TenantID:          "t-2",
			},
			want: "user-9",
		},
		{
			name: "両方不在なら tenant_id",
			req: FlagEvaluateRequest{
				TenantID: "t-3",
			},
			want: "t-3",
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			ec := makeEvalCtx(c.req)
			if ec.TargetingKey() != c.want {
				t.Fatalf("targetingKey: want %q got %q", c.want, ec.TargetingKey())
			}
		})
	}
}

// contains は strings.Contains の小さな代替（test-only ヘルパ）。
// 本ファイルは strings 依存を増やさないため自前実装する。
func contains(s, sub string) bool {
	if len(sub) == 0 {
		return true
	}
	for i := 0; i+len(sub) <= len(s); i++ {
		if s[i:i+len(sub)] == sub {
			return true
		}
	}
	return false
}
