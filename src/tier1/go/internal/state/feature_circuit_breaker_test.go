// 本ファイルは FR-T1-FEATURE-003 Circuit Breaker rules のテスト。

package state

import (
	"context"
	"errors"
	"sync"
	"testing"
	"time"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	featurev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/feature/v1"
)

// TestOverrideStore_LookupAndForce は強制 false override の書き込みと参照を確認する。
func TestOverrideStore_LookupAndForce(t *testing.T) {
	s := NewFeatureFlagOverrideStore()
	if _, _, ok := s.Lookup("flag.x"); ok {
		t.Fatal("empty store should not have any override")
	}
	s.Force("flag.x", false, "CIRCUIT_BREAKER", time.Now().Add(1*time.Hour))
	v, reason, ok := s.Lookup("flag.x")
	if !ok {
		t.Fatal("override should be found after Force")
	}
	if v != false || reason != "CIRCUIT_BREAKER" {
		t.Errorf("override: value=%v reason=%q, want false / CIRCUIT_BREAKER", v, reason)
	}
}

// TestOverrideStore_AutoExpires は until 経過後 override が自動解除されることを確認する。
func TestOverrideStore_AutoExpires(t *testing.T) {
	s := NewFeatureFlagOverrideStore()
	// すでに過去時刻の until で書き込む。
	s.Force("flag.x", false, "CIRCUIT_BREAKER", time.Now().Add(-1*time.Second))
	if _, _, ok := s.Lookup("flag.x"); ok {
		t.Error("expired override should not be returned")
	}
}

// TestOverrideStore_ManualClear は手動切戻（Clear）を確認する。
func TestOverrideStore_ManualClear(t *testing.T) {
	s := NewFeatureFlagOverrideStore()
	s.Force("flag.x", false, "CIRCUIT_BREAKER", time.Now().Add(1*time.Hour))
	s.Clear("flag.x")
	if _, _, ok := s.Lookup("flag.x"); ok {
		t.Error("override should be cleared after Clear")
	}
}

// fakeMetricSource は MetricThresholdSource の fake。返り値を制御できる。
type fakeMetricSource struct {
	mu     sync.Mutex
	value  float64
	err    error
	called int
}

func (f *fakeMetricSource) Evaluate(_ context.Context, _ FeatureCBRule) (float64, error) {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.called++
	return f.value, f.err
}

// TestEvaluator_TripsOverrideOnThresholdExceeded は閾値超過で override が発動することを確認する。
func TestEvaluator_TripsOverrideOnThresholdExceeded(t *testing.T) {
	src := &fakeMetricSource{value: 0.10} // 10% > threshold 5%
	store := NewFeatureFlagOverrideStore()
	ev := NewFeatureCircuitBreakerEvaluator(src, store, 1*time.Hour)
	ev.SetRules([]FeatureCBRule{{
		FlagKey:      "tenant.app.feature1",
		PromQL:       `sum(rate(http_errors[1m])) / sum(rate(http_requests[1m]))`,
		Threshold:    0.05,
		Comparator:   "gt",
		RecoverAfter: 5 * time.Minute,
		ForcedFalse:  true,
	}})
	// 直接 evaluateOnce を呼んで決定論的にテスト。
	ev.evaluateOnce(context.Background())
	v, reason, ok := store.Lookup("tenant.app.feature1")
	if !ok {
		t.Fatal("override should be set after threshold exceeded")
	}
	if v != false {
		t.Errorf("forced value = %v, want false", v)
	}
	if reason != "CIRCUIT_BREAKER" {
		t.Errorf("reason = %q, want CIRCUIT_BREAKER", reason)
	}
}

// TestEvaluator_DoesNotTripWhenWithinThreshold は閾値内なら override が立たないことを確認する。
func TestEvaluator_DoesNotTripWhenWithinThreshold(t *testing.T) {
	src := &fakeMetricSource{value: 0.01} // 1% < 5%
	store := NewFeatureFlagOverrideStore()
	ev := NewFeatureCircuitBreakerEvaluator(src, store, 1*time.Hour)
	ev.SetRules([]FeatureCBRule{{
		FlagKey:    "tenant.app.feature1",
		Threshold:  0.05,
		Comparator: "gt",
	}})
	ev.evaluateOnce(context.Background())
	if _, _, ok := store.Lookup("tenant.app.feature1"); ok {
		t.Error("override should not be set when within threshold")
	}
}

