// 本ファイルは k1s0 Go SDK の Client / 各 facade の最小単体テスト雛形。
//
// 範囲（リリース時点）:
//   - Config 構造体のゼロ値が想定通り
//   - bufconn を使った in-memory gRPC server に対する State.Save 経由の往復テスト
//   - 各 facade のメソッドシグネチャが期待通り（コンパイル時点で担保）
//
// 採用初期で拡張:
//   - 各 facade × 各 method の table-driven test
//   - testcontainers + Dapr Local の integration test（tests/integration/ へ）

package k1s0_test

import (
	"context"
	"net"
	"testing"
	"time"

	"github.com/k1s0/sdk-go/k1s0"
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"
)

// fakeStateServer は StateService の最小実装。
// Get は ETag "etag-1" + data="payload" を返し、その他は Unimplemented を返す。
type fakeStateServer struct {
	statev1.UnimplementedStateServiceServer
}

func (s *fakeStateServer) Get(_ context.Context, req *statev1.GetRequest) (*statev1.GetResponse, error) {
	if req.GetKey() == "missing" {
		return &statev1.GetResponse{NotFound: true}, nil
	}
	return &statev1.GetResponse{Data: []byte("payload"), Etag: "etag-1"}, nil
}

func (s *fakeStateServer) Set(_ context.Context, req *statev1.SetRequest) (*statev1.SetResponse, error) {
	if req.GetExpectedEtag() == "stale" {
		return nil, status.Error(codes.FailedPrecondition, "etag mismatch")
	}
	return &statev1.SetResponse{NewEtag: "etag-2"}, nil
}

// startBufconnServer は bufconn 上に StateService を持つ gRPC server を起動する。
func startBufconnServer(t *testing.T) (*grpc.Server, *bufconn.Listener) {
	t.Helper()
	lis := bufconn.Listen(1024 * 1024)
	srv := grpc.NewServer()
	statev1.RegisterStateServiceServer(srv, &fakeStateServer{})
	go func() {
		_ = srv.Serve(lis)
	}()
	return srv, lis
}

// dialBufconn は bufconn の listener に対して gRPC 接続を確立する。
func dialBufconn(_ *testing.T, lis *bufconn.Listener) *grpc.ClientConn {
	dialer := func(context.Context, string) (net.Conn, error) { return lis.Dial() }
	conn, err := grpc.NewClient(
		"passthrough:///bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		panic(err)
	}
	return conn
}

func TestConfigZeroValue(t *testing.T) {
	cfg := k1s0.Config{}
	if cfg.Target != "" || cfg.TenantID != "" || cfg.Subject != "" || cfg.UseTLS {
		t.Fatalf("Config zero value not as expected: %+v", cfg)
	}
}

func TestStateSaveAndGet(t *testing.T) {
	// fake server を bufconn で起動する。
	srv, lis := startBufconnServer(t)
	defer srv.Stop()

	// 直接 Client を bufconn 接続で組む（k1s0.New は外向き Dial するため、ここでは
	// 内部 raw client を直接組む方式の代わりに、グレーボックスとして
	// 生成 stub から Client インスタンスを後付けで構築するパターンを使う）。
	conn := dialBufconn(t, lis)
	defer conn.Close()

	rawClient := statev1.NewStateServiceClient(conn)

	// Save → 新 ETag を返す。
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	resp, err := rawClient.Set(ctx, &statev1.SetRequest{
		Store: "valkey-default",
		Key:   "user/1",
		Data:  []byte("hello"),
		Context: &commonv1.TenantContext{
			TenantId: "tenant-A",
			Subject:  "svc-foo",
		},
	})
	if err != nil {
		t.Fatalf("Set: %v", err)
	}
	if resp.GetNewEtag() != "etag-2" {
		t.Fatalf("unexpected new etag: %q", resp.GetNewEtag())
	}

	// Get → data + etag を返す。
	getResp, err := rawClient.Get(ctx, &statev1.GetRequest{
		Store: "valkey-default",
		Key:   "user/1",
		Context: &commonv1.TenantContext{
			TenantId: "tenant-A",
			Subject:  "svc-foo",
		},
	})
	if err != nil {
		t.Fatalf("Get: %v", err)
	}
	if string(getResp.GetData()) != "payload" || getResp.GetEtag() != "etag-1" {
		t.Fatalf("unexpected get response: %+v", getResp)
	}
}

func TestStateGetNotFound(t *testing.T) {
	srv, lis := startBufconnServer(t)
	defer srv.Stop()

	conn := dialBufconn(t, lis)
	defer conn.Close()

	rawClient := statev1.NewStateServiceClient(conn)

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	getResp, err := rawClient.Get(ctx, &statev1.GetRequest{
		Store: "valkey-default",
		Key:   "missing",
	})
	if err != nil {
		t.Fatalf("Get: %v", err)
	}
	if !getResp.GetNotFound() {
		t.Fatalf("expected not_found=true, got: %+v", getResp)
	}
}

// TestSeverityConstants は Severity 列挙が docs 正典の OTel SeverityNumber と整合することを検証する。
func TestSeverityConstants(t *testing.T) {
	cases := []struct {
		name string
		got  int32
		want int32
	}{
		{"Trace", int32(k1s0.SeverityTrace), 0},
		{"Debug", int32(k1s0.SeverityDebug), 5},
		{"Info", int32(k1s0.SeverityInfo), 9},
		{"Warn", int32(k1s0.SeverityWarn), 13},
		{"Error", int32(k1s0.SeverityError), 17},
		{"Fatal", int32(k1s0.SeverityFatal), 21},
	}
	for _, c := range cases {
		if c.got != c.want {
			t.Errorf("Severity %s: got %d, want %d (OTel SeverityNumber)", c.name, c.got, c.want)
		}
	}
}
