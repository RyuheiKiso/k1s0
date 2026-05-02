// 本ファイルは観測性 E2E の検証 3（log↔trace 結合率の最小成立形）。
// 設計正典: ADR-TEST-006（観測性 E2E を 5 検証で構造化）
// 関連 ADR: ADR-OBS-001（Grafana LGTM）/ ADR-OBS-002（OTel Collector）
//
// 検証対象（リリース時点での最小成立形）:
//   1. OTLP HTTP /v1/logs で log を OTel Collector へ送信できる
//   2. OTel Collector が otlphttp/loki exporter で Loki へ転送する
//   3. Loki HTTP API（/loki/api/v1/query_range）で送信した log を取得できる
//   4. 取得した log line に同 record の trace_id 文字列が含まれる
//
// 結合率 ≥ 95% SLO の本格 assertion は採用初期で本テストを拡張する形で行う。
// リリース時点では「log と trace が同一 trace_id で結合できる経路」の最小確認に留める。
//
// 当初 promtail 経由で k8s log を Loki に送る経路だったが、promtail Helm chart が
// deprecated + nested fs の inotify 制約で CrashLoopBackOff、log 経路が機能しない問題を
// 実観測した。代替として OTel Collector の otlphttp/loki exporter を主経路に切替えた。
//
// 前提:
//   K1S0_OTLP_HTTP_TARGET=http://localhost:4318  OTel Collector の OTLP HTTP endpoint
//   K1S0_LOKI_HTTP_TARGET=http://localhost:3100  Loki HTTP endpoint
package observability

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"strings"
	"testing"
	"time"
)

// lokiQueryResponse は Loki /loki/api/v1/query_range の応答。
type lokiQueryResponse struct {
	// "success" / "error"
	Status string `json:"status"`
	// query 結果（result type に応じた構造）
	Data struct {
		// "matrix" / "streams" / "vector" / "scalar"
		ResultType string `json:"resultType"`
		// 各 stream の labels と values
		Result []struct {
			// stream の label set
			Stream map[string]string `json:"stream"`
			// 値 [timestamp_ns_string, log_line_string][] のペア
			Values [][]string `json:"values"`
		} `json:"result"`
	} `json:"data"`
}

