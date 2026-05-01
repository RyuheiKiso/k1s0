// PII の REST エンドポイント。
//
//	POST /api/pii/classify — PII 分類（マスクせず検出のみ）
//	POST /api/pii/mask     — PII マスク

package rest

// 標準 / 内部 import。
import (
	// HTTP server。
	"net/http"

	// k1s0client の PiiFindingSummary 型を参照する。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/k1s0client"
)

// piiClassifyRequest は POST /api/pii/classify の入力。
type piiClassifyRequest struct {
	Text string `json:"text"`
}

// piiFindingOut は応答 1 件分の PII 検出結果。
type piiFindingOut struct {
	Type       string  `json:"type"`
	Start      int32   `json:"start"`
	End        int32   `json:"end"`
	Confidence float64 `json:"confidence"`
}

// piiClassifyResponse は POST /api/pii/classify の出力。
type piiClassifyResponse struct {
	Findings    []piiFindingOut `json:"findings"`
	ContainsPii bool            `json:"contains_pii"`
}

// piiMaskRequest は POST /api/pii/mask の入力。
type piiMaskRequest struct {
	Text string `json:"text"`
}

// piiMaskResponse は POST /api/pii/mask の出力。
type piiMaskResponse struct {
	MaskedText string          `json:"masked_text"`
	Findings   []piiFindingOut `json:"findings"`
}

// registerPii は pii 系 endpoint を mux に登録する。
func (r *Router) registerPii(mux *http.ServeMux) {
	// PII.Classify。
	mux.HandleFunc("POST /api/pii/classify", r.handlePiiClassify)
	// PII.Mask。
	mux.HandleFunc("POST /api/pii/mask", r.handlePiiMask)
}

// handlePiiClassify は POST /api/pii/classify を処理する。
func (r *Router) handlePiiClassify(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body piiClassifyRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する（空文字でも tier1 に投げて findings 空応答を期待する設計もあるが、明示拒否）。
	if body.Text == "" {
		writeBadRequest(w, "E-T3-BFF-PII-100", "text is required")
		return
	}
	// facade 経由で SDK を呼ぶ。
	findings, contains, err := r.facade.PiiClassify(req.Context(), body.Text)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-PII-200", "pii classify failed: "+err.Error())
		return
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, piiClassifyResponse{
		Findings:    convertPiiFindings(findings),
		ContainsPii: contains,
	})
}

// handlePiiMask は POST /api/pii/mask を処理する。
func (r *Router) handlePiiMask(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body piiMaskRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する。
	if body.Text == "" {
		writeBadRequest(w, "E-T3-BFF-PII-101", "text is required")
		return
	}
	// facade 経由で SDK を呼ぶ。
	masked, findings, err := r.facade.PiiMask(req.Context(), body.Text)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-PII-201", "pii mask failed: "+err.Error())
		return
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, piiMaskResponse{
		MaskedText: masked,
		Findings:   convertPiiFindings(findings),
	})
}

// convertPiiFindings は k1s0client の構造体を BFF 応答用構造体に詰め替える。
func convertPiiFindings(in []k1s0client.PiiFindingSummary) []piiFindingOut {
	out := make([]piiFindingOut, 0, len(in))
	for _, f := range in {
		out = append(out, piiFindingOut{
			Type:       f.Type,
			Start:      f.Start,
			End:        f.End,
			Confidence: f.Confidence,
		})
	}
	return out
}
