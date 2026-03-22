package telemetry

// metrics_test.go: metrics.go のユニットテスト。
// NewMetrics が正しいフィールドを初期化し、MetricsHandler がメトリクスを配信することを検証する。
// 注意: prometheus グローバルレジストリへの重複登録を防ぐため、
// テストバイナリ内で1度だけ NewMetrics を呼び出している。

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// sharedTestMetrics はテストバイナリ内で1度だけ NewMetrics を呼び出す。
// prometheus グローバルレジストリは同一メトリクス名の重複登録を許可しないため、
// テスト関数ごとに NewMetrics を呼ぶと AlreadyRegisteredError が発生する。
// NewMetrics はエラーを返すため、テスト初期化で panic によりエラーを伝播させる。
var sharedTestMetrics = func() *Metrics {
	m, err := NewMetrics("test-telemetry-service")
	if err != nil {
		panic(err)
	}
	return m
}()

// TestNewMetrics_FieldsInitialized は NewMetrics が4つのメトリクスフィールドを全て初期化することを確認する。
func TestNewMetrics_FieldsInitialized(t *testing.T) {
	require.NotNil(t, sharedTestMetrics, "NewMetrics が非 nil を返すこと")
	// HTTP メトリクスが初期化されていること。
	assert.NotNil(t, sharedTestMetrics.HTTPRequestsTotal, "HTTPRequestsTotal が初期化されること")
	assert.NotNil(t, sharedTestMetrics.HTTPRequestDuration, "HTTPRequestDuration が初期化されること")
	// gRPC メトリクスが初期化されていること。
	assert.NotNil(t, sharedTestMetrics.GRPCHandledTotal, "GRPCHandledTotal が初期化されること")
	assert.NotNil(t, sharedTestMetrics.GRPCHandlingDuration, "GRPCHandlingDuration が初期化されること")
}

// TestNewMetrics_HTTPCounterLabels は HTTP カウンターが method/path/status ラベルで
// インクリメント可能なことを確認する（ラベル定義の動作検証）。
func TestNewMetrics_HTTPCounterLabels(t *testing.T) {
	// WithLabelValues が正常に呼べること（パニックしないこと）。
	require.NotPanics(t, func() {
		sharedTestMetrics.HTTPRequestsTotal.WithLabelValues("GET", "/api/v1/test", "200").Inc()
	}, "HTTP カウンターのラベル呼び出しがパニックしないこと")
}

// TestNewMetrics_GRPCCounterLabels は gRPC カウンターが grpc_service/grpc_method/grpc_code ラベルで
// インクリメント可能なことを確認する。
func TestNewMetrics_GRPCCounterLabels(t *testing.T) {
	// gRPC カウンターが正常なラベルで呼べること（パニックしないこと）。
	require.NotPanics(t, func() {
		sharedTestMetrics.GRPCHandledTotal.WithLabelValues("AuthService", "Login", "OK").Inc()
	}, "gRPC カウンターのラベル呼び出しがパニックしないこと")
}

// TestMetricsHandler_ReturnsHandler は MetricsHandler が非 nil の HTTP ハンドラを返すことを確認する。
func TestMetricsHandler_ReturnsHandler(t *testing.T) {
	handler := MetricsHandler()
	require.NotNil(t, handler, "MetricsHandler が非 nil を返すこと")
}

// TestMetricsHandler_ServesPrometheusFormat は /metrics エンドポイントが
// Prometheus テキストフォーマットで 200 を返すことを確認する。
func TestMetricsHandler_ServesPrometheusFormat(t *testing.T) {
	handler := MetricsHandler()
	require.NotNil(t, handler)

	// HTTP テストリクエストでメトリクスエンドポイントが 200 を返すこと。
	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/metrics", nil)
	handler.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code, "メトリクスエンドポイントが 200 を返すこと")
	// Prometheus のメトリクスレスポンスが text/plain 形式であること。
	contentType := w.Header().Get("Content-Type")
	assert.True(t, strings.HasPrefix(contentType, "text/plain"), "Content-Type が text/plain であること")
}