// TestLokiLogTraceCorrelation は OTLP HTTP /v1/logs → Loki の貫通検証。
// ID/trace_id を埋めた log を送信し、Loki で取得した上で trace_id を確認する。
func TestLokiLogTraceCorrelation(t *testing.T) {
	// OTel Collector の OTLP HTTP endpoint
	otlpHTTP := os.Getenv("K1S0_OTLP_HTTP_TARGET")
	if otlpHTTP == "" {
		t.Skip("K1S0_OTLP_HTTP_TARGET 未設定: OTel Collector の OTLP HTTP endpoint を指定（例: http://localhost:4318）")
	}
	// Loki HTTP API endpoint
	lokiTarget := os.Getenv("K1S0_LOKI_HTTP_TARGET")
	if lokiTarget == "" {
		t.Skip("K1S0_LOKI_HTTP_TARGET 未設定: Loki HTTP endpoint を指定（例: http://localhost:3100）")
	}

	// 全体タイムアウト 60 秒（Loki ingester の取り込み遅延を見込む）
	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	// テスト固有の ID を生成して log line に埋め込む（後段の Loki query で絞り込みに使う）
	corrID := fmt.Sprintf("e2e-%d", time.Now().UnixNano())
	// fake trace_id（16 byte = 32 hex chars）。production では OTel SDK が span から付与する。
	traceID := "deadbeefcafebabe1234567890abcdef"
	// 送信時刻（OTLP の time_unix_nano は string of int）
	nowNs := time.Now().UnixNano()

	// OTLP HTTP /v1/logs の最小 JSON 構造を組み立てる
	logBody := fmt.Sprintf(`{
		"resourceLogs": [{
			"resource": {
				"attributes": [
					{"key": "service.name", "value": {"stringValue": "k1s0-e2e-log-trace"}},
					{"key": "service.version", "value": {"stringValue": "e2e-test"}}
				]
			},
			"scopeLogs": [{
				"scope": {"name": "k1s0-e2e-log-trace"},
				"logRecords": [{
					"timeUnixNano": "%d",
					"observedTimeUnixNano": "%d",
					"severityNumber": 9,
					"severityText": "INFO",
					"body": {"stringValue": "e2e log trace_id=%s corr_id=%s"},
					"traceId": "%s",
					"spanId": "1234567890abcdef",
					"attributes": [
						{"key": "corr_id", "value": {"stringValue": "%s"}},
						{"key": "trace_id", "value": {"stringValue": "%s"}}
					]
				}]
			}]
		}]
	}`, nowNs, nowNs, traceID, corrID, traceID, corrID, traceID)

	// /v1/logs に POST
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, otlpHTTP+"/v1/logs", bytes.NewReader([]byte(logBody)))
	if err != nil {
		t.Fatalf("http.NewRequest: %v", err)
	}
	// Content-Type は application/json（OTLP HTTP/JSON 経路）
	req.Header.Set("Content-Type", "application/json")
	httpClient := &http.Client{Timeout: 10 * time.Second}
	resp, err := httpClient.Do(req)
	if err != nil {
		t.Fatalf("OTLP /v1/logs POST: %v", err)
	}
	body, _ := io.ReadAll(resp.Body)
	_ = resp.Body.Close()
	// OTLP は 200 OK を返す（json body は partialSuccess または空 object）
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("OTLP /v1/logs POST: status=%d body=%s", resp.StatusCode, string(body))
	}
	t.Logf("OTLP /v1/logs POST: status=200 corr_id=%s trace_id=%s body=%s", corrID, traceID, string(body))

	// Loki ingester の取り込み遅延を考慮し最大 30 秒 polling で log 取得を試みる
	// LogQL: corr_id を含む log line を抽出する（全 stream を対象）
	deadline := time.Now().Add(30 * time.Second)
	var matchedLine string
	for time.Now().Before(deadline) {
		// 過去 5 分の範囲で query_range
		queryEnd := time.Now()
		queryStart := queryEnd.Add(-5 * time.Minute)
		q := url.Values{}
		// `{service_name="k1s0-e2e-log-trace"}` は OTLP resource attribute service.name から
		// otlphttp/loki が自動付与する label。`|= corr_id` で本 test の log に絞り込む。
		q.Set("query", fmt.Sprintf(`{service_name="k1s0-e2e-log-trace"} |= %q`, corrID))
		q.Set("start", fmt.Sprintf("%d", queryStart.UnixNano()))
		q.Set("end", fmt.Sprintf("%d", queryEnd.UnixNano()))
		q.Set("limit", "10")
		queryURL := fmt.Sprintf("%s/loki/api/v1/query_range?%s", lokiTarget, q.Encode())

		qreq, err := http.NewRequestWithContext(ctx, http.MethodGet, queryURL, nil)
		if err != nil {
			t.Fatalf("Loki query_range req: %v", err)
		}
		qresp, err := httpClient.Do(qreq)
		if err != nil {
			// network エラーは polling 継続
			time.Sleep(2 * time.Second)
			continue
		}
		qbody, _ := io.ReadAll(qresp.Body)
		_ = qresp.Body.Close()
		if qresp.StatusCode != http.StatusOK {
			t.Fatalf("Loki query_range: status=%d body=%s", qresp.StatusCode, string(qbody))
		}
		var qres lokiQueryResponse
		if err := json.Unmarshal(qbody, &qres); err != nil {
			t.Fatalf("Loki query_range JSON parse: %v body=%s", err, string(qbody))
		}
		if qres.Status != "success" {
			t.Fatalf("Loki query_range: status=%s", qres.Status)
		}
		// 結果 stream を走査して corr_id を含む line を探す
		for _, stream := range qres.Data.Result {
			for _, vals := range stream.Values {
				if len(vals) < 2 {
					continue
				}
				// vals[0]=timestamp_ns vals[1]=log_line
				line := vals[1]
				if strings.Contains(line, corrID) {
					matchedLine = line
					break
				}
			}
			if matchedLine != "" {
				break
			}
		}
		if matchedLine != "" {
			break
		}
		time.Sleep(2 * time.Second)
	}

	// 取得できなかった場合は経路不通
	if matchedLine == "" {
		t.Fatalf("Loki query_range: corr_id=%s を含む log line が 30 秒以内に取得できなかった（otlphttp/loki exporter または Loki ingester 異常）", corrID)
	}
	// 取得できた log line に trace_id が含まれているか検証（log↔trace 結合の最小成立）
	if !strings.Contains(matchedLine, traceID) {
		t.Fatalf("Loki log line に trace_id=%s が含まれていない: line=%s", traceID, matchedLine)
	}
	t.Logf("Loki query_range: corr_id=%s trace_id=%s を含む log line 取得成功 line=%s", corrID, traceID, matchedLine)
}
