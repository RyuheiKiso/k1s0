// 本ファイルは tier1 Go ファサードの OTel Traces アダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - DS-SW-COMP-109（k1s0-otel 共通ライブラリ、tracer / propagator 集約）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/08_Telemetry_API.md
//
// 役割:
//   TelemetryService.EmitSpan で受け取った「終了済 Span」（trace_id / span_id /
//   parent_span_id を持つ完全データ）を OTel Tracer に渡して送出する。
//   通常 OTel Tracer.Start は新規 Span を作るが、本 RPC は他プロセスで作成済の
//   Span を後追いで送るため、SDK 内部の SpanProcessor 経路を直接呼ぶ手は使えず、
//   Tracer.Start で SpanContext を override → 即 End する形で擬似的に再構築する。
//
// 制約:
//   - StartTime / EndTime を Tracer.Start の SpanStartOption で指定すれば既存時刻が
//     反映される。Span Kind / Status / Events / Links は今後拡張で追加可能。
//   - SDK が完全準拠で trace_id / span_id を尊重するかどうかは Tracer 実装依存。
//     一般的な OTel SDK Tracer は WithSpanContext で親をセットすると新規 Span は
//     parent としてその context を使うため、emit する Span ID は SDK 側生成になる。
//     完全な「再現 Span」が必要な場合は OTLP exporter を直接叩く別経路（plan 04-13）を要する。
//   - 現状は context に SpanContext を attach し、Span 名 + 開始/終了時刻 + 属性のみを
//     SDK で送る簡易実装に留める（trace_id 完全保持は限定的）。

package otel

import (
	// 全 RPC で context を伝搬する。
	"context"
	// 16 進文字列 → byte 配列の変換に使う。
	"encoding/hex"
	// 観測時刻 conversion。
	"time"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/trace"
)

// SpanEntry は handler が emitter に渡す中間表現。
type SpanEntry struct {
	// W3C trace_id（32 hex chars）。
	TraceID string
	// W3C span_id（16 hex chars）。
	SpanID string
	// 親 Span ID（ルート Span は空文字列）。
	ParentSpanID string
	// Span 名（操作名）。
	Name string
	// 開始時刻（unix nanoseconds、必須）。
	StartTimeUnixNanos int64
	// 終了時刻（unix nanoseconds、必須）。
	EndTimeUnixNanos int64
	// 属性。
	Attributes map[string]string
}

// TraceEmitter は handler が依存する Span 送出 interface。
type TraceEmitter interface {
	Emit(ctx context.Context, entry SpanEntry) error
}

// otelTraceEmitter は OTel Tracer を保持する Span emitter 実装。
type otelTraceEmitter struct {
	tracer trace.Tracer
}

// NewTraceEmitter は OTel Tracer から TraceEmitter を生成する。
func NewTraceEmitter(tracer trace.Tracer) TraceEmitter {
	return &otelTraceEmitter{tracer: tracer}
}

// Emit は SpanEntry を OTel Tracer に渡して Span を生成・終了する。
// trace_id / span_id を context に attach することで「親 Span が trace_id の値を
// 持つ」という形で trace ID を保持する。emit される Span 自体は SDK が新規 ID を
// 採番するため、完全な ID 保持には別経路（OTLP exporter 直接送出）が必要。
func (e *otelTraceEmitter) Emit(ctx context.Context, entry SpanEntry) error {
	// 親 SpanContext を構築（trace_id を保持するため）。
	parent, err := buildSpanContext(entry.TraceID, entry.SpanID)
	if err != nil {
		return err
	}
	if parent.IsValid() {
		// trace_id を子 Span に伝搬させるため、context に親 SpanContext を attach。
		ctx = trace.ContextWithSpanContext(ctx, parent)
	}
	// 開始時刻と終了時刻を SpanStartOption で指定する。
	startTime := time.Unix(0, entry.StartTimeUnixNanos).UTC()
	endTime := time.Unix(0, entry.EndTimeUnixNanos).UTC()

	_, span := e.tracer.Start(ctx, entry.Name,
		trace.WithTimestamp(startTime),
	)
	// 属性を一括追加。
	if attrs := labelsToAttributes(entry.Attributes); len(attrs) > 0 {
		span.SetAttributes(attrs...)
	}
	// 終了時刻を指定して End。
	span.End(trace.WithTimestamp(endTime))
	return nil
}

// buildSpanContext は hex 文字列 trace_id / span_id から OTel SpanContext を作る。
// いずれか不正な hex なら error を返し、空文字なら無効 SpanContext を返す。
func buildSpanContext(traceIDHex, spanIDHex string) (trace.SpanContext, error) {
	// 両方空ならルート Span（無効 context）として扱う。
	if traceIDHex == "" && spanIDHex == "" {
		return trace.SpanContext{}, nil
	}
	var (
		tid trace.TraceID
		sid trace.SpanID
	)
	// trace_id は 32 hex chars (16 bytes)。
	if traceIDHex != "" {
		raw, err := hex.DecodeString(traceIDHex)
		if err != nil || len(raw) != 16 {
			return trace.SpanContext{}, errInvalidTraceID
		}
		copy(tid[:], raw)
	}
	// span_id は 16 hex chars (8 bytes)。
	if spanIDHex != "" {
		raw, err := hex.DecodeString(spanIDHex)
		if err != nil || len(raw) != 8 {
			return trace.SpanContext{}, errInvalidSpanID
		}
		copy(sid[:], raw)
	}
	return trace.NewSpanContext(trace.SpanContextConfig{
		TraceID:    tid,
		SpanID:     sid,
		TraceFlags: trace.FlagsSampled,
	}), nil
}

// 属性 OTel KeyValue 化はメトリクスと共有するため metric.go の labelsToAttributes を流用する。
// 静的検査用の dummy reference（Go の package-internal 関数同士は明示 import 不要）。
var _ = attribute.String
