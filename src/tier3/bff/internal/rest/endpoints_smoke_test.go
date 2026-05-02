// 本ファイルは BFF rest router が 14 公開サービスの全 endpoint を mux に登録したことの
// smoke test。各 endpoint 実装の詳細テストは個別のサービス test に切り出すが、本 test は
// 「endpoint が漏れなく registered されている」ことを保証する回帰防止層として機能する。

package rest

// 標準 / 内部 import。
import (
	// HTTP server。
	"net/http"
	// テスト用 HTTP server。
	"net/http/httptest"
	// テスト frame。
	"testing"
)

// expectedEndpoints は Register が登録すべき全 endpoint の path 一覧。
// 新 endpoint を rest 配下に追加した時は本リストにも追記すること。
var expectedEndpoints = []string{
	// State (3)。
	"/api/state/get",
	"/api/state/save",
	"/api/state/delete",
	// PubSub (1)。
	"/api/pubsub/publish",
	// Secrets (2)。
	"/api/secrets/get",
	"/api/secrets/rotate",
	// Decision (1)。
	"/api/decision/evaluate",
	// Workflow (1)。
	"/api/workflow/start",
	// Invoke (1)。
	"/api/invoke/call",
	// Audit (2)。
	"/api/audit/record",
	"/api/audit/query",
	// Log (1)。
	"/api/log/send",
	// Telemetry (1)。
	"/api/telemetry/emit-metric",
	// PII (2)。
	"/api/pii/classify",
	"/api/pii/mask",
	// Feature (1)。
	"/api/feature/evaluate-boolean",
	// Binding (1)。
	"/api/binding/invoke",
}

// TestAllEndpointsRegistered は Register 後に各 endpoint へ POST すると、
// 「mux level でハンドラが見つからない (404)」では*ない*ことを確認する。
// no-op fake を渡しているので、handler が呼ばれた結果は 400（必須項目欠落）か
// 200（完全な省略可能項目で構成された endpoint）になる。404 / 405 が出たら登録漏れ。
func TestAllEndpointsRegistered(t *testing.T) {
	mux := http.NewServeMux()
	NewRouter(unimplementedFacade{}).Register(mux)
	srv := httptest.NewServer(mux)
	defer srv.Close()
	for _, path := range expectedEndpoints {
		resp, _ := postBody(t, srv.URL, path, map[string]any{})
		if resp.StatusCode == http.StatusNotFound || resp.StatusCode == http.StatusMethodNotAllowed {
			t.Errorf("%s should be registered, got %d", path, resp.StatusCode)
		}
	}
}
