package middleware

import (
	"fmt"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/prometheus/client_golang/prometheus"
)

var (
	httpRequestsTotal = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "http_requests_total",
			Help: "Total number of HTTP requests.",
		},
		[]string{"method", "path", "status"},
	)

	httpRequestDurationSeconds = prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "http_request_duration_seconds",
			Help:    "HTTP request duration in seconds.",
			Buckets: prometheus.DefBuckets,
		},
		[]string{"method", "path"},
	)
)

func init() {
	prometheus.MustRegister(httpRequestsTotal)
	prometheus.MustRegister(httpRequestDurationSeconds)
}

// PrometheusMiddleware records HTTP request metrics (count and duration).
func PrometheusMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		start := time.Now()

		c.Next()

		duration := time.Since(start).Seconds()
		// c.FullPath() は Gin のルートテンプレート（例: /api/v1/users/:id）を返すため
		// カーディナリティは低く保たれる。ただし未登録パス（"unknown"）が大量に来た場合は
		// 別途アラートで検知すること（H-002）。
		// 改善案: status を "2xx"/"4xx"/"5xx" バケットにすると cardinality をさらに削減できる。
		status := fmt.Sprintf("%d", c.Writer.Status())
		path := c.FullPath()
		if path == "" {
			path = "unknown"
		}

		httpRequestsTotal.WithLabelValues(c.Request.Method, path, status).Inc()
		httpRequestDurationSeconds.WithLabelValues(c.Request.Method, path).Observe(duration)
	}
}
