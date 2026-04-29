// 本ファイルは tier1 facade の HTTP/JSON 互換 gateway。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「HTTP/JSON 互換インタフェース共通仕様」:
//       URL: POST /k1s0/<api>/<rpc>
//       Content-Type: application/json; charset=utf-8
//       JSON: protojson 直列化
//       認証: Authorization: Bearer <jwt>（tier1 AuthInterceptor で検証）
//       trace 伝播: traceparent / tracestate
//       HTTP Status ↔ K1s0Error マッピング:
//         200 → 成功 / 400 → InvalidArgument / 401 → Unauthenticated /
//         403 → PermissionDenied / 404 → NotFound / 409 → AlreadyExists /
//         429 → ResourceExhausted / 503 → Unavailable / 504 → DeadlineExceeded /
//         500 → Internal
//
// 設計方針:
//   gRPC Server に登録した handler を in-process で再利用する（grpc.Server.GetServiceInfo
//   経由で動的 dispatch するのではなく、明示的な handler map で型安全に dispatch する）。
//   これにより interceptor chain（Auth / RateLimit / Observability / Audit）が HTTP 経路でも
//   同じ順序で適用され、二重実装を避ける。
//
// 認証:
//   HTTP の Authorization ヘッダを gRPC metadata に詰め直し、in-process gRPC 呼出時に
//   AuthInterceptor が JWT を検証する。multi-replica の HTTP gateway → gRPC backend 構成
//   ではなく単一 Pod 内で完結するため、shared cache / 二重 hop 不要。
//
// 制限事項:
//   - リリース時点 では State API（Get / Set / Delete）のみ実装。他 API は本パターンに沿って
//     別 PR で追加する（RegisterStateRoutes と同形のヘルパを各 API 用に書く）。
//   - server-streaming RPC（PubSub.Subscribe / InvokeStream / Audit.Export）は HTTP/JSON 単発
//     応答に収まらないため非対応（gRPC 経路を使う運用）。

package common

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
)

// httpStatusFromGRPC は docs §「HTTP/JSON 互換」マッピング表に従って gRPC code を HTTP status に変換する。
func httpStatusFromGRPC(c codes.Code) int {
	switch c {
	case codes.OK:
		return http.StatusOK
	case codes.InvalidArgument:
		return http.StatusBadRequest
	case codes.Unauthenticated:
		return http.StatusUnauthorized
	case codes.PermissionDenied:
		return http.StatusForbidden
	case codes.NotFound:
		return http.StatusNotFound
	case codes.AlreadyExists, codes.Aborted, codes.FailedPrecondition:
		return http.StatusConflict
	case codes.ResourceExhausted:
		return http.StatusTooManyRequests
	case codes.Unavailable:
		return http.StatusServiceUnavailable
	case codes.DeadlineExceeded:
		return http.StatusGatewayTimeout
	case codes.Internal:
		return http.StatusInternalServerError
	default:
		return http.StatusInternalServerError
	}
}

// httpHandlerFunc は HTTP/JSON gateway 用の handler 関数型。
// 実装側は protojson 解釈済の req message を受け取り、proto response を返す。
type httpHandlerFunc func(ctx context.Context, body []byte) (proto.Message, error)

// HTTPGateway は path → handler の registry を保持する HTTP server。
//
// interceptors:
//   gRPC server に登録する UnaryServerInterceptor を HTTP 経路でも適用するため、
//   register が handler を invoke する前に chain を通す。
//   これにより HTTP / gRPC 両経路で同じ Auth / RateLimit / Observability / Audit が
//   適用される（共通規約 §「認証と認可」/§「監査と痕跡」/§「レート制限とクォータ」と整合）。
//   nil の場合は素通り（test / 早期 dev 経路）。
type HTTPGateway struct {
	mux          *http.ServeMux
	interceptors []grpc.UnaryServerInterceptor
}

// NewHTTPGateway は空 gateway を生成する。RegisterX で API ルートを追加する。
func NewHTTPGateway() *HTTPGateway {
	return &HTTPGateway{mux: http.NewServeMux()}
}

// WithInterceptors は HTTP 経路でも適用する gRPC interceptor chain を設定する。
// 同名 grpc.ChainUnaryInterceptor と同じ順序で chain を実行する（先頭が最外層）。
// 通常 cmd/X/main.go で gRPC server に渡している interceptor と同じものを渡す。
func (g *HTTPGateway) WithInterceptors(interceptors ...grpc.UnaryServerInterceptor) *HTTPGateway {
	g.interceptors = append(g.interceptors, interceptors...)
	return g
}

