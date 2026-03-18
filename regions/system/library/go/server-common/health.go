package servercommon

import (
	"context"
	"encoding/json"
	"net/http"
	"sync"
	"time"
)

// HealthStatus はヘルスチェックレスポンスの構造体。
type HealthStatus struct {
	Status  string            `json:"status"`
	Service string            `json:"service,omitempty"`
	Checks  map[string]string `json:"checks,omitempty"`
}

// HealthChecker は依存サービスの死活確認を行うインターフェース。
type HealthChecker interface {
	// Check は依存サービスの状態を確認し、異常時にエラーを返す。
	Check(ctx context.Context) error
	// Name は依存サービスの名前を返す。
	Name() string
}

// readinessCheckers は readyz エンドポイントで検査する依存サービス一覧。
var (
	readinessCheckersMu sync.RWMutex
	readinessCheckers   []HealthChecker
)

// RegisterReadinessChecker は readyz の依存チェックを登録する。
func RegisterReadinessChecker(checker HealthChecker) {
	readinessCheckersMu.Lock()
	defer readinessCheckersMu.Unlock()
	readinessCheckers = append(readinessCheckers, checker)
}

// RegisterHealthHandlers は /healthz と /readyz エンドポイントを mux に登録する。
func RegisterHealthHandlers(mux *http.ServeMux, serviceName string) {
	// liveness probe: サービスが起動しているか確認する（軽量チェックのみ）
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(HealthStatus{Status: "ok", Service: serviceName})
	})

	// readiness probe: 依存サービス（DB/Redis/Kafka）の死活を確認する
	mux.HandleFunc("/readyz", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")

		ctx, cancel := context.WithTimeout(r.Context(), 5*time.Second)
		defer cancel()

		readinessCheckersMu.RLock()
		checkers := readinessCheckers
		readinessCheckersMu.RUnlock()

		checks := make(map[string]string)
		allOk := true
		for _, checker := range checkers {
			if err := checker.Check(ctx); err != nil {
				checks[checker.Name()] = err.Error()
				allOk = false
			} else {
				checks[checker.Name()] = "ok"
			}
		}

		status := "ok"
		httpStatus := http.StatusOK
		if !allOk {
			status = "not_ready"
			httpStatus = http.StatusServiceUnavailable
		}

		w.WriteHeader(httpStatus)
		json.NewEncoder(w).Encode(HealthStatus{
			Status:  status,
			Service: serviceName,
			Checks:  checks,
		})
	})
}
