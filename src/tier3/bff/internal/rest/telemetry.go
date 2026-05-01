// Telemetry の REST エンドポイント。
//
//	POST /api/telemetry/emit-metric — Counter メトリクスの送信

package rest

// 標準 / 内部 import。
import (
	// HTTP server。
	"net/http"

	// k1s0client の MetricPoint 型を参照する。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/k1s0client"
)

// metricPointIn は POST /api/telemetry/emit-metric の入力 1 件。
type metricPointIn struct {
	Name   string            `json:"name"`
	Value  float64           `json:"value"`
	Labels map[string]string `json:"labels,omitempty"`
}

// telemetryEmitMetricRequest は POST /api/telemetry/emit-metric の入力。
type telemetryEmitMetricRequest struct {
	Points []metricPointIn `json:"points"`
}

// registerTelemetry は telemetry 系 endpoint を mux に登録する。
func (r *Router) registerTelemetry(mux *http.ServeMux) {
	// Telemetry.EmitMetric。
	mux.HandleFunc("POST /api/telemetry/emit-metric", r.handleTelemetryEmitMetric)
}

// handleTelemetryEmitMetric は POST /api/telemetry/emit-metric を処理する。
func (r *Router) handleTelemetryEmitMetric(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body telemetryEmitMetricRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する（少なくとも 1 件、各 point に name 必須）。
	if len(body.Points) == 0 {
		writeBadRequest(w, "E-T3-BFF-TELEMETRY-100", "at least one point is required")
		return
	}
	for i, p := range body.Points {
		if p.Name == "" {
			writeBadRequest(w, "E-T3-BFF-TELEMETRY-101", "point name is required")
			return
		}
		_ = i
	}
	// k1s0client.MetricPoint に詰め替える。
	points := make([]k1s0client.MetricPoint, 0, len(body.Points))
	for _, p := range body.Points {
		points = append(points, k1s0client.MetricPoint{
			Name:   p.Name,
			Value:  p.Value,
			Labels: p.Labels,
		})
	}
	// facade 経由で SDK を呼ぶ。
	if err := r.facade.TelemetryEmitMetric(req.Context(), points); err != nil {
		writeBadGateway(w, "E-T3-BFF-TELEMETRY-200", "emit metric failed: "+err.Error())
		return
	}
	// 応答 JSON を返す（成功時は空 body）。
	writeJSON(w, http.StatusOK, struct{}{})
}
