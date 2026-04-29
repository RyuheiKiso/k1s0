// 本ファイルは HTTP/JSON 互換 gateway の Secrets API end-to-end テスト。
//
// 検証する組み合わせ:
//   1. HTTPGateway: POST /k1s0/secrets/{get,rotate} のルート登録
//   2. MakeHTTPSecretsHandlers: protojson Unmarshal → in-process SecretsServiceServer
//   3. secretHandler: tenant_id 必須 + name 必須
//   4. openbao SecretsAdapter: L2 物理 prefix（<tenantID>/<name>）
//   5. InMemoryKV backend: secret 取得

package secret

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/openbao"
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
)

// startSecretsHTTPGateway は Secrets HTTP gateway を立てる test helper。
// in-memory OpenBao に seed 機能付きで返す。
func startSecretsHTTPGateway(t *testing.T) (*httptest.Server, *openbao.InMemoryKV, func()) {
	t.Helper()
	// kv インスタンスを直接生成して、Client / adapter / seed 経路で共有する。
	kv := openbao.NewInMemoryKV()
	client := openbao.NewClientFromInMemoryKV(kv)
	deps := Deps{
		SecretsAdapter: openbao.NewSecretsAdapter(client),
	}
	g := common.NewHTTPGateway()
	g.RegisterSecretsRoutes(MakeHTTPSecretsHandlers(NewSecretsServiceServer(deps)))
	srv := httptest.NewServer(g.Handler())
	return srv, kv, srv.Close
}

// HTTP/JSON 経由で Get が成功し、protojson 応答に values が入る。
func TestHTTPGateway_Secrets_Get_OK(t *testing.T) {
	srv, kv, cleanup := startSecretsHTTPGateway(t)
	defer cleanup()
	// adapter は path "T1/db" で Get するため、in-memory に同じ path で seed する。
	if _, err := kv.Put(t.Context(), "T1/db", map[string]any{"password": "p@ss"}); err != nil {
		t.Fatalf("seed: %v", err)
	}
	body := `{"name":"db","context":{"tenant_id":"T1"}}`
	resp, err := http.Post(srv.URL+"/k1s0/secrets/get", "application/json", strings.NewReader(body))
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusOK {
		buf := new(bytes.Buffer)
		_, _ = buf.ReadFrom(resp.Body)
		t.Fatalf("status = %d body=%s", resp.StatusCode, buf.String())
	}
	var got map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&got); err != nil {
		t.Fatalf("decode: %v", err)
	}
	values, ok := got["values"].(map[string]any)
	if !ok {
		t.Fatalf("values missing: %v", got)
	}
	if values["password"] != "p@ss" {
		t.Errorf("password = %v want p@ss", values["password"])
	}
}

// tenant_id 不在は 400。
func TestHTTPGateway_Secrets_Get_MissingTenantID_400(t *testing.T) {
	srv, _, cleanup := startSecretsHTTPGateway(t)
	defer cleanup()
	body := `{"name":"db","context":{}}`
	resp, err := http.Post(srv.URL+"/k1s0/secrets/get", "application/json", strings.NewReader(body))
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("status = %d want 400", resp.StatusCode)
	}
}

// クロステナント越境テスト: HTTP 経由でも secretPath で隔離される。
func TestHTTPGateway_Secrets_CrossTenantIsolation(t *testing.T) {
	srv, kv, cleanup := startSecretsHTTPGateway(t)
	defer cleanup()
	// tenant A と B の同一論理 name "db" を別 path で seed。
	if _, err := kv.Put(t.Context(), "A/db", map[string]any{"value": "secret-A"}); err != nil {
		t.Fatalf("seed A: %v", err)
	}
	if _, err := kv.Put(t.Context(), "B/db", map[string]any{"value": "secret-B"}); err != nil {
		t.Fatalf("seed B: %v", err)
	}
	// A の Get は "secret-A" を返す。
	resp, _ := http.Post(srv.URL+"/k1s0/secrets/get", "application/json",
		strings.NewReader(`{"name":"db","context":{"tenant_id":"A"}}`))
	defer func() { _ = resp.Body.Close() }()
	var got map[string]any
	_ = json.NewDecoder(resp.Body).Decode(&got)
	values, _ := got["values"].(map[string]any)
	if values["value"] != "secret-A" {
		t.Errorf("A leak: got %v want secret-A", values["value"])
	}
}
