// 本ファイルは tier3 BFF（portal-bff）と tier1 facade の round-trip 統合テスト。
//
// 設計正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//
// 検証目的:
//   tier3 BFF が tier1 facade（cmd/state）に gRPC で接続して
//   StateService.Get を呼び出し、in-memory backend を経由した
//   round-trip 応答を返せることを binary level で保証する。

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

// startPortalBFF は portal-bff を build & 起動し、HTTP base URL を返す。
func startPortalBFF(t *testing.T, k1s0Target string) (httpURL string, cleanup func()) {
	t.Helper()
	root := repoRoot(t)
	out := filepath.Join(t.TempDir(), "portal-bff")
	build := exec.Command("go", "build", "-o", out, "./cmd/portal-bff")
	build.Dir = filepath.Join(root, "src/tier3/bff")
	build.Env = append(os.Environ(), "CGO_ENABLED=0")
	if outBytes, err := build.CombinedOutput(); err != nil {
		t.Fatalf("go build portal-bff: %v\n%s", err, outBytes)
	}
	httpPort := findFreePort(t)
	cmd := exec.Command(out)
	cmd.Env = append(os.Environ(),
		fmt.Sprintf("HTTP_ADDR=127.0.0.1:%d", httpPort),
		"K1S0_TARGET="+k1s0Target,
		"K1S0_TENANT_ID=T-tier3",
		"K1S0_SUBJECT=tier3/portal-bff-test",
		"BFF_AUTH_MODE=off",
	)
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	if err := cmd.Start(); err != nil {
		t.Fatalf("start portal-bff: %v", err)
	}
	httpURL = fmt.Sprintf("http://127.0.0.1:%d", httpPort)

	// /readyz が 200 を返すまで wait。
	deadline := time.Now().Add(5 * time.Second)
	for time.Now().Before(deadline) {
		c, err := net.DialTimeout("tcp", fmt.Sprintf("127.0.0.1:%d", httpPort), 200*time.Millisecond)
		if err == nil {
			_ = c.Close()
			break
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

// portal-bff → tier1 facade の round-trip。
// in-memory state backend は seed なし → Get は found=false の正常応答。
// 重要なのは「BFF が tier1 を呼び出して proto serialization を含めて応答が返る」こと。
func TestPortalBFF_To_Tier1_StateGet(t *testing.T) {
	if testing.Short() {
		t.Skip("skip multi-binary test in -short mode")
	}
	tier1Addr, tier1Cleanup := startStatePodForTier2(t)
	defer tier1Cleanup()
	bffURL, bffCleanup := startPortalBFF(t, tier1Addr)
	defer bffCleanup()

	body := `{"store":"valkey-default","key":"missing-key"}`
	// off mode でも Bearer 必須。
	code, respBody := postJSONWithBearer(t, bffURL+"/api/state/get", body, "any")
	if code != http.StatusOK {
		t.Fatalf("/api/state/get: status=%d body=%s", code, respBody)
	}
	// 値なしなら "found":false が返る。tier1 round-trip 成立の証跡。
	if !strings.Contains(respBody, `"found":false`) {
		t.Errorf("expected found=false (empty in-memory KV), got: %s", respBody)
	}
}

// portal-bff: tier1 へ届かない場合は 502 Bad Gateway を返す。
func TestPortalBFF_Tier1Unreachable(t *testing.T) {
	if testing.Short() {
		t.Skip("skip multi-binary test in -short mode")
	}
	bffURL, bffCleanup := startPortalBFF(t, "127.0.0.1:1") // unreachable
	defer bffCleanup()

	body := `{"store":"x","key":"y"}`
	code, respBody := postJSONWithBearer(t, bffURL+"/api/state/get", body, "any")
	// SDK 接続失敗 → 502 BadGateway 期待。
	if code != http.StatusBadGateway {
		t.Errorf("unreachable tier1: status=%d body=%s want 502", code, respBody)
	}
}

// portal-bff: 認証必須経路は Bearer 不在時 401。
func TestPortalBFF_Auth_Required(t *testing.T) {
	if testing.Short() {
		t.Skip("skip multi-binary test in -short mode")
	}
	tier1Addr, tier1Cleanup := startStatePodForTier2(t)
	defer tier1Cleanup()
	bffURL, bffCleanup := startPortalBFF(t, tier1Addr)
	defer bffCleanup()

	// Bearer 不在は 401。
	code, _ := postJSON(t, bffURL+"/api/state/get", `{"store":"x","key":"y"}`)
	if code != http.StatusUnauthorized {
		t.Errorf("missing bearer: status=%d want 401", code)
	}
}
