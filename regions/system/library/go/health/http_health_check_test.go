package health_test

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	health "github.com/k1s0-platform/system-library-go-health"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// HttpHealthCheck_Healthyが200レスポンスを返すエンドポイントに対してヘルスチェックが成功することを検証する。
func TestHttpHealthCheck_Healthy(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	check := health.NewHttpHealthCheck(srv.URL, health.WithName("test-http"))
	assert.Equal(t, "test-http", check.Name())

	err := check.Check(context.Background())
	require.NoError(t, err)
}

// HttpHealthCheck_Unhealthyが503レスポンスを返すエンドポイントに対してエラーを返すことを検証する。
func TestHttpHealthCheck_Unhealthy(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusServiceUnavailable)
	}))
	defer srv.Close()

	check := health.NewHttpHealthCheck(srv.URL, health.WithName("test-http"))
	err := check.Check(context.Background())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "status 503")
}

// HttpHealthCheck_ConnectionRefusedが接続拒否時に「HTTP check failed」を含むエラーを返すことを検証する。
func TestHttpHealthCheck_ConnectionRefused(t *testing.T) {
	check := health.NewHttpHealthCheck("http://127.0.0.1:1", health.WithName("test-http"))
	err := check.Check(context.Background())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "HTTP check failed")
}

// HttpHealthCheck_DefaultNameがオプション未指定時にデフォルト名「http」が設定されることを検証する。
func TestHttpHealthCheck_DefaultName(t *testing.T) {
	check := health.NewHttpHealthCheck("http://example.com")
	assert.Equal(t, "http", check.Name())
}

// HttpHealthCheck_WithTimeoutがWithTimeoutオプションで任意のタイムアウトを設定できることを検証する。
func TestHttpHealthCheck_WithTimeout(t *testing.T) {
	check := health.NewHttpHealthCheck("http://example.com", health.WithTimeout(2*time.Second))
	assert.Equal(t, "http", check.Name())
}

// HttpHealthCheck_IntegrationWithCheckerがCheckerと組み合わせて正常なHTTPエンドポイントをHealthyと判定することを検証する。
func TestHttpHealthCheck_IntegrationWithChecker(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	c := health.NewChecker()
	c.Add(health.NewHttpHealthCheck(srv.URL, health.WithName("upstream")))
	resp := c.RunAll(context.Background())

	assert.Equal(t, health.StatusHealthy, resp.Status)
	assert.Equal(t, health.StatusHealthy, resp.Checks["upstream"].Status)
}
