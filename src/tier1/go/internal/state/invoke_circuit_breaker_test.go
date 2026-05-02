// 本ファイルは Service Invoke handler の Circuit Breaker 連携テスト（FR-T1-INVOKE-004）。

package state

import (
	"context"
	"testing"
	"time"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	serviceinvokev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/serviceinvoke/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// TestInvoke_OpensCircuitBreakerAfterRepeatedFailures は連続失敗で CB が open になり、
// 後続呼出が即時 Unavailable で弾かれることを確認する。
func TestInvoke_OpensCircuitBreakerAfterRepeatedFailures(t *testing.T) {
	failingAdapter := &fakeInvokeAdapter{
		fn: func(_ context.Context, _ dapr.InvokeRequest) (dapr.InvokeResponse, error) {
			return dapr.InvokeResponse{}, status.Error(codes.Unavailable, "downstream is down")
		},
	}
	registry := common.NewCircuitBreakerRegistry(common.CBConfig{
		FailureThreshold:  3,
		HalfOpenAfter:     1 * time.Second,
		HalfOpenMaxProbes: 1,
	}, nil)
	h := &invokeHandler{deps: Deps{InvokeAdapter: failingAdapter, InvokeCircuitBreakers: registry}}

	req := &serviceinvokev1.InvokeRequest{
		AppId:   "billing-svc",
		Method:  "charge",
		Context: makeTenantCtx("T"),
	}
	// 最初の 3 回は handler が adapter を呼んで Unavailable を返す（CB が closed）。
	for i := 0; i < 3; i++ {
		_, err := h.Invoke(context.Background(), req)
		if status.Code(err) != codes.Unavailable {
			t.Fatalf("call %d: status = %v, want Unavailable", i+1, status.Code(err))
		}
	}
	// 4 回目は CB が open になっているため adapter は呼ばれず即 Unavailable。
	cb := registry.Get("billing-svc")
	if cb.State() != common.CBOpen {
		t.Fatalf("CB state = %v, want open after threshold", cb.State())
	}
	_, err := h.Invoke(context.Background(), req)
	if status.Code(err) != codes.Unavailable {
		t.Fatalf("4th call: status = %v, want Unavailable (CB open)", status.Code(err))
	}
}

// TestInvoke_RecordsSuccessOnHappyPath は成功呼出で CB が closed のまま維持されることを確認する。
func TestInvoke_RecordsSuccessOnHappyPath(t *testing.T) {
	a := &fakeInvokeAdapter{
		fn: func(_ context.Context, _ dapr.InvokeRequest) (dapr.InvokeResponse, error) {
			return dapr.InvokeResponse{Data: []byte("ok"), ContentType: "text/plain", Status: 200}, nil
		},
	}
	registry := common.NewCircuitBreakerRegistry(common.DefaultCBConfig(), nil)
	h := &invokeHandler{deps: Deps{InvokeAdapter: a, InvokeCircuitBreakers: registry}}
	for i := 0; i < 10; i++ {
		_, err := h.Invoke(context.Background(), &serviceinvokev1.InvokeRequest{
			AppId: "svc", Method: "m", Context: makeTenantCtx("T"),
		})
		if err != nil {
			t.Fatalf("call %d: unexpected error: %v", i+1, err)
		}
	}
	cb := registry.Get("svc")
	if cb.State() != common.CBClosed {
		t.Errorf("CB state = %v, want closed after only successes", cb.State())
	}
}

// TestInvoke_BusinessErrorDoesNotOpenCB は PermissionDenied 等の業務エラーが CB を開けないことを確認する。
// 業務ロジック誤りで CB を開けると呼出元側の修正中に全呼出が遮断される過剰反応になる。
func TestInvoke_BusinessErrorDoesNotOpenCB(t *testing.T) {
	a := &fakeInvokeAdapter{
		fn: func(_ context.Context, _ dapr.InvokeRequest) (dapr.InvokeResponse, error) {
			return dapr.InvokeResponse{}, status.Error(codes.PermissionDenied, "denied")
		},
	}
	registry := common.NewCircuitBreakerRegistry(common.CBConfig{
		FailureThreshold:  3,
		HalfOpenAfter:     1 * time.Second,
		HalfOpenMaxProbes: 1,
	}, nil)
	h := &invokeHandler{deps: Deps{InvokeAdapter: a, InvokeCircuitBreakers: registry}}
	for i := 0; i < 5; i++ {
		_, err := h.Invoke(context.Background(), &serviceinvokev1.InvokeRequest{
			AppId: "svc", Method: "m", Context: makeTenantCtx("T"),
		})
		if status.Code(err) != codes.PermissionDenied {
			t.Fatalf("call %d: status = %v, want PermissionDenied", i+1, status.Code(err))
		}
	}
	cb := registry.Get("svc")
	if cb.State() != common.CBClosed {
		t.Errorf("CB state = %v, want closed (PermissionDenied should not open CB)", cb.State())
	}
}

// TestInvoke_PerTargetCircuitBreaker は appId 単位で CB が独立していることを確認する。
func TestInvoke_PerTargetCircuitBreaker(t *testing.T) {
	a := &fakeInvokeAdapter{
		fn: func(_ context.Context, req dapr.InvokeRequest) (dapr.InvokeResponse, error) {
			if req.AppID == "broken" {
				return dapr.InvokeResponse{}, status.Error(codes.Unavailable, "down")
			}
			return dapr.InvokeResponse{Data: []byte("ok"), Status: 200}, nil
		},
	}
	registry := common.NewCircuitBreakerRegistry(common.CBConfig{
		FailureThreshold:  2,
		HalfOpenAfter:     1 * time.Second,
		HalfOpenMaxProbes: 1,
	}, nil)
	h := &invokeHandler{deps: Deps{InvokeAdapter: a, InvokeCircuitBreakers: registry}}
	// "broken" を 2 回失敗させて open に。
	for i := 0; i < 2; i++ {
		_, _ = h.Invoke(context.Background(), &serviceinvokev1.InvokeRequest{
			AppId: "broken", Method: "m", Context: makeTenantCtx("T"),
		})
	}
	if registry.Get("broken").State() != common.CBOpen {
		t.Fatal("broken CB should be open")
	}
	// "healthy" は問題なく成功し続ける（独立 CB）。
	for i := 0; i < 3; i++ {
		_, err := h.Invoke(context.Background(), &serviceinvokev1.InvokeRequest{
			AppId: "healthy", Method: "m", Context: makeTenantCtx("T"),
		})
		if err != nil {
			t.Fatalf("healthy call %d: unexpected error: %v", i+1, err)
		}
	}
	if registry.Get("healthy").State() != common.CBClosed {
		t.Errorf("healthy CB state = %v, want closed", registry.Get("healthy").State())
	}
}