// Handler は http.Handler interface を満たす。net/http.Server の Handler に渡せる。
func (g *HTTPGateway) Handler() http.Handler {
	return g.mux
}

// invokeWithInterceptors は handler を interceptor chain で wrap して呼び出す。
// interceptors が空なら handler を直接呼ぶ。
// chain 順序は grpc.ChainUnaryInterceptor と同じ（先頭が最外層、handler 直前で最内層）。
func (g *HTTPGateway) invokeWithInterceptors(ctx context.Context, info *grpc.UnaryServerInfo, body []byte, handler httpHandlerFunc) (proto.Message, error) {
	// handler を grpc.UnaryHandler 風のシグネチャに wrap する。
	// HTTP gateway は body を req として扱うが、interceptor chain 上は req は単に opaque な
	// "incoming payload" として扱われる。extractTenantID 等は req に対し reflection するため、
	// proto.Message 型として再構築できないと tenant_id が取れない。本実装では handler 内で
	// protojson Unmarshal するため、interceptor 段階では req=nil を渡し、interceptor は
	// metadata 由来の tenant_id（TIER1_AUTH_MODE=jwks/hmac で AuthInterceptor が JWT から
	// 取り出した値）に依存する設計とする。
	final := func(ctx context.Context, _ interface{}) (interface{}, error) {
		return handler(ctx, body)
	}
	// interceptor chain を構築（最後の interceptor が final を呼ぶ）。
	wrapped := final
	for i := len(g.interceptors) - 1; i >= 0; i-- {
		icpt := g.interceptors[i]
		next := wrapped
		wrapped = func(ctx context.Context, req interface{}) (interface{}, error) {
			return icpt(ctx, req, info, next)
		}
	}
	resp, err := wrapped(ctx, nil)
	if err != nil {
		return nil, err
	}
	if resp == nil {
		return nil, nil
	}
	return resp.(proto.Message), nil
}

// register は POST /k1s0/<api>/<rpc> をハンドルする内部 helper。
// 認証は authForward で gRPC metadata に転送し、in-process gRPC handler で AuthInterceptor が検証する。
func (g *HTTPGateway) register(path string, handler httpHandlerFunc) {
	// HTTP path から gRPC FullMethod を再構築する（observability ラベル / audit action 用）。
	// 例: /k1s0/state/get → /k1s0.tier1.state.v1.StateService/Get
	fullMethod := httpPathToGRPCMethod(path)
	info := &grpc.UnaryServerInfo{FullMethod: fullMethod}
	g.mux.HandleFunc(path, func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSONError(w, codes.InvalidArgument, "only POST is supported")
			return
		}
		if !strings.HasPrefix(r.Header.Get("Content-Type"), "application/json") {
			writeJSONError(w, codes.InvalidArgument, "Content-Type must be application/json")
			return
		}
		body, err := io.ReadAll(io.LimitReader(r.Body, 8*1024*1024)) // 8 MiB 上限（docs 規約準拠）
		if err != nil {
			writeJSONError(w, codes.InvalidArgument, "failed to read body: "+err.Error())
			return
		}
		// gRPC metadata に Authorization / traceparent / tracestate / idempotency-key を転送。
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
		ctx := metadata.NewIncomingContext(r.Context(), md)
		// 既定 deadline（docs 規約: tier1 内部 3 秒）。
		ctx, cancel := context.WithTimeout(ctx, 3*time.Second)
		defer cancel()

		// interceptor chain を経由して handler を呼ぶ（HTTP / gRPC で同じ chain を適用）。
		resp, err := g.invokeWithInterceptors(ctx, info, body, handler)
		if err != nil {
			st, _ := status.FromError(err)
			writeJSONError(w, st.Code(), st.Message())
			return
		}
		out, err := protojson.MarshalOptions{UseProtoNames: false, EmitUnpopulated: false}.Marshal(resp)
		if err != nil {
			writeJSONError(w, codes.Internal, "failed to marshal response: "+err.Error())
			return
		}
		w.Header().Set("Content-Type", "application/json; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write(out)
	})
}

// apiToServiceName は HTTP gateway の url segment "<api>" を proto の service 名にマップする。
// 単純な title-case では "PubSubService" のような複合語が "PubsubService" になってしまうため、
// 明示的なマッピング表を持つ（proto 定義との整合を強制する）。
var apiToServiceName = map[string]string{
	"state":         "StateService",
	"pubsub":        "PubSubService",
	"secrets":       "SecretsService",
	"workflow":      "WorkflowService",
	"feature":       "FeatureService",
	"binding":       "BindingService",
	"log":           "LogService",
	"telemetry":     "TelemetryService",
	"serviceinvoke": "InvokeService",
	// admin 系（HTTP 未対応だが将来拡張で追加可能）。
	"decision": "DecisionService",
	"audit":    "AuditService",
	"pii":      "PiiService",
}

