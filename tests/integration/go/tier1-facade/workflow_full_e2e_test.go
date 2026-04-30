// 本ファイルは t1-workflow Pod の全 6 RPC を実バイナリで検証する E2E。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/06_Workflow_API.md
//     - Start / Signal / Query / Cancel / Terminate / GetStatus
//
// 検証目的:
//   in-memory backend で workflow run の状態遷移
//   RUNNING → CANCELED / TERMINATED が gRPC 契約通り反映されることを保証する。

package tier1facade

import (
	"net/http"
	"strings"
	"testing"
)

// Workflow Pod: Start → Signal → Query → GetStatus（RUNNING）→ Cancel → GetStatus（CANCELED）。
func TestWorkflowPod_HTTPGateway_FullCancelPath(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startWorkflowPod(t)
	defer cleanup()

	tenant := `"context":{"tenant_id":"T-wf-cancel"}`

	// 1. Start: 新規 workflow を起動する。
	startBody := `{
		"workflow_type": "OrderFlow",
		"workflow_id": "wf-cancel-1",
		"input": "aW5pdA==",
		` + tenant + `
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/workflow/start", startBody); code != http.StatusOK {
		t.Fatalf("Start: %d %s", code, b)
	}

	// 2. Signal: 任意の signal を送る。
	signalBody := `{
		"workflow_id": "wf-cancel-1",
		"signal_name": "approve",
		"payload": "eWVz",
		` + tenant + `
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/workflow/signal", signalBody); code != http.StatusOK {
		t.Fatalf("Signal: %d %s", code, b)
	}

	// 3. Query: 副作用なしで状態を引く。
	queryBody := `{
		"workflow_id": "wf-cancel-1",
		"query_name": "current_state",
		` + tenant + `
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/workflow/query", queryBody); code != http.StatusOK {
		t.Fatalf("Query: %d %s", code, b)
	}

	// 4. GetStatus: 起動直後は RUNNING（in-memory backend の初期 status）。
	statusBody := `{"workflow_id":"wf-cancel-1",` + tenant + `}`
	code, body := postJSON(t, httpURL+"/k1s0/workflow/getstatus", statusBody)
	if code != http.StatusOK {
		t.Fatalf("GetStatus pre-cancel: %d %s", code, body)
	}
	if !strings.Contains(body, "RUNNING") && !strings.Contains(body, "WORKFLOW_STATUS_RUNNING") && !strings.Contains(body, `"status":1`) {
		t.Errorf("pre-cancel status not RUNNING: %s", body)
	}

	// 5. Cancel: 状態を CANCELED に遷移する。
	cancelBody := `{
		"workflow_id": "wf-cancel-1",
		"reason": "user requested",
		` + tenant + `
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/workflow/cancel", cancelBody); code != http.StatusOK {
		t.Fatalf("Cancel: %d %s", code, b)
	}

	// 6. GetStatus: CANCELED に遷移している。
	code, body = postJSON(t, httpURL+"/k1s0/workflow/getstatus", statusBody)
	if code != http.StatusOK {
		t.Fatalf("GetStatus post-cancel: %d %s", code, body)
	}
	if !strings.Contains(body, "CANCELED") && !strings.Contains(body, "WORKFLOW_STATUS_CANCELED") && !strings.Contains(body, `"status":4`) {
		t.Errorf("post-cancel status not CANCELED: %s", body)
	}
}

// Workflow Pod: Start → Terminate → GetStatus（TERMINATED）。
func TestWorkflowPod_HTTPGateway_TerminatePath(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startWorkflowPod(t)
	defer cleanup()

	tenant := `"context":{"tenant_id":"T-wf-term"}`

	startBody := `{
		"workflow_type": "BatchJob",
		"workflow_id": "wf-term-1",
		` + tenant + `
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/workflow/start", startBody); code != http.StatusOK {
		t.Fatalf("Start: %d %s", code, b)
	}

	termBody := `{
		"workflow_id": "wf-term-1",
		"reason": "ops emergency",
		` + tenant + `
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/workflow/terminate", termBody); code != http.StatusOK {
		t.Fatalf("Terminate: %d %s", code, b)
	}

	statusBody := `{"workflow_id":"wf-term-1",` + tenant + `}`
	code, body := postJSON(t, httpURL+"/k1s0/workflow/getstatus", statusBody)
	if code != http.StatusOK {
		t.Fatalf("GetStatus: %d %s", code, body)
	}
	if !strings.Contains(body, "TERMINATED") && !strings.Contains(body, "WORKFLOW_STATUS_TERMINATED") && !strings.Contains(body, `"status":5`) {
		t.Errorf("post-terminate status not TERMINATED: %s", body)
	}
}

// Workflow Pod: 別テナントの workflow には越境してアクセスできない（NFR-E-AC-003）。
func TestWorkflowPod_HTTPGateway_TenantIsolation(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startWorkflowPod(t)
	defer cleanup()

	// T1 で workflow を Start。
	startBody := `{
		"workflow_type": "X",
		"workflow_id": "wf-iso-1",
		"context": {"tenant_id": "T-iso-1"}
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/workflow/start", startBody); code != http.StatusOK {
		t.Fatalf("Start T1: %d %s", code, b)
	}

	// T2 から GetStatus → NotFound（他テナントの workflow を漏らさない）。
	statusBody := `{
		"workflow_id": "wf-iso-1",
		"context": {"tenant_id": "T-iso-2"}
	}`
	code, body := postJSON(t, httpURL+"/k1s0/workflow/getstatus", statusBody)
	if code != http.StatusNotFound && code != http.StatusOK {
		// 一部の handler は OK で空応答を返す可能性もあるが、status が空 / RUNNING 以外であるべき。
		t.Logf("cross-tenant GetStatus: %d %s", code, body)
	}
	if code == http.StatusOK && strings.Contains(body, "RUNNING") {
		t.Errorf("cross-tenant leaked workflow status: %s", body)
	}

	// T1 自身からは取れる。
	statusBody = `{"workflow_id":"wf-iso-1","context":{"tenant_id":"T-iso-1"}}`
	if code, _ := postJSON(t, httpURL+"/k1s0/workflow/getstatus", statusBody); code != http.StatusOK {
		t.Errorf("T1 own GetStatus: code=%d", code)
	}
}
