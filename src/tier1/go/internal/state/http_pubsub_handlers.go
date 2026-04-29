// 本ファイルは PubSub API の HTTP/JSON gateway 用 RPC ハンドラ adapter。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md §「HTTP/JSON 互換」
//
// 役割:
//   common.HTTPGateway.RegisterPubSubRoutes に渡す common.PubSubRPCHandlers を組み立てる。
//   Subscribe は server-streaming のため HTTP/JSON 非対応（docs §「HTTP/JSON 互換」内に
//   "新規 API 設計の優先路ではない" との記載あり、stream は gRPC 経路を使う運用）。

package state

import (
	"context"

	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
)

// MakeHTTPPubSubHandlers は HTTP/JSON gateway 用 PubSub handler 集合を組み立てる。
// pubsubSvc は in-process PubSubServiceServer 実装（Register hook が登録するもの）。
func MakeHTTPPubSubHandlers(pubsubSvc pubsubv1.PubSubServiceServer) common.PubSubRPCHandlers {
	return common.PubSubRPCHandlers{
		Publish: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &pubsubv1.PublishRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if pubsubSvc == nil {
				return nil, status.Error(codes.Unavailable, "pubsub service not wired")
			}
			return pubsubSvc.Publish(ctx, req)
		},
		BulkPublish: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &pubsubv1.BulkPublishRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if pubsubSvc == nil {
				return nil, status.Error(codes.Unavailable, "pubsub service not wired")
			}
			return pubsubSvc.BulkPublish(ctx, req)
		},
	}
}

// NewPubSubServiceServer は HTTP gateway / 統合テスト用に pubsubHandler を直接生成する exported helper。
func NewPubSubServiceServer(deps Deps) pubsubv1.PubSubServiceServer {
	return &pubsubHandler{deps: deps}
}
