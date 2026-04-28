// 本ファイルは StateService を in-memory Dapr backend 経由で結線する gRPC テスト。
//
// 試験戦略:
//   bufconn で in-memory Listener を構築し、本番と同じ Register hook で gRPC server を起動する。
//   Dapr sidecar の代わりに dapr.NewClientWithInMemoryBackends() を使うことで、
//   cmd/state/main.go の dev / CI モードと完全に同じ adapter 経路を辿る。
//   これにより「sidecar 不在環境でも State.Set/Get/Delete/BulkGet/Transact が実値を返す」
//   ことを実 gRPC レイヤを通した形で証明する。
//
// 本テストは log_telemetry_integration_test.go と並行して、cmd/state バイナリの
// dev モード起動が「ハンドラが返す内容が proto 規約に従う」レベルで動作することを保証する。

package state

import (
	// 全 RPC で context を伝搬する。
	"context"
	// bufconn の net.Conn 型。
	"net"
	// テストハーネス。
	"testing"

	// in-memory Dapr backend 構築。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// proto stub。
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
	// gRPC server / client。
	"google.golang.org/grpc"
	// 認証なし credential。
	"google.golang.org/grpc/credentials/insecure"
	// bufconn の Listener 型。
	"google.golang.org/grpc/test/bufconn"
)

