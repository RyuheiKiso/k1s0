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
		Context:     makeTenantCtx("T"),
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

// adapter が一般エラーを返した場合 → Internal。
func TestPubSubHandler_Publish_AdapterError(t *testing.T) {
	a := &fakePubSubAdapter{
		publishFn: func(_ context.Context, _ dapr.PublishRequest) (dapr.PublishResponse, error) {
			return dapr.PublishResponse{}, errors.New("kafka unavailable")
		},
	}
	h := newPubSubHandler(a)
	_, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{Topic: "t", Data: []byte("d"), Context: makeTenantCtx("T")})
	if got := status.Code(err); got != codes.Internal {
		t.Fatalf("status code: got %v want Internal", got)
	}
}

// NFR-E-AC-003: tenant_id 未設定時に InvalidArgument を返すことを検証する。
func TestPubSubHandler_Publish_RequiresTenant(t *testing.T) {
	h := newPubSubHandler(&fakePubSubAdapter{})
	_, err := h.Publish(context.Background(), &pubsubv1.PublishRequest{Topic: "t", Data: []byte("d")})
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("expected InvalidArgument for missing tenant, got %v", got)
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
			{Data: []byte("a"), Context: makeTenantCtx("T")},
			{Data: []byte("b"), Context: makeTenantCtx("T")},
			{Data: []byte("c"), Context: makeTenantCtx("T")},
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
		Topic: "t", ConsumerGroup: "g", Context: makeTenantCtx("T"),
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

// Subscribe: adapter が一般エラー（subscribeFn nil 経由）を返した時 Internal に翻訳される。
func TestPubSubService_Subscribe_AdapterError(t *testing.T) {
	lis := bufconn.Listen(bufSize)
	// fakePubSubAdapter.subscribeFn が nil の時、fake は便宜上 dapr.ErrNotWired を返すが、
	// production / dev では adapter は必ず注入されるため handler は ErrNotWired を特別扱いせず
	// generic な Internal にフォールバックする。
	a := &fakePubSubAdapter{}
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
	// FR-T1-PUBSUB-003: 空 ConsumerGroup は新挙動で InvalidArgument に倒れるため、
	// adapter 一般エラー → Internal の翻訳経路を試したい本テストでは ConsumerGroup を
	// 明示指定する。"g" は normalize で "k1s0.T.g" に変換されて adapter に渡る。
	stream, _ := client.Subscribe(context.Background(), &pubsubv1.SubscribeRequest{Topic: "t", ConsumerGroup: "g", Context: makeTenantCtx("T")})
	_, err := stream.Recv()
	if got := status.Code(err); got != codes.Internal {
		t.Fatalf("status: got %v want Internal", got)
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
		Context:     makeTenantCtx("T"),
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

// FR-T1-PUBSUB-003: 純関数 normalizeConsumerGroup の 4 ケース + handler 結線 2 ケース
// = 計 6 件の regression test。docs 受け入れ基準
// （k1s0.<tenant>.<service_name> 自動付与 / 重複付与なし / 空文字拒否 / ドット拒否）を
// 機械的に検証し、coverage.sh の grep ヒット (impl_refs) 1 → 6+ に底上げする。

// TestNormalizeConsumerGroup_PrefixesServiceName は service_name のみ受けて
// k1s0.<tenant>.<svc> を生成することを確認する。
func TestNormalizeConsumerGroup_PrefixesServiceName(t *testing.T) {
	got, err := normalizeConsumerGroup("acme", "billing")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if got != "k1s0.acme.billing" {
		t.Fatalf("got %q want k1s0.acme.billing", got)
	}
}

// TestNormalizeConsumerGroup_AlreadyPrefixedKept は前方互換のために
// 既に k1s0.<tenant>. で始まっている値を二重 prefix しないことを確認する。
func TestNormalizeConsumerGroup_AlreadyPrefixedKept(t *testing.T) {
	got, err := normalizeConsumerGroup("acme", "k1s0.acme.legacy")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if got != "k1s0.acme.legacy" {
		t.Fatalf("got %q want k1s0.acme.legacy (prefix duplication must be avoided)", got)
	}
}

// TestNormalizeConsumerGroup_EmptyRejected は空文字を InvalidArgument で弾くことを確認する。
// service_name 不在では「k1s0.<tenant>.<service_name>」を生成不能のため。
func TestNormalizeConsumerGroup_EmptyRejected(t *testing.T) {
	_, err := normalizeConsumerGroup("acme", "")
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("status: got %v want InvalidArgument", got)
	}
}

// TestNormalizeConsumerGroup_DotInServiceNameRejected は <service_name> 部の
// ドットを禁止することを確認する（階層境界 collision を構造的に防ぐ）。
func TestNormalizeConsumerGroup_DotInServiceNameRejected(t *testing.T) {
	_, err := normalizeConsumerGroup("acme", "billing.invoice")
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("status: got %v want InvalidArgument for %q", got, "billing.invoice")
	}
}

// TestPubSubService_Subscribe_AutoPrefixesConsumerGroup は Subscribe handler が
// service_name を受け取って adapter には k1s0.<tenant>.<svc> を渡すことを検証する。
func TestPubSubService_Subscribe_AutoPrefixesConsumerGroup(t *testing.T) {
	lis := bufconn.Listen(bufSize)
	sub := &fakeSubscription{events: make(chan *dapr.SubscribedEvent, 1)}
	// 1 件投入してから close して関数が戻るようにする。
	go func() {
		sub.events <- &dapr.SubscribedEvent{Topic: "t", Data: []byte("e1"), Ack: func() error { return nil }}
		close(sub.events)
		sub.closed = true
	}()
	// adapter に渡る ConsumerGroup を capture する。
	var capturedCG string
	a := &fakePubSubAdapter{
		subscribeFn: func(_ context.Context, req dapr.SubscribeAdapterRequest) (dapr.PubSubSubscription, error) {
			capturedCG = req.ConsumerGroup
			return sub, nil
		},
	}
	srv := grpc.NewServer()
	pubsubv1.RegisterPubSubServiceServer(srv, &pubsubHandler{deps: Deps{PubSubAdapter: a}})
	go func() { _ = srv.Serve(lis) }()
	defer srv.Stop()

	dialer := func(context.Context, string) (net.Conn, error) { return lis.Dial() }
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
		Topic: "t", ConsumerGroup: "billing", Context: makeTenantCtx("acme"),
	})
	if err != nil {
		t.Fatalf("Subscribe: %v", err)
	}
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("Recv: %v", err)
	}
	if capturedCG != "k1s0.acme.billing" {
		t.Fatalf("adapter ConsumerGroup: got %q want k1s0.acme.billing", capturedCG)
	}
}

