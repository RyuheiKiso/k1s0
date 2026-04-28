// 本ファイルは WorkflowService の in-process gRPC 結線テスト。
//
// 試験戦略:
//   bufconn で in-memory Listener を構築し、本番と同じ Register hook で gRPC server と
//   client を結ぶ。Temporal の InMemoryTemporal backend を経由して proto serialization /
//   gRPC routing / handler 委譲 / adapter 戻り値の全パスが本番と同じコードで動くことを保証する。
//
// 本テストが PASS すれば「WorkflowService の 6 RPC を gRPC client から呼んで実値が
// 往復する」が単体テストではなく実 gRPC レイヤを通した形で証明される。

package workflow

import (
	// 全 RPC で context を伝搬する。
	"context"
	// bufconn の net.Conn 型。
	"net"
	// テストハーネス。
	"testing"

	// Temporal adapter（in-memory backend 構築）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/temporal"
	// proto stub の WorkflowService 型。
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	workflowv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/workflow/v1"
	// gRPC server / client。
	"google.golang.org/grpc"
	// 認証なし credential。
	"google.golang.org/grpc/credentials/insecure"
	// bufconn の Listener 型。
	"google.golang.org/grpc/test/bufconn"
)

// bufSize は bufconn の buffer size（1 MiB）。
const integrationBufSize = 1024 * 1024

