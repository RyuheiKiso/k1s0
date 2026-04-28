// 本ファイルは tier1 Go ファサードの OTel Logs アダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - DS-SW-COMP-037（Log Adapter: stdout JSON Lines / OTel Collector / Loki 集約）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/07_Log_API.md
//
// 役割:
//   LogService.Send / BulkSend 経由で受け取った proto LogEntry を、
//   `go.opentelemetry.io/otel/log` の Logger を用いて OTel Logs パイプライン
//   （tier1 Pod 内で Init された LoggerProvider → OTLP gRPC → OTel Collector → Loki）
//   に流し込む。Logger インスタンスは cmd 側で `otel.Init(...)` を呼んで取得し、
//   handler の依存として注入される。
//
// テスタビリティ:
//   `LogEmitter` interface を介すことで、test では fake を注入できる。
//   production の `otelLogEmitter` は OTel SDK の Logger を保持し、
//   Severity / Body / Attributes を OTel LogRecord に詰めて Emit する。

package otel

import (
	// 全 RPC で context を伝搬する。
	"context"
	// proto LogEntry の Severity enum を使う。
	logv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/log/v1"
	// OTel Logs API。
	otellog "go.opentelemetry.io/otel/log"
)

// LogEntry は handler が adapter に渡す中間表現。
// proto を直接渡すと OTel SDK 側に proto 依存が漏れるため、ここで詰め替える。
type LogEntry struct {
	// 観測時刻（UTC）。
	Timestamp int64 // unix nanoseconds, 0 で「未指定 → emitter が現在時刻を補完」
	// 重大度（OTel Severity 1-24 にマップ済の値）。
	Severity otellog.Severity
	// 重大度テキスト（DEBUG / INFO / WARN / ERROR / FATAL のような表現）。
	SeverityText string
	// メッセージ本文。
	Body string
	// 属性（service.name / trace_id / span_id 等を含む）。
	Attributes map[string]string
	// 例外スタック（あれば）。
	StackTrace string
}

// LogEmitter は OTel Logs パイプラインへの emit 操作を抽象化する interface。
// production は otelLogEmitter（実 SDK 経由）、test は fake を注入する。
type LogEmitter interface {
	// 単一エントリを emit する。OTel SDK の Logger.Emit() を呼ぶ。
	Emit(ctx context.Context, entry LogEntry) error
}

// otelLogEmitter は実 OTel Logger を保持する emitter 実装。
type otelLogEmitter struct {
	logger otellog.Logger
}

// NewLogEmitter は OTel Logger から LogEmitter を生成する。
func NewLogEmitter(logger otellog.Logger) LogEmitter {
	return &otelLogEmitter{logger: logger}
}

// Emit は LogEntry を OTel LogRecord に変換して Logger.Emit を呼ぶ。
func (e *otelLogEmitter) Emit(ctx context.Context, entry LogEntry) error {
	// OTel LogRecord を構築。
	var record otellog.Record
	if entry.Timestamp > 0 {
		record.SetTimestamp(otelLogTimestampFromUnixNanos(entry.Timestamp))
	}
	record.SetSeverity(entry.Severity)
	if entry.SeverityText != "" {
		record.SetSeverityText(entry.SeverityText)
	}
	record.SetBody(otellog.StringValue(entry.Body))
	// 属性は string→string で全件 attach。OTel は他の型もサポートするが、
	// k1s0 proto は map<string,string> のみのため string で統一する。
	for k, v := range entry.Attributes {
		record.AddAttributes(otellog.String(k, v))
	}
	if entry.StackTrace != "" {
		record.AddAttributes(otellog.String("exception.stacktrace", entry.StackTrace))
	}
	// SDK は Emit 内部で context から ObservedTimestamp / Resource / Scope を補完する。
	e.logger.Emit(ctx, record)
	return nil
}

// SeverityFromProto は proto の Severity enum を OTel Severity に変換する。
// k1s0 proto は OTel SeverityNumber（1〜24）に整合した値を採用しているため
// 直接 cast 可能だが、型 alias で意図を明確にする。
func SeverityFromProto(s logv1.Severity) otellog.Severity {
	switch s {
	case logv1.Severity_TRACE:
		return otellog.SeverityTrace
	case logv1.Severity_DEBUG:
		return otellog.SeverityDebug
	case logv1.Severity_INFO:
		return otellog.SeverityInfo
	case logv1.Severity_WARN:
		return otellog.SeverityWarn
	case logv1.Severity_ERROR:
		return otellog.SeverityError
	case logv1.Severity_FATAL:
		return otellog.SeverityFatal
	default:
		return otellog.SeverityUndefined
	}
}

// SeverityText は proto Severity をログテキストに変換する。
// SeverityText は OTel が SeverityNumber 補完用に使うため、人間可読の名前を返す。
func SeverityText(s logv1.Severity) string {
	switch s {
	case logv1.Severity_TRACE:
		return "TRACE"
	case logv1.Severity_DEBUG:
		return "DEBUG"
	case logv1.Severity_INFO:
		return "INFO"
	case logv1.Severity_WARN:
		return "WARN"
	case logv1.Severity_ERROR:
		return "ERROR"
	case logv1.Severity_FATAL:
		return "FATAL"
	default:
		return ""
	}
}
