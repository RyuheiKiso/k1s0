// 本ファイルは t1-state Pod の StateService handler の単体テスト。
//
// 試験戦略:
//   handler は StateAdapter interface に依存している。本テストでは fake StateAdapter
//   を注入し、handler の責務（proto ↔ adapter 変換、テナント抽出、エラー翻訳）を
//   adapter / Dapr SDK の都合と切り離して検証する。

package state

import (
	"context"
	"errors"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// fakeStateAdapter は dapr.StateAdapter の最小 fake 実装。
// 各メソッドの fn を試験ごとに差し替える。
type fakeStateAdapter struct {
	getFn    func(ctx context.Context, req dapr.StateGetRequest) (dapr.StateGetResponse, error)
	setFn    func(ctx context.Context, req dapr.StateSetRequest) (dapr.StateSetResponse, error)
	deleteFn func(ctx context.Context, req dapr.StateSetRequest) error
}

func (f *fakeStateAdapter) Get(ctx context.Context, req dapr.StateGetRequest) (dapr.StateGetResponse, error) {
	return f.getFn(ctx, req)
}
func (f *fakeStateAdapter) Set(ctx context.Context, req dapr.StateSetRequest) (dapr.StateSetResponse, error) {
	return f.setFn(ctx, req)
}
func (f *fakeStateAdapter) Delete(ctx context.Context, req dapr.StateSetRequest) error {
	return f.deleteFn(ctx, req)
}

// newHandler は handler を fake adapter で構築する。
func newHandler(adapter dapr.StateAdapter) *stateHandler {
	return &stateHandler{deps: Deps{StateAdapter: adapter}}
}

// makeTenantCtx は TenantContext を含む proto request の context フィールド用 helper。
func makeTenantCtx(tenantID string) *commonv1.TenantContext {
	return &commonv1.TenantContext{TenantId: tenantID}
}

// Get の正常系: adapter 経由で値・etag が返ることを検証する。
func TestStateHandler_Get_Found(t *testing.T) {
	want := []byte("payload")
	a := &fakeStateAdapter{
		getFn: func(_ context.Context, req dapr.StateGetRequest) (dapr.StateGetResponse, error) {
			if req.Store != "valkey-default" || req.Key != "user:42" || req.TenantID != "tenant-A" {
				t.Fatalf("adapter args mismatch: %+v", req)
			}
			return dapr.StateGetResponse{Data: want, Etag: "v3", NotFound: false}, nil
		},
	}
	h := newHandler(a)
	resp, err := h.Get(context.Background(), &statev1.GetRequest{
		Store:   "valkey-default",
		Key:     "user:42",
		Context: makeTenantCtx("tenant-A"),
	})
	if err != nil {
		t.Fatalf("Get returned error: %v", err)
	}
	if string(resp.GetData()) != string(want) {
		t.Fatalf("data mismatch: got %q want %q", resp.GetData(), want)
	}
	if resp.GetEtag() != "v3" {
		t.Fatalf("etag mismatch: got %q want v3", resp.GetEtag())
	}
	if resp.GetNotFound() {
		t.Fatalf("expected NotFound=false")
	}
}

// Get の未存在: adapter が NotFound=true を返した時に proto 側で透過することを検証する。
func TestStateHandler_Get_NotFound(t *testing.T) {
	a := &fakeStateAdapter{
		getFn: func(_ context.Context, _ dapr.StateGetRequest) (dapr.StateGetResponse, error) {
			return dapr.StateGetResponse{NotFound: true}, nil
		},
	}
	h := newHandler(a)
	resp, err := h.Get(context.Background(), &statev1.GetRequest{Store: "s", Key: "x"})
	if err != nil {
		t.Fatalf("Get error: %v", err)
	}
	if !resp.GetNotFound() {
		t.Fatalf("expected NotFound=true")
	}
}

// Get の nil 入力: codes.InvalidArgument を返すことを検証する（defensive）。
func TestStateHandler_Get_NilRequest(t *testing.T) {
	h := newHandler(&fakeStateAdapter{})
	_, err := h.Get(context.Background(), nil)
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("status code: got %v want InvalidArgument", got)
	}
}

