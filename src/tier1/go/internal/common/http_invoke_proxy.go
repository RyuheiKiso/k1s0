// 本ファイルは Service Invoke API の HTTP/1.1 互換プロキシ（FR-T1-INVOKE-002）。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/01_Service_Invoke_API.md FR-T1-INVOKE-002
//     - tier1 が HTTP/1.1 エンドポイント（POST /invoke/<target>/<method> 形式）を提供
//     - 内部で gRPC に変換して対象サービスを呼び出す
//     - .NET Framework 4.x アプリは既存の HttpClient をそのまま使える
//     - レスポンスは JSON エンコーディングで返される（あるいは raw bytes、上流次第）
//     - gRPC ステータスコードを HTTP ステータスコードに適切にマッピング
//
// 共通 HTTP/JSON gateway（POST /k1s0/serviceinvoke/invoke）との違い:
//   - URL に target / method を含める（path-based dispatch）
//   - body は protojson 化された InvokeRequest ではなく **raw bytes**（呼出先サービスが解釈する payload）
//   - ContentType は HTTP request の Content-Type をそのまま転送（既定 application/octet-stream）
//   - レスポンスは Content-Type が来ればそれを尊重、そうでなければ application/octet-stream
//
// 認証 / トレース / テナント:
//   共通 gateway と同じ interceptor chain を経由する。Authorization / traceparent /
//   tracestate / X-K1s0-Tenant-Id / X-K1s0-Idempotency-Key を gRPC metadata に転写する。
//
// 制限:
//   - server-streaming（InvokeStream）は HTTP/1.1 単発応答に収まらないため非対応。
//     クライアントが streaming を要する場合は gRPC 経路を使う運用とする。

package common

