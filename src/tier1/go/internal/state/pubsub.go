// 本ファイルは t1-state Pod の PubSubService 3 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/03_PubSub_API.md
//
// scope（リリース時点 placeholder）: 実 Dapr Pub/Sub（Kafka）結線は plan 04-05。

package state

// 標準 / 内部パッケージ。
import (
	// context 伝搬。
	"context"
	// Dapr adapter。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// SDK 生成 stub の PubSubService 型。
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// pubsubHandler は PubSubService の handler 実装。
type pubsubHandler struct {
	// 将来 RPC 用埋め込み。
	pubsubv1.UnimplementedPubSubServiceServer
	// adapter 集合。
	deps Deps
}

// Publish は単発 Publish。
func (h *pubsubHandler) Publish(ctx context.Context, req *pubsubv1.PublishRequest) (*pubsubv1.PublishResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/pubsub: nil request")
	}
	// adapter 入力に変換。
	areq := dapr.PublishRequest{
		// Component は Publish の topic と分離（運用設定で確定）。本リリース時点 は固定値「pubsub-kafka」。
		Component: "pubsub-kafka",
		// トピック名。
		Topic: req.GetTopic(),
		// データ本文。
		Data: req.GetData(),
		// Content-Type。
		ContentType: req.GetContentType(),
		// 冪等性キー。
		IdempotencyKey: req.GetIdempotencyKey(),
		// メタデータ。
		Metadata: req.GetMetadata(),
		// テナント。
		TenantID: tenantIDOf(req.GetContext()),
	}
	// adapter 呼出。
	aresp, err := h.deps.PubSubAdapter.Publish(ctx, areq)
	// エラー翻訳。
	if err != nil {
		// 翻訳して返却する。
		return nil, translatePubSubErr(err, "Publish")
	}
	// 応答返却。
	return &pubsubv1.PublishResponse{
		// Kafka offset。
		Offset: aresp.Offset,
	}, nil
}

// BulkPublish はバッチ Publish。本リリース時点 では Unimplemented。
func (h *pubsubHandler) BulkPublish(_ context.Context, _ *pubsubv1.BulkPublishRequest) (*pubsubv1.BulkPublishResponse, error) {
	// 直接 Unimplemented 返却。
	return nil, status.Error(codes.Unimplemented, "tier1/pubsub: BulkPublish not yet wired (plan 04-05)")
}

// Subscribe はサブスクリプション stream。本リリース時点 では Unimplemented。
func (h *pubsubHandler) Subscribe(_ *pubsubv1.SubscribeRequest, _ pubsubv1.PubSubService_SubscribeServer) error {
	// stream は plan 04-05 で実装。
	return status.Error(codes.Unimplemented, "tier1/pubsub: Subscribe not yet wired (plan 04-05)")
}

// translatePubSubErr は PubSub 用エラー翻訳。
func translatePubSubErr(err error, rpc string) error {
	// ErrNotWired → Unimplemented。
	if isNotWired(err) {
		// 翻訳メッセージ。
		return status.Errorf(codes.Unimplemented, "tier1/pubsub: %s not yet wired to Dapr backend (plan 04-05)", rpc)
	}
	// その他 → Internal。
	return status.Errorf(codes.Internal, "tier1/pubsub: %s adapter error: %v", rpc, err)
}