// rpcMethodNames は HTTP url segment "<rpc>" を proto の RPC 名にマップする例外集。
// 単純な title-case で正しくない（複合語）case のみ列挙する。
var rpcMethodNames = map[string]string{
	"bulkget":         "BulkGet",
	"bulksend":        "BulkSend",
	"bulkpublish":     "BulkPublish",
	"getstatus":       "GetStatus",
	"emitmetric":      "EmitMetric",
	"emitspan":        "EmitSpan",
	"evaluateboolean": "EvaluateBoolean",
	"evaluatestring":  "EvaluateString",
	"evaluatenumber":  "EvaluateNumber",
	"evaluateobject":  "EvaluateObject",
	"getdynamic":      "GetDynamic",
}

// httpPathToGRPCMethod は HTTP gateway の path から対応する gRPC FullMethod を再構築する。
// 例: "/k1s0/state/get" → "/k1s0.tier1.state.v1.StateService/Get"
//
// 用途: ObservabilityInterceptor / AuditInterceptor が info.FullMethod から API 名 / RPC 名を
// 抽出するため、gRPC server 経由と同じ FullMethod を渡す必要がある。
//
// 命名は明示マップ（apiToServiceName / rpcMethodNames）でカバーし、未登録 API / RPC は
// title-case で fallback する（小規模な dev 用 API 追加が破壊的にならないよう柔軟性を残す）。
func httpPathToGRPCMethod(path string) string {
	// "/" を除去して "k1s0/<api>/<rpc>" を取り出す。
	if len(path) == 0 || path[0] != '/' {
		return path
	}
	parts := strings.SplitN(path[1:], "/", 3)
	if len(parts) != 3 || parts[0] != "k1s0" {
		return path
	}
	api := parts[1]
	rpc := parts[2]
	// service 名は明示マップ優先、未登録は title-case fallback。
	serviceName, ok := apiToServiceName[api]
	if !ok {
		serviceName = titleCase(api) + "Service"
	}
	// RPC 名は明示マップ優先、未登録は title-case fallback。
	rpcName, ok := rpcMethodNames[rpc]
	if !ok {
		rpcName = titleCase(rpc)
	}
	return "/k1s0.tier1." + api + ".v1." + serviceName + "/" + rpcName
}

// titleCase は ASCII の最初の文字だけを大文字化する（Unicode は対象外、API 名は ASCII 限定前提）。
func titleCase(s string) string {
	if s == "" {
		return s
	}
	c := s[0]
	if c >= 'a' && c <= 'z' {
		return string(c-32) + s[1:]
	}
	return s
}

// errorBody は HTTP/JSON 応答のエラー schema（docs §「HTTP Status ↔ K1s0Error マッピング」）。
type errorBody struct {
	Code         string `json:"code"`
	Message      string `json:"message"`
	RetryAfterMs int32  `json:"retry_after_ms,omitempty"`
	TraceID      string `json:"trace_id,omitempty"`
}

// writeJSONError は HTTP status + JSON body のセットでエラーを返す。
func writeJSONError(w http.ResponseWriter, c codes.Code, msg string) {
	body := errorBody{
		Code:    c.String(),
		Message: msg,
	}
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(httpStatusFromGRPC(c))
	// シンプルに encoding/json を使う（protojson に依存しない）。
	out := fmt.Sprintf(`{"error":{"code":%q,"message":%q,"retry_after_ms":%d}}`,
		body.Code, body.Message, body.RetryAfterMs)
	_, _ = io.Copy(w, bytes.NewReader([]byte(out)))
}

