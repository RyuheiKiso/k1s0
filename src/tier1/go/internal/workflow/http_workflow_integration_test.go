// 本ファイルは Workflow API の HTTP/JSON gateway 経由 e2e テスト。
//
// 役割:
//   tenant_id 防御 + ルート登録 + JSON parse 検証の確認に絞る。adapter 未注入時の
//   挙動は handler の契約上想定しない（cmd/workflow/main.go で必ず注入される）ため、
//   本テストの Deps{} は tenant_id / JSON 検証が adapter 呼出より前に短絡することの
//   検証に使う。実 backend 結合は別テストで代表する。

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
	// adapter 未注入の Deps を渡すが、本テストの assertion は tenant_id / JSON
	// validation で adapter 呼出より前に短絡するため adapter nil でも到達しない。
	deps := Deps{}
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

