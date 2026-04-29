// 本ファイルは tier2 example services（notification-hub / stock-reconciler）と
// tier1 facade の round-trip 統合テスト。
//
// 設計正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 検証目的:
//   tier2 サービスが tier1 facade（cmd/state）に gRPC で接続して RPC を呼び出し、
//   in-memory backend を経由した round-trip 応答を返せることを binary level で保証する。

package tier1facade

import (
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

// startStatePodCustomEnv は cmd/state を任意の env で起動し、gRPC port と HTTP URL を返す。
// httpEnabled=true なら HTTP gateway も同時に起動する（このテストでは false）。
func startStatePodForTier2(t *testing.T) (grpcAddr string, cleanup func()) {
	t.Helper()
	root := repoRoot(t)
	out := filepath.Join(t.TempDir(), "k1s0-state")
	build := exec.Command("go", "build", "-o", out, "./cmd/state")
	build.Dir = filepath.Join(root, "src/tier1/go")
	build.Env = append(os.Environ(), "CGO_ENABLED=0")
	if outBytes, err := build.CombinedOutput(); err != nil {
		t.Fatalf("go build cmd/state: %v\n%s", err, outBytes)
	}
	port := findFreePort(t)
	cmd := exec.Command(out,
		"-listen", fmt.Sprintf("127.0.0.1:%d", port),
		"-http-listen", "off",
	)
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	if err := cmd.Start(); err != nil {
		t.Fatalf("start state: %v", err)
	}
	grpcAddr = fmt.Sprintf("127.0.0.1:%d", port)

	// gRPC port が listen するまで wait（直接 dial で確認）。
	deadline := time.Now().Add(5 * time.Second)
	for time.Now().Before(deadline) {
		c, err := net.DialTimeout("tcp", grpcAddr, 200*time.Millisecond)
		if err == nil {
			_ = c.Close()
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
	return grpcAddr, cleanup
}

// startNotificationHub は notification-hub を build & 起動し、HTTP base URL を返す。
func startNotificationHub(t *testing.T, k1s0Target string) (httpURL string, cleanup func()) {
	t.Helper()
	root := repoRoot(t)
	out := filepath.Join(t.TempDir(), "notification-hub")
	build := exec.Command("go", "build", "-o", out, "./services/notification-hub/cmd")
	build.Dir = filepath.Join(root, "src/tier2/go")
	build.Env = append(os.Environ(), "CGO_ENABLED=0")
	if outBytes, err := build.CombinedOutput(); err != nil {
		t.Fatalf("go build notification-hub: %v\n%s", err, outBytes)
	}
	httpPort := findFreePort(t)
	cmd := exec.Command(out)
	cmd.Env = append(os.Environ(),
		fmt.Sprintf("HTTP_ADDR=127.0.0.1:%d", httpPort),
		"K1S0_TARGET="+k1s0Target,
		"K1S0_TENANT_ID=T-tier2",
		"K1S0_SUBJECT=tier2/notification-hub-test",
		"T2_AUTH_MODE=off",
		// 各 channel に任意の Binding Component 名を割当てる（in-memory backend は no-op success）。
		"NOTIFY_BINDING_EMAIL=smtp-outbound",
		"NOTIFY_BINDING_SLACK=slack-webhook",
		"NOTIFY_BINDING_WEBHOOK=http-outbound",
	)
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	if err := cmd.Start(); err != nil {
		t.Fatalf("start notification-hub: %v", err)
	}
	httpURL = fmt.Sprintf("http://127.0.0.1:%d", httpPort)

	// /readyz が 200 を返すまで wait。
	deadline := time.Now().Add(5 * time.Second)
	for time.Now().Before(deadline) {
		resp, err := http.Get(httpURL + "/readyz")
		if err == nil {
			_ = resp.Body.Close()
			if resp.StatusCode == http.StatusOK {
				break
			}
		}
		time.Sleep(80 * time.Millisecond)
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

// postJSONWithBearer は Bearer ヘッダ付きで POST する helper。
func postJSONWithBearer(t *testing.T, url, body, token string) (int, string) {
	t.Helper()
	req, err := http.NewRequest(http.MethodPost, url, strings.NewReader(body))
	if err != nil {
		t.Fatalf("NewRequest: %v", err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+token)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("POST %s: %v", url, err)
	}
	defer func() { _ = resp.Body.Close() }()
	b, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(b)
}

// notification-hub → tier1 facade の round-trip。
// in-memory binding adapter が echo で受理するため、HTTP 200 / success が返る。
func TestNotificationHub_To_Tier1_RoundTrip(t *testing.T) {
	if testing.Short() {
		t.Skip("skip multi-binary test in -short mode")
	}
	tier1Addr, tier1Cleanup := startStatePodForTier2(t)
	defer tier1Cleanup()
	hubURL, hubCleanup := startNotificationHub(t, tier1Addr)
	defer hubCleanup()

	body := `{
		"channel": "email",
		"recipient": "user@example.com",
		"subject": "test",
		"body": "hello",
		"metadata": {"trace_id": "tier2-e2e"}
	}`
	// off mode でも Bearer 必須（middleware 仕様）。値は何でも良い。
	code, respBody := postJSONWithBearer(t, hubURL+"/notify", body, "any")
	if code != http.StatusOK {
		t.Fatalf("/notify: status=%d body=%s", code, respBody)
	}
	// 成功時は notification_id, success=true が返る。
	if !strings.Contains(respBody, `"success":true`) {
		t.Errorf("expected success=true, got: %s", respBody)
	}
	if !strings.Contains(respBody, `"binding_name":"smtp-outbound"`) {
		t.Errorf("expected binding_name=smtp-outbound, got: %s", respBody)
	}
}

// notification-hub: tier1 facade が起動していない / unreachable な場合の挙動確認。
func TestNotificationHub_Tier1Unreachable(t *testing.T) {
	if testing.Short() {
		t.Skip("skip multi-binary test in -short mode")
	}
	// 存在しない target を指定する。
	hubURL, hubCleanup := startNotificationHub(t, "127.0.0.1:1") // port 1 = unreachable
	defer hubCleanup()

	body := `{"channel":"email","recipient":"x@y.z","subject":"s","body":"b"}`
	code, respBody := postJSONWithBearer(t, hubURL+"/notify", body, "any")
	// tier1 unreachable → SDK Connect タイムアウト → 5xx 期待。
	if code/100 != 5 && code/100 != 4 {
		t.Errorf("unreachable tier1: status=%d body=%s want 4xx/5xx", code, respBody)
	}
	// notification-hub プロセス自体は健全であること（/readyz 200）。
	resp, err := http.Get(hubURL + "/readyz")
	if err != nil {
		t.Fatalf("readyz: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusOK {
		t.Errorf("readyz after failed tier1 call: %d", resp.StatusCode)
	}
}
