// 本ファイルは Feature / Binding / Log / Telemetry / ServiceInvoke の HTTP/JSON
// gateway 経由 e2e テスト。adapter 未注入時の Unimplemented / InvalidArgument 翻訳を
// HTTP Status マッピング込みで確認する。
//
// 注意:
//   個別 RPC の正常系（実 backend 結合）は他テスト（pubsub / state / secret の HTTP
//   integration）で代表されているため、本ファイルではルート登録 + tenant_id 必須の
//   防御層が機能していることのみを検証する。

package state

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
)

// startMiscHTTPGateway は Feature / Binding / Log / Telemetry / Invoke を 1 gateway に登録する。
func startMiscHTTPGateway(t *testing.T) (*httptest.Server, func()) {
	t.Helper()
	client := dapr.NewClientWithInMemoryBackends()
	deps := NewDepsFromClient(client)
	g := common.NewHTTPGateway()
	g.RegisterFeatureRoutes(MakeHTTPFeatureHandlers(NewFeatureServiceServer(deps)))
	g.RegisterBindingRoutes(MakeHTTPBindingHandlers(NewBindingServiceServer(deps)))
	g.RegisterLogRoutes(MakeHTTPLogHandlers(NewLogServiceServer(deps)))
	g.RegisterTelemetryRoutes(MakeHTTPTelemetryHandlers(NewTelemetryServiceServer(deps)))
	g.RegisterInvokeRoutes(MakeHTTPInvokeHandlers(NewInvokeServiceServer(deps)))
	srv := httptest.NewServer(g.Handler())
	return srv, srv.Close
}

// 各 API で tenant_id 不在は 400 を返す（ルート登録 + handler の防御層が機能している）。
func TestHTTPGateway_Misc_TenantIDRequired(t *testing.T) {
	srv, cleanup := startMiscHTTPGateway(t)
	defer cleanup()

	cases := []struct {
		path string
		body string
	}{
		{"/k1s0/feature/evaluateboolean", `{"flag_key":"f","context":{}}`},
		{"/k1s0/feature/evaluatestring", `{"flag_key":"f","context":{}}`},
		{"/k1s0/feature/evaluatenumber", `{"flag_key":"f","context":{}}`},
		{"/k1s0/feature/evaluateobject", `{"flag_key":"f","context":{}}`},
		{"/k1s0/binding/invoke", `{"name":"x","operation":"create","context":{}}`},
		{"/k1s0/log/send", `{"entry":{"body":"x"},"context":{}}`},
		{"/k1s0/log/bulksend", `{"entries":[{"body":"x"}],"context":{}}`},
		{"/k1s0/telemetry/emitmetric", `{"metric":{"name":"x"},"context":{}}`},
		{"/k1s0/telemetry/emitspan", `{"span":{"name":"x"},"context":{}}`},
		{"/k1s0/serviceinvoke/invoke", `{"app_id":"x","method":"y","context":{}}`},
	}
	for _, c := range cases {
		t.Run(c.path, func(t *testing.T) {
			resp, err := http.Post(srv.URL+c.path, "application/json", strings.NewReader(c.body))
			if err != nil {
				t.Fatalf("POST: %v", err)
			}
			defer func() { _ = resp.Body.Close() }()
			if resp.StatusCode != http.StatusBadRequest {
				t.Fatalf("status = %d want 400", resp.StatusCode)
			}
		})
	}
}

// 不正 JSON は全 path で 400。
func TestHTTPGateway_Misc_InvalidJSON_400(t *testing.T) {
	srv, cleanup := startMiscHTTPGateway(t)
	defer cleanup()
	paths := []string{
		"/k1s0/feature/evaluateboolean",
		"/k1s0/binding/invoke",
		"/k1s0/log/send",
		"/k1s0/telemetry/emitmetric",
		"/k1s0/serviceinvoke/invoke",
	}
	for _, p := range paths {
		t.Run(p, func(t *testing.T) {
			resp, err := http.Post(srv.URL+p, "application/json", strings.NewReader("not json"))
			if err != nil {
				t.Fatalf("POST: %v", err)
			}
			defer func() { _ = resp.Body.Close() }()
			if resp.StatusCode != http.StatusBadRequest {
				t.Fatalf("status = %d want 400", resp.StatusCode)
			}
		})
	}
}
