// 本ファイルは workflow_saga.go の単体テスト。
//
// 試験戦略:
//   bufconn + fakeWorkflowServer で in-process gRPC server を立て、k1s0.Client を
//   bufconn dialer で初期化、Saga.Execute() の制御フローを検証する。
//
// 検証する不変式:
//   1. 全 Step が COMPLETED → CompletedSteps が順次蓄積、補償発火なし
//   2. 中央 Step が FAILED → 直前まで COMPLETED した Step を逆順に補償発火
//   3. 補償スキップ Step (CompensationWorkflowType="") は補償発火対象外

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
	// poll interval / timeout 設定。
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

// fakeSagaWorkflowServer は Saga test 用の WorkflowServiceServer 実装。
// Start は受信した workflow_type を順序付きで記録する。GetStatus は workflow_id ごとに
// 事前設定された最終状態を返す (default は COMPLETED)。
type fakeSagaWorkflowServer struct {
	// 未実装メソッド埋め込み。
	workflowv1.UnimplementedWorkflowServiceServer
	// state ロック。
	mu sync.Mutex
	// Start で受信した workflow_type を順序通り記録。
	startedTypes []string
	// 特定 workflow_type を Start で fail させる集合 (Start 失敗のテスト用)。
	failOnStart map[string]bool
	// workflow_type ごとの GetStatus 戻り値 (default COMPLETED)。
	statusByType map[string]workflowv1.WorkflowStatus
}

// Start は workflow_type を記録し、failOnStart に含まれていれば error を返す。
func (s *fakeSagaWorkflowServer) Start(_ context.Context, req *workflowv1.StartRequest) (*workflowv1.StartResponse, error) {
	// 状態を保護する。
	s.mu.Lock()
	// unlock を defer で保証する。
	defer s.mu.Unlock()
	// workflow_type を記録する。
	s.startedTypes = append(s.startedTypes, req.GetWorkflowType())
	// failOnStart 一致なら error 返却 (Start 失敗の simulate)。
	if s.failOnStart[req.GetWorkflowType()] {
		// gRPC 標準 error として返却する。
		return nil, context.DeadlineExceeded
	}
	// 成功時は workflow_id として workflow_type をエコーする (test での識別用)。
	return &workflowv1.StartResponse{
		WorkflowId: req.GetWorkflowType(),
		RunId:      "run-" + req.GetWorkflowType(),
	}, nil
}

// GetStatus は workflow_id (= workflow_type) に応じた最終状態を返す。
func (s *fakeSagaWorkflowServer) GetStatus(_ context.Context, req *workflowv1.GetStatusRequest) (*workflowv1.GetStatusResponse, error) {
	// 状態を保護する。
	s.mu.Lock()
	// unlock を defer で保証する。
	defer s.mu.Unlock()
	// 設定された status を取得する (default COMPLETED)。
	st, ok := s.statusByType[req.GetWorkflowId()]
	if !ok {
		st = workflowv1.WorkflowStatus_COMPLETED
	}
	// status を返却する。
	return &workflowv1.GetStatusResponse{Status: st, RunId: "run-" + req.GetWorkflowId()}, nil
}

// startSagaServer は bufconn 上に fakeSagaWorkflowServer を持つ gRPC server を起動する。
func startSagaServer(t *testing.T, fake *fakeSagaWorkflowServer) (*grpc.Server, *bufconn.Listener) {
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
	// listener と server を返す (テスト終了時に Stop / Close)。
	return srv, lis
}

