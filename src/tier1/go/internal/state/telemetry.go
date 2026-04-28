// 本ファイルは t1-state Pod の TelemetryService 2 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/08_Telemetry_API.md
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - DS-SW-COMP-038（Metrics Emitter: RED モデル / Prometheus ServiceMonitor）
//   src/tier1/README.md（t1-state Pod の責務に Telemetry を含む）
//
// scope（リリース時点 placeholder）:
//   実 OTel Collector → Mimir / Tempo への結線は plan 04-13 同期。
//   現状は SDK 接続点を提供するため skeleton として登録し、全 RPC は Unimplemented を返す。
//   FR-T1-TELEMETRY-001〜004 のうち SDK 側 facade（src/sdk/*/telemetry）は同梱済、
//   本 handler はそれを受け止める空 server として機能する。

package state

// 標準 / 内部パッケージ。
import (
	// context 伝搬。
	"context"
	// SDK 生成 stub の TelemetryService 型。
	telemetryv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/telemetry/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// telemetryHandler は TelemetryService の handler 実装。
// Telemetry は OTel Collector を経由して Mimir（メトリクス）/ Tempo（トレース）へ
// 流す設計のため、本 handler は SDK 側 facade からの gRPC 入口を確保するだけで、
// 本格実装は plan 04-13 で OTel SDK / Collector 連携の文脈で行う。
type telemetryHandler struct {
	// 将来 RPC 用埋め込み（forward compatibility）。
	telemetryv1.UnimplementedTelemetryServiceServer
	// adapter 集合（Telemetry は OTel 側へ流すため Dapr adapter は使わない）。
	deps Deps
}

// EmitMetric はメトリクス送信（Counter / Gauge / Histogram の混在可）。
func (h *telemetryHandler) EmitMetric(_ context.Context, req *telemetryv1.EmitMetricRequest) (*telemetryv1.EmitMetricResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/telemetry: nil request")
	}
	// 実 OTel Collector 結線は plan 04-13。
	return nil, status.Error(codes.Unimplemented, "tier1/telemetry: EmitMetric not yet wired to OTel Collector (plan 04-13)")
}

// EmitSpan は終了済 Span の送信。
func (h *telemetryHandler) EmitSpan(_ context.Context, req *telemetryv1.EmitSpanRequest) (*telemetryv1.EmitSpanResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/telemetry: nil request")
	}
	// 実 OTel Collector 結線は plan 04-13。
	return nil, status.Error(codes.Unimplemented, "tier1/telemetry: EmitSpan not yet wired to OTel Collector (plan 04-13)")
}
