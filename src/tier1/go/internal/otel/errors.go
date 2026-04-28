// 本ファイルは otel adapter 内部で使うセンチネルエラー集合。

package otel

import "errors"

// errMetricUnknownKind は MetricEntry.Kind が未定義値の時に返される。
var errMetricUnknownKind = errors.New("otel: unknown metric kind")

// errInvalidTraceID は SpanEntry.TraceID が不正 hex 文字列の時に返される。
var errInvalidTraceID = errors.New("otel: invalid trace_id (must be 32 hex chars)")

// errInvalidSpanID は SpanEntry.SpanID が不正 hex 文字列の時に返される。
var errInvalidSpanID = errors.New("otel: invalid span_id (must be 16 hex chars)")
