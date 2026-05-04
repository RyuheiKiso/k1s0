// 本ファイルは workflow_wait.go の単体テスト。
//
// 試験戦略:
//   bufconn + fakeWaitWorkflowServer で Signal RPC 送信 + GetStatus polling の
//   round-trip を検証する。FR-T1-WORKFLOW-005 の外部送信側 helper。
//
// 検証する不変式:
//   1. Signal 送信後、Workflow が COMPLETED に到達 → SignalAndAwait が COMPLETED 返却
//   2. Workflow が timeout 内に終端しない → context.DeadlineExceeded を返却

package k1s0_test

import (
	// 標準 context。
	"context"
	// gRPC 接続用 net.Conn。
	"net"
	// 同期プリミティブ (server side state 保護)。
	"sync"
	// テスト fail / 報告。
	"testing"
	// 時間制御 (status 切替の delay simulate)。
	"time"

	// SDK 本体。
	"github.com/k1s0/sdk-go/k1s0"
	// proto Workflow stub。
	workflowv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/workflow/v1"

	// gRPC ランタイム。
	"google.golang.org/grpc"
	// in-process gRPC listener。
	"google.golang.org/grpc/test/bufconn"
)

// fakeWaitWorkflowServer は SignalAndAwait test 用の WorkflowServiceServer 実装。
// Signal を受信したことを記録し、GetStatus はその後 N 回目から COMPLETED を返す。
type fakeWaitWorkflowServer struct {
	// 未実装メソッド埋め込み。
	workflowv1.UnimplementedWorkflowServiceServer
	// state ロック。
	mu sync.Mutex
	// 受信した Signal 件数。
	signalCount int
	// GetStatus 呼出回数。
	statusCallCount int
	// completionAfterStatusCalls 回 (≧0) の GetStatus で COMPLETED を返す閾値。
	completionAfterStatusCalls int
	// 常に RUNNING を返す flag (timeout test 用)。
	stuckRunning bool
}

// Signal は受信を記録する (副作用は計数のみ)。
func (s *fakeWaitWorkflowServer) Signal(_ context.Context, _ *workflowv1.SignalRequest) (*workflowv1.SignalResponse, error) {
	// 状態を保護する。
	s.mu.Lock()
	// unlock を defer。
	defer s.mu.Unlock()
	// 受信件数を加算する。
	s.signalCount++
	// 空応答を返す (proto 上は空 message)。
	return &workflowv1.SignalResponse{}, nil
}

// GetStatus は閾値を超えると COMPLETED に切り替わる (Workflow の進行 simulate)。
func (s *fakeWaitWorkflowServer) GetStatus(_ context.Context, _ *workflowv1.GetStatusRequest) (*workflowv1.GetStatusResponse, error) {
	// 状態を保護する。
	s.mu.Lock()
	// unlock を defer。
	defer s.mu.Unlock()
	// 呼出回数を加算する。
	s.statusCallCount++
	// stuckRunning なら常に RUNNING を返す (timeout test 経路)。
	if s.stuckRunning {
		return &workflowv1.GetStatusResponse{Status: workflowv1.WorkflowStatus_RUNNING}, nil
	}
	// 閾値未満は RUNNING、超えたら COMPLETED を返す (Workflow が進行する simulate)。
	if s.statusCallCount < s.completionAfterStatusCalls {
		return &workflowv1.GetStatusResponse{Status: workflowv1.WorkflowStatus_RUNNING}, nil
	}
	// 閾値到達 → COMPLETED。
	return &workflowv1.GetStatusResponse{Status: workflowv1.WorkflowStatus_COMPLETED}, nil
}

// startWaitServer は bufconn 上に fakeWaitWorkflowServer を起動する。
func startWaitServer(t *testing.T, fake *fakeWaitWorkflowServer) (*grpc.Server, *bufconn.Listener) {
	// helper を marker。
	t.Helper()
	// bufconn listener を 1 MiB バッファで生成する。
	lis := bufconn.Listen(1024 * 1024)
	// gRPC server を生成する。
	srv := grpc.NewServer()
	// fakeWorkflowServer を WorkflowService として登録する。
	workflowv1.RegisterWorkflowServiceServer(srv, fake)
	// goroutine で listener を消費する。
	go func() { _ = srv.Serve(lis) }()
	// listener と server を返す。
	return srv, lis
}