// TestEvaluator_SkipsRuleOnSourceError は PromQL 評価エラー時に override を立てないことを確認する。
func TestEvaluator_SkipsRuleOnSourceError(t *testing.T) {
	src := &fakeMetricSource{err: errors.New("prom unreachable")}
	store := NewFeatureFlagOverrideStore()
	ev := NewFeatureCircuitBreakerEvaluator(src, store, 1*time.Hour)
	ev.SetRules([]FeatureCBRule{{
		FlagKey:   "tenant.app.feature1",
		Threshold: 0.05,
	}})
	ev.evaluateOnce(context.Background())
	if _, _, ok := store.Lookup("tenant.app.feature1"); ok {
		t.Error("override should not be set when source errors (avoid false positive)")
	}
}

// TestEvaluator_DefaultIntervalIs30s は interval=0 で 30 秒既定が使われることを確認する。
func TestEvaluator_DefaultIntervalIs30s(t *testing.T) {
	ev := NewFeatureCircuitBreakerEvaluator(&fakeMetricSource{}, NewFeatureFlagOverrideStore(), 0)
	if ev.interval != 30*time.Second {
		t.Errorf("interval = %v, want 30s (FR-T1-FEATURE-003 default)", ev.interval)
	}
}

// TestEvaluator_LtComparator は "lt" 演算子が「未満」判定で発火することを確認する。
func TestEvaluator_LtComparator(t *testing.T) {
	src := &fakeMetricSource{value: 5} // 5 < threshold 10
	store := NewFeatureFlagOverrideStore()
	ev := NewFeatureCircuitBreakerEvaluator(src, store, 1*time.Hour)
	ev.SetRules([]FeatureCBRule{{
		FlagKey:    "x",
		Threshold:  10,
		Comparator: "lt",
	}})
	ev.evaluateOnce(context.Background())
	if _, _, ok := store.Lookup("x"); !ok {
		t.Error("lt comparator should trip when value < threshold")
	}
}

// TestEvaluator_AuditEmittedOnTrip は override 発動時に audit event が emit されることを確認する。
func TestEvaluator_AuditEmittedOnTrip(t *testing.T) {
	src := &fakeMetricSource{value: 0.10}
	store := NewFeatureFlagOverrideStore()
	ev := NewFeatureCircuitBreakerEvaluator(src, store, 1*time.Hour)
	auditEmitter := &fakeLogEmitter{}
	ev.SetAuditEmitter(auditEmitter)
	ev.SetRules([]FeatureCBRule{{
		FlagKey:    "x",
		PromQL:     "q",
		Threshold:  0.05,
		Comparator: "gt",
	}})
	ev.evaluateOnce(context.Background())
	if len(auditEmitter.calls) != 1 {
		t.Fatalf("expected 1 audit emit, got %d", len(auditEmitter.calls))
	}
	got := auditEmitter.calls[0]
	if got.Body != "feature flag forced to false by circuit breaker" {
		t.Errorf("audit body unexpected: %q", got.Body)
	}
	if got.Attributes["flag_key"] != "x" {
		t.Errorf("audit flag_key: %q", got.Attributes["flag_key"])
	}
}

// TestFeatureHandler_HonorsOverride は override 中は adapter を呼ばずに強制値を返すことを確認する。
func TestFeatureHandler_HonorsOverride(t *testing.T) {
	store := NewFeatureFlagOverrideStore()
	store.Force("tenant.app.feature1", false, "CIRCUIT_BREAKER", time.Now().Add(1*time.Hour))
	adapterCalled := false
	a := &fakeFeatureAdapter{
		boolFn: func(_ context.Context, _ dapr.FlagEvaluateRequest) (dapr.FlagBooleanResponse, error) {
			adapterCalled = true
			return dapr.FlagBooleanResponse{Value: true}, nil
		},
	}
	h := &featureHandler{deps: Deps{FeatureAdapter: a, FeatureOverrides: store}}
	resp, err := h.EvaluateBoolean(context.Background(), &featurev1.EvaluateRequest{
		FlagKey: "tenant.app.feature1",
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if resp.GetValue() != false {
		t.Errorf("value = %v, want false (override forced)", resp.GetValue())
	}
	if resp.GetMetadata().GetReason() != "CIRCUIT_BREAKER" {
		t.Errorf("reason = %q, want CIRCUIT_BREAKER", resp.GetMetadata().GetReason())
	}
	if adapterCalled {
		t.Error("adapter should not be called when override is active")
	}
}
