// Binding の REST エンドポイント。
//
//	POST /api/binding/invoke — Output Binding 呼出（HTTP / SMTP / S3 等）

package rest

// 標準 import。
import (
	// HTTP server。
	"net/http"
)

// bindingInvokeRequest は POST /api/binding/invoke の入力。
type bindingInvokeRequest struct {
	Name      string            `json:"name"`
	Operation string            `json:"operation"`
	Data      string            `json:"data,omitempty"`
	Metadata  map[string]string `json:"metadata,omitempty"`
}

// bindingInvokeResponse は POST /api/binding/invoke の出力。
type bindingInvokeResponse struct {
	Data     string            `json:"data,omitempty"`
	Metadata map[string]string `json:"metadata,omitempty"`
}

// registerBinding は binding 系 endpoint を mux に登録する。
func (r *Router) registerBinding(mux *http.ServeMux) {
	// Binding.Invoke。
	mux.HandleFunc("POST /api/binding/invoke", r.handleBindingInvoke)
}

// handleBindingInvoke は POST /api/binding/invoke を処理する。
func (r *Router) handleBindingInvoke(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body bindingInvokeRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する。
	if body.Name == "" || body.Operation == "" {
		writeBadRequest(w, "E-T3-BFF-BINDING-100", "name and operation are required")
		return
	}
	// facade 経由で SDK を呼ぶ。
	respData, respMeta, err := r.facade.BindingInvoke(req.Context(), body.Name, body.Operation, []byte(body.Data), body.Metadata)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-BINDING-200", "binding invoke failed: "+err.Error())
		return
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, bindingInvokeResponse{
		Data:     string(respData),
		Metadata: respMeta,
	})
}
