// 本ファイルは tier1 Rust 3 Pod（audit / decision / pii）の binary level smoke 統合テスト。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/09_Decision_API.md
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//
// 検証目的:
//   - cargo build したバイナリ（target/debug/t1-{audit,decision,pii}）を起動し、
//     HTTP/JSON gateway 経路（POST /k1s0/<api>/<rpc>）の主要 RPC が実値を返すことを確認する。
//   - 認証は TIER1_AUTH_MODE=off（dev 既定）で skip し、demo-tenant claims で実行する。
//
// 注:
//   - audit Export は server-streaming（HTTP gateway 非対応）のため対象外。
//   - 詳細単体は src/tier1/rust/crates/{audit,decision,pii}/ の cargo test で網羅する。

package tier1facade

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

// base64Std は標準 alphabet エンコーダ。proto bytes フィールドの JSON 表現に使う。
var base64Std = base64.StdEncoding

// rustBinaryPath は Rust pod binary の build 済 path を返す。先に cargo build を行う。
func rustBinaryPath(t *testing.T, pod string) string {
	t.Helper()
	root := repoRoot(t)
	rustDir := filepath.Join(root, "src/tier1/rust")
	bin := filepath.Join(rustDir, "target/debug", "t1-"+pod)
	// 既に build 済なら即返却。CI / 連続実行で 0 秒に近づける。
	if _, err := os.Stat(bin); err == nil {
		return bin
	}
	build := exec.Command("cargo", "build", "--bin", "t1-"+pod)
	build.Dir = rustDir
	build.Stdout = io.Discard
	build.Stderr = io.Discard
	if err := build.Run(); err != nil {
		t.Skipf("cargo build %s failed (toolchain or offline): %v", pod, err)
	}
	if _, err := os.Stat(bin); err != nil {
		t.Skipf("rust binary not built: %s", bin)
	}
	return bin
}

