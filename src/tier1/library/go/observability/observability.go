// k1s0-impl: IMPL-tier1-0003 realizes=FR-tier1-003
// observability.go — k1s0 tier1 Library Go 実装: Logging / Tracing / Metrics の L3 interface
// 01_オブザーバビリティ適合仕様.md §観測信号の 3 型（ログ・トレース・メトリクス）に準拠する。
// OSS 型（zap / otel SDK 等）を公開シグネチャに一切露出しない L3 抽象 interface を宣言する。
// 全シグナルは OpenTelemetry OTLP 経由で collector に送信される（transport 抽象化）。

// パッケージ名: observability（tier1 Library の観測 API を提供する）
package observability

import (
	// context: context.Context（非同期操作 / トレース伝播に使用する）
	"context"
	// time: HLC ベースのタイムスタンプ型に使用する
	"time"
)

// Attr は構造化ログ / スパン / メトリクスに付与するキー・バリュー属性を宣言する型。
// OSS の attribute 型（otel attribute.KeyValue 等）を露出しない独自語彙とする。
type Attr struct {
	// Key: 属性名（小文字 snake_case を推奨する）
	Key string
	// Value: 属性値（string 型のみ受け付ける: 型安全性を優先する）
	Value string
}

// A は Attr を生成するショートハンドヘルパー関数。
// ログ呼び出し側のコードを簡潔にする（A("tenant_id", id) 形式）。
func A(key, value string) Attr {
	// Attr 構造体を生成して返す
	return Attr{Key: key, Value: value}
}

// Severity は構造化ログの重要度レベルを宣言する型。
// OpenTelemetry Log Data Model §SeverityNumber に準拠した語彙とする。
type Severity string

const (
	// SeverityDebug: 詳細デバッグ情報（開発環境での詳細追跡用）
	SeverityDebug Severity = "debug"
	// SeverityInfo: 通常の動作情報（運用ダッシュボードに表示するレベル）
	SeverityInfo Severity = "info"
	// SeverityWarn: 警告（処理は継続するが注意が必要な状態）
	SeverityWarn Severity = "warn"
	// SeverityError: エラー（処理が失敗した状態、alert 対象）
	SeverityError Severity = "error"
	// SeverityFatal: 致命的エラー（アプリケーション停止が必要な状態）
	SeverityFatal Severity = "fatal"
)

// Logger は構造化ログ出力の L3 抽象 interface を宣言する。
// OpenTelemetry OTLP Logs Signal に準拠したシグネチャとする。
// OSS 型（zap.Logger / logrus.Logger 等）を引数・戻り値に一切含まない。
type Logger interface {
	// Debug は SeverityDebug レベルのログを出力する（詳細デバッグ用）。
	// ctx からトレース context を自動伝播する（trace_id / span_id を付与する）。
	Debug(ctx context.Context, msg string, attrs ...Attr)

	// Info は SeverityInfo レベルのログを出力する（通常動作記録用）。
	// ctx からトレース context を自動伝播する（trace_id / span_id を付与する）。
	Info(ctx context.Context, msg string, attrs ...Attr)

	// Warn は SeverityWarn レベルのログを出力する（警告記録用）。
	// ctx からトレース context を自動伝播する（trace_id / span_id を付与する）。
	Warn(ctx context.Context, msg string, attrs ...Attr)

	// Error は SeverityError レベルのログを出力する（エラー記録用）。
	// err は nil でも可（エラーがない場合でも呼び出せるようにする）。
	// ctx からトレース context を自動伝播する（trace_id / span_id を付与する）。
	Error(ctx context.Context, msg string, err error, attrs ...Attr)

	// Fatal は SeverityFatal レベルのログを出力してプロセスを終了する（致命的エラー用）。
	// 実装側は log 出力後に os.Exit(1) 等を呼び出すことを保証する。
	Fatal(ctx context.Context, msg string, err error, attrs ...Attr)

	// With は追加属性を持つ派生 Logger を返す（子 logger を生成する）。
	// 全てのログ出力に attrs が自動付与される（tenant_id 等の常駐属性に使用する）。
	With(attrs ...Attr) Logger
}

// SpanKind は OpenTelemetry SpanKind を宣言する型。
// Server / Client / Producer / Consumer / Internal の 5 種を定義する。
type SpanKind string

const (
	// SpanKindServer: 着信 RPC / HTTP リクエストを受け取る側（Gateway / API 側）
	SpanKindServer SpanKind = "server"
	// SpanKindClient: 発信 RPC / HTTP リクエストを送る側（呼び出し元）
	SpanKindClient SpanKind = "client"
	// SpanKindProducer: Messaging キューへの publish 側
	SpanKindProducer SpanKind = "producer"
	// SpanKindConsumer: Messaging キューからの consume 側
	SpanKindConsumer SpanKind = "consumer"
	// SpanKindInternal: プロセス内部の操作（外部通信なし）
	SpanKindInternal SpanKind = "internal"
)