// makeWaitClient は bufconn 経由で k1s0.Client を作る helper。
func makeWaitClient(t *testing.T, lis *bufconn.Listener) *k1s0.Client {
	// helper を marker。
	t.Helper()
	// bufconn 用 dialer。
	dialer := func(context.Context, string) (net.Conn, error) { return lis.Dial() }
	// Config を構築する。
	cfg := k1s0.Config{
		Target:      "passthrough:///bufnet",
		TenantID:    "T",
		Subject:     "wait-test",
		UseTLS:      false,
		DialOptions: []grpc.DialOption{grpc.WithContextDialer(dialer)},
	}
	// Client を生成する。
	c, err := k1s0.New(context.Background(), cfg)
	// 失敗時はテスト中断。
	if err != nil {
		t.Fatalf("k1s0.New: %v", err)
	}
	// Client を返却する。
	return c
}

// TestSignalAndAwait_HappyPath_ReachesCompleted は Signal 送信後、Workflow が
// COMPLETED に到達する経路を検証する。FR-T1-WORKFLOW-005 の外部送信側経路。
func TestSignalAndAwait_HappyPath_ReachesCompleted(t *testing.T) {
	// 3 回目の GetStatus で COMPLETED に切り替わる fakeServer を準備する。
	fake := &fakeWaitWorkflowServer{completionAfterStatusCalls: 3}
	// gRPC server を起動する。
	srv, lis := startWaitServer(t, fake)
	// テスト終了時に server を停止する。
	defer srv.Stop()
	// k1s0.Client を作成する。
	c := makeWaitClient(t, lis)
	// テスト終了時に Client を閉じる。
	defer c.Close()
	// 短い poll interval で素早く検出する設定。
	cfg := k1s0.SignalAndAwaitConfig{
		PollInterval: 10 * time.Millisecond,
		Timeout:      1 * time.Second,
	}
	// SignalAndAwait を呼ぶ。
	status, err := c.Workflow().SignalAndAwait(context.Background(), "wf-1", "approve", []byte("ok"), cfg)
	// 期待: error 無し、status = COMPLETED。
	if err != nil {
		t.Fatalf("SignalAndAwait error: %v", err)
	}
	// 終端 status が COMPLETED であること。
	if status != workflowv1.WorkflowStatus_COMPLETED {
		t.Fatalf("status = %v, want COMPLETED", status)
	}
	// fakeServer 側の Signal 受信が 1 件であること。
	fake.mu.Lock()
	// signal が一度だけ呼ばれたこと。
	if fake.signalCount != 1 {
		t.Fatalf("signalCount = %d, want 1", fake.signalCount)
	}
	// state ロックを解除する。
	fake.mu.Unlock()
}

// TestSignalAndAwait_Timeout_ReturnsContextError は Workflow が終端しない
// 経路で context.DeadlineExceeded を返すことを検証する。
func TestSignalAndAwait_Timeout_ReturnsContextError(t *testing.T) {
	// 常に RUNNING を返す fakeServer を準備する。
	fake := &fakeWaitWorkflowServer{stuckRunning: true}
	// gRPC server を起動する。
	srv, lis := startWaitServer(t, fake)
	// テスト終了時に server を停止する。
	defer srv.Stop()
	// k1s0.Client を作成する。
	c := makeWaitClient(t, lis)
	// テスト終了時に Client を閉じる。
	defer c.Close()
	// 50ms で全体 timeout する設定 (poll interval も短く)。
	cfg := k1s0.SignalAndAwaitConfig{
		PollInterval: 5 * time.Millisecond,
		Timeout:      50 * time.Millisecond,
	}
	// SignalAndAwait を呼ぶ。
	_, err := c.Workflow().SignalAndAwait(context.Background(), "wf-stuck", "tick", nil, cfg)
	// 期待: ctx timeout 由来の error。
	if err == nil {
		t.Fatalf("SignalAndAwait should fail with timeout")
	}
	// fakeServer 側の Signal は 1 件 (失敗前に送信成功)、GetStatus は 1 件以上呼ばれている。
	fake.mu.Lock()
	// Signal が 1 回呼ばれたこと。
	if fake.signalCount != 1 {
		t.Fatalf("signalCount = %d, want 1", fake.signalCount)
	}
	// GetStatus が少なくとも 1 回呼ばれた (poll が 1 周以上回ったこと)。
	if fake.statusCallCount < 1 {
		t.Fatalf("statusCallCount = %d, want >=1", fake.statusCallCount)
	}
	// state ロックを解除する。
	fake.mu.Unlock()
}
