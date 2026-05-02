// Audit の REST エンドポイント。
//
//	POST /api/audit/record — 監査イベント記録
//	POST /api/audit/query  — 監査イベント検索

package rest

// 標準 / 内部 import。
import (
	// HTTP server。
	"net/http"
	// 時刻パース。
	"time"
)

// auditRecordRequest は POST /api/audit/record の入力。
type auditRecordRequest struct {
	Actor          string            `json:"actor"`
	Action         string            `json:"action"`
	Resource       string            `json:"resource"`
	Outcome        string            `json:"outcome"`
	Attributes     map[string]string `json:"attributes,omitempty"`
	IdempotencyKey string            `json:"idempotency_key,omitempty"`
}

// auditRecordResponse は POST /api/audit/record の出力。
type auditRecordResponse struct {
	AuditID string `json:"audit_id"`
}

// auditQueryRequest は POST /api/audit/query の入力。
// from / to は RFC3339（zero 値なら範囲未指定として SDK にそのまま渡す）。
type auditQueryRequest struct {
	From    string            `json:"from,omitempty"`
	To      string            `json:"to,omitempty"`
	Filters map[string]string `json:"filters,omitempty"`
	Limit   int32             `json:"limit,omitempty"`
}

// auditEventOut は応答 1 件分の監査イベント（PII Mask 適用済）。
type auditEventOut struct {
	OccurredAtMillis int64             `json:"occurred_at_millis"`
	Actor            string            `json:"actor"`
	Action           string            `json:"action"`
	Resource         string            `json:"resource"`
	Outcome          string            `json:"outcome"`
	Attributes       map[string]string `json:"attributes,omitempty"`
}

// auditQueryResponse は POST /api/audit/query の出力。
type auditQueryResponse struct {
	Events []auditEventOut `json:"events"`
}

// registerAudit は audit 系 endpoint を mux に登録する。
func (r *Router) registerAudit(mux *http.ServeMux) {
	// Audit.Record。
	mux.HandleFunc("POST /api/audit/record", r.handleAuditRecord)
	// Audit.Query。
	mux.HandleFunc("POST /api/audit/query", r.handleAuditQuery)
}

// handleAuditRecord は POST /api/audit/record を処理する。
func (r *Router) handleAuditRecord(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body auditRecordRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する（idempotency_key は任意）。
	if body.Actor == "" || body.Action == "" || body.Resource == "" || body.Outcome == "" {
		writeBadRequest(w, "E-T3-BFF-AUDIT-100", "actor, action, resource, outcome are required")
		return
	}
	// facade 経由で SDK を呼ぶ。
	auditID, err := r.facade.AuditRecord(req.Context(), body.Actor, body.Action, body.Resource, body.Outcome, body.Attributes, body.IdempotencyKey)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-AUDIT-200", "audit record failed: "+err.Error())
		return
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, auditRecordResponse{AuditID: auditID})
}

// handleAuditQuery は POST /api/audit/query を処理する。
func (r *Router) handleAuditQuery(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body auditQueryRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// from / to を RFC3339 でパースする（空文字は zero time として SDK に渡す）。
	var from, to time.Time
	if body.From != "" {
		t, err := time.Parse(time.RFC3339, body.From)
		if err != nil {
			writeBadRequest(w, "E-T3-BFF-AUDIT-101", "from must be RFC3339: "+err.Error())
			return
		}
		from = t
	}
	if body.To != "" {
		t, err := time.Parse(time.RFC3339, body.To)
		if err != nil {
			writeBadRequest(w, "E-T3-BFF-AUDIT-102", "to must be RFC3339: "+err.Error())
			return
		}
		to = t
	}
	// limit は負値を 0 に正規化する（SDK 側既定 = サーバ既定 limit）。
	limit := body.Limit
	if limit < 0 {
		limit = 0
	}
	// facade 経由で SDK を呼ぶ。
	events, err := r.facade.AuditQuery(req.Context(), from, to, body.Filters, limit)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-AUDIT-201", "audit query failed: "+err.Error())
		return
	}
	// 応答 JSON 用に詰め替える。
	out := make([]auditEventOut, 0, len(events))
	for _, e := range events {
		out = append(out, auditEventOut{
			OccurredAtMillis: e.OccurredAtMillis,
			Actor:            e.Actor,
			Action:           e.Action,
			Resource:         e.Resource,
			Outcome:          e.Outcome,
			Attributes:       e.Attributes,
		})
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, auditQueryResponse{Events: out})
}
