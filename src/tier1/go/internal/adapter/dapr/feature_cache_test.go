// 本ファイルは CachedFeatureAdapter の単体テスト。
//
// 検証観点（FR-T1-FEATURE-001 / FR-T1-FEATURE-004 受け入れ基準と直結）:
//   - 同一 (tenant_id, flag_key, evaluation_context) の 2 回目以降は base が呼ばれない（cache hit）
//   - evaluation_context が異なれば base が呼ばれる（cache miss）
//   - tenant_id が異なれば base が呼ばれる（テナント間で flag 値を漏らさない）
//   - boolean / string / number / object で互いに干渉しない
//   - TTL 経過後は base が再度呼ばれる
//   - base error は cache に保存されない（次回呼出で再試行する fail-open）

package dapr

import (
	"context"
	"errors"
	"testing"
	"time"
)

// fakeFeatureAdapter は call count を観測する FeatureAdapter のスタブ。
type fakeFeatureAdapter struct {
	boolCalls   int
	stringCalls int
	numberCalls int
	objectCalls int
	// boolErr が non-nil なら EvaluateBoolean は error を返す（fail-soft 動作テスト用）。
	boolErr error
	// retVal は次回 evaluate でフィクスチャとして返す値。
	retVal string
}

func (f *fakeFeatureAdapter) EvaluateBoolean(_ context.Context, req FlagEvaluateRequest) (FlagBooleanResponse, error) {
	f.boolCalls++
	if f.boolErr != nil {
		return FlagBooleanResponse{}, f.boolErr
	}
	return FlagBooleanResponse{Value: true, Variant: "on", Reason: "TARGETING_MATCH"}, nil
}

func (f *fakeFeatureAdapter) EvaluateString(_ context.Context, req FlagEvaluateRequest) (FlagStringResponse, error) {
	f.stringCalls++
	return FlagStringResponse{Value: f.retVal, Variant: "v1", Reason: "TARGETING_MATCH"}, nil
}

func (f *fakeFeatureAdapter) EvaluateNumber(_ context.Context, req FlagEvaluateRequest) (FlagNumberResponse, error) {
	f.numberCalls++
	return FlagNumberResponse{Value: 42, Variant: "v1", Reason: "TARGETING_MATCH"}, nil
}

func (f *fakeFeatureAdapter) EvaluateObject(_ context.Context, req FlagEvaluateRequest) (FlagObjectResponse, error) {
	f.objectCalls++
	return FlagObjectResponse{ValueJSON: []byte(`{"x":1}`), Variant: "v1", Reason: "TARGETING_MATCH"}, nil
}

func TestCachedFeatureAdapter_BooleanHitMiss(t *testing.T) {
	base := &fakeFeatureAdapter{}
	c := NewCachedFeatureAdapter(base, 0) // 既定 30 秒
	req := FlagEvaluateRequest{TenantID: "T1", FlagKey: "feature.x", EvaluationContext: map[string]string{"u": "u1"}}
	if _, err := c.EvaluateBoolean(context.Background(), req); err != nil {
		t.Fatalf("first call err = %v", err)
	}
	if _, err := c.EvaluateBoolean(context.Background(), req); err != nil {
		t.Fatalf("second call err = %v", err)
	}
	if base.boolCalls != 1 {
		t.Errorf("base.boolCalls = %d; want 1 (cache hit on second call, FR-T1-FEATURE-001)", base.boolCalls)
	}
}

func TestCachedFeatureAdapter_DifferentContextMisses(t *testing.T) {
	base := &fakeFeatureAdapter{}
	c := NewCachedFeatureAdapter(base, 0)
	a := FlagEvaluateRequest{TenantID: "T1", FlagKey: "feature.x", EvaluationContext: map[string]string{"u": "u1"}}
	b := FlagEvaluateRequest{TenantID: "T1", FlagKey: "feature.x", EvaluationContext: map[string]string{"u": "u2"}}
	_, _ = c.EvaluateBoolean(context.Background(), a)
	_, _ = c.EvaluateBoolean(context.Background(), b)
	if base.boolCalls != 2 {
		t.Errorf("base.boolCalls = %d; want 2 (different evaluation_context must miss cache)", base.boolCalls)
	}
}

func TestCachedFeatureAdapter_DifferentTenantsMiss(t *testing.T) {
	base := &fakeFeatureAdapter{}
	c := NewCachedFeatureAdapter(base, 0)
	a := FlagEvaluateRequest{TenantID: "T1", FlagKey: "feature.x"}
	b := FlagEvaluateRequest{TenantID: "T2", FlagKey: "feature.x"}
	_, _ = c.EvaluateBoolean(context.Background(), a)
	_, _ = c.EvaluateBoolean(context.Background(), b)
	if base.boolCalls != 2 {
		t.Errorf("base.boolCalls = %d; want 2 (cross-tenant must not share cache)", base.boolCalls)
	}
}