// startWorkflowServer は in-memory bufconn 上に gRPC server を起動し、WorkflowService を登録する。
func startWorkflowServer(t *testing.T) (workflowv1.WorkflowServiceClient, func()) {
	// テストヘルパであることをマーク。
	t.Helper()
	// 1 MiB バッファの bufconn を生成する。
	lis := bufconn.Listen(integrationBufSize)
	// in-memory Temporal backend を構築する（cmd/workflow/main.go の dev モードと同じ Client）。
	client := temporal.NewClientWithInMemory()
	// adapter を生成する。
	adapter := temporal.NewWorkflowAdapter(client)
	// gRPC server を生成する。
	srv := grpc.NewServer()
	// 本番の Register hook を使う。
	Register(Deps{WorkflowAdapter: adapter})(srv)
	// 別 goroutine で listen ループを回す。
	go func() {
		// listen 失敗（bufconn 終了）は test 終了時の自然停止なので無視する。
		_ = srv.Serve(lis)
	}()
	// bufconn を経由する dialer を構築する。
	dialer := func(context.Context, string) (net.Conn, error) {
		// bufconn から Conn を取得して返す。
		return lis.Dial()
	}
	// gRPC client を bufconn 越しに接続する。
	conn, err := grpc.NewClient(
		// passthrough scheme で resolver を簡略化する。
		"passthrough://bufnet",
		// bufconn dialer を注入する。
		grpc.WithContextDialer(dialer),
		// テスト中は TLS なし。
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	// dial 失敗は即座に Fatal。
	if err != nil {
		// fatal で test を停止する。
		t.Fatalf("grpc.NewClient failed: %v", err)
	}
	// WorkflowService の typed client を生成する。
	c := workflowv1.NewWorkflowServiceClient(conn)
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

// gRPC client から Start → Signal → Query → GetStatus → Cancel → GetStatus の round-trip を行い、
// in-memory backend と handler の連携が proto レイヤを通して正しく動くことを検証する。
func TestWorkflowService_RoundTrip_OverGRPC(t *testing.T) {
	// bufconn server を起動する。
	c, cleanup := startWorkflowServer(t)
	// テスト終了時に cleanup する。
	defer cleanup()
	// Background context を使う。
	ctx := context.Background()

	// 1. Start: 新規 workflow 起動。
	startResp, err := c.Start(ctx, &workflowv1.StartRequest{
		// workflow 種別。
		WorkflowType: "ProcessOrder",
		// workflow ID。
		WorkflowId: "order-001",
		// 入力 payload。
		Input: []byte(`{"orderId":"X1"}`),
		// テナント context。
		Context: &commonv1.TenantContext{TenantId: "T1"},
	})
	// Start 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("Start failed: %v", err)
	}
	// workflow_id が echo されることを確認する。
	if startResp.GetWorkflowId() != "order-001" {
		// 不一致は test 失敗。
		t.Fatalf("workflow_id mismatch: got %s", startResp.GetWorkflowId())
	}
	// run_id が発行されている（in-memory backend は 32 文字 hex）。
	if len(startResp.GetRunId()) != 32 {
		// 不一致は test 失敗。
		t.Fatalf("run_id length mismatch: got %d", len(startResp.GetRunId()))
	}

	// 2. Signal: signal が backend に届くこと。
	if _, err := c.Signal(ctx, &workflowv1.SignalRequest{
		// workflow ID。
		WorkflowId: "order-001",
		// signal 名。
		SignalName: "approve",
		// payload。
		Payload: []byte("approved"),
	}); err != nil {
		// fatal。
		t.Fatalf("Signal failed: %v", err)
	}

	// 3. Query: 値なしレスポンスでも成功する。
	queryResp, err := c.Query(ctx, &workflowv1.QueryRequest{
		// workflow ID。
		WorkflowId: "order-001",
		// query 名。
		QueryName: "status",
	})
	// Query 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("Query failed: %v", err)
	}
	// in-memory backend は値を持たないため Result は nil/empty。
	if len(queryResp.GetResult()) != 0 {
		// 不一致は test 失敗。
		t.Fatalf("Query result should be empty for in-memory backend, got %v", queryResp.GetResult())
	}

	// 4. GetStatus: Running 状態のはず。
	statusResp, err := c.GetStatus(ctx, &workflowv1.GetStatusRequest{
		// workflow ID。
		WorkflowId: "order-001",
	})
	// GetStatus 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("GetStatus failed: %v", err)
	}
	// 状態は WORKFLOW_STATUS_RUNNING（0）。
	if statusResp.GetStatus() != workflowv1.WorkflowStatus_RUNNING {
		// 不一致は test 失敗。
		t.Fatalf("status mismatch: got %v want WORKFLOW_STATUS_RUNNING", statusResp.GetStatus())
	}

	// 5. Cancel: 状態を Canceled に遷移させる。
	if _, err := c.Cancel(ctx, &workflowv1.CancelRequest{
		// workflow ID。
		WorkflowId: "order-001",
		// cancel 理由。
		Reason: "user-aborted",
	}); err != nil {
		// fatal。
		t.Fatalf("Cancel failed: %v", err)
	}

	// 6. GetStatus 再実行: Canceled に遷移したはず。
	statusAfter, err := c.GetStatus(ctx, &workflowv1.GetStatusRequest{
		// workflow ID。
		WorkflowId: "order-001",
	})
	// GetStatus 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("GetStatus (after cancel) failed: %v", err)
	}
	// 状態は WORKFLOW_STATUS_CANCELED（3）。
	if statusAfter.GetStatus() != workflowv1.WorkflowStatus_CANCELED {
		// 不一致は test 失敗。
		t.Fatalf("status after cancel: got %v want WORKFLOW_STATUS_CANCELED", statusAfter.GetStatus())
	}

	// 7. 別 workflow を Terminate して Terminated に遷移する経路も検証する。
	if _, err := c.Start(ctx, &workflowv1.StartRequest{
		// workflow 種別。
		WorkflowType: "ProcessOrder",
		// workflow ID。
		WorkflowId: "order-002",
		// テナント context。
		Context: &commonv1.TenantContext{TenantId: "T1"},
	}); err != nil {
		// fatal。
		t.Fatalf("Start order-002 failed: %v", err)
	}
	// Terminate を呼ぶ。
	if _, err := c.Terminate(ctx, &workflowv1.TerminateRequest{
		// workflow ID。
		WorkflowId: "order-002",
		// terminate 理由。
		Reason: "operator-action",
	}); err != nil {
		// fatal。
		t.Fatalf("Terminate failed: %v", err)
	}
	// 状態を確認する。
	terminatedStatus, err := c.GetStatus(ctx, &workflowv1.GetStatusRequest{
		// workflow ID。
		WorkflowId: "order-002",
	})
	// GetStatus 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("GetStatus (after terminate) failed: %v", err)
	}
	// Terminated 状態のはず。
	if terminatedStatus.GetStatus() != workflowv1.WorkflowStatus_TERMINATED {
		// 不一致は test 失敗。
		t.Fatalf("status after terminate: got %v want WORKFLOW_STATUS_TERMINATED", terminatedStatus.GetStatus())
	}
}
