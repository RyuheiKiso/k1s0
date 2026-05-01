// 本ファイルは WorkflowService handler の単体テスト + gRPC 結線テスト。

package workflow

import (
	"context"
	"errors"
	"net"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/temporal"
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	workflowv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/workflow/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"
)

const bufSize = 1024 * 1024

// makeTenantCtx は WorkflowService テストで TenantContext を組み立てる helper。
// NFR-E-AC-003 で tenant_id 必須化されたためすべての RPC で渡す。
func makeTenantCtx(tenantID string) *commonv1.TenantContext {
	return &commonv1.TenantContext{TenantId: tenantID}
}

// fakeWorkflowAdapter は temporal.WorkflowAdapter の最小 fake 実装。
type fakeWorkflowAdapter struct {
	startFn     func(ctx context.Context, req temporal.StartRequest) (temporal.StartResponse, error)
	signalFn    func(ctx context.Context, req temporal.SignalRequest) error
	queryFn     func(ctx context.Context, req temporal.QueryRequest) (temporal.QueryResponse, error)
	cancelFn    func(ctx context.Context, req temporal.CancelRequest) error
	terminateFn func(ctx context.Context, req temporal.TerminateRequest) error
	statusFn    func(ctx context.Context, req temporal.GetStatusRequest) (temporal.GetStatusResponse, error)
}

func (f *fakeWorkflowAdapter) Start(ctx context.Context, req temporal.StartRequest) (temporal.StartResponse, error) {
	return f.startFn(ctx, req)
}
func (f *fakeWorkflowAdapter) Signal(ctx context.Context, req temporal.SignalRequest) error {
	return f.signalFn(ctx, req)
}
func (f *fakeWorkflowAdapter) Query(ctx context.Context, req temporal.QueryRequest) (temporal.QueryResponse, error) {
	return f.queryFn(ctx, req)
}
func (f *fakeWorkflowAdapter) Cancel(ctx context.Context, req temporal.CancelRequest) error {
	return f.cancelFn(ctx, req)
}
func (f *fakeWorkflowAdapter) Terminate(ctx context.Context, req temporal.TerminateRequest) error {
	return f.terminateFn(ctx, req)
}
func (f *fakeWorkflowAdapter) GetStatus(ctx context.Context, req temporal.GetStatusRequest) (temporal.GetStatusResponse, error) {
	return f.statusFn(ctx, req)
}

// Start: 正常系。
func TestWorkflowHandler_Start_OK(t *testing.T) {
	a := &fakeWorkflowAdapter{
		startFn: func(_ context.Context, req temporal.StartRequest) (temporal.StartResponse, error) {
			if req.WorkflowType != "ProcessOrder" {
				t.Fatalf("workflow type mismatch: %s", req.WorkflowType)
			}
			return temporal.StartResponse{WorkflowID: "wf-1", RunID: "run-1"}, nil
		},
	}
	h := &workflowHandler{deps: Deps{WorkflowAdapter: a}}
	resp, err := h.Start(context.Background(), &workflowv1.StartRequest{
		WorkflowType: "ProcessOrder",
		WorkflowId:   "wf-1",
		Input:        []byte("payload"),
		Context:      makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Start error: %v", err)
	}
	if resp.GetWorkflowId() != "wf-1" || resp.GetRunId() != "run-1" {
		t.Fatalf("response mismatch: %+v", resp)
	}
}

// NFR-E-AC-003: tenant_id 未設定時に InvalidArgument を返す。
// adapter 呼出前に短絡するため Deps{} で OK（adapter は呼ばれない）。
func TestWorkflowHandler_Start_RequiresTenant(t *testing.T) {
	h := &workflowHandler{deps: Deps{}}
	_, err := h.Start(context.Background(), &workflowv1.StartRequest{WorkflowType: "X"})
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("expected InvalidArgument for missing tenant, got %v", got)
	}
}

// Signal: SDK 経由で payload が伝搬する。
func TestWorkflowHandler_Signal_OK(t *testing.T) {
	a := &fakeWorkflowAdapter{
		signalFn: func(_ context.Context, req temporal.SignalRequest) error {
			if req.SignalName != "approve" {
				t.Fatalf("signal name mismatch: %s", req.SignalName)
			}
			return nil
		},
	}
	h := &workflowHandler{deps: Deps{WorkflowAdapter: a}}
	if _, err := h.Signal(context.Background(), &workflowv1.SignalRequest{
		WorkflowId: "wf-1",
		SignalName: "approve",
		Context:    makeTenantCtx("T"),
	}); err != nil {
		t.Fatalf("Signal error: %v", err)
	}
}

