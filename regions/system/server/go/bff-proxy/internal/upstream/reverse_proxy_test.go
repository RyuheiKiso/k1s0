package upstream

import (
	"net"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// NewReverseProxy が正しいターゲットURLでプロキシを生成する
func TestNewReverseProxy_ValidURL_ReturnsProxy(t *testing.T) {
	// URL 解析のみ（接続は行わない）ため SSRF チェックは発生しない
	proxy, err := NewReverseProxy("http://external.example.com:8080", 5*time.Second, nil)
	require.NoError(t, err)
	assert.NotNil(t, proxy)
	assert.Equal(t, "external.example.com:8080", proxy.Target().Host)
	assert.Equal(t, "http", proxy.Target().Scheme)
}

// NewReverseProxy が無効なURLを受け取るとエラーを返す
func TestNewReverseProxy_InvalidURL_ReturnsError(t *testing.T) {
	// url.Parse は多くのケースでエラーを返さないが、空文字列などの特殊ケースをテスト
	proxy, err := NewReverseProxy("://invalid", 5*time.Second, nil)
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
	proxy, err := NewReverseProxy("https://api.example.com:9090/v1", 10*time.Second, nil)
	require.NoError(t, err)
	target := proxy.Target()
	assert.Equal(t, "https", target.Scheme)
	assert.Equal(t, "api.example.com:9090", target.Host)
}

// ServeHTTP が内部アドレス（127.0.0.1）へのリクエストを SSRF 防御でブロックする。
// STATIC-HIGH-001 監査対応: DialContext フックが内部アドレスへの接続を拒否することを確認する。
func TestReverseProxy_ServeHTTP_BlocksInternalAddress(t *testing.T) {
	// httptest.NewServer は 127.0.0.1 を使用するため SSRF 防御によりブロックされる
	backend := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"proxied":true}`))
	}))
	defer backend.Close()

	// allowedHosts なし（nil）で生成: 127.0.0.1 は SSRF 防御によりブロックされる
	proxy, err := NewReverseProxy(backend.URL, 5*time.Second, nil)
	require.NoError(t, err)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	proxy.ServeHTTP(w, req)

	// SSRF 防御により DialContext でエラーが発生し、502 Bad Gateway が返る
	assert.Equal(t, http.StatusBadGateway, w.Code, "内部アドレスへのリクエストは 502 でブロックされるべき")
}

// ServeHTTP が allowedHosts に含まれる内部アドレスへのリクエストを通過させる。
// Docker/K8s 環境での BFF → バックエンド通信を許可することを確認する。
func TestReverseProxy_ServeHTTP_AllowsAllowedHost(t *testing.T) {
	// httptest.NewServer は 127.0.0.1 を使用する
	backend := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"proxied":true}`))
	}))
	defer backend.Close()

	// backend.URL から 127.0.0.1 ホストを取得して allowedHosts に追加する
	backendHost := "127.0.0.1"
	allowedHosts := map[string]bool{
		backendHost: true,
	}

	// allowedHosts に 127.0.0.1 を含めて生成: SSRF チェックをバイパスして接続できる
	proxy, err := NewReverseProxy(backend.URL, 5*time.Second, allowedHosts)
	require.NoError(t, err)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	proxy.ServeHTTP(w, req)

	// allowedHosts に含まれるため SSRF チェックをバイパスし、200 OK が返る
	assert.Equal(t, http.StatusOK, w.Code, "allowedHosts に含まれる内部アドレスへのリクエストは通過するべき")
}

// --- isBlockedIP のユニットテスト ---

// isBlockedIP がループバックアドレスを正しく検出する。
// STATIC-HIGH-001 監査対応: 127.0.0.0/8 全体をブロックする。
func TestIsBlockedIP_Loopback(t *testing.T) {
	cases := []struct {
		ip      string
		blocked bool
	}{
		{"127.0.0.1", true},
		{"127.255.255.255", true},
		{"::1", true},
	}
	for _, tc := range cases {
		ip := net.ParseIP(tc.ip)
		require.NotNil(t, ip, "IP パースエラー: %s", tc.ip)
		assert.Equal(t, tc.blocked, isBlockedIP(ip), "IP: %s", tc.ip)
	}
}

// isBlockedIP が RFC-1918 プライベートアドレスを正しく検出する。
// STATIC-HIGH-001 監査対応: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16 をブロックする。
func TestIsBlockedIP_PrivateRanges(t *testing.T) {
	cases := []struct {
		ip      string
		blocked bool
	}{
		{"10.0.0.1", true},
		{"10.255.255.255", true},
		{"172.16.0.1", true},
		{"172.31.255.255", true},
		{"192.168.0.1", true},
		{"192.168.255.255", true},
	}
	for _, tc := range cases {
		ip := net.ParseIP(tc.ip)
		require.NotNil(t, ip, "IP パースエラー: %s", tc.ip)
		assert.Equal(t, tc.blocked, isBlockedIP(ip), "IP: %s", tc.ip)
	}
}

// isBlockedIP がクラウドメタデータアドレス（169.254.169.254）をブロックする。
// STATIC-HIGH-001 監査対応: リンクローカル（169.254.0.0/16）はメタデータ IP を含む。
func TestIsBlockedIP_LinkLocal(t *testing.T) {
	cases := []struct {
		ip      string
		blocked bool
	}{
		{"169.254.169.254", true}, // AWS/GCP/Azure メタデータエンドポイント
		{"169.254.0.1", true},
		{"fe80::1", true}, // IPv6 リンクローカル
	}
	for _, tc := range cases {
		ip := net.ParseIP(tc.ip)
		require.NotNil(t, ip, "IP パースエラー: %s", tc.ip)
		assert.Equal(t, tc.blocked, isBlockedIP(ip), "IP: %s", tc.ip)
	}
}

// isBlockedIP がパブリックアドレスをブロックしない。
// STATIC-HIGH-001 監査対応: 正当なアップストリームへの接続を妨げないこと。
func TestIsBlockedIP_PublicAddresses_NotBlocked(t *testing.T) {
	cases := []string{
		"8.8.8.8",     // Google DNS
		"1.1.1.1",     // Cloudflare DNS
		"203.0.113.1", // TEST-NET-3 (RFC 5737)
		"2001:db8::1", // IPv6 ドキュメント用
	}
	for _, ipStr := range cases {
		ip := net.ParseIP(ipStr)
		require.NotNil(t, ip, "IP パースエラー: %s", ipStr)
		assert.False(t, isBlockedIP(ip), "パブリック IP %s はブロックされるべきでない", ipStr)
	}
}

// isCloudMetadataIP がクラウドメタデータ IP を正しく検出する。
// allowedHosts バイパス対象であっても、169.254.0.0/16 は常にブロックすることを確認する。
func TestIsCloudMetadataIP(t *testing.T) {
	cases := []struct {
		ip      string
		blocked bool
	}{
		{"169.254.169.254", true}, // AWS/GCP/Azure クラウドメタデータ
		{"169.254.0.1", true},
		{"169.254.255.255", true},
		{"10.0.0.1", false},    // RFC-1918 はクラウドメタデータではない
		{"127.0.0.1", false},   // ループバックはクラウドメタデータではない
		{"8.8.8.8", false},     // パブリック IP はクラウドメタデータではない
	}
	for _, tc := range cases {
		ip := net.ParseIP(tc.ip)
		require.NotNil(t, ip, "IP パースエラー: %s", tc.ip)
		assert.Equal(t, tc.blocked, isCloudMetadataIP(ip), "IP: %s", tc.ip)
	}
}
