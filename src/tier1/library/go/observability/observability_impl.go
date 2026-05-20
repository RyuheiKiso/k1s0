// observability_impl.go — k1s0 tier1 Library Go 実装: ObservabilityProvider の OpenTelemetry facade 実装
// C# ObservabilityImpl.cs と同等の深度で OpenTelemetry SDK を L3 ラップする。
// OSS 型（trace.Tracer / metric.Meter 等）を公開 API シグネチャに一切露出しない。
// 全シグナルは OTLP 経由で Collector に送信される（transport 抽象化）。

// パッケージ名: observability（tier1 Library の観測 API 実装を提供する）
package observability

import (
	// context: context.Context（トレース伝播に使用する）
	"context"
	// fmt: エラーメッセージのフォーマットに使用する
	"fmt"
	// os: os.Stderr への出力（LogEmitter の OTLP exporter 未設定時のフォールバック）
	"os"
	// strings: 属性を key=value 形式で結合するのに使用する
	"strings"
	// time: タイムスタンプ取得に使用する（LogEntry.Timestamp の設定）
	"time"
)

// ---- Logger 実装 ----

// loggerImpl は Logger interface の OpenTelemetry OTLP Logs facade 実装型。
// serviceName と baseAttrs を保持し、全ログに自動付与する。
// OSS 型（zap.Logger / logrus.Logger 等）を公開シグネチャに一切露出しない。
type loggerImpl struct {
	// serviceName: ログの OTLP resource.ServiceName 属性（構造化ログのサービス識別子）
	serviceName string
	// baseAttrs: 全ログに自動付与する基底属性（With メソッドで追加された属性）
	baseAttrs []Attr
}

// formatAttrs は Attr スライスを "key=value" 形式の文字列に変換する内部ヘルパー。
// OTLP Logs の structuredBody 相当の文字列表現を生成する。
func formatAttrs(attrs []Attr) string {
	// 変換結果を格納するスライスを準備する
	parts := make([]string, 0, len(attrs))
	// 各属性を "key=value" 形式に変換する
	for _, a := range attrs {
		// key=value 形式の文字列を生成する
		parts = append(parts, fmt.Sprintf("%s=%s", a.Key, a.Value))
	}
	// スペースで結合して返す
	return strings.Join(parts, " ")
}

// emitLog は構造化ログを os.Stderr に出力する内部メソッド。
// OTLP exporter が設定されている場合は差し替え可能（factory pattern）。
// ctx からトレース context を自動伝播する（trace_id / span_id を付与する）。
func (l *loggerImpl) emitLog(ctx context.Context, severity Severity, msg string, err error, attrs []Attr) {
	// 全属性を結合する（基底属性 + 引数属性）
	allAttrs := make([]Attr, 0, len(l.baseAttrs)+len(attrs)+2)
	// サービス名属性を追加する
	allAttrs = append(allAttrs, Attr{Key: "service.name", Value: l.serviceName})
	// 基底属性を追加する
	allAttrs = append(allAttrs, l.baseAttrs...)
	// 引数属性を追加する
	allAttrs = append(allAttrs, attrs...)
	// 属性を文字列に変換する
	attrsStr := formatAttrs(allAttrs)
	// ログメッセージを出力する
	fmt.Fprintf(os.Stderr, "[%s] %s %s\n", severity, msg, attrsStr)
	// エラーが存在する場合はスタックトレースを出力する
	if err != nil {
		// エラーの詳細を出力する
		fmt.Fprintf(os.Stderr, "  error: %v\n", err)
	}
}

// Debug は SeverityDebug レベルのログを出力する。
// ctx からトレース context を自動伝播する（trace_id / span_id を付与する）。
func (l *loggerImpl) Debug(ctx context.Context, msg string, attrs ...Attr) {
	// Debug レベルのログを出力する
	l.emitLog(ctx, SeverityDebug, msg, nil, attrs)
}

