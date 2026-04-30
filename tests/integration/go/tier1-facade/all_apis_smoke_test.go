// 本ファイルは tier1 全公開 API の binary level smoke 統合テスト。
//
// 設計正典:
//   docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/ 全 11 IDL
//
// 検証目的:
//   - cmd/state, cmd/secret, cmd/workflow の 3 binary が in-memory backend で起動し、
//     11 API ぶんの公開 RPC（HTTP/JSON gateway 経路）が「proto 検証 → adapter 実呼出
//     → 値返却」を最後まで通すことを保証する。
//   - 「正しく動かないものには価値はない」原則のため、placeholder Unimplemented を
//     返さず実値が返ることを最低限の goldens で確認する。
//
// 注:
//   server-streaming (PubSub.Subscribe / ServiceInvoke.InvokeStream / Audit.Export) は
//   gRPC 経路で別途検証する（src/tier1/go/internal/state/pubsub_inmemory_test.go ほか）。

package tier1facade

import (
	"context"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

// 各 binary の build & run helper。Pod 名と build 対象を引数で受ける。
func buildAndStart(t *testing.T, podName, target string) (httpURL string, cleanup func()) {
	t.Helper()
	root := repoRoot(t)
	out := filepath.Join(t.TempDir(), "k1s0-"+podName)
	build := exec.Command("go", "build", "-o", out, target)
	build.Dir = filepath.Join(root, "src/tier1/go")
	build.Env = append(os.Environ(), "CGO_ENABLED=0")
	if outBytes, err := build.CombinedOutput(); err != nil {
		t.Fatalf("go build %s failed: %v\n%s", target, err, outBytes)
	}

	grpcPort := findFreePort(t)
	httpPort := findFreePort(t)
	cmd := exec.Command(out,
		"-listen", fmt.Sprintf(":%d", grpcPort),
		"-http-listen", fmt.Sprintf("127.0.0.1:%d", httpPort),
	)
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	if err := cmd.Start(); err != nil {
		t.Fatalf("start %s: %v", podName, err)
	}
	httpURL = fmt.Sprintf("http://127.0.0.1:%d", httpPort)

	// HTTP gateway が ready になるまで poll する（最大 5 秒）。
	deadline := time.Now().Add(5 * time.Second)
	probePath := "/k1s0/state/get"
	if podName == "secret" {
		probePath = "/k1s0/secrets/get"
	} else if podName == "workflow" {
		probePath = "/k1s0/workflow/start"
	}
	for time.Now().Before(deadline) {
		resp, err := http.Post(httpURL+probePath, "application/json", strings.NewReader("{}"))
		if err == nil {
			_ = resp.Body.Close()
			break
		}
		time.Sleep(50 * time.Millisecond)
	}

	cleanup = func() {
		_ = cmd.Process.Signal(os.Interrupt)
		done := make(chan error, 1)
		go func() { done <- cmd.Wait() }()
		select {
		case <-done:
		case <-time.After(3 * time.Second):
			_ = cmd.Process.Kill()
			<-done
		}
	}
	return httpURL, cleanup
}

// postJSON は test 用の HTTP POST helper。
func postJSON(t *testing.T, url, body string) (int, string) {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost, url, strings.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("POST %s: %v", url, err)
	}
	defer func() { _ = resp.Body.Close() }()
	b, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(b)
}

// 起動 helper を秘密 / Workflow にも展開できるようにする。
func startSecretPod(t *testing.T) (httpURL string, cleanup func()) {
	return buildAndStart(t, "secret", "./cmd/secret")
}

func startWorkflowPod(t *testing.T) (httpURL string, cleanup func()) {
	return buildAndStart(t, "workflow", "./cmd/workflow")
}

// State Pod: BulkGet と Transact の HTTP 経路が機能することを確認する。
func TestStatePod_HTTPGateway_BulkGetAndTransact(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startStatePod(t)
	defer cleanup()

	// 1. Transact で 2 件 set する。
	transactBody := `{
		"store": "v",
		"operations": [
			{"set": {"key":"k1", "data":"djE="}},
			{"set": {"key":"k2", "data":"djI="}}
		],
		"context": {"tenant_id": "T-bulk"}
	}`
	if code, body := postJSON(t, httpURL+"/k1s0/state/transact", transactBody); code != http.StatusOK {
		t.Fatalf("Transact: %d %s", code, body)
	}

	// 2. BulkGet で 2 件まとめて取得する。
	bulkBody := `{
		"store": "v",
		"keys": ["k1", "k2", "k3"],
		"context": {"tenant_id": "T-bulk"}
	}`
	code, body := postJSON(t, httpURL+"/k1s0/state/bulkget", bulkBody)
	if code != http.StatusOK {
		t.Fatalf("BulkGet: %d %s", code, body)
	}
	// k1=v1 / k2=v2 のデータが含まれている。k3 は存在しないため空 data。
	if !strings.Contains(body, `"data":"djE="`) || !strings.Contains(body, `"data":"djI="`) {
		t.Errorf("BulkGet missing values: %s", body)
	}
}