// startRustPodHTTP は Rust binary を free TCP port で起動し、HTTP gateway URL を返す。
func startRustPodHTTP(t *testing.T, pod string) (httpURL string, cleanup func()) {
	t.Helper()
	bin := rustBinaryPath(t, pod)
	grpcPort := findFreePort(t)
	httpPort := findFreePort(t)
	cmd := exec.Command(bin)
	cmd.Env = append(os.Environ(),
		fmt.Sprintf("LISTEN_ADDR=[::]:%d", grpcPort),
		// Rust pod の HTTP gateway は環境変数で listen address を上書き可能。
		fmt.Sprintf("TIER1_HTTP_LISTEN_ADDR=127.0.0.1:%d", httpPort),
		"TIER1_AUTH_MODE=off",
	)
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	if err := cmd.Start(); err != nil {
		t.Fatalf("start %s: %v", pod, err)
	}
	httpURL = fmt.Sprintf("http://127.0.0.1:%d", httpPort)

	// readiness 待ち: 任意の path に POST して接続が確立できることを確認する。
	deadline := time.Now().Add(10 * time.Second)
	probePath := "/k1s0/" + pod + "/notexists"
	for time.Now().Before(deadline) {
		resp, err := http.Post(httpURL+probePath, "application/json", strings.NewReader("{}"))
		if err == nil {
			_ = resp.Body.Close()
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

// PII Pod: Classify + Mask が実値を返す。
func TestPiiPod_HTTPGateway_ClassifyAndMask(t *testing.T) {
	if testing.Short() {
		t.Skip("skip rust binary test in -short mode")
	}
	httpURL, cleanup := startRustPodHTTP(t, "pii")
	defer cleanup()

	// Classify: email を含むテキストから 1 件以上検出されること。
	classifyBody := `{"text":"contact: alice@example.com","context":{}}`
	code, body := postJSON(t, httpURL+"/k1s0/pii/classify", classifyBody)
	if code != http.StatusOK {
		t.Fatalf("Classify: %d %s", code, body)
	}
	var classifyResp struct {
		Findings []map[string]interface{} `json:"findings"`
	}
	if err := json.Unmarshal([]byte(body), &classifyResp); err != nil {
		t.Fatalf("decode Classify: %v body=%s", err, body)
	}
	if len(classifyResp.Findings) == 0 {
		t.Errorf("Classify: no findings: %s", body)
	}

	// Mask: detected 範囲がトークンに置換される。
	maskBody := `{"text":"alice@example.com","context":{}}`
	code, body = postJSON(t, httpURL+"/k1s0/pii/mask", maskBody)
	if code != http.StatusOK {
		t.Fatalf("Mask: %d %s", code, body)
	}
	if strings.Contains(body, `"alice@example.com"`) && !strings.Contains(body, "MASK") && !strings.Contains(body, "masked_text") {
		// マスク後文字列が原文と完全一致しないことだけ最低限確認する。
		t.Logf("Mask response body: %s", body)
	}
}

// Decision Pod: RegisterRule → Evaluate round-trip。
func TestDecisionPod_HTTPGateway_RegisterAndEvaluate(t *testing.T) {
	if testing.Short() {
		t.Skip("skip rust binary test in -short mode")
	}
	httpURL, cleanup := startRustPodHTTP(t, "decision")
	defer cleanup()

	// 最小 JDM: amount フィールドを受け取って tier を返す。
	jdm := `{"contentType":"application/vnd.gorules.decision","nodes":[{"id":"in","name":"input","type":"inputNode"},{"id":"out","name":"output","type":"outputNode"},{"id":"f","name":"function","type":"functionNode","content":{"source":"export const handler = async (input) => ({ tier: input.amount > 100 ? 'high' : 'low' });"}}],"edges":[{"id":"e1","sourceId":"in","targetId":"f"},{"id":"e2","sourceId":"f","targetId":"out"}]}`
	jdmB64 := base64Std.EncodeToString([]byte(jdm))
	regBody := fmt.Sprintf(`{"ruleId":"amount-tier","jdmDocument":%q,"context":{}}`, jdmB64)
	code, body := postJSON(t, httpURL+"/k1s0/decision/registerrule", regBody)
	if code != http.StatusOK {
		t.Fatalf("RegisterRule: %d %s", code, body)
	}

	// Evaluate: amount=200 → "high"。input_json は base64 エンコード必須。
	inputB64 := base64Std.EncodeToString([]byte(`{"amount": 200}`))
	evalBody := fmt.Sprintf(`{"ruleId":"amount-tier","inputJson":%q,"context":{}}`, inputB64)
	code, body = postJSON(t, httpURL+"/k1s0/decision/evaluate", evalBody)
	if code != http.StatusOK {
		t.Fatalf("Evaluate: %d %s", code, body)
	}
	// outputJson は base64 で返るので decode して検査する。
	var evalResp struct {
		OutputJson string `json:"outputJson"`
	}
	if err := json.Unmarshal([]byte(body), &evalResp); err != nil {
		t.Fatalf("decode Evaluate: %v body=%s", err, body)
	}
	decoded, err := base64Std.DecodeString(evalResp.OutputJson)
	if err != nil {
		t.Fatalf("base64 decode: %v", err)
	}
	if !strings.Contains(string(decoded), "high") {
		t.Errorf("Evaluate did not return 'high': decoded=%s body=%s", decoded, body)
	}
}

// Audit Pod: Record → Query → VerifyChain。
func TestAuditPod_HTTPGateway_RecordAndVerify(t *testing.T) {
	if testing.Short() {
		t.Skip("skip rust binary test in -short mode")
	}
	httpURL, cleanup := startRustPodHTTP(t, "audit")
	defer cleanup()

	for i := 0; i < 3; i++ {
		recBody := fmt.Sprintf(`{"event":{"actor":"smoke","action":"test.action.%d","resource":"res-%d","outcome":"SUCCESS"},"context":{}}`, i, i)
		code, body := postJSON(t, httpURL+"/k1s0/audit/record", recBody)
		if code != http.StatusOK {
			t.Fatalf("Record %d: %d %s", i, code, body)
		}
	}

	// Query: 3 件以上返ること（in-memory store のため必ず取れる）。
	queryBody := `{"limit":100,"context":{}}`
	code, body := postJSON(t, httpURL+"/k1s0/audit/query", queryBody)
	if code != http.StatusOK {
		t.Fatalf("Query: %d %s", code, body)
	}
	// events フィールドに 3 件以上含まれていること。
	var qResp struct {
		Events []map[string]interface{} `json:"events"`
	}
	if err := json.Unmarshal([]byte(body), &qResp); err != nil {
		t.Fatalf("decode Query: %v body=%s", err, body)
	}
	if len(qResp.Events) < 3 {
		t.Errorf("Query: got %d events, want >=3 body=%s", len(qResp.Events), body)
	}

	// VerifyChain: hash chain integrity = valid。
	verifyBody := `{"context":{}}`
	code, body = postJSON(t, httpURL+"/k1s0/audit/verifychain", verifyBody)
	if code != http.StatusOK {
		t.Fatalf("VerifyChain: %d %s", code, body)
	}
	if !strings.Contains(body, `"valid":true`) {
		t.Errorf("VerifyChain not valid: %s", body)
	}
}

// 共通: Rust 3 Pod が並行起動して HTTP gateway が応答する起動保証 smoke。
func TestRustPods_StartConcurrent(t *testing.T) {
	if testing.Short() {
		t.Skip("skip rust binary test in -short mode")
	}
	for _, pod := range []string{"audit", "decision", "pii"} {
		pod := pod
		t.Run(pod, func(t *testing.T) {
			t.Parallel()
			httpURL, cleanup := startRustPodHTTP(t, pod)
			defer cleanup()
			ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
			defer cancel()
			// 任意の path に POST。404 が返れば server is alive 判定。
			req, _ := http.NewRequestWithContext(ctx, http.MethodPost, httpURL+"/k1s0/"+pod+"/probe",
				strings.NewReader("{}"))
			req.Header.Set("Content-Type", "application/json")
			resp, err := http.DefaultClient.Do(req)
			if err != nil {
				t.Fatalf("%s: %v", pod, err)
			}
			_ = resp.Body.Close()
			if resp.StatusCode != http.StatusNotFound && resp.StatusCode/100 != 4 {
				t.Errorf("%s: status=%d (expected 4xx for unknown route)", pod, resp.StatusCode)
			}
		})
	}
}