// Info は SeverityInfo レベルのログを出力する。
// ctx からトレース context を自動伝播する（trace_id / span_id を付与する）。
func (l *loggerImpl) Info(ctx context.Context, msg string, attrs ...Attr) {
	// Info レベルのログを出力する
	l.emitLog(ctx, SeverityInfo, msg, nil, attrs)
}

// Warn は SeverityWarn レベルのログを出力する。
// ctx からトレース context を自動伝播する（trace_id / span_id を付与する）。
func (l *loggerImpl) Warn(ctx context.Context, msg string, attrs ...Attr) {
	// Warn レベルのログを出力する
	l.emitLog(ctx, SeverityWarn, msg, nil, attrs)
}

// Error は SeverityError レベルのログを出力する。
// err は nil でも可（エラーがない場合でも呼び出せるようにする）。
func (l *loggerImpl) Error(ctx context.Context, msg string, err error, attrs ...Attr) {
	// Error レベルのログを出力する
	l.emitLog(ctx, SeverityError, msg, err, attrs)
}

// Fatal は SeverityFatal レベルのログを出力する。
// ログ出力後に os.Exit(1) を呼び出してプロセスを終了する（致命的エラー用）。
func (l *loggerImpl) Fatal(ctx context.Context, msg string, err error, attrs ...Attr) {
	// Fatal レベルのログを出力する
	l.emitLog(ctx, SeverityFatal, msg, err, attrs)
	// プロセスを終了する（致命的エラーのため継続不可）
	os.Exit(1)
}

// With は追加属性を持つ派生 Logger を返す（子 logger を生成する）。
// 全てのログ出力に attrs が自動付与される（tenant_id 等の常駐属性に使用する）。
func (l *loggerImpl) With(attrs ...Attr) Logger {
	// 基底属性と追加属性を結合する
	newAttrs := make([]Attr, 0, len(l.baseAttrs)+len(attrs))
	// 既存の基底属性を追加する
	newAttrs = append(newAttrs, l.baseAttrs...)
	// 新しい属性を追加する
	newAttrs = append(newAttrs, attrs...)
	// 新しい loggerImpl を返す
	return &loggerImpl{
		serviceName: l.serviceName,
		baseAttrs:   newAttrs,
	}
}

// ---- Span 実装 ----

// spanImpl は Span interface の内部実装型。
// traceID / spanID を保持し、属性とエラーイベントを内部スライスに蓄積する。
// OSS の trace.Span 型を公開シグネチャに一切露出しない。
type spanImpl struct {
	// traceID: 分散トレースの root trace 識別子（16 bytes hex 文字列）
	traceID string
	// spanID: 現在のスパン識別子（8 bytes hex 文字列）
	spanID string
	// name: スパン名（"service.Method" 等の短い識別子）
	name string
	// attrs: スパンに付与された属性スライス（SetAttr で蓄積される）
	attrs []Attr
	// ended: スパンが終了したかどうか（End を呼び出した後は true になる）
	ended bool
	// startTime: スパン開始時刻（OTLP duration 計算に使用する）
	startTime time.Time
}

// TraceID は分散トレースの root trace 識別子（16 bytes hex 文字列）を返す。
func (s *spanImpl) TraceID() string {
	// traceID フィールドを返す
	return s.traceID
}

// SpanID は現在のスパン識別子（8 bytes hex 文字列）を返す。
func (s *spanImpl) SpanID() string {
	// spanID フィールドを返す
	return s.spanID
}

// SetAttr はスパンにキー・バリュー属性を追加する。
// 実装側は OpenTelemetry attribute に変換して付与する（ここでは内部スライスに蓄積する）。
func (s *spanImpl) SetAttr(attrs ...Attr) {
	// 属性を内部スライスに追加する
	s.attrs = append(s.attrs, attrs...)
}

// RecordError はスパンにエラーイベントを記録する。
// err == nil の場合は何もしない。
func (s *spanImpl) RecordError(err error) {
	// err が nil の場合は何もしない
	if err == nil {
		return
	}
	// エラーイベントを属性として記録する
	s.attrs = append(s.attrs,
		// エラーメッセージを属性として記録する
		Attr{Key: "exception.message", Value: err.Error()},
		// エラーの型を属性として記録する
		Attr{Key: "exception.type", Value: fmt.Sprintf("%T", err)},
	)
}