// makeSagaClient は bufconn 経由で k1s0.Client を作る helper。
func makeSagaClient(t *testing.T, lis *bufconn.Listener) *k1s0.Client {
	// helper を marker。
	t.Helper()
	// bufconn 用 dialer を作る。
	dialer := func(context.Context, string) (net.Conn, error) { return lis.Dial() }
	// Config に DialOptions を注入する形で bufconn 接続。
	cfg := k1s0.Config{
		Target:      "passthrough:///bufnet",
		TenantID:    "T",
		Subject:     "saga-test",
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

// TestSagaExecute_HappyPath_AllStepsCompleted は全 Step が COMPLETED になる経路を検証する。
// FR-T1-WORKFLOW-003 受け入れ基準: 「途中失敗が無ければ補償は発火しない」。
func TestSagaExecute_HappyPath_AllStepsCompleted(t *testing.T) {
	// 全 Step が COMPLETED 帰着の fakeWorkflowServer を準備する。
	fake := &fakeSagaWorkflowServer{
		statusByType: map[string]workflowv1.WorkflowStatus{
			// 3 Step すべて COMPLETED。
			"step1": workflowv1.WorkflowStatus_COMPLETED,
			"step2": workflowv1.WorkflowStatus_COMPLETED,
			"step3": workflowv1.WorkflowStatus_COMPLETED,
		},
	}
	// gRPC server を起動する。
	srv, lis := startSagaServer(t, fake)
	// テスト終了時に server を停止する。
	defer srv.Stop()
	// k1s0.Client を作成する。
	c := makeSagaClient(t, lis)
	// テスト終了時に Client を閉じる。
	defer c.Close()
	// Saga を構築し、3 Step を順次 chain する。
	saga := c.Workflow().NewSaga().
		Step("step1", "step1", []byte{}, "compensate1", []byte{}).
		Step("step2", "step2", []byte{}, "compensate2", []byte{}).
		Step("step3", "step3", []byte{}, "compensate3", []byte{})
	// poll を高速化するため interval を短く取る。
	cfg := k1s0.SagaConfig{PollInterval: 10 * time.Millisecond, StepTimeout: 1 * time.Second}
	// Saga を実行する。
	result, err := saga.Execute(context.Background(), cfg)
	// 期待: error 無し、3 Step 全完了、補償発火なし。
	if err != nil {
		t.Fatalf("Execute returned error: %v", err)
	}
	// 完了 Step 数。
	if got := len(result.CompletedSteps); got != 3 {
		t.Fatalf("CompletedSteps len = %d, want 3", got)
	}
	// 補償発火が無いこと。
	if got := len(result.CompensatedSteps); got != 0 {
		t.Fatalf("CompensatedSteps len = %d, want 0 (no failure means no compensation)", got)
	}
	// 補償エラーも無いこと。
	if got := len(result.CompensationErrors); got != 0 {
		t.Fatalf("CompensationErrors len = %d, want 0", got)
	}
}

// TestSagaExecute_MiddleStepFailed_CompensatesInReverse は中央 Step が FAILED になる
// 経路を検証する。FR-T1-WORKFLOW-003 受け入れ基準: 「実行済み Activity の補償が逆順で
// 自動実行される」。
func TestSagaExecute_MiddleStepFailed_CompensatesInReverse(t *testing.T) {
	// step2 を FAILED に倒す fakeServer を準備する。
	fake := &fakeSagaWorkflowServer{
		statusByType: map[string]workflowv1.WorkflowStatus{
			"step1": workflowv1.WorkflowStatus_COMPLETED,
			// step2 で FAILED 帰着 → 補償経路へ。
			"step2": workflowv1.WorkflowStatus_FAILED,
		},
	}
	// gRPC server を起動する。
	srv, lis := startSagaServer(t, fake)
	// テスト終了時に server を停止する。
	defer srv.Stop()
	// k1s0.Client を作成する。
	c := makeSagaClient(t, lis)
	// テスト終了時に Client を閉じる。
	defer c.Close()
	// 3 Step Saga を chain する (step3 は実行されない)。
	saga := c.Workflow().NewSaga().
		Step("step1", "step1", []byte{}, "compensate1", []byte{}).
		Step("step2", "step2", []byte{}, "compensate2", []byte{}).
		Step("step3", "step3", []byte{}, "compensate3", []byte{})
	// poll を高速化する。
	cfg := k1s0.SagaConfig{PollInterval: 10 * time.Millisecond, StepTimeout: 1 * time.Second}
	// Saga を実行する。
	result, err := saga.Execute(context.Background(), cfg)
	// 期待: error あり (step2 FAILED)、補償は step2 → step1 の逆順発火。
	if err == nil {
		t.Fatalf("Execute should return error when middle step FAILED")
	}
	// 完了 Step は step1 のみ。
	if got := len(result.CompletedSteps); got != 1 {
		t.Fatalf("CompletedSteps len = %d, want 1 (only step1)", got)
	}
	// 補償は step2 → step1 の 2 件 (step3 は未実行のため対象外)。
	if got := len(result.CompensatedSteps); got != 2 {
		t.Fatalf("CompensatedSteps len = %d, want 2 (step2, step1)", got)
	}
	// 補償順序が逆順 (step2 が先、step1 が後)。
	if result.CompensatedSteps[0] != "step2" || result.CompensatedSteps[1] != "step1" {
		t.Fatalf("CompensatedSteps order = %v, want [step2 step1]", result.CompensatedSteps)
	}
	// fakeServer 側で記録された Start 順序を検証する (順方向 step1/step2 → 補償 compensate2/compensate1)。
	fake.mu.Lock()
	// 期待: step1 → step2 → compensate2 → compensate1 の 4 件。
	wantOrder := []string{"step1", "step2", "compensate2", "compensate1"}
	// 長さ一致を確認する。
	if got := len(fake.startedTypes); got != len(wantOrder) {
		t.Fatalf("server received %d Start calls, want %d (got=%v)", got, len(wantOrder), fake.startedTypes)
	}
	// 各順序を検証する。
	for i, want := range wantOrder {
		// 順序一致を確認する。
		if fake.startedTypes[i] != want {
			t.Fatalf("Start[%d] = %q, want %q (full=%v)", i, fake.startedTypes[i], want, fake.startedTypes)
		}
	}
	// state ロックを解除する。
	fake.mu.Unlock()
}

// TestSagaExecute_StepWithoutCompensationSkipped は CompensationWorkflowType="" の
// Step が補償発火対象外であることを検証する。
func TestSagaExecute_StepWithoutCompensationSkipped(t *testing.T) {
	// step2 を FAILED に倒す。
	fake := &fakeSagaWorkflowServer{
		statusByType: map[string]workflowv1.WorkflowStatus{
			"readonly": workflowv1.WorkflowStatus_COMPLETED,
			"step2":    workflowv1.WorkflowStatus_FAILED,
		},
	}
	// gRPC server を起動する。
	srv, lis := startSagaServer(t, fake)
	// テスト終了時に server を停止する。
	defer srv.Stop()
	// k1s0.Client を作成する。
	c := makeSagaClient(t, lis)
	// テスト終了時に Client を閉じる。
	defer c.Close()
	// 1 つ目の Step は補償スキップ (副作用なし read-only step を想定)。
	saga := c.Workflow().NewSaga().
		Step("readonly", "readonly", []byte{}, "", nil).
		Step("step2", "step2", []byte{}, "compensate2", []byte{})
	// poll を高速化する。
	cfg := k1s0.SagaConfig{PollInterval: 10 * time.Millisecond, StepTimeout: 1 * time.Second}
	// Saga を実行する。
	_, _ = saga.Execute(context.Background(), cfg)
	// fakeServer 側で記録された Start 順序: readonly → step2 → compensate2 (readonly の補償はスキップ)。
	fake.mu.Lock()
	// 期待 3 件 (readonly / step2 / compensate2)。
	if got := len(fake.startedTypes); got != 3 {
		t.Fatalf("server received %d Start calls, want 3 (readonly/step2/compensate2)", got)
	}
	// readonly に対する補償は呼ばれてはならない (CompensationWorkflowType="")。
	for _, name := range fake.startedTypes {
		// 補償名が呼ばれていないことを確認する。
		if name == "compensate-readonly" {
			t.Fatalf("compensate-readonly must not be invoked (CompensationWorkflowType=\"\" should skip)")
		}
	}
	// state ロックを解除する。
	fake.mu.Unlock()
}
