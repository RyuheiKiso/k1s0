// 本ファイルは FR-T1-PUBSUB-004 (Dead Letter Queue) の handler / helper 単体テスト。
//
// 試験戦略:
//   - 純関数 (dlqTopicName / serviceNameFromConsumerGroup) の代表ケース
//   - handler 経由 (Subscribe RPC) での adapter 引数注入の bufconn 結線テスト
//
// 検証する不変式:
//   1. DLQ topic 名規則 `k1s0.<tenant_id>.dlq.<service_name>`
//   2. consumer_group `k1s0.<tenant>.<service>` から service 部抽出
//   3. handler が adapter.SubscribeAdapterRequest.DeadLetterTopic に上記規則の値を設定する

package state

import (
	// 標準 context。
	"context"
	// gRPC bufconn による in-process 結線。
	"net"
	// テスト用ロガー / fail。
	"testing"

	// adapter 型参照。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// SDK proto stub。
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	// gRPC server / client。
	"google.golang.org/grpc"
	// 認証なしクレデンシャル。
	"google.golang.org/grpc/credentials/insecure"
	// in-process listener。
	"google.golang.org/grpc/test/bufconn"
)

// TestDLQTopicName_HappyPath は受け入れ基準
// 「`k1s0.<tenant_id>.dlq.<service_name>` 形式」を機械検証する。
func TestDLQTopicName_HappyPath(t *testing.T) {
	// tenant=acme / service=billing の正常入力。
	got := dlqTopicName("acme", "billing")
	// 期待値は 4 セグメントの厳密一致。
	want := "k1s0.acme.dlq.billing"
	// 期待値と異なれば fail。
	if got != want {
		t.Fatalf("dlqTopicName(acme, billing) = %q, want %q", got, want)
	}
}

// TestDLQTopicName_EmptyTenantReturnsEmpty は tenant_id 空入力で空文字を返すことを確認する
// （NFR-E-AC-003 越境防止の defense-in-depth）。
func TestDLQTopicName_EmptyTenantReturnsEmpty(t *testing.T) {
	// tenant 空 / service 非空。
	got := dlqTopicName("", "billing")
	// 結果は空文字（DLQ 不有効化）。
	if got != "" {
		t.Fatalf("dlqTopicName('', billing) = %q, want empty", got)
	}
}

// TestDLQTopicName_EmptyServiceReturnsEmpty は service_name 空入力で空文字を返すことを確認する
// （consumer_group 形式違反時の安全側 fallback）。
func TestDLQTopicName_EmptyServiceReturnsEmpty(t *testing.T) {
	// tenant 非空 / service 空。
	got := dlqTopicName("acme", "")
	// 結果は空文字。
	if got != "" {
		t.Fatalf("dlqTopicName(acme, '') = %q, want empty", got)
	}
}

// TestServiceNameFromConsumerGroup_PrefixedExtracted は
// 正規化済 consumer_group から service 部の抽出を確認する。
func TestServiceNameFromConsumerGroup_PrefixedExtracted(t *testing.T) {
	// 正規化済 consumer_group。
	got := serviceNameFromConsumerGroup("acme", "k1s0.acme.billing")
	// 期待値は service 部の `billing` のみ。
	if got != "billing" {
		t.Fatalf("serviceNameFromConsumerGroup(acme, k1s0.acme.billing) = %q, want billing", got)
	}
}

// TestServiceNameFromConsumerGroup_BadPrefixReturnsEmpty は
// prefix 不一致の consumer_group で空文字を返すことを確認する。
func TestServiceNameFromConsumerGroup_BadPrefixReturnsEmpty(t *testing.T) {
	// tenant 不一致 prefix。
	got := serviceNameFromConsumerGroup("acme", "other.acme.billing")
	// 結果は空文字。
	if got != "" {
		t.Fatalf("serviceNameFromConsumerGroup(acme, other.acme.billing) = %q, want empty", got)
	}
}

