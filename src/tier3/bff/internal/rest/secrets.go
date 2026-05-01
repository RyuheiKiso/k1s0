// Secrets の REST エンドポイント。
//
//	POST /api/secrets/get    — secret を取得する
//	POST /api/secrets/rotate — secret をローテートする

package rest

// 標準 import。
import (
	// HTTP server。
	"net/http"
)

// secretsGetRequest は POST /api/secrets/get の入力。
type secretsGetRequest struct {
	Name string `json:"name"`
}

// secretsGetResponse は POST /api/secrets/get の出力。
// values は VAULT 互換の key/value マップ。
type secretsGetResponse struct {
	Values  map[string]string `json:"values"`
	Version int32             `json:"version"`
}

// secretsRotateRequest は POST /api/secrets/rotate の入力。
type secretsRotateRequest struct {
	Name           string `json:"name"`
	GracePeriodSec int32  `json:"grace_period_sec,omitempty"`
	IdempotencyKey string `json:"idempotency_key,omitempty"`
}

// secretsRotateResponse は POST /api/secrets/rotate の出力。
type secretsRotateResponse struct {
	NewVersion      int32 `json:"new_version"`
	PreviousVersion int32 `json:"previous_version"`
}

// registerSecrets は secrets 系 endpoint を mux に登録する。
func (r *Router) registerSecrets(mux *http.ServeMux) {
	// Secrets.Get。
	mux.HandleFunc("POST /api/secrets/get", r.handleSecretsGet)
	// Secrets.Rotate。
	mux.HandleFunc("POST /api/secrets/rotate", r.handleSecretsRotate)
}

// handleSecretsGet は POST /api/secrets/get を処理する。
func (r *Router) handleSecretsGet(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body secretsGetRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する。
	if body.Name == "" {
		writeBadRequest(w, "E-T3-BFF-SECRETS-100", "name is required")
		return
	}
	// facade 経由で SDK を呼ぶ。
	values, version, err := r.facade.SecretsGet(req.Context(), body.Name)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-SECRETS-200", "secret get failed: "+err.Error())
		return
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, secretsGetResponse{Values: values, Version: version})
}

// handleSecretsRotate は POST /api/secrets/rotate を処理する。
func (r *Router) handleSecretsRotate(w http.ResponseWriter, req *http.Request) {
	// JSON body をデコードする。
	var body secretsRotateRequest
	if !decodeJSON(w, req, &body) {
		return
	}
	// 必須項目を検証する。
	if body.Name == "" {
		writeBadRequest(w, "E-T3-BFF-SECRETS-101", "name is required")
		return
	}
	// facade 経由で SDK を呼ぶ。
	newV, prevV, err := r.facade.SecretsRotate(req.Context(), body.Name, body.GracePeriodSec, body.IdempotencyKey)
	if err != nil {
		writeBadGateway(w, "E-T3-BFF-SECRETS-201", "secret rotate failed: "+err.Error())
		return
	}
	// 応答 JSON を返す。
	writeJSON(w, http.StatusOK, secretsRotateResponse{NewVersion: newV, PreviousVersion: prevV})
}
