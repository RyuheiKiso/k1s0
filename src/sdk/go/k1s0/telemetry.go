// 本ファイルは k1s0 Go SDK の Telemetry 動詞統一 facade。
package k1s0

import (
	"context"

	telemetryv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/telemetry/v1"
)

// TelemetryClient は TelemetryService の動詞統一 facade。
type TelemetryClient struct{ client *Client }

// Telemetry は親 Client から TelemetryClient を返す。
func (c *Client) Telemetry() *TelemetryClient { return c.telemetry }

// EmitMetric はメトリクス送信（Counter / Gauge / Histogram の混在可）。
func (t *TelemetryClient) EmitMetric(ctx context.Context, metrics []*telemetryv1.Metric) error {
	_, e := t.client.raw.Telemetry.EmitMetric(ctx, &telemetryv1.EmitMetricRequest{
		Metrics: metrics,
		Context: t.client.tenantContext(),
	})
	return e
}

// EmitSpan は Span 送信（既に終了済 Span のみ）。
func (t *TelemetryClient) EmitSpan(ctx context.Context, spans []*telemetryv1.Span) error {
	_, e := t.client.raw.Telemetry.EmitSpan(ctx, &telemetryv1.EmitSpanRequest{
		Spans:   spans,
		Context: t.client.tenantContext(),
	})
	return e
}
