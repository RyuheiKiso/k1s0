// 本ファイルは Dapr Workflow production adapter の単体テスト。
//
// 検証対象:
//   - Start: SDK の StartWorkflowBeta1 に正しく Component / WorkflowName / Options を渡す
//   - Signal: SDK の RaiseEventWorkflowBeta1 に正しく EventName / EventData を渡す
//   - Cancel: SDK の PauseWorkflowBeta1 にマップされる
//   - Terminate: SDK の TerminateWorkflowBeta1 にマップされる
//   - GetStatus: Runtime status 文字列が WorkflowStatusValue に正しく翻訳される
//   - Query: Dapr Beta1 では非対応のため明示エラー
//   - エラー翻訳: SDK が "not found" を含むエラーを返す → ErrNotFound

package daprwf

import (
	// 全 RPC で context を伝搬する。
	"context"
	// errors.Is で sentinel 判定。
	"errors"
	// テスト fixture のフォーマット。
	"fmt"
	// テスト本体。
	"testing"

	// Dapr SDK の workflow 型。
	daprclient "github.com/dapr/go-sdk/client"
)

// fakeWorkflowClient は daprWorkflowClient を満たす fake。各メソッドの呼出引数を記録する。
type fakeWorkflowClient struct {
	// Start の応答制御。
	startResp *daprclient.StartWorkflowResponse
	startErr  error
	// Get の応答制御。
	getResp *daprclient.GetWorkflowResponse
	getErr  error
	// Terminate / Pause / Resume / RaiseEvent のエラー制御。
	terminateErr  error
	raiseEventErr error
	pauseErr      error
	resumeErr     error
	// 最後に渡されたリクエスト（assertion 用）。
	lastStart      *daprclient.StartWorkflowRequest
	lastGet        *daprclient.GetWorkflowRequest
	lastTerminate  *daprclient.TerminateWorkflowRequest
	lastRaiseEvent *daprclient.RaiseEventWorkflowRequest
	lastPause      *daprclient.PauseWorkflowRequest
}

func (f *fakeWorkflowClient) StartWorkflowBeta1(_ context.Context, req *daprclient.StartWorkflowRequest) (*daprclient.StartWorkflowResponse, error) {
	f.lastStart = req
	return f.startResp, f.startErr
}

func (f *fakeWorkflowClient) GetWorkflowBeta1(_ context.Context, req *daprclient.GetWorkflowRequest) (*daprclient.GetWorkflowResponse, error) {
	f.lastGet = req
	return f.getResp, f.getErr
}

func (f *fakeWorkflowClient) TerminateWorkflowBeta1(_ context.Context, req *daprclient.TerminateWorkflowRequest) error {
	f.lastTerminate = req
	return f.terminateErr
}

func (f *fakeWorkflowClient) RaiseEventWorkflowBeta1(_ context.Context, req *daprclient.RaiseEventWorkflowRequest) error {
	f.lastRaiseEvent = req
	return f.raiseEventErr
}

func (f *fakeWorkflowClient) PauseWorkflowBeta1(_ context.Context, req *daprclient.PauseWorkflowRequest) error {
	f.lastPause = req
	return f.pauseErr
}

func (f *fakeWorkflowClient) ResumeWorkflowBeta1(_ context.Context, _ *daprclient.ResumeWorkflowRequest) error {
	return f.resumeErr
}