// State Pod: PubSub HTTP Publish が in-memory backend で成功する。
func TestStatePod_HTTPGateway_PubSubPublish(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startStatePod(t)
	defer cleanup()

	body := `{
		"topic": "orders",
		"data": "ZGF0YQ==",
		"content_type": "application/octet-stream",
		"context": {"tenant_id": "T-pub"}
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/pubsub/publish", body); code != http.StatusOK {
		t.Fatalf("Publish: %d %s", code, b)
	}
}

// State Pod: Binding HTTP Invoke が in-memory backend で成功する。
func TestStatePod_HTTPGateway_BindingInvoke(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startStatePod(t)
	defer cleanup()

	body := `{
		"name": "smtp-out",
		"operation": "create",
		"data": "ZGF0YQ==",
		"context": {"tenant_id": "T-bind"}
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/binding/invoke", body); code != http.StatusOK {
		t.Fatalf("Binding.Invoke: %d %s", code, b)
	}
}

// State Pod: ServiceInvoke HTTP Invoke が echo（in-memory）で成功する。
func TestStatePod_HTTPGateway_ServiceInvoke(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startStatePod(t)
	defer cleanup()

	body := `{
		"app_id": "downstream",
		"method": "ping",
		"data": "aGVsbG8=",
		"content_type": "text/plain",
		"context": {"tenant_id": "T-inv"}
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/serviceinvoke/invoke", body); code != http.StatusOK {
		t.Fatalf("Invoke: %d %s", code, b)
	}
}

// State Pod: Log / Telemetry HTTP は stdout JSON Lines emitter 経由で成功する。
func TestStatePod_HTTPGateway_LogAndTelemetry(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startStatePod(t)
	defer cleanup()

	logBody := `{
		"entry": {
			"timestamp": "2026-04-29T00:00:00Z",
			"severity": "SEVERITY_INFO",
			"body": "hello",
			"attributes": {"k": "v"}
		},
		"context": {"tenant_id": "T-log"}
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/log/send", logBody); code != http.StatusOK {
		t.Fatalf("Log.Send: %d %s", code, b)
	}

	metricBody := `{
		"metrics": [{"name":"req_total","kind":"COUNTER","value":1.0,"labels":{"svc":"x"}}],
		"context": {"tenant_id": "T-tel"}
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/telemetry/emitmetric", metricBody); code != http.StatusOK {
		t.Fatalf("Telemetry.EmitMetric: %d %s", code, b)
	}

	spanBody := `{
		"spans": [{
			"trace_id":"00000000000000000000000000000001",
			"span_id":"0000000000000001",
			"name":"op",
			"start_time":"2026-04-29T00:00:00Z",
			"end_time":"2026-04-29T00:00:01Z"
		}],
		"context": {"tenant_id": "T-tel"}
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/telemetry/emitspan", spanBody); code != http.StatusOK {
		t.Fatalf("Telemetry.EmitSpan: %d %s", code, b)
	}
}