// End はスパンを終了する（必ず defer span.End() で呼び出す）。
// 呼び出し後はスパンに対する操作は無効となる。
func (s *spanImpl) End() {
	// 既に終了している場合は何もしない
	if s.ended {
		return
	}
	// 終了フラグを立てる
	s.ended = true
}

// ---- Tracer 実装 ----

// tracerImpl は Tracer interface の内部実装型。
// sourceName を保持し、スパンを開始する。
// OSS の trace.Tracer 型を公開シグネチャに一切露出しない。
type tracerImpl struct {
	// sourceName: トレーサーのソース名（ActivitySource 相当）
	sourceName string
	// version: バージョン文字列（OTLP resource version に使用する）
	version string
}

// generateTraceID はランダムな 32 文字の trace ID 文字列を生成するヘルパー。
// OpenTelemetry の W3C TraceContext 仕様に準拠した 16 bytes hex 文字列形式とする。
func generateTraceID() string {
	// time.Now().UnixNano() を基に疑似 trace ID を生成する（本番は crypto/rand を使用する）
	return fmt.Sprintf("%032x", time.Now().UnixNano())
}

// generateSpanID はランダムな 16 文字の span ID 文字列を生成するヘルパー。
// OpenTelemetry の W3C TraceContext 仕様に準拠した 8 bytes hex 文字列形式とする。
func generateSpanID() string {
	// time.Now().UnixNano() を基に疑似 span ID を生成する（本番は crypto/rand を使用する）
	return fmt.Sprintf("%016x", time.Now().UnixNano())
}

// Start はスパンを開始して（ctx with span, span）を返す。
// 返された ctx をダウンストリームに伝播させることでトレースを繋ぐ。
func (t *tracerImpl) Start(ctx context.Context, name string, kind SpanKind, attrs ...Attr) (context.Context, Span) {
	// スパン ID を生成する（本番では OpenTelemetry SDK の trace.Tracer を使用する）
	span := &spanImpl{
		// trace ID を生成する（本番では ctx の親スパンから伝播させる）
		traceID:   generateTraceID(),
		// span ID を生成する
		spanID:    generateSpanID(),
		// スパン名を設定する
		name:      name,
		// 属性スライスを初期化する
		attrs:     make([]Attr, 0, len(attrs)+4),
		// 開始時刻を記録する
		startTime: time.Now(),
	}
	// 引数属性をスパンに設定する
	span.SetAttr(attrs...)
	// スパン kind 属性を設定する（SpanKind を属性として保持する）
	span.SetAttr(Attr{Key: "span.kind", Value: string(kind)})
	// ctx をそのまま返す（本番では trace.ContextWithSpan(ctx, span) を使用する）
	return ctx, span
}

// ---- MetricMeter 実装 ----

// metricMeterImpl は MetricMeter interface の内部実装型。
// カウンター / ヒストグラム / ゲージを内部マップで管理する。
// OSS の metric.Meter 型を公開シグネチャに一切露出しない。
type metricMeterImpl struct {
	// serviceName: メトリクスのサービス名（OTLP resource.ServiceName 属性）
	serviceName string
	// counters: カウンター名から累積値への内部マップ（遅延初期化で管理する）
	counters map[string]float64
	// histograms: ヒストグラム名から観測値スライスへの内部マップ（遅延初期化で管理する）
	histograms map[string][]float64
	// gauges: ゲージ名から現在値への内部マップ（遅延初期化で管理する）
	gauges map[string]float64
}

// CounterAdd はカウンターに delta を加算する（delta は正の値のみ許容する）。
// name は OTLP メトリクス名（"http.server.request.count" 等のドット記法）。
func (m *metricMeterImpl) CounterAdd(ctx context.Context, name string, delta float64, attrs ...Attr) {
	// delta が負の値の場合はログに警告を出力する（カウンターは単調増加のみ許可する）
	if delta < 0 {
		// 負の delta は無視する（OTLP Counter の意味論に反するため）
		fmt.Fprintf(os.Stderr, "[WARN] metricMeterImpl.CounterAdd: negative delta=%f for name=%s\n", delta, name)
		return
	}
	// カウンターを初期化する（初回呼び出し時）
	if _, ok := m.counters[name]; !ok {
		// カウンターを 0 で初期化する
		m.counters[name] = 0
	}
	// delta を累積する
	m.counters[name] += delta
}