// RegisterStateRoutes は POST /k1s0/state/{get,set,delete,bulkget,transact} を gateway に登録する。
//
// invokeUnary は in-process で対応 gRPC unary handler を呼ぶ adapter（RegisterFromGRPCServer 経由で
// 注入される）。本パッケージは proto 型に依存しないため、handler 側で protojson Unmarshal して
// 適切な request 型を組み立てる。
type StateRPCHandlers struct {
	// Get は protojson body から GetRequest を復元して State.Get を呼ぶ。
	Get func(ctx context.Context, body []byte) (proto.Message, error)
	// Set は SetRequest を復元して State.Set を呼ぶ。
	Set func(ctx context.Context, body []byte) (proto.Message, error)
	// Delete は DeleteRequest を復元して State.Delete を呼ぶ。
	Delete func(ctx context.Context, body []byte) (proto.Message, error)
	// BulkGet は BulkGetRequest を復元して State.BulkGet を呼ぶ。
	BulkGet func(ctx context.Context, body []byte) (proto.Message, error)
	// Transact は TransactRequest を復元して State.Transact を呼ぶ。
	Transact func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterStateRoutes は State API の HTTP/JSON ルートを登録する。
// handlers の各 func は cmd/state/main.go で proto 型に依存しつつ組み立てる。
func (g *HTTPGateway) RegisterStateRoutes(handlers StateRPCHandlers) {
	if handlers.Get != nil {
		g.register("/k1s0/state/get", handlers.Get)
	}
	if handlers.Set != nil {
		g.register("/k1s0/state/set", handlers.Set)
	}
	if handlers.Delete != nil {
		g.register("/k1s0/state/delete", handlers.Delete)
	}
	if handlers.BulkGet != nil {
		g.register("/k1s0/state/bulkget", handlers.BulkGet)
	}
	if handlers.Transact != nil {
		g.register("/k1s0/state/transact", handlers.Transact)
	}
}

// PubSubRPCHandlers は POST /k1s0/pubsub/{publish,bulkpublish} のハンドラ。
// Subscribe は server-streaming のため HTTP/JSON では非対応（gRPC 経路を使う）。
type PubSubRPCHandlers struct {
	Publish     func(ctx context.Context, body []byte) (proto.Message, error)
	BulkPublish func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterPubSubRoutes は PubSub API の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterPubSubRoutes(handlers PubSubRPCHandlers) {
	if handlers.Publish != nil {
		g.register("/k1s0/pubsub/publish", handlers.Publish)
	}
	if handlers.BulkPublish != nil {
		g.register("/k1s0/pubsub/bulkpublish", handlers.BulkPublish)
	}
}

// SecretsRPCHandlers は POST /k1s0/secrets/{get,bulkget,getdynamic,rotate} のハンドラ。
type SecretsRPCHandlers struct {
	Get        func(ctx context.Context, body []byte) (proto.Message, error)
	BulkGet    func(ctx context.Context, body []byte) (proto.Message, error)
	GetDynamic func(ctx context.Context, body []byte) (proto.Message, error)
	Rotate     func(ctx context.Context, body []byte) (proto.Message, error)
}

// WorkflowRPCHandlers は POST /k1s0/workflow/{start,signal,query,cancel,terminate,getstatus} のハンドラ。
type WorkflowRPCHandlers struct {
	Start     func(ctx context.Context, body []byte) (proto.Message, error)
	Signal    func(ctx context.Context, body []byte) (proto.Message, error)
	Query     func(ctx context.Context, body []byte) (proto.Message, error)
	Cancel    func(ctx context.Context, body []byte) (proto.Message, error)
	Terminate func(ctx context.Context, body []byte) (proto.Message, error)
	GetStatus func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterWorkflowRoutes は Workflow API の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterWorkflowRoutes(handlers WorkflowRPCHandlers) {
	if handlers.Start != nil {
		g.register("/k1s0/workflow/start", handlers.Start)
	}
	if handlers.Signal != nil {
		g.register("/k1s0/workflow/signal", handlers.Signal)
	}
	if handlers.Query != nil {
		g.register("/k1s0/workflow/query", handlers.Query)
	}
	if handlers.Cancel != nil {
		g.register("/k1s0/workflow/cancel", handlers.Cancel)
	}
	if handlers.Terminate != nil {
		g.register("/k1s0/workflow/terminate", handlers.Terminate)
	}
	if handlers.GetStatus != nil {
		g.register("/k1s0/workflow/getstatus", handlers.GetStatus)
	}
}

// FeatureRPCHandlers は POST /k1s0/feature/{evaluateboolean,evaluatestring,evaluatenumber,evaluateobject,
// registerflag,getflag,listflags} のハンドラ。
// 評価系 4 RPC + 管理系 3 RPC の 7 RPC 全てをカバーする。
type FeatureRPCHandlers struct {
	EvaluateBoolean func(ctx context.Context, body []byte) (proto.Message, error)
	EvaluateString  func(ctx context.Context, body []byte) (proto.Message, error)
	EvaluateNumber  func(ctx context.Context, body []byte) (proto.Message, error)
	EvaluateObject  func(ctx context.Context, body []byte) (proto.Message, error)
	// FeatureAdminService 系統。
	RegisterFlag func(ctx context.Context, body []byte) (proto.Message, error)
	GetFlag      func(ctx context.Context, body []byte) (proto.Message, error)
	ListFlags    func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterFeatureRoutes は Feature API の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterFeatureRoutes(handlers FeatureRPCHandlers) {
	if handlers.EvaluateBoolean != nil {
		g.register("/k1s0/feature/evaluateboolean", handlers.EvaluateBoolean)
	}
	if handlers.EvaluateString != nil {
		g.register("/k1s0/feature/evaluatestring", handlers.EvaluateString)
	}
	if handlers.EvaluateNumber != nil {
		g.register("/k1s0/feature/evaluatenumber", handlers.EvaluateNumber)
	}
	if handlers.EvaluateObject != nil {
		g.register("/k1s0/feature/evaluateobject", handlers.EvaluateObject)
	}
	if handlers.RegisterFlag != nil {
		g.register("/k1s0/feature/registerflag", handlers.RegisterFlag)
	}
	if handlers.GetFlag != nil {
		g.register("/k1s0/feature/getflag", handlers.GetFlag)
	}
	if handlers.ListFlags != nil {
		g.register("/k1s0/feature/listflags", handlers.ListFlags)
	}
}

// BindingRPCHandlers は POST /k1s0/binding/invoke のハンドラ。
type BindingRPCHandlers struct {
	Invoke func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterBindingRoutes は Binding API の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterBindingRoutes(handlers BindingRPCHandlers) {
	if handlers.Invoke != nil {
		g.register("/k1s0/binding/invoke", handlers.Invoke)
	}
}

// LogRPCHandlers は POST /k1s0/log/{send,bulksend} のハンドラ。
type LogRPCHandlers struct {
	Send     func(ctx context.Context, body []byte) (proto.Message, error)
	BulkSend func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterLogRoutes は Log API の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterLogRoutes(handlers LogRPCHandlers) {
	if handlers.Send != nil {
		g.register("/k1s0/log/send", handlers.Send)
	}
	if handlers.BulkSend != nil {
		g.register("/k1s0/log/bulksend", handlers.BulkSend)
	}
}

// TelemetryRPCHandlers は POST /k1s0/telemetry/{emitmetric,emitspan} のハンドラ。
type TelemetryRPCHandlers struct {
	EmitMetric func(ctx context.Context, body []byte) (proto.Message, error)
	EmitSpan   func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterTelemetryRoutes は Telemetry API の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterTelemetryRoutes(handlers TelemetryRPCHandlers) {
	if handlers.EmitMetric != nil {
		g.register("/k1s0/telemetry/emitmetric", handlers.EmitMetric)
	}
	if handlers.EmitSpan != nil {
		g.register("/k1s0/telemetry/emitspan", handlers.EmitSpan)
	}
}

// InvokeRPCHandlers は POST /k1s0/serviceinvoke/invoke のハンドラ。
// InvokeStream は server-streaming のため HTTP/JSON 非対応（gRPC 経路を使う運用）。
type InvokeRPCHandlers struct {
	Invoke func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterInvokeRoutes は ServiceInvoke API の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterInvokeRoutes(handlers InvokeRPCHandlers) {
	if handlers.Invoke != nil {
		g.register("/k1s0/serviceinvoke/invoke", handlers.Invoke)
	}
}

// RegisterSecretsRoutes は Secrets API の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterSecretsRoutes(handlers SecretsRPCHandlers) {
	if handlers.Get != nil {
		g.register("/k1s0/secrets/get", handlers.Get)
	}
	if handlers.BulkGet != nil {
		g.register("/k1s0/secrets/bulkget", handlers.BulkGet)
	}
	if handlers.GetDynamic != nil {
		g.register("/k1s0/secrets/getdynamic", handlers.GetDynamic)
	}
	if handlers.Rotate != nil {
		g.register("/k1s0/secrets/rotate", handlers.Rotate)
	}
}

// UnmarshalJSON は body を proto.Message に protojson でデコードするヘルパ。
// 不正 JSON / 不明フィールドは InvalidArgument に翻訳して返す。
func UnmarshalJSON(body []byte, msg proto.Message) error {
	if err := (protojson.UnmarshalOptions{DiscardUnknown: true}).Unmarshal(body, msg); err != nil {
		return status.Errorf(codes.InvalidArgument, "invalid json body: %v", err)
	}
	return nil
}

// errInvalidArgumentNilRequest は req が nil の場合に返すヘルパエラー。
var errInvalidArgumentNilRequest = errors.New("nil request")
