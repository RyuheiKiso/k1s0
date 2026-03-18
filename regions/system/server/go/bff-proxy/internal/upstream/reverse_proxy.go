package upstream

import (
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
// http.DefaultTransport をクローンして ResponseHeaderTimeout のみ上書きすることで、
// デフォルトの接続プール・TLS・プロキシ設定を維持しつつタイムアウトを設定する。
func NewReverseProxy(upstreamURL string, timeout time.Duration) (*ReverseProxy, error) {
	target, err := url.Parse(upstreamURL)
	if err != nil {
		return nil, err
	}

	// DefaultTransport をクローンしてデフォルト設定（接続プール、TLS等）を引き継ぐ
	transport := http.DefaultTransport.(*http.Transport).Clone()
	// レスポンスヘッダーのタイムアウトを設定（アップストリームの応答遅延を検出するため）
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
