// 本ファイルは t1-state Pod の TelemetryService 2 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/08_Telemetry_API.md
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - DS-SW-COMP-038（Metrics Emitter: RED モデル / Prometheus ServiceMonitor）
//
// 役割（plan 04-13 結線済）:
//   SDK 側 facade からの gRPC 入口で proto Metric / Span を受け取り、
//   internal/otel.MetricEmitter / TraceEmitter 越しに OTel Metrics / Traces
//   パイプライン（→ Mimir / Tempo）へ流す。

package state

import (
	"context"

	"github.com/k1s0/k1s0/src/tier1/go/internal/otel"
	telemetryv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/telemetry/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// telemetryHandler は TelemetryService の handler 実装。
type telemetryHandler struct {
	telemetryv1.UnimplementedTelemetryServiceServer
	deps Deps
}

// convertMetric は proto Metric を otel.MetricEntry に詰め替える。
func convertMetric(m *telemetryv1.Metric) otel.MetricEntry {
	if m == nil {
		return otel.MetricEntry{}
	}
	return otel.MetricEntry{
		Name:   m.GetName(),
		Kind:   metricKindFromProto(m.GetKind()),
		Value:  m.GetValue(),
		Labels: m.GetLabels(),
	}
}

// metricKindFromProto は proto MetricKind を otel.MetricKind に変換する。
func metricKindFromProto(k telemetryv1.MetricKind) otel.MetricKind {
	switch k {
	case telemetryv1.MetricKind_COUNTER:
		return otel.MetricKindCounter
	case telemetryv1.MetricKind_GAUGE:
		return otel.MetricKindGauge
	case telemetryv1.MetricKind_HISTOGRAM:
		return otel.MetricKindHistogram
	default:
		return otel.MetricKindCounter
	}
}

// convertSpan は proto Span を otel.SpanEntry に詰め替える。
func convertSpan(s *telemetryv1.Span) otel.SpanEntry {
	if s == nil {
		return otel.SpanEntry{}
	}
	var startNs, endNs int64
	if s.GetStartTime() != nil {
		startNs = s.GetStartTime().AsTime().UnixNano()
	}
	if s.GetEndTime() != nil {
		endNs = s.GetEndTime().AsTime().UnixNano()
	}
	return otel.SpanEntry{
		TraceID:            s.GetTraceId(),
		SpanID:             s.GetSpanId(),
		ParentSpanID:       s.GetParentSpanId(),
		Name:               s.GetName(),
		StartTimeUnixNanos: startNs,
		EndTimeUnixNanos:   endNs,
		Attributes:         s.GetAttributes(),
	}
}

// EmitMetric はメトリクス送信（Counter / Gauge / Histogram の混在可）。
func (h *telemetryHandler) EmitMetric(ctx context.Context, req *telemetryv1.EmitMetricRequest) (*telemetryv1.EmitMetricResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/telemetry: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	if _, err := requireTenantIDFromCtx(ctx, req.GetContext(), "Telemetry.EmitMetric"); err != nil {
		return nil, err
	}
	if h.deps.MetricEmitter == nil {
		return nil, status.Error(codes.Unimplemented, "tier1/telemetry: EmitMetric not yet wired to OTel Collector")
	}
	// 必須入力（各 metric の name）を事前検証。OTel SDK は空 instrument 名を
	// rejecting し plain error を返すため、handler 段で InvalidArgument に
	// 変換しないと codes.Internal に潰れて HTTP 500 になる。
	for i, m := range req.GetMetrics() {
		if m.GetName() == "" {
			return nil, status.Errorf(codes.InvalidArgument,
				"tier1/telemetry: metric[%d].name required (non-empty)", i)
		}
	}
	for i, m := range req.GetMetrics() {
		if err := h.deps.MetricEmitter.Record(ctx, convertMetric(m)); err != nil {
			return nil, status.Errorf(codes.Internal, "tier1/telemetry: metric record failed at %d: %v", i, err)
		}
	}
	return &telemetryv1.EmitMetricResponse{}, nil
}

// EmitSpan は終了済 Span の送信。
func (h *telemetryHandler) EmitSpan(ctx context.Context, req *telemetryv1.EmitSpanRequest) (*telemetryv1.EmitSpanResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/telemetry: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	if _, err := requireTenantIDFromCtx(ctx, req.GetContext(), "Telemetry.EmitSpan"); err != nil {
		return nil, err
	}
	if h.deps.TraceEmitter == nil {
		return nil, status.Error(codes.Unimplemented, "tier1/telemetry: EmitSpan not yet wired to OTel Collector")
	}
	for i, s := range req.GetSpans() {
		if err := h.deps.TraceEmitter.Emit(ctx, convertSpan(s)); err != nil {
			return nil, status.Errorf(codes.Internal, "tier1/telemetry: span emit failed at %d: %v", i, err)
		}
	}
	return &telemetryv1.EmitSpanResponse{}, nil
}
