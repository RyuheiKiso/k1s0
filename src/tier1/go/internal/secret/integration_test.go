// 本ファイルは SecretsService の in-process gRPC 結線テスト。
//
// 試験戦略:
//   bufconn で in-memory Listener を構築し、本番と同じ Register hook で gRPC server と
//   client を結ぶ。OpenBao の InMemoryKV backend を経由して proto serialization /
//   gRPC routing / handler 委譲 / adapter 戻り値の全パスが本番と同じコードで動くことを保証する。
//
// 本テストが PASS すれば「SecretsService.Get / BulkGet / Rotate を gRPC client から
// 呼んで実値が往復する」が単体テストではなく実 gRPC レイヤを通した形で証明される。

package secret

import (
	// 全 RPC で context を伝搬する。
	"context"
	// bufconn 用 net.Conn 型。
	"net"
	// テストハーネス。
	"testing"

	// OpenBao adapter（in-memory backend 構築）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/openbao"
	// proto stub の SecretsService 型。
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	secretsv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/secrets/v1"
	// gRPC server / client。
	"google.golang.org/grpc"
	// bufconn の Listener 型。
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"
)

// bufSize は bufconn の buffer size（1 MiB）。
const integrationBufSize = 1024 * 1024

// seedSecret は openbao.InMemoryKV に直接 Put して事前条件 secret を準備する。
// ListAndGet テストの前提条件構築に使う（adapter には Set がないため backend に直接書く）。
func seedSecret(t *testing.T, kv *openbao.InMemoryKV, path string, data map[string]interface{}) {
	// テストヘルパであることをマーク。
	t.Helper()
	// 直接 Put する。
	if _, err := kv.Put(context.Background(), path, data); err != nil {
		// 失敗時は test を停止する。
		t.Fatalf("seed put failed: %v", err)
	}
}

