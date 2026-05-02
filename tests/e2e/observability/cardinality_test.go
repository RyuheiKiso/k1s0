// 本ファイルは観測性 E2E の検証 2（Prometheus cardinality regression）。
// 設計正典: ADR-TEST-006（観測性 E2E を 5 検証で構造化）
// 関連 ADR: ADR-OBS-001（Grafana LGTM）/ ADR-OBS-002（OTel Collector）
//
// 検証対象（リリース時点での最小成立形）:
//   1. Prometheus HTTP API に疎通できる
//   2. labels endpoint から少なくとも 1 つの label name が取得できる
//   3. 各 metric の cardinality（label 組み合わせ数）が baseline 上限を超えていない
//
// baseline JSON（cardinality/baselines/<metric>.json）を版管理する設計は ADR-TEST-006 で
// 確定済だが、リリース時点では baseline JSON が未作成のため本実装では「baseline 不在なら
// 現状の cardinality を log 出力して採用初期で baseline 化する経路」に留める。
//
// 前提:
//   K1S0_PROMETHEUS_HTTP_TARGET=http://localhost:9090
//   tools/local-stack/up.sh --observability で Prometheus が起動している。
package observability

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"testing"
	"time"
)

// promLabelsResponse は Prometheus /api/v1/labels の応答型。
type promLabelsResponse struct {
	// "success" / "error" の文字列、success 以外は問題あり
	Status string `json:"status"`
	// label name のリスト
	Data []string `json:"data"`
}

// promSeriesResponse は Prometheus /api/v1/series の応答型。
type promSeriesResponse struct {
	// "success" / "error"
	Status string `json:"status"`
	// metric の label set リスト（各 element が key=value map）
	Data []map[string]string `json:"data"`
}

// TestPrometheusCardinality は Prometheus HTTP API に疎通し、metric が記録されていることを確認する。
// label 数 ≥ 1 + 代表 metric の cardinality 取得を assert。
func TestPrometheusCardinality(t *testing.T) {
	// Prometheus HTTP API の endpoint
	target := os.Getenv("K1S0_PROMETHEUS_HTTP_TARGET")
	if target == "" {
		t.Skip("K1S0_PROMETHEUS_HTTP_TARGET 未設定: tools/local-stack/up.sh --observability で起動した Prometheus の HTTP endpoint を指定（例: http://localhost:9090）")
	}

	// 全体タイムアウト 30 秒
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	// Step 1: /api/v1/labels で label name のリストを取得
	labelsURL := fmt.Sprintf("%s/api/v1/labels", target)
	labelsResp := getPromJSON[promLabelsResponse](t, ctx, labelsURL)
	// status が success でなければ Prometheus が異常
	if labelsResp.Status != "success" {
		t.Fatalf("/api/v1/labels: status=%s（success 以外は Prometheus 異常）", labelsResp.Status)
	}
	// label が 1 つも記録されていなければ scrape が一切動いていない
	if len(labelsResp.Data) == 0 {
		t.Fatalf("/api/v1/labels: data 空（Prometheus が metric を一切 scrape していない）")
	}
	t.Logf("/api/v1/labels: %d 個の label name 取得（最初: %v）", len(labelsResp.Data), firstFew(labelsResp.Data, 5))

	// Step 2: 代表 metric（up）の cardinality を取得して baseline 想定上限と比較
	// `up` は Prometheus の自前 metric（各 scrape target の up/down）で、必ず存在する
	const probeMetric = "up"
	seriesURL := fmt.Sprintf("%s/api/v1/series?match[]=%s", target, probeMetric)
	seriesResp := getPromJSON[promSeriesResponse](t, ctx, seriesURL)
	if seriesResp.Status != "success" {
		t.Fatalf("/api/v1/series?match[]=%s: status=%s", probeMetric, seriesResp.Status)
	}
	// `up` metric の cardinality（scrape target 数と等しい、最低 1）
	if len(seriesResp.Data) == 0 {
		t.Fatalf("/api/v1/series?match[]=%s: data 空（scrape target なし）", probeMetric)
	}
	t.Logf("metric=%s cardinality=%d", probeMetric, len(seriesResp.Data))

	// Step 3: cardinality 上限の baseline 比較
	// 採用初期で tests/e2e/observability/cardinality/baselines/<metric>.json を整備し、
	// 1.2 倍超過で fail させる本格 regression test に拡張する。本リリースでは
	// ADR-TEST-006 検証 2 の最小成立形として「現状の cardinality 値を log 化」する。
	t.Logf("baseline 比較は採用初期で baselines/<metric>.json 整備後に有効化（ADR-TEST-006 検証 2）")
}

// getPromJSON は HTTP GET → JSON parse を 1 関数化したヘルパ。
// generic で response 型を受けて型安全に decode する。
func getPromJSON[T any](t *testing.T, ctx context.Context, url string) T {
	t.Helper()
	// HTTP GET request
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		t.Fatalf("http.NewRequest(%s): %v", url, err)
	}
	// 5 秒の timeout で 1 リクエスト
	client := &http.Client{Timeout: 10 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		t.Fatalf("http.Do(%s): %v", url, err)
	}
	defer func() { _ = resp.Body.Close() }()
	// 200 以外は Prometheus 異常 / endpoint 不正
	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		t.Fatalf("GET %s: status=%d body=%s", url, resp.StatusCode, string(body))
	}
	// body 全体を読み取る
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("read body: %v", err)
	}
	// JSON parse
	var out T
	if err := json.Unmarshal(body, &out); err != nil {
		t.Fatalf("JSON parse(%s): %v body=%s", url, err, string(body))
	}
	return out
}

// firstFew は slice の先頭 n 要素を返す（log に長いリストを出さないための切り詰め）。
func firstFew[T any](s []T, n int) []T {
	if len(s) <= n {
		return s
	}
	return s[:n]
}