import (
	"context"
	"io"
	"net/http"
	"strings"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// InvokeProxyHandler は HTTP/1.1 経路の InvokeService.Invoke を呼ぶ adapter。
//
// HTTPGateway が path から target / method を抽出して body と合わせて本 adapter を呼ぶ。
// adapter 側では proto InvokeRequest を組み立てて in-process gRPC handler を呼ぶ。
type InvokeProxyHandler interface {
	// ProxyInvoke は target / method / body / contentType / tenantID を受けて呼出先の応答を返す。
	// tenantID は X-K1s0-Tenant-Id ヘッダ由来（HTTP 経路では JWT decode 済の tenant_id を期待）。
	// 戻り値の data は呼出先サービスからの応答 body、status は HTTP status 相当（Dapr が exposing せず 200 固定の場合あり）。
	ProxyInvoke(ctx context.Context, req InvokeProxyRequest) (InvokeProxyResponse, error)
}

// InvokeProxyRequest はプロキシ呼出の入力。
type InvokeProxyRequest struct {
	// 呼出先アプリ ID（URL の <target> 部）。
	Target string
	// 呼出先メソッド（URL の <method> 部）。
	Method string
	// 呼出側 body（raw bytes、プロキシは透過転送）。
	Body []byte
	// HTTP request の Content-Type をそのまま転送。
	ContentType string
	// テナント ID（X-K1s0-Tenant-Id 由来、または JWT 由来の AuthInfo から interceptor が補完）。
	TenantID string
	// 呼出ごとの timeout（HTTP request の X-K1s0-Timeout-Ms ヘッダ由来、0 は既定値）。
	TimeoutMs int32
}

// InvokeProxyResponse はプロキシ応答。
type InvokeProxyResponse struct {
	// 呼出先からの応答 bytes（透過）。
	Data []byte
	// 呼出先 Content-Type（指定がなければ application/octet-stream）。
	ContentType string
	// HTTP status 相当（200 または K1s0Error 翻訳後）。
	Status int32
}

// RegisterInvokeProxyRoute は POST /invoke/<target>/<method> を gateway に登録する。
//
// adapter は通常 cmd/state/main.go で InvokeService 実装に bind して渡す。
//
// FR-T1-INVOKE-002:
//   - /invoke/ prefix の **2 セグメント以上** を target / method に分解
//   - method は最後のセグメント、target は中間（例: /invoke/myapp/v1/orders/create
//     → target="myapp/v1/orders", method="create"）。Dapr の app_id は "/" を許容するため。
//   - body は raw、gRPC へは bytes フィールドにそのまま投入
//   - 失敗時は HTTP/JSON マッピング表通り status code を変換（共通 gateway と同じ writeJSONError 使用）
func (g *HTTPGateway) RegisterInvokeProxyRoute(adapter InvokeProxyHandler) {
	// path prefix を登録する（/invoke/ 配下を全部キャッチ）。
	g.mux.HandleFunc("/invoke/", func(w http.ResponseWriter, r *http.Request) {
		// POST 以外は 405。
		if r.Method != http.MethodPost {
			w.Header().Set("Allow", http.MethodPost)
			writeJSONError405(w, "only POST is supported for /invoke/<target>/<method>")
			return
		}
		// path から /invoke/ を除去して残り部分を分解する。
		rest := strings.TrimPrefix(r.URL.Path, "/invoke/")
		// 末尾 / は不正（method が空）。
		if rest == "" || strings.HasSuffix(rest, "/") {
			writeJSONError(w, codes.InvalidArgument, "URL must be POST /invoke/<target>/<method>")
			return
		}
		// 最後の "/" で分割し、それより前を target、後を method とする。
		idx := strings.LastIndex(rest, "/")
		if idx <= 0 || idx == len(rest)-1 {
			writeJSONError(w, codes.InvalidArgument, "URL must be POST /invoke/<target>/<method>")
			return
		}
		target := rest[:idx]
		method := rest[idx+1:]
		// body を読み取る（8 MiB 上限、共通規約準拠）。
		body, err := io.ReadAll(io.LimitReader(r.Body, 8*1024*1024))
		if err != nil {
			writeJSONError(w, codes.InvalidArgument, "failed to read body: "+err.Error())
			return
		}
		// Content-Type は呼出側が指定したものをそのまま転送する（既定 application/octet-stream）。
		contentType := r.Header.Get("Content-Type")
		if contentType == "" {
			contentType = "application/octet-stream"
		}
		// 認証 / トレース / テナント / 冪等性ヘッダを gRPC metadata に転送。
		md := metadata.New(nil)
		if v := r.Header.Get("Authorization"); v != "" {
			md.Set("authorization", v)
		}
		if v := r.Header.Get("Traceparent"); v != "" {
			md.Set("traceparent", v)
		}
		if v := r.Header.Get("Tracestate"); v != "" {
			md.Set("tracestate", v)
		}
		if v := r.Header.Get("X-K1s0-Idempotency-Key"); v != "" {
			md.Set("x-k1s0-idempotency-key", v)
		}
		// テナント ID は明示ヘッダで受け取る（JWT 認証 off の場合の補完経路）。
		tenantID := r.Header.Get("X-K1s0-Tenant-Id")
		if tenantID != "" {
			md.Set("x-k1s0-tenant-id", tenantID)
		}
		// Timeout ヘッダ（任意）を解釈する（X-K1s0-Timeout-Ms）。
		var timeoutMs int32
		if v := r.Header.Get("X-K1s0-Timeout-Ms"); v != "" {
			if parsed, perr := parseTimeoutMs(v); perr == nil && parsed > 0 {
				timeoutMs = parsed
			}
		}
		// gRPC incoming context を構築する。
		ctx := metadata.NewIncomingContext(r.Context(), md)
		// HTTP gateway 側のフォールバック deadline（共通 gateway と同じ 3 秒）。
		// InvokeService.Invoke 側で TimeoutMs があればさらに短縮される。
		ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
		defer cancel()
		// FullMethod（observability ラベル / audit 用）。
		info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.serviceinvoke.v1.InvokeService/Invoke"}
		// adapter を interceptor chain で wrap して呼ぶ。
		final := func(ctx context.Context, _ interface{}) (interface{}, error) {
			return adapter.ProxyInvoke(ctx, InvokeProxyRequest{
				Target:      target,
				Method:      method,
				Body:        body,
				ContentType: contentType,
				TenantID:    tenantID,
				TimeoutMs:   timeoutMs,
			})
		}
		wrapped := final
		for i := len(g.interceptors) - 1; i >= 0; i-- {
			icpt := g.interceptors[i]
			next := wrapped
			wrapped = func(ctx context.Context, req interface{}) (interface{}, error) {
				return icpt(ctx, req, info, next)
			}
		}
		raw, err := wrapped(ctx, nil)
		if err != nil {
			st, _ := status.FromError(err)
			writeJSONError(w, st.Code(), st.Message())
			return
		}
		// 応答を取り出す（adapter が InvokeProxyResponse を返す前提）。
		resp, ok := raw.(InvokeProxyResponse)
		if !ok {
			writeJSONError(w, codes.Internal, "invoke proxy: unexpected response type")
			return
		}
		// Content-Type は呼出先指定があればそれ、無ければ呼出側の指定を返す。
		respCT := resp.ContentType
		if respCT == "" {
			respCT = contentType
		}
		w.Header().Set("Content-Type", respCT)
		// HTTP status: Dapr が status を exposing する場合（200 固定）はそのまま、
		// adapter が「呼出先」の HTTP status を伝えるなら respect。
		statusCode := int(resp.Status)
		if statusCode <= 0 {
			statusCode = http.StatusOK
		}
		w.WriteHeader(statusCode)
		_, _ = w.Write(resp.Data)
	})
}

// parseTimeoutMs は string ms 値を int32 に変換する。負値・非数値は error を返す。
func parseTimeoutMs(s string) (int32, error) {
	// strconv 依存を避けるため軽量パースを行う（ASCII 数字のみ）。
	var n int64
	for i := 0; i < len(s); i++ {
		c := s[i]
		if c < '0' || c > '9' {
			return 0, errInvalidTimeout
		}
		n = n*10 + int64(c-'0')
		if n > 1<<31-1 {
			return 0, errInvalidTimeout
		}
	}
	return int32(n), nil
}

// errInvalidTimeout は parseTimeoutMs が無効な数値文字列を受けた場合のセンチネル。
var errInvalidTimeout = status.Error(codes.InvalidArgument, "invalid X-K1s0-Timeout-Ms")
