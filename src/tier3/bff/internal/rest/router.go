// 本ファイルは portal-bff / admin-bff 共通の REST エンドポイント定義。
// `/healthz` `/readyz` の k8s probe と JSON エラー応答ヘルパを束ねる。

// Package rest は portal-bff / admin-bff 共通の REST エンドポイントを提供する。
package rest

// 標準 / 内部 import。
import (
	// JSON エンコード / デコード。
	"encoding/json"
	// HTTP server。
	"net/http"

	// k1s0 SDK ラッパー。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/k1s0client"
)

// Router は REST ルートを mux に登録する。
type Router struct {
	// k1s0 SDK Client。
	k1s0Client *k1s0client.Client
}

// NewRouter は Router を組み立てる。
func NewRouter(client *k1s0client.Client) *Router {
	return &Router{k1s0Client: client}
}

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

// Register は mux に REST endpoint を登録する。
func (r *Router) Register(mux *http.ServeMux) {
	// 簡易 State.Get（GraphQL を使えないクライアント用）。
	mux.HandleFunc("POST /api/state/get", func(w http.ResponseWriter, req *http.Request) {
		var body stateGetRequest
		if err := json.NewDecoder(req.Body).Decode(&body); err != nil {
			http.Error(w, "invalid json: "+err.Error(), http.StatusBadRequest)
			return
		}
		if body.Store == "" || body.Key == "" {
			http.Error(w, "store and key are required", http.StatusBadRequest)
			return
		}
		data, etag, found, err := r.k1s0Client.StateGet(req.Context(), body.Store, body.Key)
		if err != nil {
			http.Error(w, "state get failed: "+err.Error(), http.StatusBadGateway)
			return
		}
		w.Header().Set("Content-Type", "application/json; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_ = json.NewEncoder(w).Encode(stateGetResponse{
			Data:  string(data),
			Etag:  etag,
			Found: found,
		})
	})
}
