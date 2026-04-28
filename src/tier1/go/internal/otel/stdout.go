// 本ファイルは OTel Collector を持たない開発 / CI 環境向けの stdout 直書き emitter。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - DS-SW-COMP-037（Log Adapter: stdout JSON Lines / OTel Collector / Loki 集約）
//     - DS-SW-COMP-038（Metrics Emitter）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/07_Log_API.md
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/08_Telemetry_API.md
//
// 役割:
//   OTel Collector / Mimir / Tempo / Loki を持たない開発・CI 環境でも、
//   tier1 cmd/state バイナリが Log / Telemetry RPC で値を返せるよう、
//   標準出力に JSON Lines として書き出す emitter を提供する。
//   production では OTel SDK の Logger / Meter / Tracer に切替えるが、SDK 経由でなくても
//   docs 正典「stdout JSON Lines」（DS-SW-COMP-037）はそれ自体が一次出力経路として規定済。
//
// 出力フォーマット（1 行 1 イベント、JSON Lines）:
//   ログ:    {"kind":"log","timestamp":N,"severity":"INFO","body":"...","attributes":{...}}
//   メトリ:  {"kind":"metric","name":"...","metric_kind":"counter","value":N,"labels":{...}}
//   スパン:  {"kind":"span","trace_id":"...","span_id":"...","name":"...","start_ns":N,"end_ns":N,"attributes":{...}}
//
// テスタビリティ:
//   io.Writer を注入できる構造にし、test では bytes.Buffer に書き出して assertion 可能。
//   production では os.Stdout を渡す。

package otel

import (
	// 全 RPC で context を伝搬する。
	"context"
	// JSON エンコードに使う。
	"encoding/json"
	// 出力先抽象（io.Writer）。
	"io"
	// 並行制御（同 io.Writer への並列 write を直列化する）。
	"sync"
)

// stdoutLogEmitter は LogEntry を JSON Lines として io.Writer に書き出す。
// 並行 RPC からの呼出で出力が交錯しないよう Mutex で write を直列化する。
type stdoutLogEmitter struct {
	// 出力先（os.Stdout / bytes.Buffer 等、test では fake を注入）。
	w io.Writer
	// write の直列化に使う Mutex。
	mu sync.Mutex
}

// NewStdoutLogEmitter は io.Writer に stdout JSON Lines を書く LogEmitter を生成する。
func NewStdoutLogEmitter(w io.Writer) LogEmitter {
	// stdoutLogEmitter を初期化して返す。
	return &stdoutLogEmitter{w: w}
}

// stdoutLogPayload は 1 行分の log 出力スキーマ。
type stdoutLogPayload struct {
	// 出力種別。
	Kind string `json:"kind"`
	// 観測時刻（unix nanoseconds）。
	Timestamp int64 `json:"timestamp"`
	// 重大度テキスト（INFO / WARN / ERROR 等）。
	Severity string `json:"severity"`
	// メッセージ本文。
	Body string `json:"body"`
	// 属性。
	Attributes map[string]string `json:"attributes,omitempty"`
	// スタックトレース（あれば）。
	StackTrace string `json:"stack_trace,omitempty"`
}

// Emit は LogEntry を JSON 1 行に変換して書き出す。
func (e *stdoutLogEmitter) Emit(_ context.Context, entry LogEntry) error {
	// 出力 payload を構築する。
	payload := stdoutLogPayload{
		// 出力種別。
		Kind: "log",
		// 観測時刻。
		Timestamp: entry.Timestamp,
		// 重大度テキスト。
		Severity: entry.SeverityText,
		// 本文。
		Body: entry.Body,
		// 属性。
		Attributes: entry.Attributes,
		// スタックトレース。
		StackTrace: entry.StackTrace,
	}
	// JSON エンコード + 改行を 1 回の write で出す。
	return e.writeLine(payload)
}

// writeLine は payload を JSON エンコードし、改行付きで io.Writer に書く。
func (e *stdoutLogEmitter) writeLine(payload interface{}) error {
	// json.Marshal でバイト列に変換する。
	buf, err := json.Marshal(payload)
	// エンコード失敗は即時返却する。
	if err != nil {
		// error をそのまま返却する。
		return err
	}
	// Mutex で write を直列化する。
	e.mu.Lock()
	defer e.mu.Unlock()
	// 1 件分を書き出す（末尾に改行を付ける）。
	if _, err := e.w.Write(append(buf, '\n')); err != nil {
		// 書込失敗は透過する。
		return err
	}
	// 成功時 nil を返す。
	return nil
}

// stdoutMetricEmitter は MetricEntry を JSON Lines として io.Writer に書き出す。
type stdoutMetricEmitter struct {
	// 出力先。
	w io.Writer
	// write の直列化に使う Mutex。
	mu sync.Mutex
}

// NewStdoutMetricEmitter は io.Writer に書く MetricEmitter を生成する。
func NewStdoutMetricEmitter(w io.Writer) MetricEmitter {
	// stdoutMetricEmitter を初期化して返す。
	return &stdoutMetricEmitter{w: w}
}

// stdoutMetricPayload は 1 行分の metric 出力スキーマ。
type stdoutMetricPayload struct {
	// 出力種別。
	Kind string `json:"kind"`
	// メトリクス名。
	Name string `json:"name"`
	// メトリクス種別（counter / gauge / histogram）。
	MetricKind string `json:"metric_kind"`
	// 値。
	Value float64 `json:"value"`
	// ラベル。
	Labels map[string]string `json:"labels,omitempty"`
}

