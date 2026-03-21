package upstream

import (
	"net"
	"net/http"
	"net/http/httputil"
	"net/url"
	"time"
)

// ReverseProxy は httputil.ReverseProxy とパース済みターゲットURLをラップする構造体。
type ReverseProxy struct {
	target *url.URL
	proxy  *httputil.ReverseProxy
}

// NewReverseProxy は指定されたアップストリームURLとタイムアウトでリバースプロキシを生成する。
// http.DefaultTransport をクローンして全タイムアウトを明示設定し、
// ハングしたバックエンド接続によるリソースリークを防止する。
func NewReverseProxy(upstreamURL string, timeout time.Duration) (*ReverseProxy, error) {
	target, err := url.Parse(upstreamURL)
	if err != nil {
		return nil, err
	}

	// DefaultTransport をクローンしてデフォルト設定（接続プール、TLS等）を引き継ぐ
	transport := http.DefaultTransport.(*http.Transport).Clone()
	// TCP 接続確立のタイムアウト（DNSルックアップ + 3way handshake の上限）
	transport.DialContext = (&net.Dialer{
		Timeout:   30 * time.Second,
		KeepAlive: 30 * time.Second,
	}).DialContext
	// TLS ハンドシェイクのタイムアウト（証明書検証を含む）
	transport.TLSHandshakeTimeout = 10 * time.Second
	// アイドル接続の保持上限（バックエンドの FIN_WAIT2 との競合を防ぐ）
	transport.IdleConnTimeout = 90 * time.Second
	// レスポンスヘッダーのタイムアウト（アップストリームの応答遅延を検出するため）
	transport.ResponseHeaderTimeout = timeout

	proxy := httputil.NewSingleHostReverseProxy(target)
	proxy.Transport = transport

	return &ReverseProxy{
		target: target,
		proxy:  proxy,
	}, nil
}

func (r *ReverseProxy) ServeHTTP(w http.ResponseWriter, req *http.Request) {
	r.proxy.ServeHTTP(w, req)
}

func (r *ReverseProxy) Target() *url.URL {
	return r.target
}
