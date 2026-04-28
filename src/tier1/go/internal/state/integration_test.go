// 本ファイルは StateService の in-process gRPC 結線テスト。
//
// 試験戦略:
//   `bufconn` で in-memory Listener を構築し、本番と同じ Register hook で
//   gRPC server と client を結ぶ。proto serialization / gRPC routing /
//   handler 委譲 / adapter 戻り値の全パスが本番と同じコードで動くことを保証する。
//   Dapr SDK / Valkey は fake adapter で切り離す（CI 上で Dapr sidecar 不要）。
//
// 本テストが PASS すれば「StateService.Get/Set/Delete を gRPC client から
// 呼んで実値が往復する」が単体テストではなく実 gRPC レイヤを通した形で証明される。

package state

import (
	"context"
	"net"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"
)

const bufSize = 1024 * 1024

// startServer は in-memory bufconn 上に gRPC server を起動し、StateService を登録する。
// 本番の Register(deps) と同じ呼び出しを行うため、registration paths も含めた回帰防止になる。
func startServer(t *testing.T, store map[string][]byte) (statev1.StateServiceClient, func()) {
	t.Helper()
	lis := bufconn.Listen(bufSize)

	// fake adapter（in-memory map で State をシミュレート）。
	a := &fakeStateAdapter{
		getFn: func(_ context.Context, req dapr.StateGetRequest) (dapr.StateGetResponse, error) {
			data, ok := store[req.Key]
			if !ok {
				return dapr.StateGetResponse{NotFound: true}, nil
			}
			// 簡易 etag として "v" + 値長を返す（fake 用、production の Valkey は別物）。
			return dapr.StateGetResponse{Data: data, Etag: "fake-etag", NotFound: false}, nil
		},
		setFn: func(_ context.Context, req dapr.StateSetRequest) (dapr.StateSetResponse, error) {
			store[req.Key] = req.Data
			return dapr.StateSetResponse{NewEtag: "fake-etag"}, nil
		},
		deleteFn: func(_ context.Context, req dapr.StateSetRequest) error {
			delete(store, req.Key)
			return nil
		},
	}
	deps := Deps{StateAdapter: a}
	srv := grpc.NewServer()
	// 本番の Register hook を使う（registerAll パスの回帰検出）。
	Register(deps)(srv)

	go func() {
		// listen ループ。bufconn 終了時に Serve がエラーを返すが test 中は無視する。
		_ = srv.Serve(lis)
	}()

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

	client := statev1.NewStateServiceClient(conn)
	cleanup := func() {
		_ = conn.Close()
		srv.Stop()
		_ = lis.Close()
	}
	return client, cleanup
}

// gRPC client から Set → Get → Delete → Get の round-trip を行い、
// 全ステップで proto と handler が期待通り動くことを検証する。
func TestStateService_RoundTrip_OverGRPC(t *testing.T) {
	store := make(map[string][]byte)
	client, cleanup := startServer(t, store)
	defer cleanup()

	ctx := context.Background()

	// 1. Set: 新規キー（NFR-E-AC-003 で tenant_id 必須）。
	_, err := client.Set(ctx, &statev1.SetRequest{
		Store:   "valkey-default",
		Key:     "session:abc",
		Data:    []byte("user-123"),
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Set failed: %v", err)
	}
	if got := string(store["session:abc"]); got != "user-123" {
		t.Fatalf("Set didn't propagate to fake store: got %q", got)
	}

	// 2. Get: 直前に Set した値が読めるか。
	getResp, err := client.Get(ctx, &statev1.GetRequest{
		Store:   "valkey-default",
		Key:     "session:abc",
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if string(getResp.GetData()) != "user-123" {
		t.Fatalf("Get value mismatch: got %q want user-123", getResp.GetData())
	}
	if getResp.GetEtag() != "fake-etag" {
		t.Fatalf("Get etag mismatch: got %q", getResp.GetEtag())
	}
	if getResp.GetNotFound() {
		t.Fatalf("Get reported NotFound after Set")
	}

	// 3. Delete: キー削除が成功するか。
	delResp, err := client.Delete(ctx, &statev1.DeleteRequest{
		Store:   "valkey-default",
		Key:     "session:abc",
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Delete failed: %v", err)
	}
	if !delResp.GetDeleted() {
		t.Fatalf("Delete returned Deleted=false")
	}
	if _, ok := store["session:abc"]; ok {
		t.Fatalf("Delete didn't remove key from fake store")
	}

	// 4. Get 再実行: 削除後は NotFound=true が返るか。
	getResp2, err := client.Get(ctx, &statev1.GetRequest{
		Store:   "valkey-default",
		Key:     "session:abc",
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Get after Delete failed: %v", err)
	}
	if !getResp2.GetNotFound() {
		t.Fatalf("Get after Delete should return NotFound=true, got data=%q", getResp2.GetData())
	}
}
