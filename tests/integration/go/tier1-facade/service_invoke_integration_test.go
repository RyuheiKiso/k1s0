// 本ファイルは tier1 facade の binary レベル結合テスト。
//
// 設計正典:
//   docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md
//
// テスト戦略:
//   `go build` で tier1/go の cmd/state バイナリを生成し、free port 上で起動して
//   gRPC + HTTP/JSON gateway の両経路を実バイナリレベルで検証する。
//   testcontainers + Dapr sidecar を使う重い経路ではなく、process-level の smoke
//   integration として位置付け（dev / CI のローカル実行で全部回せる軽量テスト）。
//
// 検証する組み合わせ:
//   1. cmd/state binary が gRPC + HTTP gateway を listen する
//   2. HTTP/JSON 経由で Set → Get round-trip
//   3. tenant_id 不在は 400
//   4. Dapr in-memory backend で State API が実値を返す
//
// 実 Dapr / Keycloak 結合テストは別 PR で kind cluster + Helm chart deploy を介して行う。

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

// findFreePort は OS から空いている TCP port を 1 つ確保して返す。
// listener を即 close するため、戻ってきた port は短時間後に他 process に取られる
// 可能性があるが、テストでは即起動するため race は実用上無視できる。
func findFreePort(t *testing.T) int {
	t.Helper()
	lis, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("listen 0: %v", err)
	}
	port := lis.Addr().(*net.TCPAddr).Port
	_ = lis.Close()
	return port
}

// repoRoot は本テストファイルから 4 階層上（リポジトリルート）を返す。
// tests/integration/go/tier1-facade/X_test.go → ../../../..
func repoRoot(t *testing.T) string {
	t.Helper()
	wd, err := os.Getwd()
	if err != nil {
		t.Fatalf("getwd: %v", err)
	}
	return filepath.Clean(filepath.Join(wd, "..", "..", "..", ".."))
}

// buildStateBinary は src/tier1/go/cmd/state を一時 path に build する。
// 失敗時は test を Fatalf で終了する（CI で go toolchain が無いと skip 候補）。
func buildStateBinary(t *testing.T) string {
	t.Helper()
	root := repoRoot(t)
	out := filepath.Join(t.TempDir(), "k1s0-state")
	cmd := exec.Command("go", "build", "-o", out, "./cmd/state")
	cmd.Dir = filepath.Join(root, "src/tier1/go")
	cmd.Env = append(os.Environ(), "CGO_ENABLED=0")
	if outBytes, err := cmd.CombinedOutput(); err != nil {
		t.Fatalf("go build cmd/state failed: %v\n%s", err, outBytes)
	}
	return out
}

// startStatePod は build 済バイナリを起動し、HTTP gateway が ready になるまで待つ。
// cleanup 関数で graceful shutdown する。
func startStatePod(t *testing.T) (httpURL string, cleanup func()) {
	t.Helper()
	bin := buildStateBinary(t)
	grpcPort := findFreePort(t)
	httpPort := findFreePort(t)
	cmd := exec.Command(bin,
		"-listen", fmt.Sprintf(":%d", grpcPort),
		"-http-listen", fmt.Sprintf("127.0.0.1:%d", httpPort),
	)
	// stderr / stdout を test ログに流す（debug 容易性）。
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	if err := cmd.Start(); err != nil {
		t.Fatalf("start binary: %v", err)
	}
	httpURL = fmt.Sprintf("http://127.0.0.1:%d", httpPort)

	// HTTP gateway が ready になるまで poll する（最大 5 秒）。
	deadline := time.Now().Add(5 * time.Second)
	for time.Now().Before(deadline) {
		// 任意の不正リクエスト（empty body）でも HTTP server が応答すれば 4xx を返す。
		resp, err := http.Post(httpURL+"/k1s0/state/get", "application/json", strings.NewReader("{}"))
		if err == nil {
			_ = resp.Body.Close()
			break
		}
		time.Sleep(50 * time.Millisecond)
	}
	cleanup = func() {
		_ = cmd.Process.Signal(os.Interrupt)
		// graceful shutdown（最大 3 秒待つ）。
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

// build → 起動 → HTTP/JSON 経由で Set → Get round-trip。
// in-memory Dapr backend を使うため、外部依存（実 Dapr / Valkey）なしで実行可能。
func TestStatePod_HTTPGateway_RoundTrip(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startStatePod(t)
	defer cleanup()

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// 1. Set: 新規キーを保存する。
	setBody := `{
		"store": "valkey-default",
		"key": "session:integration",
		"data": "aW50LXY=",
		"context": {"tenant_id": "T-int"}
	}`
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost, httpURL+"/k1s0/state/set",
		strings.NewReader(setBody))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("Set POST: %v", err)
	}
	body, _ := io.ReadAll(resp.Body)
	_ = resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("Set status = %d body=%s", resp.StatusCode, body)
	}

	// 2. Get: 直前の値が返る。
	getBody := `{
		"store": "valkey-default",
		"key": "session:integration",
		"context": {"tenant_id": "T-int"}
	}`
	req2, _ := http.NewRequestWithContext(ctx, http.MethodPost, httpURL+"/k1s0/state/get",
		strings.NewReader(getBody))
	req2.Header.Set("Content-Type", "application/json")
	resp2, err := http.DefaultClient.Do(req2)
	if err != nil {
		t.Fatalf("Get POST: %v", err)
	}
	body2, _ := io.ReadAll(resp2.Body)
	_ = resp2.Body.Close()
	if resp2.StatusCode != http.StatusOK {
		t.Fatalf("Get status = %d body=%s", resp2.StatusCode, body2)
	}
	// data は base64 で返る（"aW50LXY=" = "int-v"）。
	if !strings.Contains(string(body2), `"data":"aW50LXY="`) {
		t.Errorf("Get response missing data: %s", body2)
	}
}

