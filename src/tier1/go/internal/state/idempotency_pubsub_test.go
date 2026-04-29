// 本ファイルは PubSub.Publish の idempotency_key dedup 動作の結合テスト。
//
// 検証観点:
//   1. 同一 idempotency_key の再試行で adapter.Publish は 1 回しか呼ばれない
//   2. 異なる idempotency_key は独立に処理される
//   3. tenant 分離: 同一 client key でも別 tenant は別 dedup
//   4. 空 idempotency_key は dedup 対象外（毎回 adapter 呼出）

package state

import (
	"context"
	"sync/atomic"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
)

// fakePublishAdapter は呼出回数だけ数える PubSubAdapter（最小 fake）。
type fakePublishAdapter struct {
	dapr.PubSubAdapter
	publishCalls int32
}

func (f *fakePublishAdapter) Publish(_ context.Context, _ dapr.PublishRequest) (dapr.PublishResponse, error) {
	atomic.AddInt32(&f.publishCalls, 1)
	return dapr.PublishResponse{Offset: 0}, nil
}

// 同一 idempotency_key の再試行で adapter は 1 回しか呼ばれない。
func TestPubSubPublish_Idempotent_DedupAcrossRetries(t *testing.T) {
	fake := &fakePublishAdapter{}
	h := &pubsubHandler{
		deps:        Deps{PubSubAdapter: fake},
		idempotency: common.NewInMemoryIdempotencyCache(0),
	}
	req := &pubsubv1.PublishRequest{
		Topic:          "events",
		Data:           []byte("v1"),
		IdempotencyKey: "client-uuid-1",
		Context:        &commonv1.TenantContext{TenantId: "T"},
	}
	for i := 0; i < 3; i++ {
		if _, err := h.Publish(context.Background(), req); err != nil {
			t.Fatalf("call %d: %v", i, err)
		}
	}
	if got := atomic.LoadInt32(&fake.publishCalls); got != 1 {
		t.Fatalf("publishCalls = %d want 1 (idempotent)", got)
	}
}

// 異なる idempotency_key は独立に adapter 呼出される。
func TestPubSubPublish_Idempotent_DistinctKeys(t *testing.T) {
	fake := &fakePublishAdapter{}
	h := &pubsubHandler{
		deps:        Deps{PubSubAdapter: fake},
		idempotency: common.NewInMemoryIdempotencyCache(0),
	}
	mk := func(k string) *pubsubv1.PublishRequest {
		return &pubsubv1.PublishRequest{
			Topic:          "t",
			Data:           []byte("v"),
			IdempotencyKey: k,
			Context:        &commonv1.TenantContext{TenantId: "T"},
		}
	}
	for _, k := range []string{"k1", "k2", "k3"} {
		if _, err := h.Publish(context.Background(), mk(k)); err != nil {
			t.Fatalf("%s: %v", k, err)
		}
	}
	if got := atomic.LoadInt32(&fake.publishCalls); got != 3 {
		t.Fatalf("publishCalls = %d want 3", got)
	}
}

// tenant 分離: 同 client key でも別 tenant は別 dedup（adapter 呼出 2 回）。
func TestPubSubPublish_Idempotent_TenantIsolation(t *testing.T) {
	fake := &fakePublishAdapter{}
	h := &pubsubHandler{
		deps:        Deps{PubSubAdapter: fake},
		idempotency: common.NewInMemoryIdempotencyCache(0),
	}
	mk := func(tenant string) *pubsubv1.PublishRequest {
		return &pubsubv1.PublishRequest{
			Topic:          "t",
			Data:           []byte("v"),
			IdempotencyKey: "shared-client-key",
			Context:        &commonv1.TenantContext{TenantId: tenant},
		}
	}
	if _, err := h.Publish(context.Background(), mk("A")); err != nil {
		t.Fatal(err)
	}
	if _, err := h.Publish(context.Background(), mk("B")); err != nil {
		t.Fatal(err)
	}
	if got := atomic.LoadInt32(&fake.publishCalls); got != 2 {
		t.Fatalf("publishCalls = %d want 2 (tenant isolation)", got)
	}
}

// 空 idempotency_key は dedup 対象外（毎回 adapter 呼出）。
func TestPubSubPublish_NoIdempotencyKey_AlwaysCallsAdapter(t *testing.T) {
	fake := &fakePublishAdapter{}
	h := &pubsubHandler{
		deps:        Deps{PubSubAdapter: fake},
		idempotency: common.NewInMemoryIdempotencyCache(0),
	}
	req := &pubsubv1.PublishRequest{
		Topic:   "t",
		Data:    []byte("v"),
		Context: &commonv1.TenantContext{TenantId: "T"},
		// IdempotencyKey 空文字 → dedup スキップ。
	}
	for i := 0; i < 3; i++ {
		if _, err := h.Publish(context.Background(), req); err != nil {
			t.Fatalf("%d: %v", i, err)
		}
	}
	if got := atomic.LoadInt32(&fake.publishCalls); got != 3 {
		t.Fatalf("publishCalls = %d want 3 (no dedup without key)", got)
	}
}
