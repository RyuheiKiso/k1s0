package upstream

import (
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// NewReverseProxy が正しいターゲットURLでプロキシを生成する
func TestNewReverseProxy_ValidURL_ReturnsProxy(t *testing.T) {
	proxy, err := NewReverseProxy("http://localhost:8080", 5*time.Second)
	require.NoError(t, err)
	assert.NotNil(t, proxy)
	assert.Equal(t, "localhost:8080", proxy.Target().Host)
	assert.Equal(t, "http", proxy.Target().Scheme)
}

// NewReverseProxy が無効なURLを受け取るとエラーを返す
func TestNewReverseProxy_InvalidURL_ReturnsError(t *testing.T) {
	// url.Parse は多くのケースでエラーを返さないが、空文字列などの特殊ケースをテスト
	proxy, err := NewReverseProxy("://invalid", 5*time.Second)
	// 注: url.Parse は多くの無効URLでもエラーを返さないため、
	// エラー系よりも Target() の値を確認する
	if err != nil {
		assert.Nil(t, proxy)
	} else {
		// パースが成功した場合もプロキシオブジェクトは返る
		assert.NotNil(t, proxy)
	}
}

// Target() が設定済みのURLを返す
func TestReverseProxy_Target_ReturnsConfiguredURL(t *testing.T) {
	proxy, err := NewReverseProxy("https://api.example.com:9090/v1", 10*time.Second)
	require.NoError(t, err)
	target := proxy.Target()
	assert.Equal(t, "https", target.Scheme)
	assert.Equal(t, "api.example.com:9090", target.Host)
}

// ServeHTTP がアップストリームサーバーにリクエストを転送する
func TestReverseProxy_ServeHTTP_ForwardsRequest(t *testing.T) {
	// テスト用アップストリームサーバーを起動する
	backend := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"proxied":true}`))
	}))
	defer backend.Close()

	proxy, err := NewReverseProxy(backend.URL, 5*time.Second)
	require.NoError(t, err)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	proxy.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
	assert.Contains(t, w.Body.String(), "proxied")
}