// startSecretsServerWithKV は seed 用 InMemoryKV を直接受け取って in-memory client を作る。
// 既存 Client コンストラクタは backend を内包するため exposing 用の補助 helper。
// 簡略のため、本テストでは backendをまず作成 → wrap という順で進める。
func startSecretsServerWithKV(t *testing.T) (secretsv1.SecretsServiceClient, *openbao.InMemoryKV, func()) {
	// テストヘルパであることをマーク。
	t.Helper()
	// in-memory backend を生成する。
	kv := openbao.NewInMemoryKV()
	// kv を内包する Client を構築する（kv / lister 両方を同一 InMemoryKV にバインド）。
	client := openbao.NewClientFromInMemoryKV(kv)
	// adapter を生成する。
	adapter := openbao.NewSecretsAdapter(client)
	// 1 MiB バッファの bufconn を生成する。
	lis := bufconn.Listen(integrationBufSize)
	// gRPC server を生成する。
	srv := grpc.NewServer()
	// 本番の Register hook を使う。
	Register(Deps{SecretsAdapter: adapter})(srv)
	// 別 goroutine で listen ループを回す。
	go func() {
		// listen 失敗は無視する。
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
	// dial 失敗は test 停止。
	if err != nil {
		// fatal。
		t.Fatalf("grpc.NewClient failed: %v", err)
	}
	// typed client を生成する。
	c := secretsv1.NewSecretsServiceClient(conn)
	// cleanup 関数。
	cleanup := func() {
		// client / server / listener を逆順に閉じる。
		_ = conn.Close()
		// server を停止する。
		srv.Stop()
		// listener を閉じる。
		_ = lis.Close()
	}
	// 返却する。
	return c, kv, cleanup
}

// gRPC client から Get / Rotate の round-trip を行い、proto serialization と
// handler / adapter / in-memory backend の全パスが期待通り動くことを検証する。
func TestSecretsService_RoundTrip_OverGRPC(t *testing.T) {
	// bufconn server を起動する。
	c, kv, cleanup := startSecretsServerWithKV(t)
	// テスト終了時に cleanup する。
	defer cleanup()
	// Background context を使う。
	ctx := context.Background()
	// 事前条件として in-memory backend に secret を seed する。
	seedSecret(t, kv,"tenant/T1/db/master", map[string]interface{}{
		// password を文字列で保存する。
		"password": "p@ss",
		// username を文字列で保存する。
		"username": "k1s0app",
	})

	// 1. Get: seed した secret が proto 応答として正しく返るか。
	getResp, err := c.Get(ctx, &secretsv1.GetSecretRequest{
		// path を指定する。
		Name: "tenant/T1/db/master",
		// テナントを指定する。
		Context: &commonv1.TenantContext{TenantId: "T1"},
	})
	// Get 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("Get failed: %v", err)
	}
	// 値検証: password を確認する。
	if getResp.GetValues()["password"] != "p@ss" {
		// 不一致は test 失敗。
		t.Fatalf("password mismatch: %s", getResp.GetValues()["password"])
	}
	// バージョンは 1（seed 直後）。
	if getResp.GetVersion() != 1 {
		// 不一致は test 失敗。
		t.Fatalf("version mismatch: got %d", getResp.GetVersion())
	}

	// 2. Rotate: バージョン bump が成功し、応答に prev/new/rotated_at_ms が詰まる。
	rotateResp, err := c.Rotate(ctx, &secretsv1.RotateSecretRequest{
		// 対象 path。
		Name: "tenant/T1/db/master",
		// テナント。
		Context: &commonv1.TenantContext{TenantId: "T1"},
	})
	// Rotate 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("Rotate failed: %v", err)
	}
	// 新バージョンは 2 のはず。
	if rotateResp.GetNewVersion() != 2 {
		// 不一致は test 失敗。
		t.Fatalf("new_version mismatch: got %d", rotateResp.GetNewVersion())
	}
	// 直前バージョンは 1。
	if rotateResp.GetPreviousVersion() != 1 {
		// 不一致は test 失敗。
		t.Fatalf("previous_version mismatch: got %d", rotateResp.GetPreviousVersion())
	}
	// rotated_at_ms は実時刻（> 0）が詰まっているはず。
	if rotateResp.GetRotatedAtMs() <= 0 {
		// 0 / 負値は test 失敗。
		t.Fatalf("rotated_at_ms not populated: got %d", rotateResp.GetRotatedAtMs())
	}

	// 3. BulkGet: tenant 配下を List + Get で全件取得する。
	// 追加 seed して 2 件にする。
	seedSecret(t, kv,"tenant/T1/api/key", map[string]interface{}{
		// API キーを文字列で保存する。
		"value": "API-KEY-XYZ",
	})
	// gRPC client から BulkGet を呼ぶ。
	bulkResp, err := c.BulkGet(ctx, &secretsv1.BulkGetSecretRequest{
		// テナント。
		Context: &commonv1.TenantContext{TenantId: "T1"},
	})
	// BulkGet 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("BulkGet failed: %v", err)
	}
	// 結果は 2 件のはず（"db/master" と "api/key"、prefix は trim 済）。
	if len(bulkResp.GetResults()) != 2 {
		// 件数不一致は test 失敗。
		t.Fatalf("BulkGet result count mismatch: got %d", len(bulkResp.GetResults()))
	}
	// "db/master" が含まれているか確認する。
	if _, ok := bulkResp.GetResults()["db/master"]; !ok {
		// 不在は test 失敗。
		t.Fatalf("BulkGet missing db/master")
	}
	// "api/key" が含まれているか確認する。
	if entry, ok := bulkResp.GetResults()["api/key"]; !ok {
		// 不在は test 失敗。
		t.Fatalf("BulkGet missing api/key")
	} else if entry.GetValues()["value"] != "API-KEY-XYZ" {
		// 値不一致は test 失敗。
		t.Fatalf("api/key value mismatch: %s", entry.GetValues()["value"])
	}
}
