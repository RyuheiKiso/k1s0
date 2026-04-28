// 本ファイルは temporalWorkflowAdapter の単体テスト。
// Temporal SDK Client を fake で差し替え、adapter ロジックを検証する。

package temporal

import (
	"context"
	"errors"
	"testing"

	tclient "go.temporal.io/sdk/client"
	"go.temporal.io/sdk/converter"
)

// fakeTemporalClient は temporalClient interface の最小 fake 実装。
type fakeTemporalClient struct {
	executeFn   func(ctx context.Context, options tclient.StartWorkflowOptions, workflow interface{}, args ...interface{}) (tclient.WorkflowRun, error)
	signalFn    func(ctx context.Context, wfID, runID, signalName string, arg interface{}) error
	queryFn     func(ctx context.Context, wfID, runID, queryType string, args ...interface{}) (converter.EncodedValue, error)
	cancelFn    func(ctx context.Context, wfID, runID string) error
	terminateFn func(ctx context.Context, wfID, runID, reason string, details ...interface{}) error
	describeFn  func(ctx context.Context, wfID, runID string) (*describeResponse, error)
}

func (f *fakeTemporalClient) ExecuteWorkflow(ctx context.Context, options tclient.StartWorkflowOptions, workflow interface{}, args ...interface{}) (tclient.WorkflowRun, error) {
	return f.executeFn(ctx, options, workflow, args...)
}
func (f *fakeTemporalClient) SignalWorkflow(ctx context.Context, wfID, runID, signalName string, arg interface{}) error {
	return f.signalFn(ctx, wfID, runID, signalName, arg)
}
func (f *fakeTemporalClient) QueryWorkflow(ctx context.Context, wfID, runID, queryType string, args ...interface{}) (converter.EncodedValue, error) {
	return f.queryFn(ctx, wfID, runID, queryType, args...)
}
func (f *fakeTemporalClient) CancelWorkflow(ctx context.Context, wfID, runID string) error {
	return f.cancelFn(ctx, wfID, runID)
}
func (f *fakeTemporalClient) TerminateWorkflow(ctx context.Context, wfID, runID, reason string, details ...interface{}) error {
	return f.terminateFn(ctx, wfID, runID, reason, details...)
}
func (f *fakeTemporalClient) DescribeWorkflowExecution(ctx context.Context, wfID, runID string) (*describeResponse, error) {
	return f.describeFn(ctx, wfID, runID)
}

// fakeWorkflowRun は WorkflowRun interface の最小 fake 実装（GetID/GetRunID のみ使う）。
type fakeWorkflowRun struct {
	id    string
	runID string
}

func (f *fakeWorkflowRun) GetID() string                                { return f.id }
func (f *fakeWorkflowRun) GetRunID() string                             { return f.runID }
func (f *fakeWorkflowRun) Get(ctx context.Context, valuePtr interface{}) error { return nil }
func (f *fakeWorkflowRun) GetWithOptions(ctx context.Context, valuePtr interface{}, options tclient.WorkflowRunGetOptions) error {
	return nil
}

func newAdapterWithFake(t *testing.T, fake *fakeTemporalClient) WorkflowAdapter {
	t.Helper()
	return NewWorkflowAdapter(NewWithClient("test://noop", fake))
}

// Start: ExecuteWorkflow に WorkflowType / TaskQueue / Input が正しく渡ることを検証。
func TestWorkflowAdapter_Start_OK(t *testing.T) {
	fake := &fakeTemporalClient{
		executeFn: func(_ context.Context, options tclient.StartWorkflowOptions, workflow interface{}, args ...interface{}) (tclient.WorkflowRun, error) {
			if workflow.(string) != "ProcessOrder" {
				t.Fatalf("workflow type mismatch: %v", workflow)
			}
			if options.ID != "order-123" {
				t.Fatalf("workflow id mismatch: %s", options.ID)
			}
			if options.TaskQueue != "k1s0-default" {
				t.Fatalf("task queue should default to k1s0-default: %s", options.TaskQueue)
			}
			if len(args) != 1 || string(args[0].([]byte)) != `{"orderId":"X"}` {
				t.Fatalf("input not propagated: %v", args)
			}
			return &fakeWorkflowRun{id: "order-123", runID: "run-abc"}, nil
		},
	}
	a := newAdapterWithFake(t, fake)
	resp, err := a.Start(context.Background(), StartRequest{
		WorkflowType: "ProcessOrder",
		WorkflowID:   "order-123",
		Input:        []byte(`{"orderId":"X"}`),
	})
	if err != nil {
		t.Fatalf("Start error: %v", err)
	}
	if resp.WorkflowID != "order-123" || resp.RunID != "run-abc" {
		t.Fatalf("response mismatch: %+v", resp)
	}
}

// TaskQueue を明示指定した場合は default ではなく指定値が使われる。
func TestWorkflowAdapter_Start_CustomTaskQueue(t *testing.T) {
	fake := &fakeTemporalClient{
		executeFn: func(_ context.Context, options tclient.StartWorkflowOptions, _ interface{}, _ ...interface{}) (tclient.WorkflowRun, error) {
			if options.TaskQueue != "billing-queue" {
				t.Fatalf("task queue not propagated: %s", options.TaskQueue)
			}
			return &fakeWorkflowRun{id: "x", runID: "y"}, nil
		},
	}
	a := newAdapterWithFake(t, fake)
	if _, err := a.Start(context.Background(), StartRequest{WorkflowType: "X", WorkflowID: "x", TaskQueue: "billing-queue"}); err != nil {
		t.Fatalf("Start error: %v", err)
	}
}