// Query: 結果バイト列が proto 応答に詰められる。
func TestWorkflowHandler_Query_OK(t *testing.T) {
	a := &fakeWorkflowAdapter{
		queryFn: func(_ context.Context, _ temporal.QueryRequest) (temporal.QueryResponse, error) {
			return temporal.QueryResponse{Result: []byte(`{"current":"in_progress"}`)}, nil
		},
	}
	h := &workflowHandler{deps: Deps{WorkflowAdapter: a}}
	resp, err := h.Query(context.Background(), &workflowv1.QueryRequest{WorkflowId: "x", QueryName: "status", Context: makeTenantCtx("T")})
	if err != nil {
		t.Fatalf("Query error: %v", err)
	}
	if string(resp.GetResult()) != `{"current":"in_progress"}` {
		t.Fatalf("result mismatch: %s", resp.GetResult())
	}
}

// Cancel: 正常系。
func TestWorkflowHandler_Cancel_OK(t *testing.T) {
	a := &fakeWorkflowAdapter{
		cancelFn: func(_ context.Context, _ temporal.CancelRequest) error { return nil },
	}
	h := &workflowHandler{deps: Deps{WorkflowAdapter: a}}
	if _, err := h.Cancel(context.Background(), &workflowv1.CancelRequest{WorkflowId: "x", Context: makeTenantCtx("T")}); err != nil {
		t.Fatalf("Cancel error: %v", err)
	}
}

// Terminate: Reason が adapter に伝搬する。
func TestWorkflowHandler_Terminate_OK(t *testing.T) {
	a := &fakeWorkflowAdapter{
		terminateFn: func(_ context.Context, req temporal.TerminateRequest) error {
			if req.Reason != "fraud-detected" {
				t.Fatalf("reason mismatch: %s", req.Reason)
			}
			return nil
		},
	}
	h := &workflowHandler{deps: Deps{WorkflowAdapter: a}}
	if _, err := h.Terminate(context.Background(), &workflowv1.TerminateRequest{
		WorkflowId: "x",
		Reason:     "fraud-detected",
		Context:    makeTenantCtx("T"),
	}); err != nil {
		t.Fatalf("Terminate error: %v", err)
	}
}

// GetStatus: Temporal status → proto WorkflowStatus 翻訳。
func TestWorkflowHandler_GetStatus_OK(t *testing.T) {
	a := &fakeWorkflowAdapter{
		statusFn: func(_ context.Context, _ temporal.GetStatusRequest) (temporal.GetStatusResponse, error) {
			return temporal.GetStatusResponse{Status: temporal.WorkflowStatusCompleted, RunID: "run-1"}, nil
		},
	}
	h := &workflowHandler{deps: Deps{WorkflowAdapter: a}}
	resp, err := h.GetStatus(context.Background(), &workflowv1.GetStatusRequest{WorkflowId: "x", Context: makeTenantCtx("T")})
	if err != nil {
		t.Fatalf("GetStatus error: %v", err)
	}
	if resp.GetStatus() != workflowv1.WorkflowStatus_COMPLETED {
		t.Fatalf("status mismatch: %v", resp.GetStatus())
	}
	if resp.GetRunId() != "run-1" {
		t.Fatalf("run_id mismatch: %s", resp.GetRunId())
	}
}

// adapter エラーが Internal に翻訳される。
func TestWorkflowHandler_Start_AdapterError(t *testing.T) {
	a := &fakeWorkflowAdapter{
		startFn: func(_ context.Context, _ temporal.StartRequest) (temporal.StartResponse, error) {
			return temporal.StartResponse{}, errors.New("connect refused")
		},
	}
	h := &workflowHandler{deps: Deps{WorkflowAdapter: a}}
	_, err := h.Start(context.Background(), &workflowv1.StartRequest{WorkflowType: "X", Context: makeTenantCtx("T")})
	if got := status.Code(err); got != codes.Internal {
		t.Fatalf("status: got %v want Internal", got)
	}
}

// in-process gRPC で Start round-trip が動くことを検証する。
func TestWorkflowService_Start_OverGRPC(t *testing.T) {
	lis := bufconn.Listen(bufSize)
	a := &fakeWorkflowAdapter{
		startFn: func(_ context.Context, _ temporal.StartRequest) (temporal.StartResponse, error) {
			return temporal.StartResponse{WorkflowID: "wf-X", RunID: "run-X"}, nil
		},
	}
	srv := grpc.NewServer()
	Register(Deps{WorkflowAdapter: a})(srv)
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
	client := workflowv1.NewWorkflowServiceClient(conn)
	resp, err := client.Start(context.Background(), &workflowv1.StartRequest{
		WorkflowType: "Test",
		WorkflowId:   "wf-test",
		Context:      makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Start over gRPC: %v", err)
	}
	if resp.GetWorkflowId() != "wf-X" || resp.GetRunId() != "run-X" {
		t.Fatalf("response: %+v", resp)
	}
}
