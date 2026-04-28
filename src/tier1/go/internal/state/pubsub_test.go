// 本ファイルは PubSubService handler の単体テスト + in-process gRPC 結線テスト。
//
// 試験戦略:
//   handler は dapr.PubSubAdapter に依存している。fake adapter で SDK / Kafka を切り
//   離し、handler の責務（proto ↔ adapter 変換、エラー翻訳）を検証する。
//   integration test では bufconn で実 gRPC を介し proto serialization 含めて round-trip する。

package state

import (
	"context"
	"errors"
	"net"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"
)

// fakePubSubAdapter は dapr.PubSubAdapter の最小 fake 実装。
type fakePubSubAdapter struct {
	publishFn   func(ctx context.Context, req dapr.PublishRequest) (dapr.PublishResponse, error)
	subscribeFn func(ctx context.Context, req dapr.SubscribeAdapterRequest) (dapr.PubSubSubscription, error)
}

func (f *fakePubSubAdapter) Publish(ctx context.Context, req dapr.PublishRequest) (dapr.PublishResponse, error) {
	return f.publishFn(ctx, req)
}
func (f *fakePubSubAdapter) Subscribe(ctx context.Context, req dapr.SubscribeAdapterRequest) (dapr.PubSubSubscription, error) {
	if f.subscribeFn == nil {
		return nil, dapr.ErrNotWired
	}
	return f.subscribeFn(ctx, req)
}

// fakeSubscription はチャネル経由でイベントを供給する subscription fake。
// テストごとに events に投入するか、Close() で終了させる。
type fakeSubscription struct {
	events chan *dapr.SubscribedEvent
	closed bool
	acked  []string
}

func (s *fakeSubscription) Receive(ctx context.Context) (*dapr.SubscribedEvent, error) {
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	case ev, ok := <-s.events:
		if !ok {
			return nil, errors.New("subscription closed")
		}
		return ev, nil
	}
}

func (s *fakeSubscription) Close() error {
	if !s.closed {
		s.closed = true
		close(s.events)
	}
	return nil
}

// newPubSubHandler は handler を fake adapter で構築する（state.go の Deps 流用）。
func newPubSubHandler(adapter dapr.PubSubAdapter) *pubsubHandler {
	return &pubsubHandler{deps: Deps{PubSubAdapter: adapter}}
}

// Publish の正常系: adapter に正しい引数を渡すことを検証する。
func TestPubSubHandler_Publish_OK(t *testing.T) {
	a := &fakePubSubAdapter{
		publishFn: func(_ context.Context, req dapr.PublishRequest) (dapr.PublishResponse, error) {
			if req.Topic != "k1s0.events.user-created" {
				t.Fatalf("topic mismatch: %s", req.Topic)
			}
			if string(req.Data) != `{"user_id":"42"}` {
				t.Fatalf("data mismatch: %s", req.Data)
			}
			if req.ContentType != "application/json" {
				t.Fatalf("content-type mismatch: %s", req.ContentType)
			}
			return dapr.PublishResponse{Offset: 0}, nil
		},
	}
	h := newPubSubHandler(a)
	resp, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{
		Topic:       "k1s0.events.user-created",
		Data:        []byte(`{"user_id":"42"}`),
		ContentType: "application/json",
	})
	if err != nil {
		t.Fatalf("Publish error: %v", err)
	}
	if resp.GetOffset() != 0 {
		t.Fatalf("offset should be 0 (Dapr SDK 非対応): got %d", resp.GetOffset())
	}
}

// Publish の nil 入力: InvalidArgument。
func TestPubSubHandler_Publish_NilRequest(t *testing.T) {
	h := newPubSubHandler(&fakePubSubAdapter{})
	_, err := h.Publish(context.Background(), nil)
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("status code: got %v want InvalidArgument", got)
	}
}

// adapter が ErrNotWired を返した場合 → Unimplemented。
func TestPubSubHandler_Publish_NotWired(t *testing.T) {
	a := &fakePubSubAdapter{
		publishFn: func(_ context.Context, _ dapr.PublishRequest) (dapr.PublishResponse, error) {
			return dapr.PublishResponse{}, dapr.ErrNotWired
		},
	}
	h := newPubSubHandler(a)
	_, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{Topic: "t", Data: []byte("d")})
	if got := status.Code(err); got != codes.Unimplemented {
		t.Fatalf("status code: got %v want Unimplemented", got)
	}
}

// adapter が一般エラーを返した場合 → Internal。
func TestPubSubHandler_Publish_AdapterError(t *testing.T) {
	a := &fakePubSubAdapter{
		publishFn: func(_ context.Context, _ dapr.PublishRequest) (dapr.PublishResponse, error) {
			return dapr.PublishResponse{}, errors.New("kafka unavailable")
		},
	}
	h := newPubSubHandler(a)
	_, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{Topic: "t", Data: []byte("d")})
	if got := status.Code(err); got != codes.Internal {
		t.Fatalf("status code: got %v want Internal", got)
	}
}

// BulkPublish: 複数 entry を順次発行する。
func TestPubSubHandler_BulkPublish_OK(t *testing.T) {
	count := 0
	a := &fakePubSubAdapter{
		publishFn: func(_ context.Context, req dapr.PublishRequest) (dapr.PublishResponse, error) {
			count++
			if req.Topic != "k1s0.events.audit" {
				t.Fatalf("topic mismatch on call %d: %s", count, req.Topic)
			}
			return dapr.PublishResponse{}, nil
		},
	}
	h := newPubSubHandler(a)
	_, err := h.BulkPublish(context.Background(), &pubsubv1.BulkPublishRequest{
		Topic: "k1s0.events.audit",
		Entries: []*pubsubv1.PublishRequest{
			{Data: []byte("a")},
			{Data: []byte("b")},
			{Data: []byte("c")},
		},
	})
	if err != nil {
		t.Fatalf("BulkPublish error: %v", err)
	}
	if count != 3 {
		t.Fatalf("Publish should be called 3 times, got %d", count)
	}
}