// Record は MetricEntry を JSON 1 行に変換して書き出す。
func (e *stdoutMetricEmitter) Record(_ context.Context, entry MetricEntry) error {
	// kind を文字列名にする。
	kindName := metricKindName(entry.Kind)
	// payload を構築する。
	payload := stdoutMetricPayload{
		// 出力種別。
		Kind: "metric",
		// メトリクス名。
		Name: entry.Name,
		// メトリクス種別名。
		MetricKind: kindName,
		// 値。
		Value: entry.Value,
		// ラベル。
		Labels: entry.Labels,
	}
	// JSON 1 行を書き出す。
	return writeJSONLine(&e.mu, e.w, payload)
}

// metricKindName は MetricKind を文字列に変換する。
func metricKindName(k MetricKind) string {
	// switch で case ごとの string を返す。
	switch k {
	// Counter 種別。
	case MetricKindCounter:
		// "counter" を返す。
		return "counter"
	// Gauge 種別。
	case MetricKindGauge:
		// "gauge" を返す。
		return "gauge"
	// Histogram 種別。
	case MetricKindHistogram:
		// "histogram" を返す。
		return "histogram"
	// 未知の種別は "unknown" を返す。
	default:
		// "unknown" を返す。
		return "unknown"
	}
}

// stdoutTraceEmitter は SpanEntry を JSON Lines として io.Writer に書き出す。
type stdoutTraceEmitter struct {
	// 出力先。
	w io.Writer
	// write の直列化に使う Mutex。
	mu sync.Mutex
}

// NewStdoutTraceEmitter は io.Writer に書く TraceEmitter を生成する。
func NewStdoutTraceEmitter(w io.Writer) TraceEmitter {
	// stdoutTraceEmitter を初期化して返す。
	return &stdoutTraceEmitter{w: w}
}

// stdoutTracePayload は 1 行分の span 出力スキーマ。
type stdoutTracePayload struct {
	// 出力種別。
	Kind string `json:"kind"`
	// W3C trace_id。
	TraceID string `json:"trace_id,omitempty"`
	// W3C span_id。
	SpanID string `json:"span_id,omitempty"`
	// 親 span_id。
	ParentSpanID string `json:"parent_span_id,omitempty"`
	// span 名。
	Name string `json:"name"`
	// 開始時刻（unix nanoseconds）。
	StartNs int64 `json:"start_ns"`
	// 終了時刻（unix nanoseconds）。
	EndNs int64 `json:"end_ns"`
	// 属性。
	Attributes map[string]string `json:"attributes,omitempty"`
}

// Emit は SpanEntry を JSON 1 行に変換して書き出す。
func (e *stdoutTraceEmitter) Emit(_ context.Context, entry SpanEntry) error {
	// payload を構築する。
	payload := stdoutTracePayload{
		// 出力種別。
		Kind: "span",
		// trace_id。
		TraceID: entry.TraceID,
		// span_id。
		SpanID: entry.SpanID,
		// parent_span_id。
		ParentSpanID: entry.ParentSpanID,
		// span 名。
		Name: entry.Name,
		// 開始時刻。
		StartNs: entry.StartTimeUnixNanos,
		// 終了時刻。
		EndNs: entry.EndTimeUnixNanos,
		// 属性。
		Attributes: entry.Attributes,
	}
	// JSON 1 行を書き出す。
	return writeJSONLine(&e.mu, e.w, payload)
}

// writeJSONLine は Mutex で write を直列化しつつ、JSON エンコード + 改行 1 行を書き出す。
// LogEmitter / MetricEmitter / TraceEmitter で共有する小ユーティリティ。
func writeJSONLine(mu *sync.Mutex, w io.Writer, payload interface{}) error {
	// JSON エンコード。
	buf, err := json.Marshal(payload)
	// エンコード失敗は即時返却する。
	if err != nil {
		// error を透過する。
		return err
	}
	// Mutex で write を直列化する。
	mu.Lock()
	defer mu.Unlock()
	// 1 件分を書き出す（末尾に改行を付ける）。
	if _, err := w.Write(append(buf, '\n')); err != nil {
		// 書込失敗は透過する。
		return err
	}
	// 成功時 nil を返す。
	return nil
}

// StdoutBundle は cmd/state main から 3 emitter をまとめて取得する utility。
// io.Writer 1 つを共有して 3 種別の emit を同じ stream に並べる。
type StdoutBundle struct {
	// 単一エントリ送信。
	LogEmitter LogEmitter
	// メトリクス記録。
	MetricEmitter MetricEmitter
	// span 送信。
	TraceEmitter TraceEmitter
}

// NewStdoutBundle は io.Writer から 3 emitter のバンドルを構築する。
// production では os.Stdout を渡し、test では bytes.Buffer を渡す。
func NewStdoutBundle(w io.Writer) StdoutBundle {
	// 3 emitter を同じ writer で構成する。
	return StdoutBundle{
		// log emitter。
		LogEmitter: NewStdoutLogEmitter(w),
		// metric emitter。
		MetricEmitter: NewStdoutMetricEmitter(w),
		// trace emitter。
		TraceEmitter: NewStdoutTraceEmitter(w),
	}
}
