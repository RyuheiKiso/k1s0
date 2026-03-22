package telemetry

import (
	"fmt"
	"net/http"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

// Metrics は Prometheus メトリクスのヘルパー構造体である。
// RED メソッド（Rate, Errors, Duration）のメトリクスを提供する。
type Metrics struct {
	HTTPRequestsTotal    *prometheus.CounterVec
	HTTPRequestDuration  *prometheus.HistogramVec
	GRPCHandledTotal     *prometheus.CounterVec
	GRPCHandlingDuration *prometheus.HistogramVec
}

// NewMetrics は Prometheus メトリクスを初期化して返す。
// serviceName はメトリクスの service ラベルに使用される。
// MustRegister の代わりに Register を使用し、AlreadyRegisteredError を無視することで
// テストの並列実行時やサービス再起動時の二重登録によるパニックを防止する。
func NewMetrics(serviceName string) (*Metrics, error) {
	m := &Metrics{
		// HTTP リクエスト数カウンター: method/path/status ラベルで分類
		HTTPRequestsTotal: prometheus.NewCounterVec(
			prometheus.CounterOpts{
				Name:        "http_requests_total",
				Help:        "Total number of HTTP requests",
				ConstLabels: prometheus.Labels{"service": serviceName},
			},
			[]string{"method", "path", "status"},
		),
		// HTTP リクエスト処理時間ヒストグラム: レイテンシ分布を計測
		HTTPRequestDuration: prometheus.NewHistogramVec(
			prometheus.HistogramOpts{
				Name:        "http_request_duration_seconds",
				Help:        "Histogram of HTTP request latency",
				ConstLabels: prometheus.Labels{"service": serviceName},
				Buckets:     []float64{0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10},
			},
			[]string{"method", "path"},
		),
		// gRPC 処理済みリクエスト数カウンター: service/method/status_code ラベルで分類
		GRPCHandledTotal: prometheus.NewCounterVec(
			prometheus.CounterOpts{
				Name:        "grpc_server_handled_total",
				Help:        "Total number of RPCs completed on the server",
				ConstLabels: prometheus.Labels{"service": serviceName},
			},
			[]string{"grpc_service", "grpc_method", "grpc_code"},
		),
		// gRPC 処理時間ヒストグラム: レスポンスレイテンシ分布を計測
		GRPCHandlingDuration: prometheus.NewHistogramVec(
			prometheus.HistogramOpts{
				Name:        "grpc_server_handling_seconds",
				Help:        "Histogram of response latency of gRPC",
				ConstLabels: prometheus.Labels{"service": serviceName},
				Buckets:     []float64{0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10},
			},
			[]string{"grpc_service", "grpc_method"},
		),
	}

	// 各メトリクスを登録する。既に登録済みの場合はエラーを無視する（テスト並列実行対策）
	// AlreadyRegisteredError 以外のエラー（例: メトリクス定義の競合）は伝播させる
	collectors := []prometheus.Collector{
		m.HTTPRequestsTotal,
		m.HTTPRequestDuration,
		m.GRPCHandledTotal,
		m.GRPCHandlingDuration,
	}
	for _, c := range collectors {
		if err := prometheus.Register(c); err != nil {
			// 既存メトリクスが登録済みの場合はエラーを無視する（テスト並列実行対策）
			if _, ok := err.(prometheus.AlreadyRegisteredError); !ok {
				return nil, fmt.Errorf("prometheus.Register failed: %w", err)
			}
		}
	}

	return m, nil
}

// MetricsHandler は /metrics エンドポイント用の HTTP ハンドラを返す。
func MetricsHandler() http.Handler {
	return promhttp.Handler()
}