func TestCachedFeatureAdapter_KindsAreIsolated(t *testing.T) {
	base := &fakeFeatureAdapter{retVal: "hello"}
	c := NewCachedFeatureAdapter(base, 0)
	req := FlagEvaluateRequest{TenantID: "T1", FlagKey: "feature.same"}
	_, _ = c.EvaluateBoolean(context.Background(), req)
	_, _ = c.EvaluateString(context.Background(), req)
	_, _ = c.EvaluateNumber(context.Background(), req)
	_, _ = c.EvaluateObject(context.Background(), req)
	// 各 kind 1 回ずつ base が呼ばれる（互いに干渉しない）。
	if base.boolCalls != 1 || base.stringCalls != 1 || base.numberCalls != 1 || base.objectCalls != 1 {
		t.Errorf("kind-isolation broken: bool=%d string=%d number=%d object=%d",
			base.boolCalls, base.stringCalls, base.numberCalls, base.objectCalls)
	}
}

func TestCachedFeatureAdapter_TTLExpires(t *testing.T) {
	base := &fakeFeatureAdapter{}
	// 1ms TTL で TTL 経過を即座にシミュレートする（test 専用）。
	c := NewCachedFeatureAdapter(base, time.Millisecond)
	req := FlagEvaluateRequest{TenantID: "T1", FlagKey: "feature.x"}
	_, _ = c.EvaluateBoolean(context.Background(), req)
	// TTL 経過まで sleep。
	time.Sleep(5 * time.Millisecond)
	_, _ = c.EvaluateBoolean(context.Background(), req)
	if base.boolCalls != 2 {
		t.Errorf("base.boolCalls = %d; want 2 (TTL expiration must re-fetch)", base.boolCalls)
	}
}

func TestCachedFeatureAdapter_ErrorNotCached(t *testing.T) {
	base := &fakeFeatureAdapter{boolErr: errors.New("flagd unavailable")}
	c := NewCachedFeatureAdapter(base, 0)
	req := FlagEvaluateRequest{TenantID: "T1", FlagKey: "feature.x"}
	_, err1 := c.EvaluateBoolean(context.Background(), req)
	_, err2 := c.EvaluateBoolean(context.Background(), req)
	if err1 == nil || err2 == nil {
		t.Fatalf("expected errors on both calls, got %v / %v", err1, err2)
	}
	if base.boolCalls != 2 {
		t.Errorf("base.boolCalls = %d; want 2 (error responses must not be cached, FR-T1-FEATURE-001 fail-soft)", base.boolCalls)
	}
}

func TestCachedFeatureAdapter_ObjectDefensiveCopy(t *testing.T) {
	base := &fakeFeatureAdapter{}
	c := NewCachedFeatureAdapter(base, 0)
	req := FlagEvaluateRequest{TenantID: "T1", FlagKey: "feature.x"}
	first, _ := c.EvaluateObject(context.Background(), req)
	// caller が ValueJSON を mutate しても cache は壊れない（defensive copy）。
	if len(first.ValueJSON) > 0 {
		first.ValueJSON[0] = 0x00
	}
	second, _ := c.EvaluateObject(context.Background(), req)
	if string(second.ValueJSON) != `{"x":1}` {
		t.Errorf("cache poisoned by caller mutation: %q", string(second.ValueJSON))
	}
}

func TestCachedFeatureAdapter_TTLOverflowFallsBackToDefault(t *testing.T) {
	base := &fakeFeatureAdapter{}
	// 上限超（10 分）は既定 30 秒にフォールバックされる。
	c := NewCachedFeatureAdapter(base, 10*time.Minute)
	if c.TTL() != defaultFeatureCacheTTL {
		t.Errorf("TTL ceiling not enforced: got %v, want %v", c.TTL(), defaultFeatureCacheTTL)
	}
}

func TestCanonicalContext_StableAcrossMapOrderings(t *testing.T) {
	// Go map の iteration 順は非決定なので、複数回 build しても同じ canonical 形になることを確認する。
	a := canonicalContext(map[string]string{"a": "1", "b": "2", "c": "3"})
	for i := 0; i < 100; i++ {
		got := canonicalContext(map[string]string{"a": "1", "b": "2", "c": "3"})
		if got != a {
			t.Fatalf("canonical drift at iteration %d: got=%q want=%q", i, got, a)
		}
	}
}
