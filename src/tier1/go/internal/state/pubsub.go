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
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantID(req.GetContext(), "PubSub.Publish")
	if err != nil {
		return nil, err
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
		TenantID: tid,
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

// BulkPublish は複数 message を順次 Publish する。
// Dapr SDK は単一 PublishEvent しか提供しないため、ループで逐次発行する。
// 1 件失敗で全体失敗（部分発行済 message のロールバックは pubsub backend
// 仕様に依存するため、呼出側で冪等性キーによる重複発行検知を前提とする）。
func (h *pubsubHandler) BulkPublish(ctx context.Context, req *pubsubv1.BulkPublishRequest) (*pubsubv1.BulkPublishResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/pubsub: nil request")
	}
	for i, entry := range req.GetEntries() {
		// 各 entry は PublishRequest（topic / data / content_type / idempotency_key /
		// metadata / context）。BulkPublishRequest 側で topic を共通化しているため
		// entry.topic と齟齬がある場合は entry を優先する。
		topic := entry.GetTopic()
		if topic == "" {
			topic = req.GetTopic()
		}
		// NFR-E-AC-003: 各 entry も tenant_id 越境防止のため必須検証。
		entTid, err := requireTenantID(entry.GetContext(), "PubSub.BulkPublish")
		if err != nil {
			return nil, err
		}
		areq := dapr.PublishRequest{
			Component:      "pubsub-kafka",
			Topic:          topic,
			Data:           entry.GetData(),
			ContentType:    entry.GetContentType(),
			IdempotencyKey: entry.GetIdempotencyKey(),
			Metadata:       entry.GetMetadata(),
			TenantID:       entTid,
		}
		if _, err := h.deps.PubSubAdapter.Publish(ctx, areq); err != nil {
			return nil, status.Errorf(codes.Internal, "tier1/pubsub: BulkPublish failed at entry %d: %v", i, err)
		}
	}
	return &pubsubv1.BulkPublishResponse{}, nil
}

// Subscribe は server-streaming RPC。Dapr Subscribe で得た subscription から
// 逐次イベントを受信し、proto Event として gRPC stream クライアントへ転送する。
//
// stream context が cancel されると subscription を Close し関数を戻す。
// 送信成功後に ev.Ack()、stream.Send 失敗時は ev.Retry() を呼んで Dapr 側で再配信させる。
func (h *pubsubHandler) Subscribe(req *pubsubv1.SubscribeRequest, stream pubsubv1.PubSubService_SubscribeServer) error {
	if req == nil {
		return status.Error(codes.InvalidArgument, "tier1/pubsub: nil request")
	}
	if h.deps.PubSubAdapter == nil {
		return status.Error(codes.Unimplemented, "tier1/pubsub: Subscribe not yet wired")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, terr := requireTenantID(req.GetContext(), "PubSub.Subscribe")
	if terr != nil {
		return terr
	}
	ctx := stream.Context()
	sub, err := h.deps.PubSubAdapter.Subscribe(ctx, dapr.SubscribeAdapterRequest{
		Component:     "pubsub-kafka",
		Topic:         req.GetTopic(),
		ConsumerGroup: req.GetConsumerGroup(),
		TenantID:      tid,
	})
	if err != nil {
		if isNotWired(err) {
			return status.Error(codes.Unimplemented, "tier1/pubsub: Subscribe not yet wired to Dapr backend")
		}
		return status.Errorf(codes.Internal, "tier1/pubsub: Subscribe failed: %v", err)
	}
	defer func() { _ = sub.Close() }()

	for {
		// stream cancel チェック（Receive が block する前に context 状況を確認）。
		if err := ctx.Err(); err != nil {
			return err
		}
		ev, err := sub.Receive(ctx)
		if err != nil {
			// adapter 側で「subscription closed」を通常終了として返す場合は io.EOF など。
			// 単純化のため、エラーは Internal として返却。
			return status.Errorf(codes.Internal, "tier1/pubsub: Subscribe receive: %v", err)
		}
		if ev == nil {
			// イベント無しは無視して次へ。
			continue
		}
		out := &pubsubv1.Event{
			Topic:       ev.Topic,
			Data:        ev.Data,
			ContentType: ev.ContentType,
			Offset:      ev.Offset,
			Metadata:    ev.Metadata,
		}
		if err := stream.Send(out); err != nil {
			// クライアントへの転送失敗 → Dapr 側で再配信させる。
			if ev.Retry != nil {
				_ = ev.Retry()
			}
			return status.Errorf(codes.Internal, "tier1/pubsub: stream.Send: %v", err)
		}
		// 転送成功 → ack。
		if ev.Ack != nil {
			_ = ev.Ack()
		}
	}
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
