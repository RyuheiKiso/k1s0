// 本ファイルは FR-T1-PUBSUB-005 の「イベントサイズ上限 1MB」を担保するテスト。
//
// Kafka の既定 message.max.bytes は 1 MiB 相当のため、handler 段で同上限を強制し、
// adapter 越しに Kafka へ "Message too large" を返させない（observability 改善）。

package state

import (
	"context"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// TestPublish_RejectsDataOver1MiB は 1 MiB + 1 byte の Data 投入が ResourceExhausted で
// 弾かれることを確認する。adapter 側にリクエストが伝播していないことも検証する。
func TestPublish_RejectsDataOver1MiB(t *testing.T) {
	adapterCalled := false
	a := &fakePubSubAdapter{
		publishFn: func(_ context.Context, _ dapr.PublishRequest) (dapr.PublishResponse, error) {
			adapterCalled = true
			return dapr.PublishResponse{}, nil
		},
	}
	h := newPubSubHandler(a)
	big := make([]byte, pubsubMaxEventBytes+1)
	_, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{
		Topic:   "orders",
		Data:    big,
		Context: makeTenantCtx("T"),
	})
	if got := status.Code(err); got != codes.ResourceExhausted {
		t.Fatalf("status: got %v want ResourceExhausted", got)
	}
	if adapterCalled {
		t.Error("adapter should not be reached when data exceeds 1 MiB")
	}
}

// TestPublish_AcceptsDataAt1MiB は境界値（ちょうど 1 MiB）が通ることを確認する。
func TestPublish_AcceptsDataAt1MiB(t *testing.T) {
	a := &fakePubSubAdapter{
		publishFn: func(_ context.Context, _ dapr.PublishRequest) (dapr.PublishResponse, error) {
			return dapr.PublishResponse{Offset: 1}, nil
		},
	}
	h := newPubSubHandler(a)
	at := make([]byte, pubsubMaxEventBytes)
	_, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{
		Topic:   "orders",
		Data:    at,
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("1 MiB should be accepted: %v", err)
	}
}

// TestBulkPublish_PartialFailureOnOversizedEntry は BulkPublish の部分成功挙動を担保する。
// 1 entry が 1 MiB 超過、もう 1 entry は許容内のとき、前者のみ ResourceExhausted で
// error_code 付き、後者は offset 付きで成功する。
func TestBulkPublish_PartialFailureOnOversizedEntry(t *testing.T) {
	publishedCount := 0
	a := &fakePubSubAdapter{
		publishFn: func(_ context.Context, _ dapr.PublishRequest) (dapr.PublishResponse, error) {
			publishedCount++
			return dapr.PublishResponse{Offset: int64(publishedCount)}, nil
		},
	}
	h := newPubSubHandler(a)
	big := make([]byte, pubsubMaxEventBytes+1)
	resp, err := h.BulkPublish(context.Background(), &pubsubv1.BulkPublishRequest{
		Topic: "orders",
		Entries: []*pubsubv1.PublishRequest{
			{Topic: "orders", Data: big, Context: makeTenantCtx("T")},
			{Topic: "orders", Data: []byte("ok"), Context: makeTenantCtx("T")},
		},
	})
	if err != nil {
		t.Fatalf("BulkPublish should not error overall: %v", err)
	}
	if got := len(resp.GetResults()); got != 2 {
		t.Fatalf("results count: got %d want 2", got)
	}
	if resp.GetResults()[0].GetErrorCode() == "" {
		t.Error("oversized entry should report error_code")
	}
	if resp.GetResults()[1].GetErrorCode() != "" {
		t.Errorf("normal entry should succeed, got error_code=%q", resp.GetResults()[1].GetErrorCode())
	}
	if publishedCount != 1 {
		t.Errorf("only the small entry should have been published, got %d", publishedCount)
	}
}
