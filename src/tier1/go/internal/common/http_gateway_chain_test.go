// 本ファイルは HTTPGateway.WithInterceptors の挙動検証。
//
// 検証観点:
//   - WithInterceptors なしは従来通り素通り（既存テスト互換）
//   - interceptor が attach されると HTTP path でも chain が走る
//   - AuthInterceptor (hmac mode) を chain に入れると JWT 不在で 401 が返る
//   - chain 順序が grpc.ChainUnaryInterceptor と一致（先頭が最外層）
//   - httpPathToGRPCMethod が docs §「HTTP/JSON 互換」path 規約から正しく FullMethod を再構築する

package common

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"google.golang.org/grpc"
	grpccodes "google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"

	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
)

// AuthInterceptor を HTTP gateway に attach すると JWT 不在で 401 を返す。
// これにより HTTP path が gRPC path と同じ認証層を共有することを保証する。
func TestHTTPGateway_WithAuthInterceptor_RejectsMissingJWT(t *testing.T) {
	authCfg := AuthInterceptorConfig{
		Mode:       AuthModeHMAC,
		HMACSecret: []byte("test-hmac-secret-32-bytes--------"),
	}
	g := NewHTTPGateway().WithInterceptors(AuthInterceptor(authCfg))
	g.register("/k1s0/state/get", func(ctx context.Context, body []byte) (proto.Message, error) {
		t.Fatal("handler should not be reached when JWT is missing")
		return nil, nil
	})
	srv := httptest.NewServer(g.Handler())
	defer srv.Close()
	resp, err := http.Post(srv.URL+"/k1s0/state/get", "application/json", strings.NewReader("{}"))
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	// docs §「HTTP Status ↔ K1s0Error」: Unauthenticated → 401。
	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("status = %d want 401 (interceptor must run on HTTP path)", resp.StatusCode)
	}
}

// chain が一切無いと従来通り handler が呼ばれる（既存挙動互換）。
func TestHTTPGateway_NoInterceptors_HandlerCalled(t *testing.T) {
	g := NewHTTPGateway()
	called := false
	g.register("/k1s0/state/get", func(ctx context.Context, body []byte) (proto.Message, error) {
		called = true
		return &commonv1.TenantContext{}, nil
	})
	srv := httptest.NewServer(g.Handler())
	defer srv.Close()
	resp, _ := http.Post(srv.URL+"/k1s0/state/get", "application/json", strings.NewReader("{}"))
	defer func() { _ = resp.Body.Close() }()
	if !called {
		t.Fatalf("handler should be called when no interceptors attached")
	}
}

// 複数 interceptor の chain 順序: 先頭が最外層、末尾が最内層（grpc.ChainUnaryInterceptor 互換）。
func TestHTTPGateway_InterceptorOrder(t *testing.T) {
	var order []string
	mark := func(name string) grpc.UnaryServerInterceptor {
		return func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
			order = append(order, "before:"+name)
			resp, err := handler(ctx, req)
			order = append(order, "after:"+name)
			return resp, err
		}
	}
	g := NewHTTPGateway().WithInterceptors(mark("outer"), mark("inner"))
	g.register("/k1s0/state/get", func(ctx context.Context, body []byte) (proto.Message, error) {
		order = append(order, "handler")
		return &commonv1.TenantContext{}, nil
	})
	srv := httptest.NewServer(g.Handler())
	defer srv.Close()
	resp, _ := http.Post(srv.URL+"/k1s0/state/get", "application/json", strings.NewReader("{}"))
	defer func() { _ = resp.Body.Close() }()
	want := []string{"before:outer", "before:inner", "handler", "after:inner", "after:outer"}
	if len(order) != len(want) {
		t.Fatalf("order length = %d want %d: %v", len(order), len(want), order)
	}
	for i := range want {
		if order[i] != want[i] {
			t.Errorf("order[%d] = %q want %q", i, order[i], want[i])
		}
	}
}

// interceptor が error を返すと handler が呼ばれず、HTTP status にマップされる。
func TestHTTPGateway_InterceptorError_PropagatedToHTTPStatus(t *testing.T) {
	deny := func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
		return nil, status.Error(grpccodes.PermissionDenied, "tenant_id mismatch")
	}
	g := NewHTTPGateway().WithInterceptors(deny)
	g.register("/k1s0/state/get", func(ctx context.Context, body []byte) (proto.Message, error) {
		t.Fatal("handler should not be reached")
		return nil, nil
	})
	srv := httptest.NewServer(g.Handler())
	defer srv.Close()
	resp, _ := http.Post(srv.URL+"/k1s0/state/get", "application/json", strings.NewReader("{}"))
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusForbidden {
		t.Fatalf("status = %d want 403", resp.StatusCode)
	}
}

// httpPathToGRPCMethod が docs §「HTTP/JSON 互換」path 規約から FullMethod を正しく再構築する。
// ObservabilityInterceptor / AuditInterceptor が info.FullMethod から API 名を抽出するため重要。
func TestHttpPathToGRPCMethod_DocsRegression(t *testing.T) {
	cases := []struct {
		path string
		want string
	}{
		{"/k1s0/state/get", "/k1s0.tier1.state.v1.StateService/Get"},
		{"/k1s0/state/set", "/k1s0.tier1.state.v1.StateService/Set"},
		{"/k1s0/pubsub/publish", "/k1s0.tier1.pubsub.v1.PubSubService/Publish"},
		{"/k1s0/secrets/get", "/k1s0.tier1.secrets.v1.SecretsService/Get"},
		{"/k1s0/serviceinvoke/invoke", "/k1s0.tier1.serviceinvoke.v1.InvokeService/Invoke"},
		{"/k1s0/workflow/start", "/k1s0.tier1.workflow.v1.WorkflowService/Start"},
		{"/unknown/path", "/unknown/path"},
		{"", ""},
	}
	for _, c := range cases {
		if got := httpPathToGRPCMethod(c.path); got != c.want {
			t.Errorf("httpPathToGRPCMethod(%q) = %q; want %q", c.path, got, c.want)
		}
	}
}
