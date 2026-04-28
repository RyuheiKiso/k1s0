// 本ファイルは otel.LogEmitter の単体テスト。
// 実 OTel SDK Logger の代わりに recordingLogger を使い、
// Emit に渡された LogRecord の内容を検証する。

package otel

import (
	"context"
	"testing"
	"time"

	logv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/log/v1"
	otellog "go.opentelemetry.io/otel/log"
	"go.opentelemetry.io/otel/log/embedded"
)

// recordingLogger は OTel Logger interface を満たす最小 fake。
// Emit に渡された Record を順番に保存する。
type recordingLogger struct {
	embedded.Logger
	records []otellog.Record
}

func (r *recordingLogger) Emit(_ context.Context, rec otellog.Record) {
	r.records = append(r.records, rec)
}

// Enabled は OTel Logger interface 必須メソッド（v0.19）。常に true を返す。
func (r *recordingLogger) Enabled(_ context.Context, _ otellog.EnabledParameters) bool {
	return true
}

// 単一 LogEntry が正しく LogRecord に変換されることを検証する。
func TestLogEmitter_Emit_BasicMapping(t *testing.T) {
	rec := &recordingLogger{}
	emitter := NewLogEmitter(rec)
	now := time.Now().UTC().UnixNano()
	if err := emitter.Emit(context.Background(), LogEntry{
		Timestamp:    now,
		Severity:     otellog.SeverityError,
		SeverityText: "ERROR",
		Body:         "DB connection lost",
		Attributes:   map[string]string{"service.name": "tier1/state"},
		StackTrace:   "main.go:42",
	}); err != nil {
		t.Fatalf("Emit error: %v", err)
	}
	if len(rec.records) != 1 {
		t.Fatalf("records count: got %d want 1", len(rec.records))
	}
	got := rec.records[0]
	if got.Severity() != otellog.SeverityError {
		t.Fatalf("severity mismatch: %v", got.Severity())
	}
	if got.SeverityText() != "ERROR" {
		t.Fatalf("severityText mismatch: %s", got.SeverityText())
	}
	if got.Body().AsString() != "DB connection lost" {
		t.Fatalf("body mismatch: %s", got.Body().AsString())
	}
	// 属性検証: service.name + exception.stacktrace の 2 件あるはず。
	attrCount := 0
	got.WalkAttributes(func(_ otellog.KeyValue) bool { attrCount++; return true })
	if attrCount != 2 {
		t.Fatalf("attr count: got %d want 2", attrCount)
	}
}

// proto Severity → OTel Severity の変換テーブル検証。
func TestSeverityFromProto(t *testing.T) {
	cases := []struct {
		in  logv1.Severity
		out otellog.Severity
	}{
		{logv1.Severity_TRACE, otellog.SeverityTrace},
		{logv1.Severity_DEBUG, otellog.SeverityDebug},
		{logv1.Severity_INFO, otellog.SeverityInfo},
		{logv1.Severity_WARN, otellog.SeverityWarn},
		{logv1.Severity_ERROR, otellog.SeverityError},
		{logv1.Severity_FATAL, otellog.SeverityFatal},
	}
	for _, c := range cases {
		if got := SeverityFromProto(c.in); got != c.out {
			t.Errorf("SeverityFromProto(%v): got %v want %v", c.in, got, c.out)
		}
	}
}

func TestSeverityText(t *testing.T) {
	if SeverityText(logv1.Severity_INFO) != "INFO" {
		t.Errorf("SeverityText(INFO) wrong")
	}
	if SeverityText(logv1.Severity_FATAL) != "FATAL" {
		t.Errorf("SeverityText(FATAL) wrong")
	}
}