// TestProductionStart_PassesOptionsAndComponent は Start で SDK に正しい Options / Component が渡ることを確認する。
// L2 分離（NFR-E-AC-003）: 物理 InstanceID は "<tenant>::<workflow_id>" に scope され、応答は raw に戻る。
func TestProductionStart_PassesOptionsAndComponent(t *testing.T) {
	// SDK 応答も scope された ID を返す前提（production の Dapr Workflow 経路と等価）。
	fake := &fakeWorkflowClient{
		startResp: &daprclient.StartWorkflowResponse{InstanceID: "tenant-a::wf-1"},
	}
	a := NewProduction(fake, "")
	resp, err := a.Start(context.Background(), StartRequest{
		WorkflowType: "OnboardWorkflow",
		WorkflowID:   "wf-1",
		Input:        []byte("payload"),
		Idempotent:   true,
		TenantID:     "tenant-a",
	})
	if err != nil {
		t.Fatalf("Start error: %v", err)
	}
	// 応答は raw（"wf-1"）に戻す。tier2/tier3 視点では prefix が見えない。
	if resp.WorkflowID != "wf-1" || resp.RunID != "wf-1" {
		t.Errorf("response (should be unscoped): got %+v", resp)
	}
	// 物理 SDK 呼出 InstanceID は scope 済（"tenant-a::wf-1"）。
	if fake.lastStart.InstanceID != "tenant-a::wf-1" {
		t.Errorf("physical InstanceID (should be scoped): got %q want %q", fake.lastStart.InstanceID, "tenant-a::wf-1")
	}
	if fake.lastStart.WorkflowComponent != defaultWorkflowComponent {
		t.Errorf("component: got %q want %q", fake.lastStart.WorkflowComponent, defaultWorkflowComponent)
	}
	if fake.lastStart.WorkflowName != "OnboardWorkflow" {
		t.Errorf("WorkflowName: got %q", fake.lastStart.WorkflowName)
	}
	if fake.lastStart.Options["tenant_id"] != "tenant-a" {
		t.Errorf("tenant_id options: got %q", fake.lastStart.Options["tenant_id"])
	}
	if fake.lastStart.Options["idempotent"] != "true" {
		t.Errorf("idempotent options: got %q", fake.lastStart.Options["idempotent"])
	}
	if !fake.lastStart.SendRawInput {
		t.Errorf("SendRawInput should be true")
	}
}

// TestProductionStart_OmitsEmptyOptions は idempotent=false / TenantID 空時に options がクリーンに空であることを確認する。
func TestProductionStart_OmitsEmptyOptions(t *testing.T) {
	fake := &fakeWorkflowClient{
		startResp: &daprclient.StartWorkflowResponse{InstanceID: "wf-2"},
	}
	a := NewProduction(fake, "custom-comp")
	if _, err := a.Start(context.Background(), StartRequest{WorkflowType: "Foo"}); err != nil {
		t.Fatalf("Start error: %v", err)
	}
	if fake.lastStart.WorkflowComponent != "custom-comp" {
		t.Errorf("component override: got %q", fake.lastStart.WorkflowComponent)
	}
	if _, ok := fake.lastStart.Options["tenant_id"]; ok {
		t.Errorf("tenant_id should be absent when empty")
	}
	if _, ok := fake.lastStart.Options["idempotent"]; ok {
		t.Errorf("idempotent should be absent when false")
	}
}

// TestProductionSignal_MapsToRaiseEvent は Signal が RaiseEventWorkflowBeta1 に変換されることを確認する。
func TestProductionSignal_MapsToRaiseEvent(t *testing.T) {
	fake := &fakeWorkflowClient{}
	a := NewProduction(fake, "")
	if err := a.Signal(context.Background(), SignalRequest{
		WorkflowID: "wf-1",
		SignalName: "approval-received",
		Payload:    []byte("approved"),
		TenantID:   "tenant-a",
	}); err != nil {
		t.Fatalf("Signal error: %v", err)
	}
	// L2 分離: 物理 InstanceID は scope 済。
	if fake.lastRaiseEvent.InstanceID != "tenant-a::wf-1" {
		t.Errorf("InstanceID (should be scoped): got %q want %q", fake.lastRaiseEvent.InstanceID, "tenant-a::wf-1")
	}
	if fake.lastRaiseEvent.EventName != "approval-received" {
		t.Errorf("EventName: got %q", fake.lastRaiseEvent.EventName)
	}
	if !fake.lastRaiseEvent.SendRawData {
		t.Errorf("SendRawData should be true")
	}
}

// TestProductionCancel_MapsToPause は Cancel が PauseWorkflowBeta1 に変換されることを確認する。
// L2 分離: TenantID 指定時は物理 InstanceID が scope 済になる。
func TestProductionCancel_MapsToPause(t *testing.T) {
	fake := &fakeWorkflowClient{}
	a := NewProduction(fake, "")
	if err := a.Cancel(context.Background(), CancelRequest{WorkflowID: "wf-1", Reason: "user-cancel", TenantID: "tenant-a"}); err != nil {
		t.Fatalf("Cancel error: %v", err)
	}
	if fake.lastPause == nil || fake.lastPause.InstanceID != "tenant-a::wf-1" {
		t.Errorf("Pause not invoked / not scoped: %+v", fake.lastPause)
	}
}

