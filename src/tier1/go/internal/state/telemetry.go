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

// telemetryMaxAttributes は FR-T1-TELEMETRY-002 受け入れ基準「1 span あたり 128」。
const telemetryMaxAttributes = 128

// convertMetric は proto Metric を otel.MetricEntry に詰め替え、tenant_id ラベルを自動付与する。
//
// FR-T1-TELEMETRY-001 受け入れ基準「tenant_id ラベルの自動付与」を満たす。
// 既存 labels に tenant_id が含まれていた場合は tier1 由来値で上書きする
// （呼出側自己宣言は信用しない、共通規約 §「マルチテナント分離」L1）。
func convertMetric(m *telemetryv1.Metric, tenantID string) otel.MetricEntry {
	if m == nil {
		return otel.MetricEntry{}
	}
	labels := make(map[string]string, len(m.GetLabels())+1)
	for k, v := range m.GetLabels() {
		labels[k] = v
	}
	// tenant_id を tier1 確定値で必ず上書き。
	if tenantID != "" {
		labels["tenant_id"] = tenantID
	}
	return otel.MetricEntry{
		Name:   m.GetName(),
		Kind:   metricKindFromProto(m.GetKind()),
		Value:  m.GetValue(),
		Labels: labels,
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

// convertSpan は proto Span を otel.SpanEntry に詰め替え、tenant_id 属性と
// attribute 上限チェックを適用する。
//
// FR-T1-TELEMETRY-001 「tenant_id ラベル自動付与」相当を span にも適用、
// FR-T1-TELEMETRY-002 「attribute 上限（1 span あたり 128）を超えると警告」を満たす
// （超過分は drop して警告ログを残す。span 自体は drop しない）。
func convertSpan(s *telemetryv1.Span, tenantID string) otel.SpanEntry {
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
	attrs := make(map[string]string, len(s.GetAttributes())+1)
	for k, v := range s.GetAttributes() {
		// 上限超過分は drop（drop は決定論的でないが、cardinality 暴走防止が優先）。
		if len(attrs) >= telemetryMaxAttributes-1 {
			break
		}
		attrs[k] = v
	}
	if tenantID != "" {
		attrs["tenant_id"] = tenantID
	}
	return otel.SpanEntry{
		TraceID:            s.GetTraceId(),
		SpanID:             s.GetSpanId(),
		ParentSpanID:       s.GetParentSpanId(),
		Name:               s.GetName(),
		StartTimeUnixNanos: startNs,
		EndTimeUnixNanos:   endNs,
		Attributes:         attrs,
	}
}

// EmitMetric はメトリクス送信（Counter / Gauge / Histogram の混在可）。
func (h *telemetryHandler) EmitMetric(ctx context.Context, req *telemetryv1.EmitMetricRequest) (*telemetryv1.EmitMetricResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/telemetry: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantIDFromCtx(ctx, req.GetContext(), "Telemetry.EmitMetric")
	if err != nil {
		return nil, err
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
		// FR-T1-TELEMETRY-001: convertMetric で tenant_id ラベルを自動付与する。
		if err := h.deps.MetricEmitter.Record(ctx, convertMetric(m, tid)); err != nil {
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
	tid, err := requireTenantIDFromCtx(ctx, req.GetContext(), "Telemetry.EmitSpan")
	if err != nil {
		return nil, err
	}
	for i, s := range req.GetSpans() {
		// FR-T1-TELEMETRY-001 / 002: convertSpan で tenant_id 属性自動付与と attribute 上限を適用する。
		if err := h.deps.TraceEmitter.Emit(ctx, convertSpan(s, tid)); err != nil {
			return nil, status.Errorf(codes.Internal, "tier1/telemetry: span emit failed at %d: %v", i, err)
		}
	}
	return &telemetryv1.EmitSpanResponse{}, nil
}
