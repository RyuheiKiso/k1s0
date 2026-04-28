// 本ファイルは ObservabilityInterceptor の単体テスト。
//
// 検証観点:
//   1. apiNameFromMethod / methodNameFromFullMethod の境界条件
//   2. extractTenantID が proto-like な GetContext / GetTenantId を辿って tenant_id を返すこと
//   3. interceptor が handler の戻り値とエラーを透過すること
//   4. TracerProvider / MeterProvider が未設定でも panic しないこと（fail-soft）

package common

import (
	"context"
	"errors"
	"testing"

	"google.golang.org/grpc"
	grpccodes "google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// API 名抽出の境界条件。
func TestApiNameFromMethod_Cases(t *testing.T) {
	cases := []struct {
		in   string
		want string
	}{
		{"/k1s0.tier1.state.v1.StateService/Get", "state"},
		{"/k1s0.tier1.pubsub.v1.PubSubService/Publish", "pubsub"},
		{"/k1s0.tier1.decision.v1.DecisionService/Evaluate", "decision"},
		{"/k1s0.tier1.audit.v1.AuditService/Record", "audit"},
		{"/grpc.health.v1.Health/Check", "unknown"},
		{"", "unknown"},
		{"no-slash", "unknown"},
	}
	for _, c := range cases {
		if got := apiNameFromMethod(c.in); got != c.want {
			t.Errorf("apiNameFromMethod(%q) = %q; want %q", c.in, got, c.want)
		}
	}
}

// メソッド名抽出。
func TestMethodNameFromFullMethod_Cases(t *testing.T) {
	cases := []struct {
		in, want string
	}{
		{"/k1s0.tier1.state.v1.StateService/Get", "Get"},
		{"/foo/bar/Baz", "Baz"},
		{"NoSlash", "NoSlash"},
	}
	for _, c := range cases {
		if got := methodNameFromFullMethod(c.in); got != c.want {
			t.Errorf("methodNameFromFullMethod(%q) = %q; want %q", c.in, got, c.want)
		}
	}
}

// fakeTenantContext は TenantContext-like な GetTenantId() のみ持つ最小実装。
type fakeTenantContext struct{ tenantID string }

func (f *fakeTenantContext) GetTenantId() string { return f.tenantID }

// fakeRequest は GetContext() で fakeTenantContext を返す proto-like 型。
type fakeRequest struct{ ctx *fakeTenantContext }

func (f *fakeRequest) GetContext() interface {
	GetTenantId() string
} {
	if f.ctx == nil {
		return nil
	}
	return f.ctx
}

// extractTenantID は GetContext() を辿って tenant_id を返す。
func TestExtractTenantID_FromProtoLike(t *testing.T) {
	cases := []struct {
		name string
		req  interface{}
		want string
	}{
		{name: "with tenant", req: &fakeRequest{ctx: &fakeTenantContext{tenantID: "T1"}}, want: "T1"},
		{name: "nil context", req: &fakeRequest{ctx: nil}, want: ""},
		{name: "non-proto", req: struct{}{}, want: ""},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			if got := extractTenantID(c.req); got != c.want {
				t.Errorf("extractTenantID = %q; want %q", got, c.want)
			}
		})
	}
}

// interceptor は handler の戻り値とエラーを透過する。
// global TracerProvider / MeterProvider が no-op 状態でも panic しない（fail-soft）。
func TestObservabilityInterceptor_PassthroughAndFailSoft(t *testing.T) {
	icpt := ObservabilityInterceptor()
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	want := "ok-resp"
	resp, err := icpt(context.Background(), &fakeRequest{ctx: &fakeTenantContext{tenantID: "T"}}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return want, nil
	})
	if err != nil {
		t.Fatalf("err = %v", err)
	}
	if resp != want {
		t.Fatalf("resp = %v; want %v", resp, want)
	}
	// エラー透過（gRPC status）。
	wantErr := status.Error(grpccodes.NotFound, "missing")
	_, err = icpt(context.Background(), &fakeRequest{}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return nil, wantErr
	})
	if !errors.Is(err, wantErr) && status.Code(err) != grpccodes.NotFound {
		t.Fatalf("expected NotFound, got %v", err)
	}
	// 未対応 method（API 抽出 unknown）でも panic しない。
	_, err = icpt(context.Background(), nil, &grpc.UnaryServerInfo{FullMethod: "/unknown/Foo"}, func(ctx context.Context, req interface{}) (interface{}, error) {
		return "u", nil
	})
	if err != nil {
		t.Fatalf("unknown api err = %v", err)
	}
}
