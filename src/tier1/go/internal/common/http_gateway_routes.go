// 本ファイルは tier1 共通 HTTP/JSON gateway の per-RPC ルート登録ヘルパ集。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「HTTP/JSON 互換インタフェース共通仕様」
//
// 分割の経緯:
//   元 http_gateway.go が 564 行となり src/CLAUDE.md「1 ファイル 500 行以内」を超過していた。
//   主軸（HTTPGateway 本体・interceptor 適用・error 整形）は http_gateway.go に残し、
//   API 別 RPCHandlers struct + RegisterXxxRoutes メソッド群を本ファイルに分離した。
//
// API 横並び:
//   - State / PubSub / Secrets / Workflow / Feature / Binding / Log / Telemetry / Invoke
//   各 API ごとに RPCHandlers struct + RegisterXxxRoutes(handlers) のペアを公開する。
//   handlers の各 func は cmd/<pod>/main.go 側で proto 型に依存しつつ組み立てる。

package common

import (
	// 標準。
	"context"
	"errors"

	// gRPC ステータス。
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	// protojson Unmarshal。
	"google.golang.org/protobuf/encoding/protojson"
	// proto.Message。
	"google.golang.org/protobuf/proto"
)

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

// PubSubRPCHandlers は PubSub API の HTTP/JSON ルート登録に必要な handler 集合。
type PubSubRPCHandlers struct {
	Publish     func(ctx context.Context, body []byte) (proto.Message, error)
	BulkPublish func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterPubSubRoutes は PubSub API（Publish / BulkPublish の 2 RPC）の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterPubSubRoutes(handlers PubSubRPCHandlers) {
	if handlers.Publish != nil {
		g.register("/k1s0/pubsub/publish", handlers.Publish)
	}
	if handlers.BulkPublish != nil {
		g.register("/k1s0/pubsub/bulkpublish", handlers.BulkPublish)
	}
}

// SecretsRPCHandlers は Secrets API の HTTP/JSON ルート登録に必要な handler 集合。
type SecretsRPCHandlers struct {
	Get        func(ctx context.Context, body []byte) (proto.Message, error)
	BulkGet    func(ctx context.Context, body []byte) (proto.Message, error)
	GetDynamic func(ctx context.Context, body []byte) (proto.Message, error)
	Rotate     func(ctx context.Context, body []byte) (proto.Message, error)
}

// WorkflowRPCHandlers は Workflow API の HTTP/JSON ルート登録に必要な handler 集合。
type WorkflowRPCHandlers struct {
	Start     func(ctx context.Context, body []byte) (proto.Message, error)
	Signal    func(ctx context.Context, body []byte) (proto.Message, error)
	Query     func(ctx context.Context, body []byte) (proto.Message, error)
	Cancel    func(ctx context.Context, body []byte) (proto.Message, error)
	Terminate func(ctx context.Context, body []byte) (proto.Message, error)
	GetStatus func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterWorkflowRoutes は Workflow API（6 RPC）の HTTP/JSON ルートを登録する。
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

// FeatureRPCHandlers は Feature API の HTTP/JSON ルート登録に必要な handler 集合。
type FeatureRPCHandlers struct {
	EvaluateBoolean func(ctx context.Context, body []byte) (proto.Message, error)
	EvaluateString  func(ctx context.Context, body []byte) (proto.Message, error)
	EvaluateNumber  func(ctx context.Context, body []byte) (proto.Message, error)
	EvaluateObject  func(ctx context.Context, body []byte) (proto.Message, error)
	RegisterFlag    func(ctx context.Context, body []byte) (proto.Message, error)
	GetFlag         func(ctx context.Context, body []byte) (proto.Message, error)
	ListFlags       func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterFeatureRoutes は Feature API（4 評価 RPC + Admin 3 RPC）の HTTP/JSON ルートを登録する。
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

// BindingRPCHandlers は Binding API の HTTP/JSON ルート登録に必要な handler 集合。
type BindingRPCHandlers struct {
	Invoke func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterBindingRoutes は Binding API（Invoke 1 RPC）の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterBindingRoutes(handlers BindingRPCHandlers) {
	if handlers.Invoke != nil {
		g.register("/k1s0/binding/invoke", handlers.Invoke)
	}
}

// LogRPCHandlers は Log API の HTTP/JSON ルート登録に必要な handler 集合。
type LogRPCHandlers struct {
	Send     func(ctx context.Context, body []byte) (proto.Message, error)
	BulkSend func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterLogRoutes は Log API（Send / BulkSend）の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterLogRoutes(handlers LogRPCHandlers) {
	if handlers.Send != nil {
		g.register("/k1s0/log/send", handlers.Send)
	}
	if handlers.BulkSend != nil {
		g.register("/k1s0/log/bulksend", handlers.BulkSend)
	}
}

// TelemetryRPCHandlers は Telemetry API の HTTP/JSON ルート登録に必要な handler 集合。
type TelemetryRPCHandlers struct {
	EmitMetric func(ctx context.Context, body []byte) (proto.Message, error)
	EmitSpan   func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterTelemetryRoutes は Telemetry API（2 RPC）の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterTelemetryRoutes(handlers TelemetryRPCHandlers) {
	if handlers.EmitMetric != nil {
		g.register("/k1s0/telemetry/emitmetric", handlers.EmitMetric)
	}
	if handlers.EmitSpan != nil {
		g.register("/k1s0/telemetry/emitspan", handlers.EmitSpan)
	}
}

// InvokeRPCHandlers は ServiceInvoke API の HTTP/JSON ルート登録に必要な handler 集合。
type InvokeRPCHandlers struct {
	Invoke func(ctx context.Context, body []byte) (proto.Message, error)
}

// RegisterInvokeRoutes は ServiceInvoke API（Invoke 1 RPC、stream は除く）の HTTP/JSON ルートを登録する。
func (g *HTTPGateway) RegisterInvokeRoutes(handlers InvokeRPCHandlers) {
	if handlers.Invoke != nil {
		g.register("/k1s0/serviceinvoke/invoke", handlers.Invoke)
	}
}

// RegisterSecretsRoutes は Secrets API（Get / BulkGet / GetDynamic / Rotate）の HTTP/JSON ルートを登録する。
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
