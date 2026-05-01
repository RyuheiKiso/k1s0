// Log の REST エンドポイント。
//
//	POST /api/log/send — 単一ログエントリ送信

package rest

// 標準 / 内部 import。
import (
	// HTTP server。
	"net/http"

	// k1s0client の LogSeverity 型を参照する。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/k1s0client"
)

// logSendRequest は POST /api/log/send の入力。
// severity は TRACE / DEBUG / INFO / WARN / ERROR / FATAL のいずれか（未指定は INFO 扱い）。
type logSendRequest struct {
	Severity   string            `json:"severity,omitempty"`
	Body       string            `json:"body"`
	Attributes map[string]string `json:"attributes,omitempty"`
}

// registerLog は log 系 endpoint を mux に登録する。
func (r *Router) registerLog(mux *http.ServeMux) {
	// Log.Send。
	mux.HandleFunc("POST /api/log/send", r.handleLogSend)
}

// handleLogSend は POST /api/log/send を処理する。
func (r *Router) handleLogSend(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body logSendRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// body は必須（severity は未指定でも k1s0client 側が INFO へ fallback する）。
	if body.Body == "" {
		writeBadRequest(w, "E-T3-BFF-LOG-100", "body is required")
		return
	}
	// 文字列 severity を k1s0client 公開定数型に詰める（未知値は k1s0client 側で INFO fallback）。
	severity := k1s0client.LogSeverity(body.Severity)
	// 空文字なら INFO 扱いに合わせる（呼出側ログから "" が出力されるのを防ぐ）。
	if severity == "" {
		severity = k1s0client.LogSeverityInfo
	}
	// facade 経由で SDK を呼ぶ。
	if err := r.facade.LogSend(req.Context(), severity, body.Body, body.Attributes); err != nil {
		writeBadGateway(w, "E-T3-BFF-LOG-200", "log send failed: "+err.Error())
		return
	}
	// 応答 JSON を返す（成功時は空 body）。
	writeJSON(w, http.StatusOK, struct{}{})
}
