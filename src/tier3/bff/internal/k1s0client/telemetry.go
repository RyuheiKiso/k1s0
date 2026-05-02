// k1s0 Telemetry ラッパー。
//
// SDK の TelemetryClient.EmitMetric を per-request tenant 伝搬付きで露出する。
// BFF からは Counter のみを最小公開する（Gauge / Histogram は tier2 が直接呼ぶべき）。
// SDK の proto Metric 型を露出させないため、軽量構造体 MetricPoint で受け取り変換する。
// EmitSpan は OTel exporter 経由で自動送信されるため BFF からは公開しない。

package k1s0client

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// 時刻。
	"time"

	// SDK 生成型（Metric / MetricKind）。
	telemetryv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/telemetry/v1"
	// proto timestamp。
	"google.golang.org/protobuf/types/known/timestamppb"
)

// MetricPoint は BFF JSON 入力用の単一メトリクス点（Counter 用途を想定）。
type MetricPoint struct {
	// メトリクス名（OTel 命名規約、ドット区切り。例: k1s0.tier3.bff.requests_total）。
	Name string
	// 加算値（Counter は double）。
	Value float64
	// ラベル（service.name / env / status_code 等の OTel attribute）。
	Labels map[string]string
}

// TelemetryEmitMetric は BFF 用の軽量 MetricPoint 列を SDK proto Metric に変換して送信する。
// SDK 型 (telemetryv1.Metric) を BFF API 表面に出さないことが本ラッパーの責務。
func (c *Client) TelemetryEmitMetric(ctx context.Context, points []MetricPoint) error {
	// 入力が空なら呼出をスキップする（無駄な RPC を発生させない）。
	if len(points) == 0 {
		return nil
	}
	// SDK proto Metric の slice に変換する。
	metrics := make([]*telemetryv1.Metric, 0, len(points))
	// 同一タイムスタンプを再利用する。
	now := timestamppb.New(time.Now().UTC())
	// 各 point を Counter として組み立てる。
	for _, p := range points {
		metrics = append(metrics, &telemetryv1.Metric{
			// メトリクス名。
			Name: p.Name,
			// Counter 種別（BFF からは累計加算値しか公開しない）。
			Kind: telemetryv1.MetricKind_COUNTER,
			// 加算値。
			Value: p.Value,
			// ラベル。
			Labels: p.Labels,
			// 観測時刻（UTC）。
			Timestamp: now,
		})
	}
	// SDK facade を呼ぶ。
	return c.client.Telemetry().EmitMetric(withTenantFromRequest(ctx), metrics)
}
