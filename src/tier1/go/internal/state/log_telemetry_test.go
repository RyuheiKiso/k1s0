// 本ファイルは Log / Telemetry handler の単体テスト。
// fake LogEmitter / MetricEmitter / TraceEmitter で OTel SDK を切り離し、
// handler の proto ↔ otel adapter 詰め替えロジックを検証する。

package state

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/k1s0/k1s0/src/tier1/go/internal/otel"
	logv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/log/v1"
	telemetryv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/telemetry/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// ---------------------------------------------------------------------------
// fake emitters
// ---------------------------------------------------------------------------

type fakeLogEmitter struct {
	calls []otel.LogEntry
	err   error
}

func (f *fakeLogEmitter) Emit(_ context.Context, e otel.LogEntry) error {
	f.calls = append(f.calls, e)
	return f.err
}

type fakeMetricEmitter struct {
	calls []otel.MetricEntry
	err   error
}

func (f *fakeMetricEmitter) Record(_ context.Context, e otel.MetricEntry) error {
	f.calls = append(f.calls, e)
	return f.err
}

type fakeTraceEmitter struct {
	calls []otel.SpanEntry
	err   error
}

func (f *fakeTraceEmitter) Emit(_ context.Context, e otel.SpanEntry) error {
	f.calls = append(f.calls, e)
	return f.err
}

// ---------------------------------------------------------------------------
// Log handler
// ---------------------------------------------------------------------------

// emitter 注入後は Send が proto entry を otel.LogEntry に詰め替えることを検証する。
func TestLogHandler_Send_OK(t *testing.T) {
	emitter := &fakeLogEmitter{}
	h := &logHandler{deps: Deps{LogEmitter: emitter}}
	ts := time.Date(2026, 4, 28, 12, 0, 0, 0, time.UTC)
	_, err := h.Send(context.Background(), &logv1.SendLogRequest{
		Entry: &logv1.LogEntry{
			Timestamp:  timestamppb.New(ts),
			Severity:   logv1.Severity_ERROR,
			Body:       "DB lost",
			Attributes: map[string]string{"service.name": "tier1"},
			StackTrace: "main.go:42",
		},
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Send error: %v", err)
	}
	if len(emitter.calls) != 1 {
		t.Fatalf("emitter called %d times", len(emitter.calls))
	}
	got := emitter.calls[0]
	if got.Severity != otel.SeverityFromProto(logv1.Severity_ERROR) {
		t.Fatalf("severity mismatch")
	}
	if got.Body != "DB lost" {
		t.Fatalf("body mismatch: %s", got.Body)
	}
	if got.SeverityText != "ERROR" {
		t.Fatalf("severity text mismatch: %s", got.SeverityText)
	}
	if got.StackTrace != "main.go:42" {
		t.Fatalf("stack mismatch: %s", got.StackTrace)
	}
}

// emitter エラーが Internal に翻訳される。
func TestLogHandler_Send_EmitterError(t *testing.T) {
	emitter := &fakeLogEmitter{err: errors.New("loki down")}
	h := &logHandler{deps: Deps{LogEmitter: emitter}}
	_, err := h.Send(context.Background(), &logv1.SendLogRequest{Entry: &logv1.LogEntry{Body: "x"}, Context: makeTenantCtx("T")})
	if got := status.Code(err); got != codes.Internal {
		t.Fatalf("status: got %v want Internal", got)
	}
}

// FR-T1-LOG-003 / NFR-E-AC-003: tenant_id 未設定時に InvalidArgument を返す。
func TestLogHandler_Send_RequiresTenant(t *testing.T) {
	emitter := &fakeLogEmitter{}
	h := &logHandler{deps: Deps{LogEmitter: emitter}}
	_, err := h.Send(context.Background(), &logv1.SendLogRequest{Entry: &logv1.LogEntry{Body: "x"}})
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("expected InvalidArgument for missing tenant, got %v", got)
	}
}