// Signal: SDK へ正しい引数が渡ることを検証。
func TestWorkflowAdapter_Signal_OK(t *testing.T) {
	called := 0
	fake := &fakeTemporalClient{
		signalFn: func(_ context.Context, wfID, runID, name string, arg interface{}) error {
			called++
			if wfID != "wf-1" || name != "approve" {
				t.Fatalf("args mismatch: %s/%s/%s", wfID, runID, name)
			}
			if string(arg.([]byte)) != "approved" {
				t.Fatalf("payload mismatch: %v", arg)
			}
			return nil
		},
	}
	a := newAdapterWithFake(t, fake)
	if err := a.Signal(context.Background(), SignalRequest{
		WorkflowID: "wf-1",
		SignalName: "approve",
		Payload:    []byte("approved"),
	}); err != nil {
		t.Fatalf("Signal error: %v", err)
	}
	if called != 1 {
		t.Fatalf("SignalWorkflow called %d times", called)
	}
}

// Cancel: SDK Cancel が呼ばれることを検証。
func TestWorkflowAdapter_Cancel_OK(t *testing.T) {
	fake := &fakeTemporalClient{
		cancelFn: func(_ context.Context, wfID, _ string) error {
			if wfID != "wf-1" {
				t.Fatalf("wfID mismatch: %s", wfID)
			}
			return nil
		},
	}
	a := newAdapterWithFake(t, fake)
	if err := a.Cancel(context.Background(), CancelRequest{WorkflowID: "wf-1"}); err != nil {
		t.Fatalf("Cancel error: %v", err)
	}
}

// Terminate: Reason が SDK へ伝搬する。
func TestWorkflowAdapter_Terminate_OK(t *testing.T) {
	var observedReason string
	fake := &fakeTemporalClient{
		terminateFn: func(_ context.Context, _, _ string, reason string, _ ...interface{}) error {
			observedReason = reason
			return nil
		},
	}
	a := newAdapterWithFake(t, fake)
	if err := a.Terminate(context.Background(), TerminateRequest{WorkflowID: "x", Reason: "tenant-suspended"}); err != nil {
		t.Fatalf("Terminate error: %v", err)
	}
	if observedReason != "tenant-suspended" {
		t.Fatalf("reason not propagated: %s", observedReason)
	}
}

// GetStatus: Temporal status を proto status へ変換する翻訳テーブルを検証。
func TestWorkflowAdapter_GetStatus_Translates(t *testing.T) {
	cases := []struct {
		temporalCode int32
		want         WorkflowStatusValue
	}{
		{temporalStatusRunning, WorkflowStatusRunning},
		{temporalStatusCompleted, WorkflowStatusCompleted},
		{temporalStatusFailed, WorkflowStatusFailed},
		{temporalStatusCanceled, WorkflowStatusCanceled},
		{temporalStatusTerminated, WorkflowStatusTerminated},
		{temporalStatusContinuedAsNew, WorkflowStatusContinuedAsNew},
		{temporalStatusTimedOut, WorkflowStatusFailed}, // TimedOut → Failed に寄せる
	}
	for _, c := range cases {
		fake := &fakeTemporalClient{
			describeFn: func(_ context.Context, _, _ string) (*describeResponse, error) {
				return &describeResponse{StatusCode: c.temporalCode, RunID: "r"}, nil
			},
		}
		a := newAdapterWithFake(t, fake)
		resp, err := a.GetStatus(context.Background(), GetStatusRequest{WorkflowID: "x"})
		if err != nil {
			t.Fatalf("GetStatus error: %v", err)
		}
		if resp.Status != c.want {
			t.Errorf("status code %d: got %v want %v", c.temporalCode, resp.Status, c.want)
		}
	}
}

// SDK エラーが透過されることを検証。
func TestWorkflowAdapter_Start_SDKError(t *testing.T) {
	want := errors.New("temporal frontend down")
	fake := &fakeTemporalClient{
		executeFn: func(_ context.Context, _ tclient.StartWorkflowOptions, _ interface{}, _ ...interface{}) (tclient.WorkflowRun, error) {
			return nil, want
		},
	}
	a := newAdapterWithFake(t, fake)
	_, err := a.Start(context.Background(), StartRequest{WorkflowType: "X", WorkflowID: "x"})
	if !errors.Is(err, want) {
		t.Fatalf("error not transparent: %v", err)
	}
}

// Query: EncodedValue が nil の時に空 result を返す。
func TestWorkflowAdapter_Query_NoValue(t *testing.T) {
	fake := &fakeTemporalClient{
		queryFn: func(_ context.Context, _, _, _ string, _ ...interface{}) (converter.EncodedValue, error) {
			return nil, nil
		},
	}
	a := newAdapterWithFake(t, fake)
	resp, err := a.Query(context.Background(), QueryRequest{WorkflowID: "x", QueryName: "q"})
	if err != nil {
		t.Fatalf("Query error: %v", err)
	}
	if resp.Result != nil {
		t.Fatalf("expected nil result, got %v", resp.Result)
	}
}
