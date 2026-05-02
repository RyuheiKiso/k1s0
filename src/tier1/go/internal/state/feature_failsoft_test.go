// 本ファイルは FR-T1-FEATURE-001 受け入れ基準
// 「評価失敗時はデフォルト値を返す（K1s0Error にしない、業務継続優先）」のテスト。

package state

import (
	"context"
	"errors"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	featurev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/feature/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// TestFeature_EvaluateBoolean_FailsSoftOnAdapterError は adapter 不能（plain error / Unavailable）
// で false + ERROR reason を返すことを確認する（FR-T1-FEATURE-001）。
func TestFeature_EvaluateBoolean_FailsSoftOnAdapterError(t *testing.T) {
	a := &fakeFeatureAdapter{
		boolFn: func(_ context.Context, _ dapr.FlagEvaluateRequest) (dapr.FlagBooleanResponse, error) {
			return dapr.FlagBooleanResponse{}, errors.New("flagd unreachable")
		},
	}
	h := &featureHandler{deps: Deps{FeatureAdapter: a}}
	resp, err := h.EvaluateBoolean(context.Background(), &featurev1.EvaluateRequest{
		FlagKey: "x",
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("expected fail-soft (no error), got: %v", err)
	}
	if resp.GetValue() != false {
		t.Errorf("expected default false, got %v", resp.GetValue())
	}
	if resp.GetMetadata().GetReason() != "ERROR" {
		t.Errorf("expected reason=ERROR, got %q", resp.GetMetadata().GetReason())
	}
}

// TestFeature_EvaluateString_FailsSoftOnUnavailable は Unavailable でも fail-soft することを確認する。
func TestFeature_EvaluateString_FailsSoftOnUnavailable(t *testing.T) {
	a := &fakeFeatureAdapter{
		stringFn: func(_ context.Context, _ dapr.FlagEvaluateRequest) (dapr.FlagStringResponse, error) {
			return dapr.FlagStringResponse{}, status.Error(codes.Unavailable, "flagd down")
		},
	}
	h := &featureHandler{deps: Deps{FeatureAdapter: a}}
	resp, err := h.EvaluateString(context.Background(), &featurev1.EvaluateRequest{
		FlagKey: "x",
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("expected fail-soft (no error), got: %v", err)
	}
	if resp.GetValue() != "" {
		t.Errorf("expected default empty string, got %q", resp.GetValue())
	}
	if resp.GetMetadata().GetReason() != "ERROR" {
		t.Errorf("expected reason=ERROR, got %q", resp.GetMetadata().GetReason())
	}
}

// TestFeature_EvaluateBoolean_PropagatesClientFault は呼出側起因のエラー
// （PermissionDenied / InvalidArgument / Unauthenticated）は fail-soft せず gRPC error として返ることを確認する。
func TestFeature_EvaluateBoolean_PropagatesClientFault(t *testing.T) {
	a := &fakeFeatureAdapter{
		boolFn: func(_ context.Context, _ dapr.FlagEvaluateRequest) (dapr.FlagBooleanResponse, error) {
			return dapr.FlagBooleanResponse{}, status.Error(codes.PermissionDenied, "denied")
		},
	}
	h := &featureHandler{deps: Deps{FeatureAdapter: a}}
	_, err := h.EvaluateBoolean(context.Background(), &featurev1.EvaluateRequest{
		FlagKey: "x",
		Context: makeTenantCtx("T"),
	})
	if got := status.Code(err); got != codes.PermissionDenied {
		t.Fatalf("expected PermissionDenied, got %v", got)
	}
}

// TestFeature_EvaluateNumber_FailsSoftOnAdapterError も同様に fail-soft する。
func TestFeature_EvaluateNumber_FailsSoftOnAdapterError(t *testing.T) {
	a := &fakeFeatureAdapter{
		numberFn: func(_ context.Context, _ dapr.FlagEvaluateRequest) (dapr.FlagNumberResponse, error) {
			return dapr.FlagNumberResponse{}, errors.New("backend dead")
		},
	}
	h := &featureHandler{deps: Deps{FeatureAdapter: a}}
	resp, err := h.EvaluateNumber(context.Background(), &featurev1.EvaluateRequest{
		FlagKey: "x",
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("expected fail-soft, got: %v", err)
	}
	if resp.GetValue() != 0 {
		t.Errorf("expected default 0, got %v", resp.GetValue())
	}
}

// TestFeature_EvaluateObject_FailsSoftOnAdapterError も同様に fail-soft する。
func TestFeature_EvaluateObject_FailsSoftOnAdapterError(t *testing.T) {
	a := &fakeFeatureAdapter{
		objectFn: func(_ context.Context, _ dapr.FlagEvaluateRequest) (dapr.FlagObjectResponse, error) {
			return dapr.FlagObjectResponse{}, errors.New("backend dead")
		},
	}
	h := &featureHandler{deps: Deps{FeatureAdapter: a}}
	resp, err := h.EvaluateObject(context.Background(), &featurev1.EvaluateRequest{
		FlagKey: "x",
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("expected fail-soft, got: %v", err)
	}
	if len(resp.GetValueJson()) != 0 {
		t.Errorf("expected default nil/empty json, got %q", resp.GetValueJson())
	}
}