// TestPubSubService_Subscribe_RejectsEmptyConsumerGroup は handler 段で
// 空 ConsumerGroup を InvalidArgument で弾き、adapter を呼ばないことを確認する。
func TestPubSubService_Subscribe_RejectsEmptyConsumerGroup(t *testing.T) {
	lis := bufconn.Listen(bufSize)
	// adapter は呼ばれてはならない（handler が事前に弾く契約）。
	a := &fakePubSubAdapter{
		subscribeFn: func(_ context.Context, _ dapr.SubscribeAdapterRequest) (dapr.PubSubSubscription, error) {
			t.Fatalf("adapter must not be called when consumer_group is empty (FR-T1-PUBSUB-003)")
			return nil, nil
		},
	}
	srv := grpc.NewServer()
	pubsubv1.RegisterPubSubServiceServer(srv, &pubsubHandler{deps: Deps{PubSubAdapter: a}})
	go func() { _ = srv.Serve(lis) }()
	defer srv.Stop()

	dialer := func(context.Context, string) (net.Conn, error) { return lis.Dial() }
	conn, _ := grpc.NewClient(
		"passthrough://bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	defer conn.Close()
	client := pubsubv1.NewPubSubServiceClient(conn)
	stream, _ := client.Subscribe(context.Background(), &pubsubv1.SubscribeRequest{Topic: "t", Context: makeTenantCtx("T")})
	_, err := stream.Recv()
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("status: got %v want InvalidArgument", got)
	}
}