// TestPubSubService_Subscribe_PassesDLQTopicToAdapter は handler 経由で
// adapter.SubscribeAdapterRequest.DeadLetterTopic に
// `k1s0.<tenant>.dlq.<service>` 形式の値が渡ることを bufconn で検証する。
//
// このテストは FR-T1-PUBSUB-004 の handler ↔ adapter 結線が正しく組まれていることを
// 機械的に守る regression test。proto 拡張不要 (Subscription metadata 経由) のため、
// handler 段で DeadLetterTopic を組み立てて adapter に渡す経路が壊れた場合に検出する。
func TestPubSubService_Subscribe_PassesDLQTopicToAdapter(t *testing.T) {
	// in-process listener。
	lis := bufconn.Listen(bufSize)
	// adapter.SubscribeAdapterRequest を capture する変数。
	var capturedDLQ string
	// 1 件投入してすぐ close する subscription fake（receive 1 回で関数を戻すため）。
	sub := &fakeSubscription{events: make(chan *dapr.SubscribedEvent, 1)}
	// イベントを 1 件投入してから channel を閉じる。
	go func() {
		// stream.Send が 1 件で Subscribe ループは次の Receive で errSubscriptionClosed を受け関数終了する。
		sub.events <- &dapr.SubscribedEvent{Topic: "t", Data: []byte("e1"), Ack: func() error { return nil }}
		// channel を閉じる。
		close(sub.events)
		// fakeSubscription.Close() の二重 close 防止フラグを立てる（handler の defer で
		// sub.Close() が呼ばれるため、ここで `closed=true` にしないと re-close で panic する）。
		sub.closed = true
	}()
	// fake adapter で DeadLetterTopic を capture する。
	a := &fakePubSubAdapter{
		subscribeFn: func(_ context.Context, req dapr.SubscribeAdapterRequest) (dapr.PubSubSubscription, error) {
			// DLQ topic 値を保存する。
			capturedDLQ = req.DeadLetterTopic
			return sub, nil
		},
	}
	// gRPC サーバ起動。
	srv := grpc.NewServer()
	// pubsubHandler を登録する。
	pubsubv1.RegisterPubSubServiceServer(srv, &pubsubHandler{deps: Deps{PubSubAdapter: a}})
	// goroutine で listener を消費する。
	go func() { _ = srv.Serve(lis) }()
	// テスト終了時にサーバを停止する。
	defer srv.Stop()

	// bufconn 用 dialer 関数。
	dialer := func(context.Context, string) (net.Conn, error) { return lis.Dial() }
	// gRPC client を作成する（認証なし、bufconn 経由）。
	conn, err := grpc.NewClient(
		"passthrough://bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	// dial 失敗時はテスト中断。
	if err != nil {
		t.Fatalf("grpc.NewClient: %v", err)
	}
	// テスト終了時に接続を閉じる。
	defer conn.Close()
	// PubSubService クライアントを作る。
	client := pubsubv1.NewPubSubServiceClient(conn)
	// Subscribe RPC を呼ぶ。tenant=acme / consumer_group=billing → 正規化後 k1s0.acme.billing。
	stream, err := client.Subscribe(context.Background(), &pubsubv1.SubscribeRequest{
		Topic: "t", ConsumerGroup: "billing", Context: makeTenantCtx("acme"),
	})
	// Subscribe 確立失敗はテスト中断。
	if err != nil {
		t.Fatalf("Subscribe: %v", err)
	}
	// 1 件受信して handler の adapter 呼び出しが完了したことを確認する。
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("Recv: %v", err)
	}
	// 期待値: handler が dlqTopicName("acme","billing") = "k1s0.acme.dlq.billing" を adapter に渡す。
	want := "k1s0.acme.dlq.billing"
	// capturedDLQ と期待値が一致することを検証する。
	if capturedDLQ != want {
		t.Fatalf("adapter DeadLetterTopic: got %q want %q (FR-T1-PUBSUB-004)", capturedDLQ, want)
	}
}