// State Pod: FeatureAdminService の RegisterFlag → GetFlag → ListFlags round-trip。
func TestStatePod_HTTPGateway_FeatureAdmin(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startStatePod(t)
	defer cleanup()

	regBody := `{
		"flag": {
			"flag_key": "T-feat.svc.experiment_a",
			"kind": "RELEASE",
			"value_type": "FLAG_VALUE_BOOLEAN",
			"default_variant": "off",
			"variants": {"off": false, "on": true},
			"state": "FLAG_STATE_ENABLED",
			"description": "test"
		},
		"change_reason": "initial",
		"context": {"tenant_id": "T-feat"}
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/feature/registerflag", regBody); code != http.StatusOK {
		t.Fatalf("RegisterFlag: %d %s", code, b)
	}

	getBody := `{
		"flag_key": "T-feat.svc.experiment_a",
		"context": {"tenant_id": "T-feat"}
	}`
	code, body := postJSON(t, httpURL+"/k1s0/feature/getflag", getBody)
	if code != http.StatusOK {
		t.Fatalf("GetFlag: %d %s", code, body)
	}
	// protojson は既定で camelCase を返す（"flag_key" → "flagKey"）。
	if !strings.Contains(body, `"flagKey":"T-feat.svc.experiment_a"`) {
		t.Errorf("GetFlag missing flagKey: %s", body)
	}

	listBody := `{"context": {"tenant_id": "T-feat"}}`
	code, body = postJSON(t, httpURL+"/k1s0/feature/listflags", listBody)
	if code != http.StatusOK {
		t.Fatalf("ListFlags: %d %s", code, body)
	}
	if !strings.Contains(body, "T-feat.svc.experiment_a") {
		t.Errorf("ListFlags missing registered flag: %s", body)
	}
}

// Secret Pod: in-memory KVv2 backend で Set 経路がないため、direct な Secret Get は
// 空応答（key not found）を返す。重要なのは binary が起動して HTTP で応答することと、
// tenant_id 必須が機能することの 2 点。
func TestSecretPod_HTTPGateway_Smoke(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startSecretPod(t)
	defer cleanup()

	// tenant_id 不在は 400。
	code, _ := postJSON(t, httpURL+"/k1s0/secrets/get",
		`{"name":"db.password","context":{}}`)
	if code != http.StatusBadRequest {
		t.Errorf("missing tenant_id: status=%d want 400", code)
	}

	// tenant_id 付き Get（in-memory KVv2 backend は seed なしのため NotFound 期待）。
	code, body := postJSON(t, httpURL+"/k1s0/secrets/get",
		`{"name":"db.password","context":{"tenant_id":"T-sec"}}`)
	if code == http.StatusInternalServerError {
		t.Fatalf("unexpected 500: %s", body)
	}
	// NotFound (404) または OK (200, 空データ) のどちらかを許容する。
	if code != http.StatusNotFound && code != http.StatusOK {
		t.Errorf("unexpected status: %d body=%s", code, body)
	}
}

// Workflow Pod: in-memory backend で Start / GetStatus round-trip が動作する。
func TestWorkflowPod_HTTPGateway_StartAndStatus(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startWorkflowPod(t)
	defer cleanup()

	startBody := `{
		"workflow_type": "OnboardTenant",
		"workflow_id": "wf-1",
		"input": "aW5pdA==",
		"context": {"tenant_id": "T-wf"}
	}`
	if code, b := postJSON(t, httpURL+"/k1s0/workflow/start", startBody); code != http.StatusOK {
		t.Fatalf("Start: %d %s", code, b)
	}

	statusBody := `{
		"workflow_id": "wf-1",
		"context": {"tenant_id": "T-wf"}
	}`
	code, body := postJSON(t, httpURL+"/k1s0/workflow/getstatus", statusBody)
	if code != http.StatusOK {
		t.Fatalf("GetStatus: %d %s", code, body)
	}
	// proto GetStatusResponse は run_id を持つ。in-memory backend は run_id を採番して返す。
	if !strings.Contains(body, `"runId"`) {
		t.Errorf("GetStatus body lacks runId: %s", body)
	}
}

// 共通: 全 3 Pod が並行起動可能で、各 HTTP gateway が listening するという起動保証 smoke。
func TestAllPods_StartConcurrent(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	pods := []struct {
		name    string
		starter func(*testing.T) (string, func())
		probe   string
		body    string
	}{
		{"state", startStatePod, "/k1s0/state/get", `{"context":{}}`},
		{"secret", startSecretPod, "/k1s0/secrets/get", `{"context":{}}`},
		{"workflow", startWorkflowPod, "/k1s0/workflow/start", `{"context":{}}`},
	}
	for _, p := range pods {
		p := p
		t.Run(p.name, func(t *testing.T) {
			t.Parallel()
			url, cleanup := p.starter(t)
			defer cleanup()
			// HTTP 応答が確認できる（status code は問わず、connection refused でないこと）。
			req, _ := http.NewRequest(http.MethodPost, url+p.probe, strings.NewReader(p.body))
			req.Header.Set("Content-Type", "application/json")
			resp, err := http.DefaultClient.Do(req)
			if err != nil {
				t.Fatalf("%s: %v", p.name, err)
			}
			_ = resp.Body.Close()
		})
	}
}

// dummy net.Conn ref keeps the package importable across compilers (vet sanity).
var _ = (net.Conn)(nil)
