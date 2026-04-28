// 本ファイルは ObservabilityInterceptor の OTel SDK 経由 end-to-end テスト。
//
// 検証観点:
//   - in-memory SpanExporter で実際に span が Export されること
//   - span.Name が full method を含むこと
//   - 属性に rpc.system / rpc.service / rpc.method / tenant_id が乗ること
//   - エラー時に span.Status が Error になること
//   - in-memory MetricReader で counter / histogram が記録されること

package common

import (
	"context"
	"testing"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/metric/metricdata"
	"go.opentelemetry.io/otel/sdk/trace"
	"go.opentelemetry.io/otel/sdk/trace/tracetest"
	"google.golang.org/grpc"
	grpccodes "google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// 共通規約 §可観測性: tier1 ファサードで必ず 1 span を発行する。
// in-memory SpanExporter で span が実際に export されることを確認する。
func TestObservabilityInterceptor_EmitsSpan(t *testing.T) {
	exporter := tracetest.NewInMemoryExporter()
	tp := trace.NewTracerProvider(
		trace.WithSyncer(exporter),
	)
	defer func() { _ = tp.Shutdown(context.Background()) }()
	otel.SetTracerProvider(tp)
	t.Cleanup(func() { otel.SetTracerProvider(otel.GetTracerProvider()) })

	icpt := ObservabilityInterceptor()
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	_, err := icpt(context.Background(), &fakeRequest{ctx: &fakeTenantContext{tenantID: "T1"}}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return "ok", nil
	})
	if err != nil {
		t.Fatalf("err = %v", err)
	}
	spans := exporter.GetSpans()
	if len(spans) != 1 {
		t.Fatalf("expected 1 span, got %d", len(spans))
	}
	got := spans[0]
	if got.Name != info.FullMethod {
		t.Errorf("span.Name = %q; want %q", got.Name, info.FullMethod)
	}
	wantAttrs := map[string]string{
		"rpc.system":  "grpc",
		"rpc.service": "state",
		"rpc.method":  "Get",
		"tenant_id":   "T1",
	}
	for _, a := range got.Attributes {
		if want, ok := wantAttrs[string(a.Key)]; ok {
			if a.Value.AsString() != want {
				t.Errorf("attr %s = %q; want %q", a.Key, a.Value.AsString(), want)
			}
			delete(wantAttrs, string(a.Key))
		}
	}
	if len(wantAttrs) > 0 {
		t.Errorf("missing attributes: %v", wantAttrs)
	}
}

// gRPC error 時に span.Status.Code が Error になることを確認する。
func TestObservabilityInterceptor_SpanStatusOnError(t *testing.T) {
	exporter := tracetest.NewInMemoryExporter()
	tp := trace.NewTracerProvider(trace.WithSyncer(exporter))
	defer func() { _ = tp.Shutdown(context.Background()) }()
	otel.SetTracerProvider(tp)
	t.Cleanup(func() { otel.SetTracerProvider(otel.GetTracerProvider()) })

	icpt := ObservabilityInterceptor()
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	_, _ = icpt(context.Background(), &fakeRequest{}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return nil, status.Error(grpccodes.NotFound, "missing")
	})
	spans := exporter.GetSpans()
	if len(spans) != 1 {
		t.Fatalf("expected 1 span, got %d", len(spans))
	}
	if spans[0].Status.Code != 1 /* otel codes.Error */ {
		t.Errorf("span.Status.Code = %v; want Error(1)", spans[0].Status.Code)
	}
}

// MeterProvider に in-memory reader を attach し、counter / histogram が記録されることを確認する。
func TestObservabilityInterceptor_RecordsMetrics(t *testing.T) {
	reader := metric.NewManualReader()
	mp := metric.NewMeterProvider(metric.WithReader(reader))
	defer func() { _ = mp.Shutdown(context.Background()) }()
	otel.SetMeterProvider(mp)
	t.Cleanup(func() { otel.SetMeterProvider(otel.GetMeterProvider()) })

	icpt := ObservabilityInterceptor()
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.pubsub.v1.PubSubService/Publish"}
	_, _ = icpt(context.Background(), &fakeRequest{ctx: &fakeTenantContext{tenantID: "T2"}}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return "ok", nil
	})

	rm := metricdata.ResourceMetrics{}
	if err := reader.Collect(context.Background(), &rm); err != nil {
		t.Fatalf("collect: %v", err)
	}
	// 期待: k1s0_tier1_pubsub_requests_total と k1s0_tier1_pubsub_duration_seconds が両方記録される。
	wantNames := map[string]bool{
		"k1s0_tier1_pubsub_requests_total":   false,
		"k1s0_tier1_pubsub_duration_seconds": false,
	}
	for _, sm := range rm.ScopeMetrics {
		for _, m := range sm.Metrics {
			if _, ok := wantNames[m.Name]; ok {
				wantNames[m.Name] = true
			}
		}
	}
	for name, found := range wantNames {
		if !found {
			t.Errorf("metric %q not recorded", name)
		}
	}
	// counter のラベルに tenant_id / method / code が乗ることも確認する。
	var observedAttrs []attribute.KeyValue
	for _, sm := range rm.ScopeMetrics {
		for _, m := range sm.Metrics {
			if m.Name != "k1s0_tier1_pubsub_requests_total" {
				continue
			}
			sum, ok := m.Data.(metricdata.Sum[int64])
			if !ok {
				t.Fatalf("requests_total not a Sum[int64]: %T", m.Data)
			}
			if len(sum.DataPoints) == 0 {
				t.Fatal("no data points")
			}
			observedAttrs = sum.DataPoints[0].Attributes.ToSlice()
		}
	}
	want := map[string]string{"tenant_id": "T2", "method": "Publish", "code": "OK"}
	for _, a := range observedAttrs {
		if w, ok := want[string(a.Key)]; ok {
			if a.Value.AsString() != w {
				t.Errorf("attr %s = %q; want %q", a.Key, a.Value.AsString(), w)
			}
			delete(want, string(a.Key))
		}
	}
	if len(want) > 0 {
		t.Errorf("missing counter attrs: %v", want)
	}
}