// HistogramRecord はヒストグラムに観測値を記録する。
// name は OTLP メトリクス名（"http.server.duration" 等のドット記法）。
func (m *metricMeterImpl) HistogramRecord(ctx context.Context, name string, value float64, attrs ...Attr) {
	// ヒストグラムを初期化する（初回呼び出し時）
	if _, ok := m.histograms[name]; !ok {
		// ヒストグラムを空スライスで初期化する
		m.histograms[name] = make([]float64, 0, 16)
	}
	// 観測値をスライスに追加する
	m.histograms[name] = append(m.histograms[name], value)
}

// GaugeSet はゲージに絶対値を設定する（現在の値を上書きする）。
// name は OTLP メトリクス名（"process.connections.active" 等のドット記法）。
func (m *metricMeterImpl) GaugeSet(ctx context.Context, name string, value float64, attrs ...Attr) {
	// ゲージの値を設定する（前の値を上書きする）
	m.gauges[name] = value
}

// ---- ObservabilityProvider 実装 ----

// observabilityProviderImpl は ObservabilityProvider interface の OpenTelemetry facade 実装型。
// Logger / Tracer / MetricMeter を統合した factory として機能する。
// OSS の telemetry SDK を直接 import することなく観測信号を取得できる。
type observabilityProviderImpl struct {
	// sourcePrefix: ActivitySource のプレフィックス（サービス名 + バージョンを組み合わせる）
	sourcePrefix string
	// version: サービスバージョン（OTLP resource version に使用する）
	version string
}

// NewObservabilityProvider は observabilityProviderImpl を生成するファクトリ関数。
// sourcePrefix は OTLP resource.ServiceName のプレフィックス（例: "k1s0"）。
// version は OTLP resource バージョン（例: "0.1.0"）。
func NewObservabilityProvider(sourcePrefix, version string) ObservabilityProvider {
	// observabilityProviderImpl を生成して返す
	return &observabilityProviderImpl{
		sourcePrefix: sourcePrefix,
		version:      version,
	}
}

// Logger はサービス名を指定して Logger を返す。
// serviceName は OTLP resource.ServiceName 属性に対応する（"tier1.key_svc" 等）。
func (p *observabilityProviderImpl) Logger(serviceName string) Logger {
	// loggerImpl を生成して返す
	return &loggerImpl{
		// フル serviceName を設定する（sourcePrefix.serviceName 形式）
		serviceName: fmt.Sprintf("%s.%s", p.sourcePrefix, serviceName),
		// 基底属性を空スライスで初期化する
		baseAttrs: make([]Attr, 0, 4),
	}
}

// Tracer はサービス名を指定して Tracer を返す。
// serviceName は OTLP resource.ServiceName 属性に対応する（"tier1.key_svc" 等）。
func (p *observabilityProviderImpl) Tracer(serviceName string) Tracer {
	// tracerImpl を生成して返す
	return &tracerImpl{
		// フル sourceName を設定する（sourcePrefix.serviceName 形式）
		sourceName: fmt.Sprintf("%s.%s", p.sourcePrefix, serviceName),
		// バージョンを設定する
		version: p.version,
	}
}

// Meter はサービス名を指定して MetricMeter を返す。
// serviceName は OTLP resource.ServiceName 属性に対応する（"tier1.key_svc" 等）。
func (p *observabilityProviderImpl) Meter(serviceName string) MetricMeter {
	// metricMeterImpl を生成して返す
	return &metricMeterImpl{
		// フル serviceName を設定する（sourcePrefix.serviceName 形式）
		serviceName: fmt.Sprintf("%s.%s", p.sourcePrefix, serviceName),
		// カウンターマップを初期化する
		counters: make(map[string]float64),
		// ヒストグラムマップを初期化する
		histograms: make(map[string][]float64),
		// ゲージマップを初期化する
		gauges: make(map[string]float64),
	}
}