// Span は OpenTelemetry SpanContext の L3 抽象 interface を宣言する。
// OSS の trace.Span 型を露出しない独自語彙とする。
type Span interface {
	// TraceID は分散トレースの root trace 識別子（16 bytes hex 文字列）を返す。
	TraceID() string

	// SpanID は現在のスパン識別子（8 bytes hex 文字列）を返す。
	SpanID() string

	// SetAttr はスパンにキー・バリュー属性を追加する。
	// 実装側は OpenTelemetry attribute に変換して付与する。
	SetAttr(attrs ...Attr)

	// RecordError はスパンにエラーイベントを記録する。
	// err == nil の場合は何もしない。
	RecordError(err error)

	// End はスパンを終了する（必ず defer span.End() で呼び出す）。
	// 呼び出し後はスパンに対する操作は無効となる。
	End()
}

// Tracer は分散トレーシングの L3 抽象 interface を宣言する。
// OpenTelemetry OTLP Traces Signal に準拠したシグネチャとする。
// OSS の trace.Tracer 型を引数・戻り値に一切含まない。
type Tracer interface {
	// Start はスパンを開始して（ctx with span, span）を返す。
	// 返された ctx をダウンストリームに伝播させることでトレースを繋ぐ。
	// name はスパン名（"service.Method" 等の短い識別子を推奨する）。
	// kind は SpanKind（省略時は SpanKindInternal とする）。
	Start(ctx context.Context, name string, kind SpanKind, attrs ...Attr) (context.Context, Span)
}

// MetricKind はメトリクスの計測種別を宣言する型。
// OpenTelemetry Metrics Data Model §Instrument に準拠した語彙とする。
type MetricKind string

const (
	// MetricKindCounter: 単調増加カウンター（リクエスト数 / エラー数等）
	MetricKindCounter MetricKind = "counter"
	// MetricKindHistogram: 分布計測（レイテンシ / ペイロードサイズ等）
	MetricKindHistogram MetricKind = "histogram"
	// MetricKindGauge: 現在値（接続数 / キューサイズ等）
	MetricKindGauge MetricKind = "gauge"
)

// HistogramBounds は Histogram の境界値スライスを宣言する型。
// 実装側はこのスライスをバケット境界として使用する（nil = 既定バケットを使用する）。
type HistogramBounds []float64

// MetricMeter は L3 メトリクス記録 interface を宣言する。
// OpenTelemetry Metrics SDK の MeterProvider / Meter を隠蔽する。
// OSS 型（prometheus.Counter 等）を公開 API に一切露出しない。
type MetricMeter interface {
	// CounterAdd はカウンターに delta を加算する（delta は正の値のみ許容する）。
	// name は OTLP メトリクス名（"http.server.request.count" 等のドット記法）。
	// attrs は計測ポイントのラベル（tenant_id / method 等）。
	CounterAdd(ctx context.Context, name string, delta float64, attrs ...Attr)

	// HistogramRecord はヒストグラムに観測値を記録する。
	// name は OTLP メトリクス名（"http.server.duration" 等のドット記法）。
	// value は観測値（単位は name に付随する規約に従う）。
	HistogramRecord(ctx context.Context, name string, value float64, attrs ...Attr)

	// GaugeSet はゲージに絶対値を設定する（現在の値を上書きする）。
	// name は OTLP メトリクス名（"process.connections.active" 等のドット記法）。
	// value は現在値（負の値も許容する）。
	GaugeSet(ctx context.Context, name string, value float64, attrs ...Attr)
}

// ObservabilityProvider は Logger / Tracer / MetricMeter を統合した L3 factory interface を宣言する。
// 呼び出し元は Provider から各 signal instrument を取得して使用する。
// OSS の telemetry SDK を直接 import することなく観測信号を取得できる。
type ObservabilityProvider interface {
	// Logger はサービス名を指定して Logger を返す。
	// serviceName は OTLP resource.ServiceName 属性に対応する（"tier1.key_svc" 等）。
	Logger(serviceName string) Logger

	// Tracer はサービス名を指定して Tracer を返す。
	// serviceName は OTLP resource.ServiceName 属性に対応する（"tier1.key_svc" 等）。
	Tracer(serviceName string) Tracer

	// Meter はサービス名を指定して MetricMeter を返す。
	// serviceName は OTLP resource.ServiceName 属性に対応する（"tier1.key_svc" 等）。
	Meter(serviceName string) MetricMeter
}

// LogEntry は Logger.Info / Logger.Error 等で記録するログエントリを宣言する型。
// テスト / ドライラン用の stub Logger が in-memory にエントリを蓄積する際に使用する。
type LogEntry struct {
	// Severity: ログの重要度レベル
	Severity Severity
	// Msg: ログメッセージ
	Msg string
	// Attrs: ログ属性スライス
	Attrs []Attr
	// Err: エラー（SeverityError / SeverityFatal の場合のみ設定される）
	Err error
	// Timestamp: HLC ベースのタイムスタンプ（wall-clock TTL 禁止規約に準拠する）
	// 実装は HLC wrapper を使用する（SystemTime::now() 等の wall clock 直接使用を禁止する）
	Timestamp time.Time
}
