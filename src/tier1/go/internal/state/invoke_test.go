// 本ファイルは Service Invoke handler の FR-T1-INVOKE-003 / 005 単体テスト。
//
// 観点:
//   - FR-T1-INVOKE-003: TimeoutMs を context.WithTimeout で適用、過大値は弾く、
//     未指定時は 3 秒（共通規約）
//   - FR-T1-INVOKE-005: 呼出元 Authorization / W3C Trace Context を outgoing
//     metadata に転写する
//
// fake adapter で context を捕捉し、context の Deadline / outgoing metadata を観測する。

package state

import (
	"context"
	"testing"
	"time"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	serviceinvokev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/serviceinvoke/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// captureCtxAdapter は adapter 呼出時の context を保存して観測できる fake。
type captureCtxAdapter struct {
	captured context.Context
}

func (c *captureCtxAdapter) Invoke(ctx context.Context, req dapr.InvokeRequest) (dapr.InvokeResponse, error) {
	// adapter に渡された context をそのまま捕捉する。
	c.captured = ctx
	// 成功応答を返す。
	return dapr.InvokeResponse{Data: []byte("ok"), ContentType: "text/plain", Status: 200}, nil
}

// TestInvoke_AppliesTimeout_DefaultIs3Seconds は TimeoutMs=0 のときに既定 3 秒が
// 子 context に設定されることを確認する（共通規約 §「タイムアウトとデッドライン伝播」）。
func TestInvoke_AppliesTimeout_DefaultIs3Seconds(t *testing.T) {
	cap := &captureCtxAdapter{}
	h := &invokeHandler{deps: Deps{InvokeAdapter: cap}}
	// 親 context は deadline を持たない（既定値が効く確認）。
	parent := context.Background()
	_, err := h.Invoke(parent, &serviceinvokev1.InvokeRequest{
		AppId:   "t",
		Method:  "m",
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Invoke unexpected error: %v", err)
	}
	dl, ok := cap.captured.Deadline()
	if !ok {
		t.Fatal("captured context has no deadline; want default 3s applied")
	}
	// Deadline が「これから 3 秒以内」に設定されていることを確認する（jitter 許容）。
	d := time.Until(dl)
	if d <= 0 || d > invokeDefaultTimeout {
		t.Errorf("deadline %v not in (0, %v]", d, invokeDefaultTimeout)
	}
	if d < 2900*time.Millisecond {
		t.Errorf("deadline %v much shorter than 3s default; expected ~3s", d)
	}
}

// TestInvoke_AppliesTimeout_RespectsRequest は TimeoutMs 指定値が context.WithTimeout に
// 反映されることを確認する。
func TestInvoke_AppliesTimeout_RespectsRequest(t *testing.T) {
	cap := &captureCtxAdapter{}
	h := &invokeHandler{deps: Deps{InvokeAdapter: cap}}
	_, err := h.Invoke(context.Background(), &serviceinvokev1.InvokeRequest{
		AppId:     "t",
		Method:    "m",
		TimeoutMs: 500,
		Context:   makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Invoke unexpected error: %v", err)
	}
	dl, ok := cap.captured.Deadline()
	if !ok {
		t.Fatal("captured context has no deadline; want 500ms applied")
	}
	d := time.Until(dl)
	if d > 500*time.Millisecond {
		t.Errorf("deadline %v exceeds requested 500ms", d)
	}
	if d < 400*time.Millisecond {
		t.Errorf("deadline %v much shorter than 500ms; expected ~500ms", d)
	}
}

// TestInvoke_RejectsExcessiveTimeout は invokeMaxTimeout 超過の TimeoutMs を InvalidArgument で弾く。
func TestInvoke_RejectsExcessiveTimeout(t *testing.T) {
	h := &invokeHandler{deps: Deps{InvokeAdapter: &captureCtxAdapter{}}}
	_, err := h.Invoke(context.Background(), &serviceinvokev1.InvokeRequest{
		AppId:     "t",
		Method:    "m",
		TimeoutMs: int32((invokeMaxTimeout + time.Second) / time.Millisecond),
		Context:   makeTenantCtx("T"),
	})
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("status: got %v want InvalidArgument", got)
	}
}

// TestInvoke_RejectsNegativeTimeout は負の TimeoutMs を InvalidArgument で弾く。
func TestInvoke_RejectsNegativeTimeout(t *testing.T) {
	h := &invokeHandler{deps: Deps{InvokeAdapter: &captureCtxAdapter{}}}
	_, err := h.Invoke(context.Background(), &serviceinvokev1.InvokeRequest{
		AppId:     "t",
		Method:    "m",
		TimeoutMs: -1,
		Context:   makeTenantCtx("T"),
	})
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("status: got %v want InvalidArgument", got)
	}
}

// TestInvoke_ForwardsAuthorizationHeader は incoming context の Authorization が
// adapter に渡される outgoing metadata に転写されることを確認する（FR-T1-INVOKE-005）。
func TestInvoke_ForwardsAuthorizationHeader(t *testing.T) {
	cap := &captureCtxAdapter{}
	h := &invokeHandler{deps: Deps{InvokeAdapter: cap}}
	// incoming に Authorization をセットする。
	parent := metadata.NewIncomingContext(context.Background(), metadata.Pairs(
		"authorization", "Bearer test-jwt",
		"traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
	))
	_, err := h.Invoke(parent, &serviceinvokev1.InvokeRequest{
		AppId: "t", Method: "m",
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Invoke error: %v", err)
	}
	// adapter が受けた context の outgoing metadata に転写されているはず。
	out, ok := metadata.FromOutgoingContext(cap.captured)
	if !ok {
		t.Fatal("adapter context has no outgoing metadata; expected forwarded headers")
	}
	if got := out.Get("authorization"); len(got) == 0 || got[0] != "Bearer test-jwt" {
		t.Errorf("authorization not forwarded: got %v", got)
	}
	if got := out.Get("traceparent"); len(got) == 0 || got[0] != "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01" {
		t.Errorf("traceparent not forwarded: got %v", got)
	}
}

// TestInvoke_PreservesExistingOutgoingMetadata は既存 outgoing metadata を上書きしないことを確認する。
func TestInvoke_PreservesExistingOutgoingMetadata(t *testing.T) {
	cap := &captureCtxAdapter{}
	h := &invokeHandler{deps: Deps{InvokeAdapter: cap}}
	// incoming に Authorization をセットする。
	parent := metadata.NewIncomingContext(context.Background(), metadata.Pairs(
		"authorization", "Bearer incoming",
	))
	// 同時に outgoing にも別値を予めセットする（呼出側の意図を尊重）。
	parent = metadata.AppendToOutgoingContext(parent, "authorization", "Bearer existing")
	_, err := h.Invoke(parent, &serviceinvokev1.InvokeRequest{
		AppId: "t", Method: "m",
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Invoke error: %v", err)
	}
	out, _ := metadata.FromOutgoingContext(cap.captured)
	got := out.Get("authorization")
	if len(got) == 0 {
		t.Fatal("authorization missing")
	}
	// 既存値が保持されることを確認する（上書きされていない）。
	if got[0] != "Bearer existing" {
		t.Errorf("existing outgoing header was overwritten: got %v, want Bearer existing", got)
	}
}

// TestInvoke_RejectsEmptyAppIDOrMethod は app_id / method 必須を確認する。
func TestInvoke_RejectsEmptyAppIDOrMethod(t *testing.T) {
	h := &invokeHandler{deps: Deps{InvokeAdapter: &captureCtxAdapter{}}}
	tests := []struct {
		name string
		req  *serviceinvokev1.InvokeRequest
	}{
		{"empty app_id", &serviceinvokev1.InvokeRequest{AppId: "", Method: "m", Context: makeTenantCtx("T")}},
		{"empty method", &serviceinvokev1.InvokeRequest{AppId: "a", Method: "", Context: makeTenantCtx("T")}},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := h.Invoke(context.Background(), tt.req)
			if got := status.Code(err); got != codes.InvalidArgument {
				t.Fatalf("status: got %v want InvalidArgument", got)
			}
		})
	}
}