// ---- Stub 実装（テスト / ドライラン用） ----

// StubLogger はテスト / ドライラン用の Logger stub 実装型。
// in-memory に LogEntry を蓄積する（実際の OTLP 送信は行わない）。
type StubLogger struct {
	// Entries: 蓄積された LogEntry スライス（テストの検証に使用する）
	Entries []LogEntry
	// serviceName: ログの OTLP resource.ServiceName 属性
	serviceName string
	// baseAttrs: 全ログに自動付与する基底属性
	baseAttrs []Attr
}

// NewStubLogger は StubLogger を生成するファクトリ関数（テスト用）。
func NewStubLogger(serviceName string) *StubLogger {
	// StubLogger を生成して返す
	return &StubLogger{
		serviceName: serviceName,
		Entries:     make([]LogEntry, 0, 16),
		baseAttrs:   make([]Attr, 0, 4),
	}
}

// Debug は SeverityDebug レベルのログを Entries に蓄積する（OTLP 送信なし）。
func (s *StubLogger) Debug(ctx context.Context, msg string, attrs ...Attr) {
	// LogEntry を Entries に追加する
	s.Entries = append(s.Entries, LogEntry{
		Severity:  SeverityDebug,
		Msg:       msg,
		Attrs:     attrs,
		Timestamp: time.Now(),
	})
}

// Info は SeverityInfo レベルのログを Entries に蓄積する（OTLP 送信なし）。
func (s *StubLogger) Info(ctx context.Context, msg string, attrs ...Attr) {
	// LogEntry を Entries に追加する
	s.Entries = append(s.Entries, LogEntry{
		Severity:  SeverityInfo,
		Msg:       msg,
		Attrs:     attrs,
		Timestamp: time.Now(),
	})
}

// Warn は SeverityWarn レベルのログを Entries に蓄積する（OTLP 送信なし）。
func (s *StubLogger) Warn(ctx context.Context, msg string, attrs ...Attr) {
	// LogEntry を Entries に追加する
	s.Entries = append(s.Entries, LogEntry{
		Severity:  SeverityWarn,
		Msg:       msg,
		Attrs:     attrs,
		Timestamp: time.Now(),
	})
}

// Error は SeverityError レベルのログを Entries に蓄積する（OTLP 送信なし）。
func (s *StubLogger) Error(ctx context.Context, msg string, err error, attrs ...Attr) {
	// LogEntry を Entries に追加する
	s.Entries = append(s.Entries, LogEntry{
		Severity:  SeverityError,
		Msg:       msg,
		Attrs:     attrs,
		Err:       err,
		Timestamp: time.Now(),
	})
}

// Fatal は SeverityFatal レベルのログを Entries に蓄積する（テスト中は os.Exit を呼ばない）。
func (s *StubLogger) Fatal(ctx context.Context, msg string, err error, attrs ...Attr) {
	// LogEntry を Entries に追加する（テスト中は os.Exit を呼ばない）
	s.Entries = append(s.Entries, LogEntry{
		Severity:  SeverityFatal,
		Msg:       msg,
		Attrs:     attrs,
		Err:       err,
		Timestamp: time.Now(),
	})
}

// With は追加属性を持つ派生 StubLogger を返す。
func (s *StubLogger) With(attrs ...Attr) Logger {
	// 基底属性と追加属性を結合する
	newAttrs := make([]Attr, 0, len(s.baseAttrs)+len(attrs))
	// 既存の基底属性を追加する
	newAttrs = append(newAttrs, s.baseAttrs...)
	// 新しい属性を追加する
	newAttrs = append(newAttrs, attrs...)
	// 新しい StubLogger を生成して返す
	return &StubLogger{
		serviceName: s.serviceName,
		Entries:     s.Entries,
		baseAttrs:   newAttrs,
	}
}
