// State の REST エンドポイント。
//
//	POST /api/state/get    — k1s0 State から指定キーを取得する
//	POST /api/state/save   — k1s0 State にキーを保存する
//	POST /api/state/delete — k1s0 State から指定キーを削除する

package rest

// 標準 import。
import (
	// HTTP server。
	"net/http"
)

// stateGetRequest は POST /api/state/get の入力。
type stateGetRequest struct {
	Store string `json:"store"`
	Key   string `json:"key"`
}

// stateGetResponse は POST /api/state/get の出力。
type stateGetResponse struct {
	Data  string `json:"data,omitempty"`
	Etag  string `json:"etag,omitempty"`
	Found bool   `json:"found"`
}

// stateSaveRequest は POST /api/state/save の入力。
type stateSaveRequest struct {
	Store string `json:"store"`
	Key   string `json:"key"`
	Data  string `json:"data"`
}

// stateSaveResponse は POST /api/state/save の出力。
type stateSaveResponse struct {
	Etag string `json:"etag,omitempty"`
}

// stateDeleteRequest は POST /api/state/delete の入力。
// expectedEtag が空でなければ optimistic concurrency control を効かせる。
type stateDeleteRequest struct {
	Store        string `json:"store"`
	Key          string `json:"key"`
	ExpectedEtag string `json:"expected_etag,omitempty"`
}

// registerState は state 系 endpoint を mux に登録する。
func (r *Router) registerState(mux *http.ServeMux) {
	// State.Get。
	mux.HandleFunc("POST /api/state/get", r.handleStateGet)
	// State.Save。
	mux.HandleFunc("POST /api/state/save", r.handleStateSave)
	// State.Delete。
	mux.HandleFunc("POST /api/state/delete", r.handleStateDelete)
}

// handleStateGet は POST /api/state/get を処理する。
func (r *Router) handleStateGet(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body stateGetRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する。
	if body.Store == "" || body.Key == "" {
		writeBadRequest(w, "E-T3-BFF-STATE-100", "store and key are required")
		return
	}
	// facade 経由で SDK を呼ぶ。
	data, etag, found, err := r.facade.StateGet(req.Context(), body.Store, body.Key)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-STATE-200", "state get failed: "+err.Error())
		return
	}
	// 応答 JSON を返す（not-found は found=false で 200）。
	writeJSON(w, http.StatusOK, stateGetResponse{
		Data:  string(data),
		Etag:  etag,
		Found: found,
	})
}

// handleStateSave は POST /api/state/save を処理する。
func (r *Router) handleStateSave(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body stateSaveRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する。
	if body.Store == "" || body.Key == "" {
		writeBadRequest(w, "E-T3-BFF-STATE-101", "store and key are required")
		return
	}
	// facade 経由で SDK を呼ぶ。
	etag, err := r.facade.StateSave(req.Context(), body.Store, body.Key, []byte(body.Data))
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-STATE-201", "state save failed: "+err.Error())
		return
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, stateSaveResponse{Etag: etag})
}

// handleStateDelete は POST /api/state/delete を処理する。
func (r *Router) handleStateDelete(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body stateDeleteRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する。
	if body.Store == "" || body.Key == "" {
		writeBadRequest(w, "E-T3-BFF-STATE-102", "store and key are required")
		return
	}
	// facade 経由で SDK を呼ぶ。
	if err := r.facade.StateDelete(req.Context(), body.Store, body.Key, body.ExpectedEtag); err != nil {
		writeBadGateway(w, "E-T3-BFF-STATE-202", "state delete failed: "+err.Error())
		return
	}
	// 応答 JSON を返す（成功時は空 body）。
	writeJSON(w, http.StatusOK, struct{}{})
}
