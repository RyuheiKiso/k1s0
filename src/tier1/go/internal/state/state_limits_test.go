// 本ファイルは FR-T1-STATE-003 / 005 の上限値検証テスト。
//
// 受け入れ基準:
//   FR-T1-STATE-003: BulkGet は 1 回の呼び出しで最大 100 キーを処理
//   FR-T1-STATE-005: トランザクションは最大 10 操作 / call

package state

import (
	"context"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// TestBulkGet_RejectsOver100Keys は 101 キー要求が ResourceExhausted で弾かれることを確認する。
func TestBulkGet_RejectsOver100Keys(t *testing.T) {
	adapterCalled := false
	a := &fakeStateAdapter{
		bulkGetFn: func(_ context.Context, _ dapr.StateBulkGetRequest) ([]dapr.StateBulkGetItem, error) {
			adapterCalled = true
			return nil, nil
		},
	}
	h := newHandler(a)
	keys := make([]string, 101)
	for i := range keys {
		keys[i] = "k"
	}
	_, err := h.BulkGet(context.Background(), &statev1.BulkGetRequest{
		Store:   "default",
		Keys:    keys,
		Context: makeTenantCtx("T"),
	})
	if got := status.Code(err); got != codes.ResourceExhausted {
		t.Fatalf("status: got %v want ResourceExhausted", got)
	}
	if adapterCalled {
		t.Error("adapter should not be reached when key count exceeds limit")
	}
}

// TestBulkGet_Accepts100Keys は境界値（ちょうど 100 キー）が通ることを確認する。
func TestBulkGet_Accepts100Keys(t *testing.T) {
	a := &fakeStateAdapter{
		bulkGetFn: func(_ context.Context, req dapr.StateBulkGetRequest) ([]dapr.StateBulkGetItem, error) {
			out := make([]dapr.StateBulkGetItem, 0, len(req.Keys))
			for _, k := range req.Keys {
				out = append(out, dapr.StateBulkGetItem{Key: k, NotFound: true})
			}
			return out, nil
		},
	}
	h := newHandler(a)
	keys := make([]string, 100)
	for i := range keys {
		keys[i] = "k"
	}
	_, err := h.BulkGet(context.Background(), &statev1.BulkGetRequest{
		Store:   "default",
		Keys:    keys,
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("100 keys should be accepted: %v", err)
	}
}

// TestTransact_RejectsOver10Ops は 11 operation 要求が ResourceExhausted で弾かれることを確認する。
func TestTransact_RejectsOver10Ops(t *testing.T) {
	adapterCalled := false
	a := &fakeStateAdapter{
		transactFn: func(_ context.Context, _ dapr.StateTransactRequest) error {
			adapterCalled = true
			return nil
		},
	}
	h := newHandler(a)
	ops := make([]*statev1.TransactOp, 11)
	for i := range ops {
		ops[i] = &statev1.TransactOp{Op: &statev1.TransactOp_Set{Set: &statev1.SetRequest{Key: "k"}}}
	}
	_, err := h.Transact(context.Background(), &statev1.TransactRequest{
		Store:      "default",
		Operations: ops,
		Context:    makeTenantCtx("T"),
	})
	if got := status.Code(err); got != codes.ResourceExhausted {
		t.Fatalf("status: got %v want ResourceExhausted", got)
	}
	if adapterCalled {
		t.Error("adapter should not be reached when ops count exceeds limit")
	}
}

// TestTransact_Accepts10Ops は境界値（ちょうど 10 ops）が通ることを確認する。
func TestTransact_Accepts10Ops(t *testing.T) {
	a := &fakeStateAdapter{}
	h := newHandler(a)
	ops := make([]*statev1.TransactOp, 10)
	for i := range ops {
		ops[i] = &statev1.TransactOp{Op: &statev1.TransactOp_Set{Set: &statev1.SetRequest{Key: "k"}}}
	}
	resp, err := h.Transact(context.Background(), &statev1.TransactRequest{
		Store:      "default",
		Operations: ops,
		Context:    makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("10 ops should be accepted: %v", err)
	}
	if !resp.GetCommitted() {
		t.Error("expected committed=true")
	}
}
