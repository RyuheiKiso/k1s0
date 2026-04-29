// GraphQL endpoint（リリース時点 minimal、リリース時点 で gqlgen に置換）。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md

// Package graphql は portal-bff の GraphQL endpoint を提供する。
package graphql

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// JSON エンコード / デコード。
	"encoding/json"
	// 文字列処理。
	"strings"
	// HTTP server。
	"net/http"
	// timeout 設定。
	"time"
)

// StateClient は Resolver が必要とする k1s0 State の最小 interface。
// 実体は k1s0client.Client が満たすが、test では in-memory mock を渡せる。
type StateClient interface {
	// StateGet は k1s0 State から指定キーを取得する。
	StateGet(ctx context.Context, store, key string) (data []byte, etag string, found bool, err error)
}

// Resolver は GraphQL クエリを解決する Resolver。
type Resolver struct {
	// k1s0 State の最小 interface（テスト容易性のため抽象化）。
	k1s0Client StateClient
}

// NewResolver は Resolver を組み立てる。
func NewResolver(client StateClient) *Resolver {
	return &Resolver{k1s0Client: client}
}

// graphqlRequest は POST /graphql の入力 JSON。
type graphqlRequest struct {
	Query         string         `json:"query"`
	Variables     map[string]any `json:"variables,omitempty"`
	OperationName string         `json:"operationName,omitempty"`
}

// graphqlResponse は GraphQL 標準応答 JSON。
type graphqlResponse struct {
	Data   any              `json:"data,omitempty"`
	Errors []map[string]any `json:"errors,omitempty"`
}

// Handler は POST /graphql ハンドラを返す。
func (r *Resolver) Handler() http.HandlerFunc {
	return func(w http.ResponseWriter, req *http.Request) {
		// POST のみ受ける。
		if req.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		// JSON body をデコードする。
		var gReq graphqlRequest
		if err := json.NewDecoder(req.Body).Decode(&gReq); err != nil {
			http.Error(w, "invalid json: "+err.Error(), http.StatusBadRequest)
			return
		}
		// timeout を被せる。
		ctx, cancel := context.WithTimeout(req.Context(), 5*time.Second)
		defer cancel()
		// 簡易 query ルータ（リリース時点 では substring match、リリース時点 で gqlgen の resolver に置換）。
		var resp graphqlResponse
		switch {
		case strings.Contains(gReq.Query, "stateGet"):
			// State.Get クエリ。
			store, _ := gReq.Variables["store"].(string)
			key, _ := gReq.Variables["key"].(string)
			data, etag, found, err := r.k1s0Client.StateGet(ctx, store, key)
			if err != nil {
				resp.Errors = []map[string]any{{"message": err.Error()}}
			} else if !found {
				resp.Data = map[string]any{"stateGet": nil}
			} else {
				resp.Data = map[string]any{"stateGet": map[string]any{"data": string(data), "etag": etag}}
			}
		case strings.Contains(gReq.Query, "currentUser"):
			// 現在認証済ユーザを返す（auth middleware が context に subject を入れる前提）。
			resp.Data = map[string]any{"currentUser": map[string]any{"id": "anonymous", "roles": []string{}}}
		default:
			// 未知のクエリは null を返す。
			resp.Errors = []map[string]any{{"message": "unsupported query"}}
		}
		// JSON で応答する。
		w.Header().Set("Content-Type", "application/json; charset=utf-8")
		_ = json.NewEncoder(w).Encode(resp)
	}
}
