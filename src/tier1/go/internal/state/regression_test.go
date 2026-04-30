// 本ファイルは F1〜H1 で潰した bug の regression を防ぐ unit test。
// state package の handler 単体テストとして CI で再発検知できるよう集約する。

package state

import (
	"context"
	"strings"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
)

// newAuthCtx は test 用に AuthInfo を context に注入するヘルパ。
func newAuthCtx(tenantID, subject string) context.Context {
	return common.NewAuthContext(context.Background(), &common.AuthInfo{
		TenantID: tenantID,
		Subject:  subject,
	})
}

// F4 regression: BulkPublish の partial success 仕様。
// docs §「PubSub API」: 「配列内の各エントリに個別の結果を返す（部分成功あり）」
// 1 entry 不正でも全 entry の結果を蓄積、tenant_id 不在のような全体前提違反のみ
// 即時 InvalidArgument。
func TestPubSubHandler_BulkPublish_PartialSuccess(t *testing.T) {
	calls := 0
	a := &fakePubSubAdapter{
		publishFn: func(_ context.Context, req dapr.PublishRequest) (dapr.PublishResponse, error) {
			calls++
			return dapr.PublishResponse{Offset: int64(calls)}, nil
		},
	}
	h := newPubSubHandler(a)
	// entries[0] / [2] は正常 topic、[1] は invalid (空白含む)
	req := &pubsubv1.BulkPublishRequest{
		Topic: "valid-topic",
		Entries: []*pubsubv1.PublishRequest{
			{
				Topic:   "valid-topic",
				Data:    []byte("ok-1"),
				Context: &commonv1.TenantContext{TenantId: "T1"},
			},
			{
				Topic:   "bad topic with spaces",
				Data:    []byte("bad"),
				Context: &commonv1.TenantContext{TenantId: "T1"},
			},
			{
				Topic:   "valid-topic",
				Data:    []byte("ok-2"),
				Context: &commonv1.TenantContext{TenantId: "T1"},
			},
		},
	}
	resp, err := h.BulkPublish(context.Background(), req)
	if err != nil {
		t.Fatalf("BulkPublish should not error on per-entry topic invalid; got %v", err)
	}
	if got := len(resp.GetResults()); got != 3 {
		t.Fatalf("expected 3 per-entry results, got %d", got)
	}
	// entries[0] / [2] は成功、entries[1] は ErrorCode に invalid topic を含む
	if resp.Results[0].GetErrorCode() != "" {
		t.Errorf("entry 0 should be successful, got error_code=%q", resp.Results[0].GetErrorCode())
	}
	if !strings.Contains(resp.Results[1].GetErrorCode(), "invalid topic") {
		t.Errorf("entry 1 should fail with 'invalid topic', got %q", resp.Results[1].GetErrorCode())
	}
	if resp.Results[2].GetErrorCode() != "" {
		t.Errorf("entry 2 should be successful, got error_code=%q", resp.Results[2].GetErrorCode())
	}
	if calls != 2 {
		t.Errorf("publish should be called for 2 valid entries, got %d", calls)
	}
}

// F4 (b): tenant_id 不在は全体停止 (security audit 整合性のため)。
func TestPubSubHandler_BulkPublish_MissingTenantStopsAll(t *testing.T) {
	a := &fakePubSubAdapter{
		publishFn: func(_ context.Context, _ dapr.PublishRequest) (dapr.PublishResponse, error) {
			t.Fatal("publish should not be called when tenant_id missing")
			return dapr.PublishResponse{}, nil
		},
	}
	h := newPubSubHandler(a)
	_, err := h.BulkPublish(context.Background(), &pubsubv1.BulkPublishRequest{
		Topic: "valid-topic",
		Entries: []*pubsubv1.PublishRequest{
			{Topic: "valid-topic", Data: []byte("x"), Context: &commonv1.TenantContext{}},
		},
	})
	if err == nil {
		t.Fatal("missing tenant_id should error")
	}
	if status.Code(err) != codes.InvalidArgument {
		t.Fatalf("expected InvalidArgument, got %v", status.Code(err))
	}
}

// G3 regression: state handler 段で cross-tenant boundary を二重防御する。
func TestStateHandler_Get_CrossTenantRejected(t *testing.T) {
	// AuthInfo を context に注入（JWT 由来 tenant-A）
	ctx := newAuthCtx("tenant-A", "alice")
	h := &stateHandler{}
	_, err := h.Get(ctx, &statev1.GetRequest{
		Context: &commonv1.TenantContext{TenantId: "tenant-B"},
		Store:   "kv",
		Key:     "k",
	})
	if err == nil {
		t.Fatal("cross-tenant should be rejected")
	}
	if status.Code(err) != codes.PermissionDenied {
		t.Fatalf("expected PermissionDenied, got %v", status.Code(err))
	}
	if !strings.Contains(err.Error(), "cross-tenant") {
		t.Fatalf("error should mention cross-tenant, got: %v", err)
	}
}

// 同 tenant_id なら通常の handler 経路に進む（store/key 検証で停止 = OK）。
func TestStateHandler_Get_MatchingTenantPassesAuthCheck(t *testing.T) {
	ctx := newAuthCtx("tenant-A", "alice")
	h := &stateHandler{}
	// store="" で adapter 到達前に弾かれる事を確認（auth check は通過したことの証明）
	_, err := h.Get(ctx, &statev1.GetRequest{
		Context: &commonv1.TenantContext{TenantId: "tenant-A"},
		Store:   "",
		Key:     "k",
	})
	if err == nil {
		t.Fatal("empty store should error")
	}
	// auth は通過、store check で fail
	if status.Code(err) != codes.InvalidArgument {
		t.Fatalf("expected InvalidArgument (store required), got %v / %v", status.Code(err), err)
	}
	if strings.Contains(err.Error(), "cross-tenant") {
		t.Fatalf("auth check should pass with matching tenant; got: %v", err)
	}
}