// tenant_id 不在は 400 を返す（binary level でも防御層が機能している）。
func TestStatePod_HTTPGateway_TenantIDRequired(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startStatePod(t)
	defer cleanup()

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost, httpURL+"/k1s0/state/get",
		strings.NewReader(`{"store":"x","key":"y","context":{}}`))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("status = %d want 400", resp.StatusCode)
	}
}

// クロステナント越境（binary level）: HTTP 経由でも L2 物理 prefix で隔離される。
func TestStatePod_HTTPGateway_CrossTenantIsolation(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	httpURL, cleanup := startStatePod(t)
	defer cleanup()

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	post := func(path, body string) (int, []byte) {
		req, _ := http.NewRequestWithContext(ctx, http.MethodPost, httpURL+path,
			strings.NewReader(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := http.DefaultClient.Do(req)
		if err != nil {
			t.Fatalf("POST %s: %v", path, err)
		}
		defer func() { _ = resp.Body.Close() }()
		b, _ := io.ReadAll(resp.Body)
		return resp.StatusCode, b
	}
	// tenant A が "shared" に "secret-A" を保存（base64 "c2VjcmV0LUE="）。
	if code, body := post("/k1s0/state/set",
		`{"store":"v","key":"shared","data":"c2VjcmV0LUE=","context":{"tenant_id":"A"}}`); code != http.StatusOK {
		t.Fatalf("A Set: %d %s", code, body)
	}
	// tenant B が同一論理キーに "secret-B"（base64 "c2VjcmV0LUI="）を保存。
	if code, body := post("/k1s0/state/set",
		`{"store":"v","key":"shared","data":"c2VjcmV0LUI=","context":{"tenant_id":"B"}}`); code != http.StatusOK {
		t.Fatalf("B Set: %d %s", code, body)
	}
	// tenant A の Get は "secret-A"（base64 "c2VjcmV0LUE="）。
	code, body := post("/k1s0/state/get",
		`{"store":"v","key":"shared","context":{"tenant_id":"A"}}`)
	if code != http.StatusOK {
		t.Fatalf("A Get: %d %s", code, body)
	}
	if !strings.Contains(string(body), `"data":"c2VjcmV0LUE="`) {
		t.Errorf("A leak: response=%s", body)
	}
	// tenant B の Get は "secret-B"。
	code, body = post("/k1s0/state/get",
		`{"store":"v","key":"shared","context":{"tenant_id":"B"}}`)
	if code != http.StatusOK {
		t.Fatalf("B Get: %d %s", code, body)
	}
	if !strings.Contains(string(body), `"data":"c2VjcmV0LUI="`) {
		t.Errorf("B leak: response=%s", body)
	}
}
