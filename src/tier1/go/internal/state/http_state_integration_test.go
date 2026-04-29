// 本ファイルは HTTP/JSON 互換 gateway の State API end-to-end テスト。
//
// 検証する組み合わせ:
//   1. HTTPGateway: POST /k1s0/state/get などのルート登録
//   2. MakeHTTPHandlers: protojson Unmarshal → in-process StateServiceServer 呼出
//   3. stateHandler: requireTenantID（NFR-E-AC-003）
//   4. dapr StateAdapter: L2 物理キー prefix（<tenant_id>/<key>）
//   5. inMemoryDapr backend: テナント分離 + First-Write-Wins
//
// 検証観点:
//   - Set → Get の HTTP/JSON round-trip が proto-equivalent な挙動をする
//   - tenant_id 不在は 400（InvalidArgument → BadRequest）
//   - 既存キーへの Set（ETag 空）は 409（AlreadyExists/Conflict）
//   - クロステナントは物理隔離される（adapter prefix）

package state

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
)

// startHTTPGateway は HTTPGateway + State route を bufconn 不要で立てる test helper。
func startHTTPGateway(t *testing.T) (*httptest.Server, func()) {
	t.Helper()
	client := dapr.NewClientWithInMemoryBackends()
	deps := NewDepsFromClient(client)
	g := common.NewHTTPGateway()
	g.RegisterStateRoutes(MakeHTTPHandlers(NewStateServiceServer(deps)))
	srv := httptest.NewServer(g.Handler())
	return srv, srv.Close
}

// HTTP/JSON 経由で Set → Get round-trip。
func TestHTTPGateway_State_RoundTrip(t *testing.T) {
	srv, cleanup := startHTTPGateway(t)
	defer cleanup()

	// Set
	setBody := `{
		"store": "valkey-default",
		"key": "session:abc",
		"data": "dXNlci0xMjM=",
		"context": {"tenant_id": "T-http"}
	}`
	resp, err := http.Post(srv.URL+"/k1s0/state/set", "application/json", strings.NewReader(setBody))
	if err != nil {
		t.Fatalf("Set POST: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		body, _ := readAllBody(resp)
		t.Fatalf("Set status = %d body=%s", resp.StatusCode, body)
	}
	_ = resp.Body.Close()

	// Get
	getBody := `{
		"store": "valkey-default",
		"key": "session:abc",
		"context": {"tenant_id": "T-http"}
	}`
	resp, err = http.Post(srv.URL+"/k1s0/state/get", "application/json", strings.NewReader(getBody))
	if err != nil {
		t.Fatalf("Get POST: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		body, _ := readAllBody(resp)
		t.Fatalf("Get status = %d body=%s", resp.StatusCode, body)
	}
	var got map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&got); err != nil {
		t.Fatalf("decode: %v", err)
	}
	_ = resp.Body.Close()
	// data は base64 で返る（protojson の bytes 規約）。"dXNlci0xMjM=" = "user-123"。
	if got["data"] != "dXNlci0xMjM=" {
		t.Errorf("data = %v want base64 of user-123", got["data"])
	}
}

// tenant_id 不在は 400（InvalidArgument）。
func TestHTTPGateway_State_MissingTenantID_400(t *testing.T) {
	srv, cleanup := startHTTPGateway(t)
	defer cleanup()
	body := `{"store":"valkey-default","key":"x","context":{}}`
	resp, err := http.Post(srv.URL+"/k1s0/state/get", "application/json", strings.NewReader(body))
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("status = %d want 400", resp.StatusCode)
	}
}

// 既存キーへの Set（ETag 空）は 409（First-Write-Wins）。
func TestHTTPGateway_State_FirstWriteConflict_409(t *testing.T) {
	srv, cleanup := startHTTPGateway(t)
	defer cleanup()

	body := `{"store":"valkey-default","key":"k1","data":"dg==","context":{"tenant_id":"T"}}`
	// 1 回目は OK。
	resp, _ := http.Post(srv.URL+"/k1s0/state/set", "application/json", strings.NewReader(body))
	_ = resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("first Set status = %d", resp.StatusCode)
	}
	// 2 回目（ETag 空）は 409 のはず。
	resp, _ = http.Post(srv.URL+"/k1s0/state/set", "application/json", strings.NewReader(body))
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusConflict {
		body, _ := readAllBody(resp)
		t.Fatalf("second Set status = %d body=%s want 409", resp.StatusCode, body)
	}
}

// 不正 JSON は 400。
func TestHTTPGateway_State_InvalidJSON_400(t *testing.T) {
	srv, cleanup := startHTTPGateway(t)
	defer cleanup()
	resp, err := http.Post(srv.URL+"/k1s0/state/get", "application/json", strings.NewReader("not json"))
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("status = %d want 400", resp.StatusCode)
	}
}

// クロステナント越境テスト: HTTP 経由でも物理 prefix で隔離される。
func TestHTTPGateway_State_CrossTenantIsolation(t *testing.T) {
	srv, cleanup := startHTTPGateway(t)
	defer cleanup()

	// tenant A が "shared" に "secret-A" を保存。
	bodyA := `{"store":"valkey-default","key":"shared","data":"c2VjcmV0LUE=","context":{"tenant_id":"A"}}`
	resp, _ := http.Post(srv.URL+"/k1s0/state/set", "application/json", strings.NewReader(bodyA))
	_ = resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("A Set: %d", resp.StatusCode)
	}
	// tenant B が同一論理キーに "secret-B" を保存（prefix で別 path になるので衝突しない）。
	bodyB := `{"store":"valkey-default","key":"shared","data":"c2VjcmV0LUI=","context":{"tenant_id":"B"}}`
	resp, _ = http.Post(srv.URL+"/k1s0/state/set", "application/json", strings.NewReader(bodyB))
	_ = resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("B Set: %d", resp.StatusCode)
	}
	// tenant A の Get は "secret-A"（B の値が見えない）。
	getA := `{"store":"valkey-default","key":"shared","context":{"tenant_id":"A"}}`
	resp, _ = http.Post(srv.URL+"/k1s0/state/get", "application/json", strings.NewReader(getA))
	defer func() { _ = resp.Body.Close() }()
	var got map[string]any
	_ = json.NewDecoder(resp.Body).Decode(&got)
	// "secret-A" の base64 = "c2VjcmV0LUE="。
	if got["data"] != "c2VjcmV0LUE=" {
		t.Errorf("A leak: got %v want secret-A", got["data"])
	}
}

// readAllBody is a tiny helper. We avoid io.ReadAll directly to keep imports small.
func readAllBody(resp *http.Response) (string, error) {
	defer func() { _ = resp.Body.Close() }()
	buf := new(bytes.Buffer)
	_, err := buf.ReadFrom(resp.Body)
	return buf.String(), err
}

// makeTenantCtx の共通 helper を本テストでも使う前提（既存ファイルが提供）。
// （placeholder: 既存テストファイルが makeTenantCtx を持つため再定義しない）
var _ = context.Background
