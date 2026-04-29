// 本ファイルは HTTP/JSON 互換 gateway の PubSub API end-to-end テスト。
//
// 検証する組み合わせ:
//   1. HTTPGateway: POST /k1s0/pubsub/publish のルート登録
//   2. MakeHTTPPubSubHandlers: protojson Unmarshal → in-process PubSubServiceServer
//   3. pubsubHandler: requireTenantID（NFR-E-AC-003）
//   4. dapr PubSubAdapter: L2 物理トピック prefix
//   5. inMemoryDapr backend: Publish 受理

package state

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
)

// startPubSubHTTPGateway は PubSub HTTP gateway を立てる test helper。
func startPubSubHTTPGateway(t *testing.T) (*httptest.Server, func()) {
	t.Helper()
	client := dapr.NewClientWithInMemoryBackends()
	deps := NewDepsFromClient(client)
	g := common.NewHTTPGateway()
	g.RegisterPubSubRoutes(MakeHTTPPubSubHandlers(NewPubSubServiceServer(deps)))
	srv := httptest.NewServer(g.Handler())
	return srv, srv.Close
}

// HTTP/JSON 経由で Publish が成功する。
func TestHTTPGateway_PubSub_Publish_OK(t *testing.T) {
	srv, cleanup := startPubSubHTTPGateway(t)
	defer cleanup()
	body := `{
		"topic": "k1s0.events.user-created",
		"data": "eyJ1IjoxfQ==",
		"content_type": "application/json",
		"context": {"tenant_id": "T-pub"}
	}`
	resp, err := http.Post(srv.URL+"/k1s0/pubsub/publish", "application/json", strings.NewReader(body))
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusOK {
		body, _ := readAllBody(resp)
		t.Fatalf("status = %d body=%s", resp.StatusCode, body)
	}
}

// tenant_id 不在は 400。
func TestHTTPGateway_PubSub_Publish_MissingTenantID_400(t *testing.T) {
	srv, cleanup := startPubSubHTTPGateway(t)
	defer cleanup()
	body := `{"topic":"x","data":"YQ==","context":{}}`
	resp, err := http.Post(srv.URL+"/k1s0/pubsub/publish", "application/json", strings.NewReader(body))
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("status = %d want 400", resp.StatusCode)
	}
}
