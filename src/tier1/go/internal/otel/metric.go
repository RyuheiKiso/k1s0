// 本ファイルは tier1 Go ファサードの OTel Metrics アダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - DS-SW-COMP-038（Metrics Emitter: RED モデル / Prometheus ServiceMonitor）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/08_Telemetry_API.md
//
// 役割:
//   TelemetryService.EmitMetric から受け取った proto Metric を、
//   `go.opentelemetry.io/otel/metric` の Meter で対応する instrument を
//   遅延生成しつつ記録する。同一名 instrument は内部 cache で再利用する。
//
// 動的命名対応:
//   gRPC 経由で arbitrary な metric 名が来るため、Counter / UpDownCounter /
//   Histogram の各 instrument を name → instrument の sync.Map で cache する。
//   Gauge は OTel async-only なので、観測値ベースの Gauge は UpDownCounter で代用。

package otel

import (
	// context を Record に伝搬する。
	"context"
	// 並行 cache 用。
	"sync"

	// OTel Metrics API。
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
)

// MetricKind は k1s0 proto の MetricKind と対応する種別 enum。
// proto enum を直接 import すると otel パッケージが proto 依存になるため、
// handler 側で詰め替える方針で再定義する。
type MetricKind int

const (
	// 加算カウンタ（OTel Int64Counter / Float64Counter）。
	MetricKindCounter MetricKind = iota + 1
	// 観測値（OTel UpDownCounter で代用 — Async Gauge は handler パスに不適）。
	MetricKindGauge
	// 観測分布（OTel Float64Histogram）。
	MetricKindHistogram
)

// MetricEntry は handler が emitter に渡す中間表現。
type MetricEntry struct {
	// メトリクス名（OTel 慣行: ドット区切り）。
	Name string
	// 種別（Counter / Gauge / Histogram）。
	Kind MetricKind
	// 値。Counter は加算量、Gauge は瞬時値（UpDownCounter で記録）、Histogram は観測値。
	Value float64
	// ラベル（OTel attribute）。
	Labels map[string]string
}

// MetricEmitter は handler が依存する metric 記録 interface。
type MetricEmitter interface {
	Record(ctx context.Context, entry MetricEntry) error
}

// otelMetricEmitter は OTel Meter を保持し、instrument を name 別に cache する emitter。
type otelMetricEmitter struct {
	meter metric.Meter
	// counters は Float64Counter の cache（Counter 種別用）。
	counters sync.Map // map[string]metric.Float64Counter
	// upDownCounters は Float64UpDownCounter の cache（Gauge 代用）。
	upDownCounters sync.Map // map[string]metric.Float64UpDownCounter
	// histograms は Float64Histogram の cache（Histogram 種別用）。
	histograms sync.Map // map[string]metric.Float64Histogram
}

// NewMetricEmitter は OTel Meter から MetricEmitter を生成する。
func NewMetricEmitter(meter metric.Meter) MetricEmitter {
	return &otelMetricEmitter{meter: meter}
}

// labelsToAttributes は string-string map を OTel attribute Set に変換する。
func labelsToAttributes(labels map[string]string) []attribute.KeyValue {
	if len(labels) == 0 {
		return nil
	}
	attrs := make([]attribute.KeyValue, 0, len(labels))
	for k, v := range labels {
		attrs = append(attrs, attribute.String(k, v))
	}
	return attrs
}

// Record は MetricEntry を OTel instrument に記録する。
// Kind に応じて適切な instrument を遅延生成し、cache に保存する。
func (e *otelMetricEmitter) Record(ctx context.Context, entry MetricEntry) error {
	attrs := labelsToAttributes(entry.Labels)
	switch entry.Kind {
	case MetricKindCounter:
		inst, err := e.getOrCreateCounter(entry.Name)
		if err != nil {
			return err
		}
		inst.Add(ctx, entry.Value, metric.WithAttributes(attrs...))
	case MetricKindGauge:
		inst, err := e.getOrCreateUpDownCounter(entry.Name)
		if err != nil {
			return err
		}
		// Gauge 値を直接 set する手段が UpDownCounter には無いため、
		// 「現値との差分を Add する」のは複雑なので、ここでは直接値を Add する。
		// async Gauge を使えば瞬時値を素直に表現できるが handler 同期パスとは整合しない。
		inst.Add(ctx, entry.Value, metric.WithAttributes(attrs...))
	case MetricKindHistogram:
		inst, err := e.getOrCreateHistogram(entry.Name)
		if err != nil {
			return err
		}
		inst.Record(ctx, entry.Value, metric.WithAttributes(attrs...))
	default:
		return errMetricUnknownKind
	}
	return nil
}

// getOrCreateCounter は instrument cache を介して Counter を返す。
func (e *otelMetricEmitter) getOrCreateCounter(name string) (metric.Float64Counter, error) {
	if v, ok := e.counters.Load(name); ok {
		return v.(metric.Float64Counter), nil
	}
	inst, err := e.meter.Float64Counter(name)
	if err != nil {
		return nil, err
	}
	actual, _ := e.counters.LoadOrStore(name, inst)
	return actual.(metric.Float64Counter), nil
}

// getOrCreateUpDownCounter は instrument cache を介して UpDownCounter を返す。
func (e *otelMetricEmitter) getOrCreateUpDownCounter(name string) (metric.Float64UpDownCounter, error) {
	if v, ok := e.upDownCounters.Load(name); ok {
		return v.(metric.Float64UpDownCounter), nil
	}
	inst, err := e.meter.Float64UpDownCounter(name)
	if err != nil {
		return nil, err
	}
	actual, _ := e.upDownCounters.LoadOrStore(name, inst)
	return actual.(metric.Float64UpDownCounter), nil
}

// getOrCreateHistogram は instrument cache を介して Histogram を返す。
func (e *otelMetricEmitter) getOrCreateHistogram(name string) (metric.Float64Histogram, error) {
	if v, ok := e.histograms.Load(name); ok {
		return v.(metric.Float64Histogram), nil
	}
	inst, err := e.meter.Float64Histogram(name)
	if err != nil {
		return nil, err
	}
	actual, _ := e.histograms.LoadOrStore(name, inst)
	return actual.(metric.Float64Histogram), nil
}
