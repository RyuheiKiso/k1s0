// 本ファイルは in-memory backend での PubSub Publish→Subscribe round-trip テスト。
// dev / CI で外部 Dapr sidecar 無しに pubsub 経路が機能することを保証する。

package state

import (
	"context"
	"net"
	"testing"
	"time"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"
)

// startPubSubInMemoryBufconn は in-memory backend で PubSubService を bufconn 上に立てる。
func startPubSubInMemoryBufconn(t *testing.T) (pubsubv1.PubSubServiceClient, func()) {
	t.Helper()
	// in-memory backend は pubsubBus を含む Client を返す。
	client := dapr.NewClientWithInMemoryBackends()
	deps := NewDepsFromClient(client)
	deps.Idempotency = common.NewInMemoryIdempotencyCache(0)

	lis := bufconn.Listen(1024 * 1024)
	srv := grpc.NewServer()
	pubsubv1.RegisterPubSubServiceServer(srv, &pubsubHandler{deps: deps, idempotency: deps.Idempotency})

	go func() {
		_ = srv.Serve(lis)
	}()

	dialer := func(_ context.Context, _ string) (net.Conn, error) {
		return lis.Dial()
	}
	conn, err := grpc.NewClient(
		"passthrough://bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatalf("grpc.NewClient: %v", err)
	}
	cleanup := func() {
		_ = conn.Close()
		srv.Stop()
	}
	return pubsubv1.NewPubSubServiceClient(conn), cleanup
}

// in-memory pubsub bus 経由で Publish → Subscribe の round-trip が機能する。
func TestPubSubService_InMemory_RoundTrip(t *testing.T) {
	cli, cleanup := startPubSubInMemoryBufconn(t)
	defer cleanup()

	// Subscribe 開始（bus 上で channel が確保される）。
	subCtx, subCancel := context.WithCancel(context.Background())
	defer subCancel()
	stream, err := cli.Subscribe(subCtx, &pubsubv1.SubscribeRequest{
		Topic:         "orders",
		ConsumerGroup: "g1",
		Context:       makeTenantCtx("T-rt"),
	})
	if err != nil {
		t.Fatalf("Subscribe: %v", err)
	}

	// Subscribe が channel を確保するまで一瞬待つ（goroutine 起動レース回避）。
	time.Sleep(20 * time.Millisecond)

	// 別 goroutine で 3 件 Publish する。
	go func() {
		ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
		defer cancel()
		for i := 0; i < 3; i++ {
			_, err := cli.Publish(ctx, &pubsubv1.PublishRequest{
				Topic:       "orders",
				Data:        []byte{byte('A' + i)},
				ContentType: "application/octet-stream",
				Context:     makeTenantCtx("T-rt"),
			})
			if err != nil {
				t.Errorf("Publish: %v", err)
				return
			}
		}
	}()

	// 3 件受信できること。
	got := make([]string, 0, 3)
	for i := 0; i < 3; i++ {
		recvCtx, recvCancel := context.WithTimeout(context.Background(), 2*time.Second)
		defer recvCancel()
		// stream.Recv は ctx 不可だが、stream の context はサブスクライブ側で管理する。
		_ = recvCtx
		ev, err := stream.Recv()
		if err != nil {
			t.Fatalf("Recv (%d): %v", i, err)
		}
		got = append(got, string(ev.GetData()))
	}
	if got[0] != "A" || got[1] != "B" || got[2] != "C" {
		t.Errorf("payload order: %v want [A B C]", got)
	}
}

// 異テナントの Publish は別 channel 扱いで配信されない（NFR-E-AC-003: テナント越境防止）。
func TestPubSubService_InMemory_TenantIsolation(t *testing.T) {
	cli, cleanup := startPubSubInMemoryBufconn(t)
	defer cleanup()

	// T1 で Subscribe する。
	subCtx, subCancel := context.WithCancel(context.Background())
	defer subCancel()
	stream, err := cli.Subscribe(subCtx, &pubsubv1.SubscribeRequest{
		Topic:         "billing",
		ConsumerGroup: "g1",
		Context:       makeTenantCtx("T1"),
	})
	if err != nil {
		t.Fatalf("Subscribe: %v", err)
	}
	time.Sleep(20 * time.Millisecond)

	// 別テナント T2 で Publish → T1 の Subscribe には届かないはず。
	go func() {
		_, _ = cli.Publish(context.Background(), &pubsubv1.PublishRequest{
			Topic:       "billing",
			Data:        []byte("T2-only"),
			ContentType: "application/octet-stream",
			Context:     makeTenantCtx("T2"),
		})
	}()

	// 100ms 待っても受信が無いことを確認する。
	type recvResult struct {
		data []byte
		err  error
	}
	resultCh := make(chan recvResult, 1)
	go func() {
		ev, err := stream.Recv()
		if ev != nil {
			resultCh <- recvResult{data: ev.GetData(), err: err}
		} else {
			resultCh <- recvResult{err: err}
		}
	}()

	select {
	case r := <-resultCh:
		t.Errorf("expected no event for T1, got data=%q err=%v", string(r.data), r.err)
	case <-time.After(150 * time.Millisecond):
		// OK: T1 には届かない（テナント分離成功）。
	}
}