// startStateServerWithInMemoryDapr は cmd/state/main.go の dev モードと同じ構成で
// gRPC server を起動する。Dapr backend は in-memory に切替える。
func startStateServerWithInMemoryDapr(t *testing.T) (statev1.StateServiceClient, func()) {
	// テストヘルパであることをマーク。
	t.Helper()
	// 1 MiB バッファの bufconn を生成する。
	lis := bufconn.Listen(1024 * 1024)
	// in-memory Dapr backend を構築する（cmd/state の DAPR_GRPC_ENDPOINT 未設定モード相当）。
	client := dapr.NewClientWithInMemoryBackends()
	// 5 building block の adapter を Client から構築する（NewDepsFromClient と同じ）。
	deps := NewDepsFromClient(client)
	// gRPC server を生成する。
	srv := grpc.NewServer()
	// 本番の Register hook で 7 service を登録する。
	Register(deps)(srv)
	// 別 goroutine で listen ループを回す。
	go func() {
		// listen 失敗（bufconn 終了）は test 終了時の自然停止なので無視する。
		_ = srv.Serve(lis)
	}()
	// bufconn dialer を構築する。
	dialer := func(context.Context, string) (net.Conn, error) {
		// Conn を取得する。
		return lis.Dial()
	}
	// gRPC client を bufconn 越しに接続する。
	conn, err := grpc.NewClient(
		// passthrough scheme。
		"passthrough://bufnet",
		// dialer を注入する。
		grpc.WithContextDialer(dialer),
		// TLS なし。
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	// dial 失敗は即座に Fatal。
	if err != nil {
		// fatal で test を停止する。
		t.Fatalf("grpc.NewClient failed: %v", err)
	}
	// StateService の typed client を生成する。
	c := statev1.NewStateServiceClient(conn)
	// cleanup 関数。
	cleanup := func() {
		// client conn を閉じる。
		_ = conn.Close()
		// server を停止する。
		srv.Stop()
		// listener を閉じる。
		_ = lis.Close()
	}
	// client / cleanup を返却する。
	return c, cleanup
}

// gRPC client から Set → Get → Delete → Get の round-trip を、
// in-memory Dapr backend を通して proto レイヤ込みで動作することを検証する。
func TestStateService_InMemoryDaprBackend_RoundTrip(t *testing.T) {
	// bufconn server を起動する。
	c, cleanup := startStateServerWithInMemoryDapr(t)
	// テスト終了時に cleanup する。
	defer cleanup()
	// Background context を使う。
	ctx := context.Background()

	// 1. Set: 新規キーを保存する。
	if _, err := c.Set(ctx, &statev1.SetRequest{
		// store 名（Dapr Component 名）。
		Store: "valkey-default",
		// key。
		Key: "session:abc",
		// data。
		Data: []byte("user-123"),
	}); err != nil {
		// fatal。
		t.Fatalf("Set failed: %v", err)
	}

	// 2. Get: 直前に Set した値が返るか。
	getResp, err := c.Get(ctx, &statev1.GetRequest{
		// store。
		Store: "valkey-default",
		// key。
		Key: "session:abc",
	})
	// Get 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("Get failed: %v", err)
	}
	// 値検証。
	if string(getResp.GetData()) != "user-123" {
		// 不一致は test 失敗。
		t.Fatalf("Get data mismatch: got %q", getResp.GetData())
	}
	// Etag は in-memory backend が "v<N>" 形式で発行する。
	if getResp.GetEtag() == "" {
		// etag 不在は test 失敗。
		t.Fatalf("Get etag should be non-empty")
	}
	// NotFound=false のはず。
	if getResp.GetNotFound() {
		// 不一致は test 失敗。
		t.Fatalf("Get reported NotFound after Set")
	}

	// 3. BulkGet: 複数 key を一括取得する。
	if _, err := c.Set(ctx, &statev1.SetRequest{
		// store。
		Store: "valkey-default",
		// key2。
		Key: "session:def",
		// data2。
		Data: []byte("user-456"),
	}); err != nil {
		// fatal。
		t.Fatalf("Set 2 failed: %v", err)
	}
	// BulkGet を実行する。
	bulkResp, err := c.BulkGet(ctx, &statev1.BulkGetRequest{
		// store。
		Store: "valkey-default",
		// keys。
		Keys: []string{"session:abc", "session:def", "session:nope"},
	})
	// BulkGet 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("BulkGet failed: %v", err)
	}
	// 結果は 3 件（うち 1 件 NotFound）。
	if len(bulkResp.GetResults()) != 3 {
		// 件数不一致は test 失敗。
		t.Fatalf("BulkGet results count: got %d", len(bulkResp.GetResults()))
	}
	// session:abc は user-123。
	if v := string(bulkResp.GetResults()["session:abc"].GetData()); v != "user-123" {
		// 不一致は test 失敗。
		t.Fatalf("BulkGet session:abc data mismatch: %q", v)
	}
	// session:nope は NotFound。
	if !bulkResp.GetResults()["session:nope"].GetNotFound() {
		// 不一致は test 失敗。
		t.Fatalf("BulkGet session:nope should be NotFound")
	}

	// 4. Transact: Set + Delete を 1 トランザクションで実行する。
	if _, err := c.Transact(ctx, &statev1.TransactRequest{
		// store。
		Store: "valkey-default",
		// 2 ops。
		Operations: []*statev1.TransactOp{
			// Set op（新規 key）。
			{Op: &statev1.TransactOp_Set{Set: &statev1.SetRequest{Store: "valkey-default", Key: "tx-key", Data: []byte("tx-value")}}},
			// Delete op（既存 key）。
			{Op: &statev1.TransactOp_Delete{Delete: &statev1.DeleteRequest{Store: "valkey-default", Key: "session:abc"}}},
		},
	}); err != nil {
		// fatal。
		t.Fatalf("Transact failed: %v", err)
	}
	// transactional 効果を確認する。
	txGet, err := c.Get(ctx, &statev1.GetRequest{Store: "valkey-default", Key: "tx-key"})
	// Get 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("Get tx-key failed: %v", err)
	}
	// 新規 key が読めるはず。
	if string(txGet.GetData()) != "tx-value" {
		// 不一致は test 失敗。
		t.Fatalf("tx-key data mismatch: %q", txGet.GetData())
	}
	// 削除済 key は NotFound のはず。
	delGet, err := c.Get(ctx, &statev1.GetRequest{Store: "valkey-default", Key: "session:abc"})
	// Get 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("Get session:abc after delete tx failed: %v", err)
	}
	// NotFound=true のはず。
	if !delGet.GetNotFound() {
		// 不一致は test 失敗。
		t.Fatalf("session:abc should be NotFound after delete tx")
	}
}
