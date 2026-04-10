package upstream

import (
	"context"
	"fmt"
	"log"
	"log/slog"
	"net"
	"net/http"
	"net/http/httputil"
	"net/url"
	"strings"
	"time"
)

// blockedCIDRs は SSRF 攻撃でアクセスされてはならない内部/ローカルアドレス範囲。
// ループバック、RFC-1918 プライベート、リンクローカル、クラウドメタデータアドレスを含む。
// STATIC-HIGH-001 監査対応: NewReverseProxy に内部 IP アクセス制限を追加する。
var blockedCIDRs = func() []*net.IPNet {
	cidrs := []string{
		"127.0.0.0/8",   // IPv4 ループバック
		"::1/128",        // IPv6 ループバック
		"10.0.0.0/8",    // RFC-1918 クラス A
		"172.16.0.0/12", // RFC-1918 クラス B
		"192.168.0.0/16", // RFC-1918 クラス C
		"169.254.0.0/16", // リンクローカル IPv4（クラウドメタデータ 169.254.169.254 を含む）
		"fe80::/10",      // リンクローカル IPv6
		"fc00::/7",       // IPv6 ユニークローカル
		"0.0.0.0/8",      // 無効なターゲット
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

// cloudMetadataCIDR はクラウドメタデータサービスのアドレス範囲（169.254.0.0/16）。
// allowedHosts によるバイパスが有効な場合でも、このアドレス範囲は常にブロックする。
// AWS/GCP/Azure のメタデータエンドポイント（169.254.169.254）への意図しないアクセスを防止する。
// M-015 監査対応: init 時の panic を log.Fatalf に置き換え、defer クリーンアップが実行されるようにする。
var cloudMetadataCIDR = func() *net.IPNet {
	// クラウドメタデータサービスの CIDR を解析する。
	// "169.254.0.0/16" はリテラル定数であり ParseCIDR が失敗することはないが、
	// M-015 監査対応として panic の代わりに log.Fatalf を使用する。
	_, ipNet, err := net.ParseCIDR("169.254.0.0/16")
	if err != nil || ipNet == nil {
		// log.Fatalf は os.Exit(1) を呼び出すため defer は実行されないが、
		// panic よりも明確なエラーメッセージとスタックトレースを提供する。
		log.Fatalf("cloudMetadataCIDRの初期化に失敗しました: %v", err)
	}
	return ipNet
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

// isCloudMetadataIP は指定された IP がクラウドメタデータサービスのアドレス範囲に属するかを返す。
// allowedHosts バイパスが有効であっても、この関数が true を返す場合は接続を拒否する。
func isCloudMetadataIP(ip net.IP) bool {
	return cloudMetadataCIDR != nil && cloudMetadataCIDR.Contains(ip)
}

// ReverseProxy は httputil.ReverseProxy とパース済みターゲットURLをラップする構造体。
// allowedHosts に含まれるホスト名は SSRF チェックをバイパスし、Docker/K8s 内部ネットワークへの
// アクセスを可能にする。クラウドメタデータ（169.254.0.0/16）は常にブロックされる。
type ReverseProxy struct {
	target *url.URL
	// allowedHosts は設定ファイル由来の静的アップストリームホスト名のセット。
	// これらのホストへの接続は SSRF チェックをスキップし、内部ネットワークへのアクセスを許可する。
	// ただし、クラウドメタデータ IP（169.254.0.0/16）は allowedHosts に含まれても常にブロックされる。
	allowedHosts map[string]bool
	proxy        *httputil.ReverseProxy
}

// ssrfSafeDialContext は DNS 解決後の接続先 IP を検証し、内部アドレスへの接続を拒否する。
// STATIC-HIGH-001 監査対応: DialContext フックで SSRF 防御を実装する。
// allowedHosts に含まれるホスト名の場合は RFC-1918 チェックをスキップするが、
// クラウドメタデータ IP（169.254.0.0/16）は常にブロックする。
func (p *ReverseProxy) ssrfSafeDialContext(ctx context.Context, network, addr string) (net.Conn, error) {
	host, port, err := net.SplitHostPort(addr)
	if err != nil {
		return nil, fmt.Errorf("SSRF防御: アドレス解析エラー: %w", err)
	}

	// 設定ファイル由来の静的アップストリームホスト名はSSRFチェックをスキップする。
	// ただし、クラウドメタデータIPは解決後に必ず検証する。
	isAllowedHost := p.allowedHosts != nil && p.allowedHosts[host]

	// DNS 解決して実際の IP アドレスを取得する
	addrs, err := net.DefaultResolver.LookupHost(ctx, host)
	if err != nil {
		return nil, fmt.Errorf("SSRF防御: DNS解決エラー (%s): %w", host, err)
	}

	// LOW-007 対応: DNS 解決結果が 0 件の場合のインデックス境界外アクセス（パニック）を防ぐ。
	// LookupHost はエラーなしで空スライスを返すことがあるため、明示的に長さを確認する。
	if len(addrs) == 0 {
		return nil, fmt.Errorf("DNS解決失敗: %s のアドレスが見つかりません", host)
	}

	// 解決されたすべての IP を検証する（DNS リバインディング対策を含む）
	for _, resolvedAddr := range addrs {
		ip := net.ParseIP(resolvedAddr)
		if ip == nil {
			continue
		}
		// クラウドメタデータIPは allowedHosts に関係なく常にブロックする。
		// Docker/K8s 環境でも 169.254.169.254 への意図しないアクセスを防止する。
		if isCloudMetadataIP(ip) {
			return nil, fmt.Errorf("SSRF防御: クラウドメタデータアドレスへのアクセスは禁止されています (%s → %s)", host, resolvedAddr)
		}
		// allowedHosts に含まれる場合は RFC-1918 チェックをスキップする
		if isAllowedHost {
			continue
		}
		// 動的ターゲット（allowedHosts 外）は通常の SSRF チェックを適用する
		if isBlockedIP(ip) {
			return nil, fmt.Errorf("SSRF防御: 内部アドレスへのアクセスは禁止されています (%s → %s)", host, resolvedAddr)
		}
	}

	// 検証済みの IP:port で接続する（addrs[0] は上記の長さチェックにより安全）
	baseDialer := &net.Dialer{Timeout: 30 * time.Second}
	return baseDialer.DialContext(ctx, network, net.JoinHostPort(addrs[0], port))
}

// NewReverseProxy は指定されたアップストリームURLとタイムアウトでリバースプロキシを生成する。
// http.DefaultTransport をクローンして全タイムアウトを明示設定し、
// ハングしたバックエンド接続によるリソースリークを防止する。
// STATIC-HIGH-001 監査対応: DialContext に SSRF 防御フックを追加し、
// ループバック・RFC-1918・リンクローカル・クラウドメタデータへのアクセスを拒否する。
// allowedHosts に含まれるホスト名（設定ファイル由来の静的アップストリーム）は
// SSRF チェックをスキップし、Docker/K8s 内部ネットワーク経由の通信を可能にする。
func NewReverseProxy(upstreamURL string, timeout time.Duration, allowedHosts map[string]bool) (*ReverseProxy, error) {
	target, err := url.Parse(upstreamURL)
	if err != nil {
		return nil, err
	}

	rp := &ReverseProxy{
		target:       target,
		allowedHosts: allowedHosts,
	}

	// DefaultTransport をクローンしてデフォルト設定（接続プール、TLS等）を引き継ぐ
	transport := http.DefaultTransport.(*http.Transport).Clone()

	// SSRF 防御: 内部アドレスへの接続を拒否するダイアラー（allowedHosts はバイパス対象）
	transport.DialContext = rp.ssrfSafeDialContext

	// TLS ハンドシェイクのタイムアウト（証明書検証を含む）
	transport.TLSHandshakeTimeout = 10 * time.Second
	// アイドル接続の保持上限（バックエンドの FIN_WAIT2 との競合を防ぐ）
	transport.IdleConnTimeout = 90 * time.Second
	// レスポンスヘッダーのタイムアウト（アップストリームの応答遅延を検出するため）
	transport.ResponseHeaderTimeout = timeout

	proxy := httputil.NewSingleHostReverseProxy(target)
	proxy.Transport = transport

	// HIGH-GO-002 監査対応: バックエンドエラー詳細を隠蔽し、内部ネットワーク情報の漏洩を防止する。
	// デフォルトの ErrorHandler はバックエンドのエラーメッセージをそのままクライアントに返す可能性があるため、
	// 502 Bad Gateway のみを返すよう上書きする。
	proxy.ErrorHandler = func(w http.ResponseWriter, r *http.Request, err error) {
		slog.Warn("アップストリームサービスへの接続に失敗しました",
			slog.String("error", err.Error()),
			slog.String("method", r.Method),
			slog.String("path", r.URL.Path),
		)
		w.WriteHeader(http.StatusBadGateway)
	}

	// HIGH-GO-002 監査対応: バックエンドのレスポンスヘッダーから内部情報を含むヘッダーを除去する。
	// X-Internal- プレフィックスのヘッダーやバックエンドサーバー情報をクライアントに返さないようにする。
	proxy.ModifyResponse = func(resp *http.Response) error {
		// X-Internal- プレフィックスを持つヘッダーをすべて削除する
		for key := range resp.Header {
			if strings.HasPrefix(strings.ToLower(key), "x-internal-") {
				resp.Header.Del(key)
			}
		}
		// バックエンドサーバー情報を隠蔽する（Apache/nginx/tonic 等のバージョン漏洩を防ぐ）
		resp.Header.Del("Server")
		return nil
	}

	rp.proxy = proxy

	return rp, nil
}

func (r *ReverseProxy) ServeHTTP(w http.ResponseWriter, req *http.Request) {
	r.proxy.ServeHTTP(w, req)
}

func (r *ReverseProxy) Target() *url.URL {
	return r.target
}
