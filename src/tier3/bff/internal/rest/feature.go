// Feature の REST エンドポイント。
//
//	POST /api/feature/evaluate-boolean — Boolean 型 Feature Flag 評価

package rest

// 標準 import。
import (
	// HTTP server。
	"net/http"
)

// featureEvaluateBooleanRequest は POST /api/feature/evaluate-boolean の入力。
// eval_ctx は OpenFeature の Evaluation Context 相当（テナント / ユーザ属性等）。
type featureEvaluateBooleanRequest struct {
	FlagKey string            `json:"flag_key"`
	EvalCtx map[string]string `json:"eval_ctx,omitempty"`
}

// featureEvaluateBooleanResponse は POST /api/feature/evaluate-boolean の出力。
type featureEvaluateBooleanResponse struct {
	Value   bool   `json:"value"`
	Variant string `json:"variant"`
	Reason  string `json:"reason"`
}

// registerFeature は feature 系 endpoint を mux に登録する。
func (r *Router) registerFeature(mux *http.ServeMux) {
	// Feature.EvaluateBoolean。
	mux.HandleFunc("POST /api/feature/evaluate-boolean", r.handleFeatureEvaluateBoolean)
}

// handleFeatureEvaluateBoolean は POST /api/feature/evaluate-boolean を処理する。
func (r *Router) handleFeatureEvaluateBoolean(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body featureEvaluateBooleanRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する。
	if body.FlagKey == "" {
		writeBadRequest(w, "E-T3-BFF-FEATURE-100", "flag_key is required")
		return
	}
	// facade 経由で SDK を呼ぶ。
	value, variant, reason, err := r.facade.FeatureEvaluateBoolean(req.Context(), body.FlagKey, body.EvalCtx)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-FEATURE-200", "feature evaluate failed: "+err.Error())
		return
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, featureEvaluateBooleanResponse{
		Value:   value,
		Variant: variant,
		Reason:  reason,
	})
}
