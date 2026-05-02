// 本ファイルは FR-T1-PUBSUB-001 / 002 メタデータ自動付与のテスト。
//
// 受け入れ基準:
//   FR-T1-PUBSUB-001: イベントに tenant_id / trace_id / event_id / published_at が自動付与される
//   FR-T1-PUBSUB-002: event_id は UUID v7 で自動生成（時系列ソート可能）

package state

import (
	"context"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	"google.golang.org/grpc/metadata"
)

// captureMetadataFn は publishFn として使えるラッパで、最後に観測した PublishRequest を保存する。
func captureMetadataFn(out *dapr.PublishRequest, mu *sync.Mutex) func(context.Context, dapr.PublishRequest) (dapr.PublishResponse, error) {
	return func(_ context.Context, req dapr.PublishRequest) (dapr.PublishResponse, error) {
		mu.Lock()
		*out = req
		mu.Unlock()
		return dapr.PublishResponse{Offset: 0}, nil
	}
}

// publishCaptureHandler は fake adapter を使って Publish の metadata を観測する handler を作る。
func publishCaptureHandler(t *testing.T) (*pubsubHandler, *dapr.PublishRequest) {
	t.Helper()
	var captured dapr.PublishRequest
	var mu sync.Mutex
	a := &fakePubSubAdapter{publishFn: captureMetadataFn(&captured, &mu)}
	return &pubsubHandler{deps: Deps{PubSubAdapter: a}}, &captured
}

// TestPublish_AutoInjectsEventID は event_id が UUID v7 で metadata に注入されることを確認する。
func TestPublish_AutoInjectsEventID(t *testing.T) {
	h, captured := publishCaptureHandler(t)
	_, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{
		Topic:   "orders",
		Data:    []byte("x"),
		Context: makeTenantCtx("T-foo"),
	})
	if err != nil {
		t.Fatalf("Publish: %v", err)
	}
	eid, ok := captured.Metadata[pubsubMetaKeyEventID]
	if !ok || eid == "" {
		t.Fatal("event_id not auto-injected")
	}
	if len(eid) != 36 || strings.Count(eid, "-") != 4 {
		t.Errorf("event_id not UUID format: %q", eid)
	}
	// UUID v7 は 15 桁目（index 14）の version nibble が "7"。
	if eid[14] != '7' {
		t.Errorf("event_id is not UUID v7 (version digit = %q): %q", eid[14:15], eid)
	}
}

// TestPublish_AutoInjectsTenantID は tier1 由来 tenant_id が metadata に必ず含まれることを確認する。
func TestPublish_AutoInjectsTenantID(t *testing.T) {
	h, captured := publishCaptureHandler(t)
	_, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{
		Topic:   "orders",
		Data:    []byte("x"),
		Context: makeTenantCtx("T-foo"),
	})
	if err != nil {
		t.Fatalf("Publish: %v", err)
	}
	if got := captured.Metadata[pubsubMetaKeyTenantID]; got != "T-foo" {
		t.Errorf("tenant_id metadata: got %q, want T-foo", got)
	}
}

// TestPublish_AutoInjectsPublishedAt は published_at が RFC 3339 で metadata に入ることを確認する。
func TestPublish_AutoInjectsPublishedAt(t *testing.T) {
	h, captured := publishCaptureHandler(t)
	_, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{
		Topic:   "orders",
		Data:    []byte("x"),
		Context: makeTenantCtx("T-foo"),
	})
	if err != nil {
		t.Fatalf("Publish: %v", err)
	}
	pat, ok := captured.Metadata[pubsubMetaKeyPublishedAt]
	if !ok {
		t.Fatal("published_at not auto-injected")
	}
	if _, err := time.Parse(time.RFC3339Nano, pat); err != nil {
		t.Errorf("published_at not RFC 3339: %q (parse error: %v)", pat, err)
	}
}

// TestPublish_ExtractsTraceIDFromTraceparent は incoming traceparent ヘッダから
// trace-id 部（32 文字 hex）を取り出して metadata に入れることを確認する。
func TestPublish_ExtractsTraceIDFromTraceparent(t *testing.T) {
	h, captured := publishCaptureHandler(t)
	traceID := "0af7651916cd43dd8448eb211c80319c"
	traceparent := "00-" + traceID + "-b7ad6b7169203331-01"
	ctx := metadata.NewIncomingContext(context.Background(), metadata.Pairs("traceparent", traceparent))
	_, err := h.Publish(ctx, &pubsubv1.PublishRequest{
		Topic:   "orders",
		Data:    []byte("x"),
		Context: makeTenantCtx("T-foo"),
	})
	if err != nil {
		t.Fatalf("Publish: %v", err)
	}
	if got := captured.Metadata[pubsubMetaKeyTraceID]; got != traceID {
		t.Errorf("trace_id metadata: got %q, want %q", got, traceID)
	}
}

// TestPublish_PreservesUserMetadata はユーザー指定 metadata を保持しつつ自動付与することを確認する。
func TestPublish_PreservesUserMetadata(t *testing.T) {
	h, captured := publishCaptureHandler(t)
	_, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{
		Topic:    "orders",
		Data:     []byte("x"),
		Metadata: map[string]string{"partition_key": "user-42", "custom": "v1"},
		Context:  makeTenantCtx("T-foo"),
	})
	if err != nil {
		t.Fatalf("Publish: %v", err)
	}
	if got := captured.Metadata["partition_key"]; got != "user-42" {
		t.Errorf("user metadata partition_key lost: %q", got)
	}
	if got := captured.Metadata["custom"]; got != "v1" {
		t.Errorf("user metadata custom lost: %q", got)
	}
	if captured.Metadata[pubsubMetaKeyEventID] == "" {
		t.Error("event_id not auto-injected when user metadata is present")
	}
}

// TestPublish_RespectsExistingEventID は呼出側が明示的に event_id を指定した場合、
// 上書きしないことを確認する（例: SDK 側 dedup 用に固定 event_id を渡したいケース）。
func TestPublish_RespectsExistingEventID(t *testing.T) {
	h, captured := publishCaptureHandler(t)
	_, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{
		Topic:    "orders",
		Data:     []byte("x"),
		Metadata: map[string]string{pubsubMetaKeyEventID: "user-supplied-id"},
		Context:  makeTenantCtx("T-foo"),
	})
	if err != nil {
		t.Fatalf("Publish: %v", err)
	}
	if got := captured.Metadata[pubsubMetaKeyEventID]; got != "user-supplied-id" {
		t.Errorf("user-supplied event_id was overwritten: got %q", got)
	}
}

// TestPublish_AlwaysOverwritesTenantID は tenant_id を必ず tier1 由来値で上書きすることを確認する
// （共通規約 §「マルチテナント分離」L1: 自己宣言は拒否、tier1 が決定）。
func TestPublish_AlwaysOverwritesTenantID(t *testing.T) {
	h, captured := publishCaptureHandler(t)
	_, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{
		Topic:    "orders",
		Data:     []byte("x"),
		Metadata: map[string]string{pubsubMetaKeyTenantID: "OTHER-TENANT"},
		Context:  makeTenantCtx("T-foo"),
	})
	if err != nil {
		t.Fatalf("Publish: %v", err)
	}
	if got := captured.Metadata[pubsubMetaKeyTenantID]; got != "T-foo" {
		t.Errorf("tenant_id should be overwritten with tier1 value: got %q, want T-foo", got)
	}
}
