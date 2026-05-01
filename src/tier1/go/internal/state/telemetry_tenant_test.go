// 本ファイルは FR-T1-TELEMETRY-001 / 002 の tenant_id 自動付与と attribute 上限のテスト。

package state

import (
	"context"
	"fmt"
	"testing"

	telemetryv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/telemetry/v1"
)

// TestEmitMetric_AutoInjectsTenantLabel は tenant_id label が convertMetric で自動付与されることを確認する。
func TestEmitMetric_AutoInjectsTenantLabel(t *testing.T) {
	me := &fakeMetricEmitter{}
	h := &telemetryHandler{deps: Deps{MetricEmitter: me}}
	_, err := h.EmitMetric(context.Background(), &telemetryv1.EmitMetricRequest{
		Metrics: []*telemetryv1.Metric{
			{Name: "http_requests_total", Kind: telemetryv1.MetricKind_COUNTER, Value: 1, Labels: map[string]string{"method": "GET"}},
		},
		Context: makeTenantCtx("T-foo"),
	})
	if err != nil {
		t.Fatalf("EmitMetric: %v", err)
	}
	if len(me.calls) != 1 {
		t.Fatalf("expected 1 call, got %d", len(me.calls))
	}
	got := me.calls[0]
	if got.Labels["tenant_id"] != "T-foo" {
		t.Errorf("tenant_id label not auto-injected: got %q", got.Labels["tenant_id"])
	}
	if got.Labels["method"] != "GET" {
		t.Errorf("existing label dropped: got %q", got.Labels["method"])
	}
}

// TestEmitMetric_AlwaysOverwritesTenantLabel は user-supplied tenant_id label を tier1 値で上書きすることを確認する。
func TestEmitMetric_AlwaysOverwritesTenantLabel(t *testing.T) {
	me := &fakeMetricEmitter{}
	h := &telemetryHandler{deps: Deps{MetricEmitter: me}}
	_, err := h.EmitMetric(context.Background(), &telemetryv1.EmitMetricRequest{
		Metrics: []*telemetryv1.Metric{
			{Name: "x", Kind: telemetryv1.MetricKind_COUNTER, Value: 1, Labels: map[string]string{"tenant_id": "OTHER"}},
		},
		Context: makeTenantCtx("T-foo"),
	})
	if err != nil {
		t.Fatalf("EmitMetric: %v", err)
	}
	if me.calls[0].Labels["tenant_id"] != "T-foo" {
		t.Errorf("tenant_id should be overwritten with tier1 value: got %q", me.calls[0].Labels["tenant_id"])
	}
}

// TestEmitSpan_AutoInjectsTenantAttribute は tenant_id 属性が span に自動付与されることを確認する。
func TestEmitSpan_AutoInjectsTenantAttribute(t *testing.T) {
	te := &fakeTraceEmitter{}
	h := &telemetryHandler{deps: Deps{TraceEmitter: te}}
	_, err := h.EmitSpan(context.Background(), &telemetryv1.EmitSpanRequest{
		Spans: []*telemetryv1.Span{
			{Name: "op", TraceId: "t1", SpanId: "s1", Attributes: map[string]string{"http.status": "200"}},
		},
		Context: makeTenantCtx("T-foo"),
	})
	if err != nil {
		t.Fatalf("EmitSpan: %v", err)
	}
	if len(te.calls) != 1 {
		t.Fatalf("expected 1 call, got %d", len(te.calls))
	}
	got := te.calls[0]
	if got.Attributes["tenant_id"] != "T-foo" {
		t.Errorf("tenant_id attribute not auto-injected: got %q", got.Attributes["tenant_id"])
	}
	if got.Attributes["http.status"] != "200" {
		t.Errorf("existing attribute dropped: got %q", got.Attributes["http.status"])
	}
}

// TestEmitSpan_EnforcesAttributeLimit は 128 を超える属性は drop されることを確認する。
func TestEmitSpan_EnforcesAttributeLimit(t *testing.T) {
	te := &fakeTraceEmitter{}
	h := &telemetryHandler{deps: Deps{TraceEmitter: te}}
	attrs := make(map[string]string, 200)
	for i := 0; i < 200; i++ {
		attrs[fmt.Sprintf("k%d", i)] = "v"
	}
	_, err := h.EmitSpan(context.Background(), &telemetryv1.EmitSpanRequest{
		Spans: []*telemetryv1.Span{
			{Name: "op", TraceId: "t1", SpanId: "s1", Attributes: attrs},
		},
		Context: makeTenantCtx("T-foo"),
	})
	if err != nil {
		t.Fatalf("EmitSpan: %v", err)
	}
	got := te.calls[0]
	if len(got.Attributes) > telemetryMaxAttributes {
		t.Errorf("attribute count exceeds limit: got %d, want <= %d", len(got.Attributes), telemetryMaxAttributes)
	}
	if _, ok := got.Attributes["tenant_id"]; !ok {
		t.Error("tenant_id missing after attribute capping")
	}
}
