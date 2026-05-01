// Workflow の REST エンドポイント。
//
//	POST /api/workflow/start — ワークフロー開始

package rest

// 標準 import。
import (
	// HTTP server。
	"net/http"
)

// workflowStartRequest は POST /api/workflow/start の入力。
type workflowStartRequest struct {
	WorkflowType string `json:"workflow_type"`
	WorkflowID   string `json:"workflow_id"`
	Input        string `json:"input,omitempty"`
	Idempotent   bool   `json:"idempotent,omitempty"`
}

// workflowStartResponse は POST /api/workflow/start の出力。
type workflowStartResponse struct {
	WorkflowID string `json:"workflow_id"`
	RunID      string `json:"run_id"`
}

// registerWorkflow は workflow 系 endpoint を mux に登録する。
func (r *Router) registerWorkflow(mux *http.ServeMux) {
	// Workflow.Start。
	mux.HandleFunc("POST /api/workflow/start", r.handleWorkflowStart)
}

// handleWorkflowStart は POST /api/workflow/start を処理する。
func (r *Router) handleWorkflowStart(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body workflowStartRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する。
	if body.WorkflowType == "" || body.WorkflowID == "" {
		writeBadRequest(w, "E-T3-BFF-WORKFLOW-100", "workflow_type and workflow_id are required")
		return
	}
	// facade 経由で SDK を呼ぶ。
	wfID, runID, err := r.facade.WorkflowStart(req.Context(), body.WorkflowType, body.WorkflowID, []byte(body.Input), body.Idempotent)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-WORKFLOW-200", "workflow start failed: "+err.Error())
		return
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, workflowStartResponse{WorkflowID: wfID, RunID: runID})
}