// TestProductionTerminate_MapsToTerminate は Terminate が TerminateWorkflowBeta1 に変換されることを確認する。
// L2 分離: TenantID 指定時は物理 InstanceID が scope 済になる。
func TestProductionTerminate_MapsToTerminate(t *testing.T) {
	fake := &fakeWorkflowClient{}
	a := NewProduction(fake, "")
	if err := a.Terminate(context.Background(), TerminateRequest{WorkflowID: "wf-1", TenantID: "tenant-a"}); err != nil {
		t.Fatalf("Terminate error: %v", err)
	}
	if fake.lastTerminate == nil || fake.lastTerminate.InstanceID != "tenant-a::wf-1" {
		t.Errorf("Terminate not invoked / not scoped: %+v", fake.lastTerminate)
	}
}

// TestProductionGetStatus_TranslatesRuntimeStatus は runtime_status 文字列が WorkflowStatusValue に翻訳されることを確認する。
func TestProductionGetStatus_TranslatesRuntimeStatus(t *testing.T) {
	cases := []struct {
		runtime  string
		expected WorkflowStatusValue
	}{
		{"RUNNING", StatusRunning},
		{"PENDING", StatusRunning},
		{"CONTINUED_AS_NEW", StatusRunning},
		{"SUSPENDED", StatusRunning},
		{"COMPLETED", StatusCompleted},
		{"FAILED", StatusFailed},
		{"CANCELED", StatusCanceled},
		{"TERMINATED", StatusTerminated},
		{"running", StatusRunning},
		{"unknown-string", StatusRunning},
	}
	for _, c := range cases {
		t.Run(c.runtime, func(t *testing.T) {
			fake := &fakeWorkflowClient{
				getResp: &daprclient.GetWorkflowResponse{
					InstanceID:    "wf-x",
					RuntimeStatus: c.runtime,
				},
			}
			a := NewProduction(fake, "")
			r, err := a.GetStatus(context.Background(), GetStatusRequest{WorkflowID: "wf-x"})
			if err != nil {
				t.Fatalf("GetStatus error: %v", err)
			}
			if r.Status != c.expected {
				t.Errorf("status %q: got %v want %v", c.runtime, r.Status, c.expected)
			}
		})
	}
}

// TestProductionQuery_AlwaysUnimplemented は Query が常に explicit error を返すことを確認する。
func TestProductionQuery_AlwaysUnimplemented(t *testing.T) {
	a := NewProduction(&fakeWorkflowClient{}, "")
	if _, err := a.Query(context.Background(), QueryRequest{WorkflowID: "wf-1"}); err == nil {
		t.Fatalf("Query should return explicit error")
	}
}

// TestProductionTranslateNotFound は SDK が "not found" を含むエラーを ErrNotFound に翻訳することを確認する。
func TestProductionTranslateNotFound(t *testing.T) {
	fake := &fakeWorkflowClient{
		terminateErr: fmt.Errorf("instance not found: wf-bogus"),
	}
	a := NewProduction(fake, "")
	err := a.Terminate(context.Background(), TerminateRequest{WorkflowID: "wf-bogus"})
	if !errors.Is(err, ErrNotFound) {
		t.Errorf("translateNotFound: got %v want ErrNotFound", err)
	}
}

// TestProductionPropagatesGenericError は "not found" 以外のエラーがそのまま伝搬することを確認する。
func TestProductionPropagatesGenericError(t *testing.T) {
	fake := &fakeWorkflowClient{
		terminateErr: errors.New("connection refused"),
	}
	a := NewProduction(fake, "")
	err := a.Terminate(context.Background(), TerminateRequest{WorkflowID: "wf-1"})
	if err == nil || errors.Is(err, ErrNotFound) {
		t.Errorf("generic error should not become ErrNotFound: %v", err)
	}
}
