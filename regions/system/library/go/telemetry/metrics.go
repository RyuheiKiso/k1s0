package telemetry

import (
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
func NewMetrics(serviceName string) *Metrics {
	m := &Metrics{
		HTTPRequestsTotal: prometheus.NewCounterVec(
			prometheus.CounterOpts{
				Name:        "http_requests_total",
				Help:        "Total number of HTTP requests",
				ConstLabels: prometheus.Labels{"service": serviceName},
			},
			[]string{"method", "path", "status"},
		),
		HTTPRequestDuration: prometheus.NewHistogramVec(
			prometheus.HistogramOpts{
				Name:        "http_request_duration_seconds",
				Help:        "Histogram of HTTP request latency",
				ConstLabels: prometheus.Labels{"service": serviceName},
				Buckets:     []float64{0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10},
			},
			[]string{"method", "path"},
		),
		GRPCHandledTotal: prometheus.NewCounterVec(
			prometheus.CounterOpts{
				Name:        "grpc_server_handled_total",
				Help:        "Total number of RPCs completed on the server",
				ConstLabels: prometheus.Labels{"service": serviceName},
			},
			[]string{"grpc_service", "grpc_method", "grpc_code"},
		),
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

	prometheus.MustRegister(m.HTTPRequestsTotal)
	prometheus.MustRegister(m.HTTPRequestDuration)
	prometheus.MustRegister(m.GRPCHandledTotal)
	prometheus.MustRegister(m.GRPCHandlingDuration)

	return m
}

// MetricsHandler は /metrics エンドポイント用の HTTP ハンドラを返す。
func MetricsHandler() http.Handler {
	return promhttp.Handler()
}
