// Decision の REST エンドポイント。
//
//	POST /api/decision/evaluate — JDM ルール評価

package rest

// 標準 import。
import (
	// HTTP server。
	"net/http"
)

// decisionEvaluateRequest は POST /api/decision/evaluate の入力。
type decisionEvaluateRequest struct {
	RuleID       string `json:"rule_id"`
	RuleVersion  string `json:"rule_version,omitempty"`
	InputJSON    string `json:"input_json"`
	IncludeTrace bool   `json:"include_trace,omitempty"`
}

// decisionEvaluateResponse は POST /api/decision/evaluate の出力。
type decisionEvaluateResponse struct {
	OutputJSON string `json:"output_json"`
	TraceJSON  string `json:"trace_json,omitempty"`
	ElapsedUs  int64  `json:"elapsed_us"`
}

// registerDecision は decision 系 endpoint を mux に登録する。
func (r *Router) registerDecision(mux *http.ServeMux) {
	// Decision.Evaluate。
	mux.HandleFunc("POST /api/decision/evaluate", r.handleDecisionEvaluate)
}

// handleDecisionEvaluate は POST /api/decision/evaluate を処理する。
func (r *Router) handleDecisionEvaluate(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body decisionEvaluateRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する。
	if body.RuleID == "" || body.InputJSON == "" {
		writeBadRequest(w, "E-T3-BFF-DECISION-100", "rule_id and input_json are required")
		return
	}
	// facade 経由で SDK を呼ぶ。
	out, trace, elapsed, err := r.facade.DecisionEvaluate(req.Context(), body.RuleID, body.RuleVersion, []byte(body.InputJSON), body.IncludeTrace)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-DECISION-200", "decision evaluate failed: "+err.Error())
		return
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, decisionEvaluateResponse{
		OutputJSON: string(out),
		TraceJSON:  string(trace),
		ElapsedUs:  elapsed,
	})
}
