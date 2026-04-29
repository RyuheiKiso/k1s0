// 本ファイルは Feature / Binding / Log / Telemetry / ServiceInvoke API の
// HTTP/JSON gateway 用 RPC ハンドラ adapter。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md §「HTTP/JSON 互換」
//
// 役割:
//   t1-state Pod が登録する 5 service handler のうち State / PubSub 以外
//   （Feature / Binding / Log / Telemetry / ServiceInvoke）の HTTP/JSON adapter を提供する。
//   Subscribe / InvokeStream は server-streaming のため非対応。

package state

import (
	"context"

	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	bindingv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/binding/v1"
	featurev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/feature/v1"
	logv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/log/v1"
	serviceinvokev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/serviceinvoke/v1"
	telemetryv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/telemetry/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
)

// errNotWired は handler 未注入時の標準エラー。
func errSvcNotWired(name string) error {
	return status.Errorf(codes.Unavailable, "%s service not wired", name)
}

// MakeHTTPFeatureHandlers は HTTP/JSON gateway 用 Feature handler 集合を組み立てる。
// 評価系 4 RPC + 管理系 3 RPC の 7 RPC 全てをカバーする。adminSvc が nil の場合は
// 評価系のみ登録し、admin route は未登録扱い（ハンドラ nil → register でスキップ）。
func MakeHTTPFeatureHandlers(svc featurev1.FeatureServiceServer, adminSvc featurev1.FeatureAdminServiceServer) common.FeatureRPCHandlers {
	out := common.FeatureRPCHandlers{
		EvaluateBoolean: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &featurev1.EvaluateRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, errSvcNotWired("feature")
			}
			return svc.EvaluateBoolean(ctx, req)
		},
		EvaluateString: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &featurev1.EvaluateRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, errSvcNotWired("feature")
			}
			return svc.EvaluateString(ctx, req)
		},
		EvaluateNumber: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &featurev1.EvaluateRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, errSvcNotWired("feature")
			}
			return svc.EvaluateNumber(ctx, req)
		},
		EvaluateObject: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &featurev1.EvaluateRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, errSvcNotWired("feature")
			}
			return svc.EvaluateObject(ctx, req)
		},
	}
	// FeatureAdminService が注入されている時のみ管理系 3 RPC を登録する。
	if adminSvc != nil {
		out.RegisterFlag = func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &featurev1.RegisterFlagRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			return adminSvc.RegisterFlag(ctx, req)
		}
		out.GetFlag = func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &featurev1.GetFlagRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			return adminSvc.GetFlag(ctx, req)
		}
		out.ListFlags = func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &featurev1.ListFlagsRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			return adminSvc.ListFlags(ctx, req)
		}
	}
	return out
}

// NewFeatureServiceServer は HTTP gateway / 統合テスト用 exported helper。
func NewFeatureServiceServer(deps Deps) featurev1.FeatureServiceServer {
	return &featureHandler{deps: deps}
}

// MakeHTTPBindingHandlers は HTTP/JSON gateway 用 Binding handler 集合を組み立てる。
func MakeHTTPBindingHandlers(svc bindingv1.BindingServiceServer) common.BindingRPCHandlers {
	return common.BindingRPCHandlers{
		Invoke: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &bindingv1.InvokeBindingRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, errSvcNotWired("binding")
			}
			return svc.Invoke(ctx, req)
		},
	}
}

// NewBindingServiceServer は HTTP gateway / 統合テスト用 exported helper。
func NewBindingServiceServer(deps Deps) bindingv1.BindingServiceServer {
	return &bindingHandler{deps: deps}
}

// MakeHTTPLogHandlers は HTTP/JSON gateway 用 Log handler 集合を組み立てる。
func MakeHTTPLogHandlers(svc logv1.LogServiceServer) common.LogRPCHandlers {
	return common.LogRPCHandlers{
		Send: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &logv1.SendLogRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, errSvcNotWired("log")
			}
			return svc.Send(ctx, req)
		},
		BulkSend: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &logv1.BulkSendLogRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, errSvcNotWired("log")
			}
			return svc.BulkSend(ctx, req)
		},
	}
}

// NewLogServiceServer は HTTP gateway / 統合テスト用 exported helper。
func NewLogServiceServer(deps Deps) logv1.LogServiceServer {
	return &logHandler{deps: deps}
}

// MakeHTTPTelemetryHandlers は HTTP/JSON gateway 用 Telemetry handler 集合を組み立てる。
func MakeHTTPTelemetryHandlers(svc telemetryv1.TelemetryServiceServer) common.TelemetryRPCHandlers {
	return common.TelemetryRPCHandlers{
		EmitMetric: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &telemetryv1.EmitMetricRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, errSvcNotWired("telemetry")
			}
			return svc.EmitMetric(ctx, req)
		},
		EmitSpan: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &telemetryv1.EmitSpanRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, errSvcNotWired("telemetry")
			}
			return svc.EmitSpan(ctx, req)
		},
	}
}

// NewTelemetryServiceServer は HTTP gateway / 統合テスト用 exported helper。
func NewTelemetryServiceServer(deps Deps) telemetryv1.TelemetryServiceServer {
	return &telemetryHandler{deps: deps}
}

// MakeHTTPInvokeHandlers は HTTP/JSON gateway 用 ServiceInvoke handler 集合を組み立てる。
// InvokeStream は server-streaming のため HTTP/JSON 非対応（gRPC 経路）。
func MakeHTTPInvokeHandlers(svc serviceinvokev1.InvokeServiceServer) common.InvokeRPCHandlers {
	return common.InvokeRPCHandlers{
		Invoke: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &serviceinvokev1.InvokeRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, errSvcNotWired("serviceinvoke")
			}
			return svc.Invoke(ctx, req)
		},
	}
}

// NewInvokeServiceServer は HTTP gateway / 統合テスト用 exported helper。
func NewInvokeServiceServer(deps Deps) serviceinvokev1.InvokeServiceServer {
	return &invokeHandler{deps: deps}
}
