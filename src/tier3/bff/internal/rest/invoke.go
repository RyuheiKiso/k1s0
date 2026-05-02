// Invoke の REST エンドポイント。
//
//	POST /api/invoke/call — 他 Dapr アプリへの unary 呼出

package rest

// 標準 import。
import (
	// HTTP server。
	"net/http"
)

// invokeCallRequest は POST /api/invoke/call の入力。
type invokeCallRequest struct {
	AppID       string `json:"app_id"`
	Method      string `json:"method"`
	Data        string `json:"data,omitempty"`
	ContentType string `json:"content_type,omitempty"`
	TimeoutMs   int32  `json:"timeout_ms,omitempty"`
}

// invokeCallResponse は POST /api/invoke/call の出力。
type invokeCallResponse struct {
	Data        string `json:"data"`
	ContentType string `json:"content_type"`
	Status      int32  `json:"status"`
}

// registerInvoke は invoke 系 endpoint を mux に登録する。
func (r *Router) registerInvoke(mux *http.ServeMux) {
	// Invoke.Call。
	mux.HandleFunc("POST /api/invoke/call", r.handleInvokeCall)
}

// handleInvokeCall は POST /api/invoke/call を処理する。
func (r *Router) handleInvokeCall(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body invokeCallRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する。
	if body.AppID == "" || body.Method == "" {
		writeBadRequest(w, "E-T3-BFF-INVOKE-100", "app_id and method are required")
		return
	}
	// content_type の既定は application/json。
	contentType := body.ContentType
	if contentType == "" {
		contentType = "application/json"
	}
	// facade 経由で SDK を呼ぶ。
	respData, respCT, status, err := r.facade.InvokeCall(req.Context(), body.AppID, body.Method, []byte(body.Data), contentType, body.TimeoutMs)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-INVOKE-200", "invoke call failed: "+err.Error())
		return
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, invokeCallResponse{
		Data:        string(respData),
		ContentType: respCT,
		Status:      status,
	})
}