// Subscribe: in-process gRPC で 3 イベントが順次届くことを検証する。
func TestPubSubService_Subscribe_OverGRPC(t *testing.T) {
	lis := bufconn.Listen(bufSize)
	sub := &fakeSubscription{events: make(chan *dapr.SubscribedEvent, 3)}
	// 3 件投入してから close。
	go func() {
		ack1Called, ack2Called, ack3Called := false, false, false
		sub.events <- &dapr.SubscribedEvent{Topic: "t", Data: []byte("e1"), Ack: func() error { ack1Called = true; return nil }}
		sub.events <- &dapr.SubscribedEvent{Topic: "t", Data: []byte("e2"), Ack: func() error { ack2Called = true; return nil }}
		sub.events <- &dapr.SubscribedEvent{Topic: "t", Data: []byte("e3"), Ack: func() error { ack3Called = true; return nil }}
		// 念のため値を抑止する用途で参照（go vet 回避）。
		_ = ack1Called
		_ = ack2Called
		_ = ack3Called
		close(sub.events)
		sub.closed = true
	}()
	a := &fakePubSubAdapter{
		subscribeFn: func(_ context.Context, _ dapr.SubscribeAdapterRequest) (dapr.PubSubSubscription, error) {
			return sub, nil
		},
	}
	srv := grpc.NewServer()
	pubsubv1.RegisterPubSubServiceServer(srv, &pubsubHandler{deps: Deps{PubSubAdapter: a}})
	go func() { _ = srv.Serve(lis) }()
	defer srv.Stop()

	dialer := func(context.Context, string) (net.Conn, error) {
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
	defer conn.Close()
	client := pubsubv1.NewPubSubServiceClient(conn)
	stream, err := client.Subscribe(context.Background(), &pubsubv1.SubscribeRequest{
		Topic: "t", ConsumerGroup: "g",
	})
	if err != nil {
		t.Fatalf("Subscribe: %v", err)
	}
	collected := []string{}
	for i := 0; i < 3; i++ {
		ev, err := stream.Recv()
		if err != nil {
			t.Fatalf("Recv (%d): %v", i, err)
		}
		collected = append(collected, string(ev.GetData()))
	}
	if len(collected) != 3 || collected[0] != "e1" || collected[2] != "e3" {
		t.Fatalf("collected mismatch: %v", collected)
	}
}

// Subscribe: adapter が ErrNotWired を返した時 Unimplemented に翻訳される。
func TestPubSubService_Subscribe_NotWired(t *testing.T) {
	lis := bufconn.Listen(bufSize)
	a := &fakePubSubAdapter{} // subscribeFn nil → ErrNotWired
	srv := grpc.NewServer()
	pubsubv1.RegisterPubSubServiceServer(srv, &pubsubHandler{deps: Deps{PubSubAdapter: a}})
	go func() { _ = srv.Serve(lis) }()
	defer srv.Stop()

	dialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}
	conn, _ := grpc.NewClient(
		"passthrough://bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	defer conn.Close()
	client := pubsubv1.NewPubSubServiceClient(conn)
	stream, _ := client.Subscribe(context.Background(), &pubsubv1.SubscribeRequest{Topic: "t"})
	_, err := stream.Recv()
	if got := status.Code(err); got != codes.Unimplemented {
		t.Fatalf("status: got %v want Unimplemented", got)
	}
}

// in-process gRPC で Publish が proto レベルで往復することを検証する。
func TestPubSubService_Publish_OverGRPC(t *testing.T) {
	lis := bufconn.Listen(bufSize)
	captured := struct {
		topic string
		data  []byte
	}{}
	a := &fakePubSubAdapter{
		publishFn: func(_ context.Context, req dapr.PublishRequest) (dapr.PublishResponse, error) {
			captured.topic = req.Topic
			captured.data = req.Data
			return dapr.PublishResponse{Offset: 0}, nil
		},
	}
	deps := Deps{PubSubAdapter: a, StateAdapter: &fakeStateAdapter{}, BindingAdapter: nil, InvokeAdapter: nil, FeatureAdapter: nil}
	srv := grpc.NewServer()
	// PubSub だけ手動登録（State は Register hook が他 adapter も期待するため非利用）。
	pubsubv1.RegisterPubSubServiceServer(srv, &pubsubHandler{deps: deps})
	go func() { _ = srv.Serve(lis) }()
	defer srv.Stop()

	dialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}
	conn, err := grpc.NewClient(
		"passthrough://bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatalf("grpc.NewClient failed: %v", err)
	}
	defer conn.Close()

	client := pubsubv1.NewPubSubServiceClient(conn)
	resp, err := client.Publish(context.Background(), &pubsubv1.PublishRequest{
		Topic:       "k1s0.events.test",
		Data:        []byte("hello"),
		ContentType: "text/plain",
	})
	if err != nil {
		t.Fatalf("Publish over gRPC failed: %v", err)
	}
	if resp.GetOffset() != 0 {
		t.Fatalf("expected offset=0, got %d", resp.GetOffset())
	}
	if captured.topic != "k1s0.events.test" || string(captured.data) != "hello" {
		t.Fatalf("captured args mismatch: topic=%q data=%q", captured.topic, captured.data)
	}
}
