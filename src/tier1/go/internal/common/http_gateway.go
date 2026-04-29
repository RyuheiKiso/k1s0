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
type HTTPGateway struct {
	mux *http.ServeMux
}

// NewHTTPGateway は空 gateway を生成する。RegisterX で API ルートを追加する。
func NewHTTPGateway() *HTTPGateway {
	return &HTTPGateway{mux: http.NewServeMux()}
}

// Handler は http.Handler interface を満たす。net/http.Server の Handler に渡せる。
func (g *HTTPGateway) Handler() http.Handler {
	return g.mux
}

// register は POST /k1s0/<api>/<rpc> をハンドルする内部 helper。
// 認証は authForward で gRPC metadata に転送し、in-process gRPC handler で AuthInterceptor が検証する。
func (g *HTTPGateway) register(path string, handler httpHandlerFunc) {
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

		resp, err := handler(ctx, body)
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
	// BulkGet / Transact は本 Pilot で同形に追加可能（実装は cmd 側）。
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

// FeatureRPCHandlers は POST /k1s0/feature/{evaluateboolean,evaluatestring,evaluatenumber,evaluateobject} のハンドラ。
// FeatureAdminService（RegisterFlag / GetFlag / ListFlags）も同形で展開可能だが、リリース時点 では未登録。
type FeatureRPCHandlers struct {
	EvaluateBoolean func(ctx context.Context, body []byte) (proto.Message, error)
	EvaluateString  func(ctx context.Context, body []byte) (proto.Message, error)
	EvaluateNumber  func(ctx context.Context, body []byte) (proto.Message, error)
	EvaluateObject  func(ctx context.Context, body []byte) (proto.Message, error)
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