// BulkSend が複数エントリを一括変換し、Accepted を返却する。
func TestLogHandler_BulkSend_OK(t *testing.T) {
	emitter := &fakeLogEmitter{}
	h := &logHandler{deps: Deps{LogEmitter: emitter}}
	resp, err := h.BulkSend(context.Background(), &logv1.BulkSendLogRequest{
		Entries: []*logv1.LogEntry{
			{Body: "e1"},
			{Body: "e2"},
			{Body: "e3"},
		},
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("BulkSend error: %v", err)
	}
	if len(emitter.calls) != 3 {
		t.Fatalf("emitter calls: %d", len(emitter.calls))
	}
	if resp.GetAccepted() != 3 {
		t.Fatalf("accepted: got %d want 3", resp.GetAccepted())
	}
}

// ---------------------------------------------------------------------------
// Telemetry handler
// ---------------------------------------------------------------------------

func TestTelemetryHandler_EmitMetric_OK(t *testing.T) {
	emitter := &fakeMetricEmitter{}
	h := &telemetryHandler{deps: Deps{MetricEmitter: emitter}}
	_, err := h.EmitMetric(context.Background(), &telemetryv1.EmitMetricRequest{
		Metrics: []*telemetryv1.Metric{
			{Name: "k1s0.invoke.duration_ms", Kind: telemetryv1.MetricKind_HISTOGRAM, Value: 12.5, Labels: map[string]string{"status": "ok"}},
			{Name: "k1s0.error.count", Kind: telemetryv1.MetricKind_COUNTER, Value: 1},
		},
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("EmitMetric error: %v", err)
	}
	if len(emitter.calls) != 2 {
		t.Fatalf("emitter calls: %d", len(emitter.calls))
	}
	if emitter.calls[0].Kind != otel.MetricKindHistogram {
		t.Fatalf("metric 0 kind mismatch")
	}
	if emitter.calls[1].Kind != otel.MetricKindCounter {
		t.Fatalf("metric 1 kind mismatch")
	}
	if emitter.calls[0].Value != 12.5 {
		t.Fatalf("metric 0 value mismatch")
	}
}

func TestTelemetryHandler_EmitSpan_OK(t *testing.T) {
	emitter := &fakeTraceEmitter{}
	h := &telemetryHandler{deps: Deps{TraceEmitter: emitter}}
	now := time.Date(2026, 4, 28, 12, 0, 0, 0, time.UTC)
	_, err := h.EmitSpan(context.Background(), &telemetryv1.EmitSpanRequest{
		Spans: []*telemetryv1.Span{
			{
				TraceId:      "0123456789abcdef0123456789abcdef",
				SpanId:       "0123456789abcdef",
				ParentSpanId: "fedcba9876543210",
				Name:         "GET /api/v1/foo",
				StartTime:    timestamppb.New(now),
				EndTime:      timestamppb.New(now.Add(10 * time.Millisecond)),
				Attributes:   map[string]string{"http.method": "GET"},
			},
		},
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("EmitSpan error: %v", err)
	}
	if len(emitter.calls) != 1 {
		t.Fatalf("emitter calls: %d", len(emitter.calls))
	}
	got := emitter.calls[0]
	if got.TraceID != "0123456789abcdef0123456789abcdef" || got.SpanID != "0123456789abcdef" {
		t.Fatalf("trace/span id mismatch: %s / %s", got.TraceID, got.SpanID)
	}
	if got.Name != "GET /api/v1/foo" {
		t.Fatalf("name mismatch: %s", got.Name)
	}
	if got.EndTimeUnixNanos-got.StartTimeUnixNanos != int64(10*time.Millisecond) {
		t.Fatalf("duration mismatch")
	}
}

// metricKindFromProto の境界条件をテーブル駆動で検証。
func TestMetricKindFromProto(t *testing.T) {
	cases := []struct {
		in  telemetryv1.MetricKind
		out otel.MetricKind
	}{
		{telemetryv1.MetricKind_COUNTER, otel.MetricKindCounter},
		{telemetryv1.MetricKind_GAUGE, otel.MetricKindGauge},
		{telemetryv1.MetricKind_HISTOGRAM, otel.MetricKindHistogram},
	}
	for _, c := range cases {
		if got := metricKindFromProto(c.in); got != c.out {
			t.Errorf("metricKindFromProto(%v): got %v want %v", c.in, got, c.out)
		}
	}
}
