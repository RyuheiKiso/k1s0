package upstream

import (
	"context"
	"fmt"
	"net"
	"net/http"
	"net/http/httputil"
	"net/url"
	"time"
)

// blockedCIDRs は SSRF 攻撃でアクセスされてはならない内部/ローカルアドレス範囲。
// ループバック、RFC-1918 プライベート、リンクローカル、クラウドメタデータアドレスを含む。
// STATIC-HIGH-001 監査対応: NewReverseProxy に内部 IP アクセス制限を追加する。
var blockedCIDRs = func() []*net.IPNet {
	cidrs := []string{
		"127.0.0.0/8",    // IPv4 ループバック
		"::1/128",         // IPv6 ループバック
		"10.0.0.0/8",      // RFC-1918 クラス A
		"172.16.0.0/12",   // RFC-1918 クラス B
		"192.168.0.0/16",  // RFC-1918 クラス C
		"169.254.0.0/16",  // リンクローカル IPv4（クラウドメタデータ 169.254.169.254 を含む）
		"fe80::/10",       // リンクローカル IPv6
		"fc00::/7",        // IPv6 ユニークローカル
		"0.0.0.0/8",       // 無効なターゲット
	}
	var nets []*net.IPNet
	for _, cidr := range cidrs {
		_, ipNet, err := net.ParseCIDR(cidr)
		if err == nil {
			nets = append(nets, ipNet)
		}
	}
	return nets
}()

// isBlockedIP は指定された IP アドレスが内部/ローカルアドレス範囲に属するかを返す。
// STATIC-HIGH-001 監査対応: SSRF 防御チェックに使用する。
func isBlockedIP(ip net.IP) bool {
	for _, cidr := range blockedCIDRs {
		if cidr.Contains(ip) {
			return true
		}
	}
	return false
}

// ssrfSafeDialContext は DNS 解決後の接続先 IP を検証し、内部アドレスへの接続を拒否する。
// STATIC-HIGH-001 監査対応: DialContext フックで SSRF 防御を実装する。
func ssrfSafeDialContext(baseDialer *net.Dialer) func(ctx context.Context, network, addr string) (net.Conn, error) {
	return func(ctx context.Context, network, addr string) (net.Conn, error) {
		host, port, err := net.SplitHostPort(addr)
		if err != nil {
			return nil, fmt.Errorf("SSRF防御: アドレス解析エラー: %w", err)
		}

		// DNS 解決して実際の IP アドレスを取得する
		addrs, err := net.DefaultResolver.LookupHost(ctx, host)
		if err != nil {
			return nil, fmt.Errorf("SSRF防御: DNS解決エラー (%s): %w", host, err)
		}

		// 解決されたすべての IP を検証する（DNS リバインディング対策を含む）
		for _, resolvedAddr := range addrs {
			ip := net.ParseIP(resolvedAddr)
			if ip == nil {
				continue
			}
			if isBlockedIP(ip) {
				return nil, fmt.Errorf("SSRF防御: 内部アドレスへのアクセスは禁止されています (%s → %s)", host, resolvedAddr)
			}
		}

		// 検証済みの IP:port で接続する
		return baseDialer.DialContext(ctx, network, net.JoinHostPort(addrs[0], port))
	}
}

// ReverseProxy は httputil.ReverseProxy とパース済みターゲットURLをラップする構造体。
type ReverseProxy struct {
	target *url.URL
	proxy  *httputil.ReverseProxy
}

// NewReverseProxy は指定されたアップストリームURLとタイムアウトでリバースプロキシを生成する。
// http.DefaultTransport をクローンして全タイムアウトを明示設定し、
// ハングしたバックエンド接続によるリソースリークを防止する。
// STATIC-HIGH-001 監査対応: DialContext に SSRF 防御フックを追加し、
// ループバック・RFC-1918・リンクローカル・クラウドメタデータへのアクセスを拒否する。
func NewReverseProxy(upstreamURL string, timeout time.Duration) (*ReverseProxy, error) {
	target, err := url.Parse(upstreamURL)
	if err != nil {
		return nil, err
	}

	// DefaultTransport をクローンしてデフォルト設定（接続プール、TLS等）を引き継ぐ
	transport := http.DefaultTransport.(*http.Transport).Clone()

	// SSRF 防御: 内部アドレスへの接続を拒否するベースダイアラー
	baseDialer := &net.Dialer{
		Timeout:   30 * time.Second,
		KeepAlive: 30 * time.Second,
	}
	transport.DialContext = ssrfSafeDialContext(baseDialer)

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
