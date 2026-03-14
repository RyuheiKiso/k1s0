package servercommon

import (
	"encoding/json"
	"net/http"
)

// HealthStatus はヘルスチェックレスポンスの構造体。
type HealthStatus struct {
	Status  string `json:"status"`
	Service string `json:"service,omitempty"`
}

// RegisterHealthHandlers は /healthz と /readyz エンドポイントを mux に登録する。
func RegisterHealthHandlers(mux *http.ServeMux, serviceName string) {
	// liveness probe: サービスが起動しているか確認する
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(HealthStatus{Status: "ok", Service: serviceName})
	})

	// readiness probe: サービスがリクエストを受け付けられるか確認する
	mux.HandleFunc("/readyz", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(HealthStatus{Status: "ok", Service: serviceName})
	})
}
