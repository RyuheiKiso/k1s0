// 本ファイルは portal-bff / admin-bff 共通の REST エンドポイント定義。
// `/healthz` `/readyz` の k8s probe と JSON エラー応答ヘルパを束ねる。

// Package rest は portal-bff / admin-bff 共通の REST エンドポイントを提供する。
package rest

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// JSON エンコード / デコード。
	"encoding/json"
	// HTTP server。
	"net/http"
)

// StateClient は Router が必要とする k1s0 State の最小 interface。
// 実体は k1s0client.Client が満たすが、test では in-memory mock を渡せる。
type StateClient interface {
	// StateGet は k1s0 State から指定キーを取得する。
	StateGet(ctx context.Context, store, key string) (data []byte, etag string, found bool, err error)
}

// Router は REST ルートを mux に登録する。
type Router struct {
	// k1s0 State の最小 interface（テスト容易性のため抽象化）。
	k1s0Client StateClient
}

// NewRouter は Router を組み立てる。
func NewRouter(client StateClient) *Router {
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
