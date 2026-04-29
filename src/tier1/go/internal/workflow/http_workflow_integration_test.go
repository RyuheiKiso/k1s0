// 本ファイルは Workflow API の HTTP/JSON gateway 経由 e2e テスト。
//
// 注意:
//   release-initial では adapter (Temporal / Dapr Workflow) が未注入のため、handler は
//   Unimplemented を返す。本テストは tenant_id 防御 + ルート登録の確認に絞る
//   （実 backend 結合は別テストで代表）。

package workflow

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
)

func startWorkflowHTTPGateway(t *testing.T) (*httptest.Server, func()) {
	t.Helper()
	deps := Deps{} // adapter 未注入、各 RPC は Unimplemented を返す
	g := common.NewHTTPGateway()
	g.RegisterWorkflowRoutes(MakeHTTPHandlers(NewWorkflowServiceServer(deps)))
	srv := httptest.NewServer(g.Handler())
	return srv, srv.Close
}

// tenant_id 不在は 400。
func TestHTTPGateway_Workflow_TenantIDRequired(t *testing.T) {
	srv, cleanup := startWorkflowHTTPGateway(t)
	defer cleanup()
	cases := []struct{ path, body string }{
		{"/k1s0/workflow/start", `{"workflow_type":"X","workflow_id":"id1","context":{}}`},
		{"/k1s0/workflow/signal", `{"workflow_id":"id1","signal_name":"s","context":{}}`},
		{"/k1s0/workflow/cancel", `{"workflow_id":"id1","context":{}}`},
		{"/k1s0/workflow/terminate", `{"workflow_id":"id1","context":{}}`},
		{"/k1s0/workflow/getstatus", `{"workflow_id":"id1","context":{}}`},
	}
	for _, c := range cases {
		t.Run(c.path, func(t *testing.T) {
			resp, err := http.Post(srv.URL+c.path, "application/json", strings.NewReader(c.body))
			if err != nil {
				t.Fatalf("POST: %v", err)
			}
			defer func() { _ = resp.Body.Close() }()
			if resp.StatusCode != http.StatusBadRequest {
				t.Fatalf("%s status = %d want 400", c.path, resp.StatusCode)
			}
		})
	}
}

// 不正 JSON は 400。
func TestHTTPGateway_Workflow_InvalidJSON_400(t *testing.T) {
	srv, cleanup := startWorkflowHTTPGateway(t)
	defer cleanup()
	resp, err := http.Post(srv.URL+"/k1s0/workflow/start", "application/json", strings.NewReader("not json"))
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("status = %d want 400", resp.StatusCode)
	}
}

// adapter 未注入時、tenant_id 付きで Start を呼ぶと 503 (Unavailable / Unimplemented) を返す。
// docs §「HTTP Status ↔ K1s0Error」では Unimplemented は明示マッピング外だが、
// 既定で 500 系（Internal）に倒れる。本テストでは「panic しない / 4xx を返さない」ことを確認する。
func TestHTTPGateway_Workflow_UnwiredBackend_NotPanic(t *testing.T) {
	srv, cleanup := startWorkflowHTTPGateway(t)
	defer cleanup()
	resp, err := http.Post(srv.URL+"/k1s0/workflow/start", "application/json",
		strings.NewReader(`{"workflow_type":"X","workflow_id":"id1","context":{"tenant_id":"T"}}`))
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	// adapter 未注入時 handler は Unimplemented を返し、HTTP gateway はそれを 500 にマップする
	// （httpStatusFromGRPC は Unimplemented を default ケースで 500 扱い）。
	if resp.StatusCode != http.StatusInternalServerError {
		t.Fatalf("status = %d want 500 (Unimplemented falls through to 500)", resp.StatusCode)
	}
}
