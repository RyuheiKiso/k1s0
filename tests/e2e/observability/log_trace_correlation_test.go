// 本ファイルは観測性 E2E の検証 3（log↔trace 結合率）。
// 設計正典: ADR-TEST-006（観測性 E2E を 5 検証で構造化）
// 関連 ADR: ADR-OBS-001（Grafana LGTM）/ ADR-OBS-002（OTel Collector）
//
// 検証対象（リリース時点での最小成立形）:
//   1. Loki HTTP API に疎通できる
//   2. label 名の取得 + 任意 LogQL クエリの実行が通る
//
// 結合率 ≥ 95% SLO の本格 assertion は採用初期で実装する。tier1 の structured logger が
// trace_id フィールドを構造化ログに含めて Loki に送る経路（ADR-OBS-002 の OTel Collector
// 経由）が運用稼働してから初めて意味のある SLO 計算ができるため、リリース時点では
// 「Loki が応答する + LogQL が動く」最小成立に留める。
//
// 前提:
//   K1S0_LOKI_HTTP_TARGET=http://localhost:3100
//   tools/local-stack/up.sh --observability で Loki が起動している。
package observability

import (
	"context"
	"encoding/json"
	"fmt"
	"net/url"
	"os"
	"testing"
	"time"
)

// lokiLabelsResponse は Loki /loki/api/v1/labels の応答。
type lokiLabelsResponse struct {
	// "success" / "error"
	Status string `json:"status"`
	// label name のリスト
	Data []string `json:"data"`
}

// lokiQueryResponse は Loki /loki/api/v1/query_range の応答。
type lokiQueryResponse struct {
	// "success" / "error"
	Status string `json:"status"`
	// query 結果（result type に応じた構造）
	Data struct {
		// "matrix" / "streams" / "vector" / "scalar"
		ResultType string `json:"resultType"`
		// 各 stream の labels と values
		Result []map[string]any `json:"result"`
	} `json:"data"`
}

// TestLokiLogTraceCorrelation は Loki HTTP API の最小疎通検証。
// 結合率 SLO ≥ 95% は採用初期で本格化、本リリースは「Loki が動く」最小確認に留める。
func TestLokiLogTraceCorrelation(t *testing.T) {
	// Loki HTTP API endpoint
	target := os.Getenv("K1S0_LOKI_HTTP_TARGET")
	if target == "" {
		t.Skip("K1S0_LOKI_HTTP_TARGET 未設定: tools/local-stack/up.sh --observability で起動した Loki HTTP endpoint を指定（例: http://localhost:3100）")
	}

	// 全体タイムアウト 30 秒
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	// Step 1: /loki/api/v1/labels で Loki 疎通 + label name 取得
	labelsURL := fmt.Sprintf("%s/loki/api/v1/labels", target)
	labelsResp := getPromJSON[lokiLabelsResponse](t, ctx, labelsURL)
	// status が success でなければ Loki 異常
	if labelsResp.Status != "success" {
		t.Fatalf("/loki/api/v1/labels: status=%s（success 以外は Loki 異常）", labelsResp.Status)
	}
	// label が 0 件 = どの app も log を Loki に送れていない
	// リリース時点で local-stack のいくつかの component（Argo CD / Backstage 等）が
	// 起動するだけで自動的に label が 1 つ以上記録される（job / namespace 等）
	if len(labelsResp.Data) == 0 {
		t.Fatalf("/loki/api/v1/labels: data 空（Loki が log を一切受信していない）")
	}
	t.Logf("/loki/api/v1/labels: %d 個の label name 取得（最初: %v）", len(labelsResp.Data), firstFew(labelsResp.Data, 5))

	// Step 2: /loki/api/v1/query_range で代表 LogQL を実行
	// `{job=~".+"}` は最も汎用的な LogQL で、何らかの job ラベルが付いたログを 1 つ以上
	// 取得できれば最小成立（cluster 上に動いている component が log を送っている証跡）
	now := time.Now()
	start := now.Add(-1 * time.Hour).UnixNano()
	end := now.UnixNano()
	queryParams := url.Values{}
	queryParams.Set("query", `{job=~".+"}`)
	queryParams.Set("start", fmt.Sprintf("%d", start))
	queryParams.Set("end", fmt.Sprintf("%d", end))
	queryParams.Set("limit", "10")
	queryURL := fmt.Sprintf("%s/loki/api/v1/query_range?%s", target, queryParams.Encode())
	queryResp := getPromJSON[lokiQueryResponse](t, ctx, queryURL)
	if queryResp.Status != "success" {
		t.Fatalf("/loki/api/v1/query_range: status=%s", queryResp.Status)
	}
	// resultType が想定外なら LogQL 仕様変更 / Loki bug
	if queryResp.Data.ResultType == "" {
		t.Fatalf("/loki/api/v1/query_range: resultType 空")
	}
	t.Logf("/loki/api/v1/query_range: resultType=%s result=%d streams", queryResp.Data.ResultType, len(queryResp.Data.Result))

	// Step 3: trace_id 結合率 SLO 計算は採用初期で本格化
	// 計算式: count_over_time({app=~".+"} | json | trace_id != "" [10m]) /
	//         count_over_time({app=~".+"} [10m]) >= 0.95
	// この計算が意味を持つには tier1 の structured logger が trace_id フィールドを
	// 出力する経路が運用稼働している必要があり、ADR-TEST-006 検証 3 の本格実装は
	// 採用初期で本テストを拡張する形で行う。
	// 本リリースでは Loki 疎通 + LogQL 動作の確認のみで「最小成立形」を満たす。
	t.Logf("trace_id 結合率 SLO（>= 95%%）の本格 assertion は採用初期で本テストを拡張（ADR-TEST-006 検証 3）")

	// 引数を未使用にしないため、json package を使った dummy 検証
	if _, err := json.Marshal(queryResp); err != nil {
		t.Fatalf("queryResp re-marshal: %v", err)
	}
}