// Get 時に adapter が ErrNotWired を返した場合、Unimplemented に翻訳されることを検証する。
func TestStateHandler_Get_NotWired(t *testing.T) {
	a := &fakeStateAdapter{
		getFn: func(_ context.Context, _ dapr.StateGetRequest) (dapr.StateGetResponse, error) {
			return dapr.StateGetResponse{}, dapr.ErrNotWired
		},
	}
	h := newHandler(a)
	_, err := h.Get(context.Background(), &statev1.GetRequest{Store: "s", Key: "k"})
	if got := status.Code(err); got != codes.Unimplemented {
		t.Fatalf("status code: got %v want Unimplemented", got)
	}
}

// Get 時の adapter エラーが Internal に翻訳されることを検証する。
func TestStateHandler_Get_AdapterError(t *testing.T) {
	a := &fakeStateAdapter{
		getFn: func(_ context.Context, _ dapr.StateGetRequest) (dapr.StateGetResponse, error) {
			return dapr.StateGetResponse{}, errors.New("connect: refused")
		},
	}
	h := newHandler(a)
	_, err := h.Get(context.Background(), &statev1.GetRequest{Store: "s", Key: "k"})
	if got := status.Code(err); got != codes.Internal {
		t.Fatalf("status code: got %v want Internal", got)
	}
}

// Set の正常系: TTL / ExpectedEtag を adapter に渡し、NewEtag が応答に乗ることを検証する。
func TestStateHandler_Set_OK(t *testing.T) {
	a := &fakeStateAdapter{
		setFn: func(_ context.Context, req dapr.StateSetRequest) (dapr.StateSetResponse, error) {
			if req.Store != "s" || req.Key != "k" || string(req.Data) != "v" {
				t.Fatalf("set args mismatch: %+v", req)
			}
			if req.TTLSeconds != 60 {
				t.Fatalf("ttl mismatch: %d", req.TTLSeconds)
			}
			if req.ExpectedEtag != "v3" {
				t.Fatalf("etag mismatch: %s", req.ExpectedEtag)
			}
			return dapr.StateSetResponse{NewEtag: "v4"}, nil
		},
	}
	h := newHandler(a)
	resp, err := h.Set(context.Background(), &statev1.SetRequest{
		Store:        "s",
		Key:          "k",
		Data:         []byte("v"),
		ExpectedEtag: "v3",
		TtlSec:       60,
	})
	if err != nil {
		t.Fatalf("Set error: %v", err)
	}
	if resp.GetNewEtag() != "v4" {
		t.Fatalf("new_etag mismatch: got %q want v4", resp.GetNewEtag())
	}
}

// Delete の正常系: deleted=true が応答に乗ることを検証する。
func TestStateHandler_Delete_OK(t *testing.T) {
	called := 0
	a := &fakeStateAdapter{
		deleteFn: func(_ context.Context, req dapr.StateSetRequest) error {
			called++
			if req.Store != "s" || req.Key != "k" {
				t.Fatalf("del args mismatch: %+v", req)
			}
			return nil
		},
	}
	h := newHandler(a)
	resp, err := h.Delete(context.Background(), &statev1.DeleteRequest{Store: "s", Key: "k"})
	if err != nil {
		t.Fatalf("Delete error: %v", err)
	}
	if !resp.GetDeleted() {
		t.Fatalf("expected Deleted=true")
	}
	if called != 1 {
		t.Fatalf("adapter Delete not called")
	}
}

// BulkGet / Transact は plan 04-04 範囲外で Unimplemented のまま。
func TestStateHandler_BulkGet_StillUnimplemented(t *testing.T) {
	h := newHandler(&fakeStateAdapter{})
	_, err := h.BulkGet(context.Background(), &statev1.BulkGetRequest{})
	if got := status.Code(err); got != codes.Unimplemented {
		t.Fatalf("BulkGet: got %v want Unimplemented", got)
	}
}

func TestStateHandler_Transact_StillUnimplemented(t *testing.T) {
	h := newHandler(&fakeStateAdapter{})
	_, err := h.Transact(context.Background(), &statev1.TransactRequest{})
	if got := status.Code(err); got != codes.Unimplemented {
		t.Fatalf("Transact: got %v want Unimplemented", got)
	}
}
