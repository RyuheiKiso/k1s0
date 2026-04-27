// 本ファイルは tier3 BFF（Backend-for-Frontend）GraphQL 最小例。
// React SPA からの GraphQL クエリを受け、k1s0 SDK 経由で tier1 State API を呼ぶ。
//
// 採用初期で gqlgen / 99designs/gqlgen 等の本格的 GraphQL ライブラリに移行予定。
// 本リリース時点 では net/http だけで GraphQL の最小処理（POST /graphql に JSON Body）を実装。

package main

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// JSON エンコード / デコード。
	"encoding/json"
	// CLI flag。
	"flag"
	// HTTP server。
	"net/http"
	// 標準ログ。
	"log"
	// シグナル受信。
	"os"
	"os/signal"
	"syscall"
	"time"

	// k1s0 高水準 SDK facade。
	"github.com/k1s0/sdk-go/k1s0"
)

// graphqlRequest は GraphQL リクエストの JSON 形式。
type graphqlRequest struct {
	// クエリ文字列。
	Query string `json:"query"`
	// 変数マップ（query の $var 引数）。
	Variables map[string]any `json:"variables,omitempty"`
	// オペレーション名（複数 query 含む場合の選択用）。
	OperationName string `json:"operationName,omitempty"`
}

// graphqlResponse は GraphQL 応答の JSON 形式。
type graphqlResponse struct {
	// 成功時の data。
	Data any `json:"data,omitempty"`
	// 失敗時の errors。
	Errors []map[string]any `json:"errors,omitempty"`
}

// プロセスエントリポイント。
func main() {
	// listen address 上書き flag。
	addr := flag.String("listen", ":8080", "HTTP server listen address")
	// tier1 facade 接続先。
	tier1Target := flag.String("tier1-target", "localhost:50001", "tier1 facade gRPC target")
	tenantID := flag.String("tenant-id", "tenant-example", "Tenant ID")
	subject := flag.String("subject", "tier3-portal-bff", "Subject")
	flag.Parse()

	// k1s0 SDK Client を生成する。
	client, err := k1s0.New(context.Background(), k1s0.Config{
		Target: *tier1Target, TenantID: *tenantID, Subject: *subject,
		UseTLS: false,
	})
	if err != nil {
		log.Fatalf("k1s0 sdk init: %v", err)
	}
	defer client.Close()

	// HTTP handler。
	mux := http.NewServeMux()
	// 健全性。
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK); _, _ = w.Write([]byte("ok"))
	})
	mux.HandleFunc("/readyz", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK); _, _ = w.Write([]byte("ready"))
	})

	// GraphQL endpoint（POST /graphql）。
	mux.HandleFunc("/graphql", func(w http.ResponseWriter, r *http.Request) {
		// POST のみ受ける。
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		// JSON body をデコードする。
		var req graphqlRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid json: "+err.Error(), http.StatusBadRequest)
			return
		}
		// 最小実装: Query 文字列に "stateGet" を含めば tier1 State.Get を呼ぶ簡易ルータ。
		// 採用初期で gqlgen の resolver にリプレースする。
		ctx, cancel := context.WithTimeout(r.Context(), 5*time.Second)
		defer cancel()
		var resp graphqlResponse
		if contains(req.Query, "stateGet") {
			// variables から store / key を取得する。
			store, _ := req.Variables["store"].(string)
			key, _ := req.Variables["key"].(string)
			data, etag, found, err := client.State().Get(ctx, store, key)
			if err != nil {
				resp.Errors = []map[string]any{{"message": err.Error()}}
			} else if !found {
				resp.Data = map[string]any{"stateGet": nil}
			} else {
				resp.Data = map[string]any{
					"stateGet": map[string]any{
						"data": string(data),
						"etag": etag,
					},
				}
			}
		} else {
			// 未知のクエリは null を返す。
			resp.Data = map[string]any{"_": nil}
		}
		// JSON で応答する。
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(resp)
	})

	// HTTP server 起動。
	srv := &http.Server{Addr: *addr, Handler: mux, ReadHeaderTimeout: 5 * time.Second}
	errCh := make(chan error, 1)
	go func() {
		log.Printf("tier3-portal-bff: HTTP listening on %s", *addr)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			errCh <- err
		}
	}()

	// シグナル待ち。
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)
	select {
	case sig := <-sigCh:
		log.Printf("received signal %s, shutting down", sig)
	case err := <-errCh:
		log.Fatalf("http server: %v", err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), 25*time.Second)
	defer cancel()
	_ = srv.Shutdown(ctx)
	log.Printf("tier3-portal-bff: shutdown complete")
}

// contains は s に sub が含まれるかを返す（軽量 strings.Contains 代替）。
func contains(s, sub string) bool {
	// 単純 byte 比較で十分（GraphQL クエリは ASCII で書かれる前提）。
	return len(sub) > 0 && len(s) >= len(sub) && index(s, sub) >= 0
}

// index は s 内に sub が現れる最小インデックスを返す（なければ -1）。
func index(s, sub string) int {
	// O(n*m) の単純実装。GraphQL クエリは短いので十分。
	for i := 0; i+len(sub) <= len(s); i++ {
		if s[i:i+len(sub)] == sub {
			return i
		}
	}
	return -1
}
