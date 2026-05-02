// 本ファイルは Feature Circuit Breaker の MetricThresholdSource を Prometheus HTTP API で
// 実装した最小クライアント。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/11_Feature_API.md
//     - FR-T1-FEATURE-003 受け入れ基準:
//       * Prometheus クエリを条件に指定可能
//       * 閾値超過から false 化まで 30 秒以内
//
// 役割:
//   FeatureCBRule.PromQL を Prometheus Query API（GET /api/v1/query?query=...）に投げ、
//   先頭ベクタ値を float64 で返す。失敗（network / 5xx / 不正応答）は error で
//   呼出側（FeatureCircuitBreakerEvaluator）にスキップさせる（次回 retry）。
//
// 範囲外（別 PR）:
//   - PromQL の構文検証は Prometheus 側に委ねる（事前 parser を持たない）
//   - 認証 (Bearer token / mTLS) は環境変数で注入する経路を別途追加する想定
//   - Range query / instant 以外の resultType（matrix）は未対応（rule は instant 想定）

package state

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strconv"
	"time"
)

// PrometheusMetricSource は Prometheus HTTP API に対する最小 instant query クライアント。
// FeatureCBRule.PromQL を都度評価し、先頭ベクタ値を返す。
type PrometheusMetricSource struct {
	// Prometheus base URL（例: "http://prometheus.k1s0-observability.svc:9090"）。
	BaseURL string
	// HTTP client（タイムアウト含む）。nil 時は 5 秒の既定 client を使う。
	HTTPClient *http.Client
}

// NewPrometheusMetricSource は base URL を指定して PrometheusMetricSource を生成する。
// httpClient nil 時は 5 秒タイムアウトの client を使う。
func NewPrometheusMetricSource(baseURL string, httpClient *http.Client) *PrometheusMetricSource {
	if httpClient == nil {
		httpClient = &http.Client{Timeout: 5 * time.Second}
	}
	return &PrometheusMetricSource{BaseURL: baseURL, HTTPClient: httpClient}
}

// promResponse は GET /api/v1/query の応答スキーマ（必要な部分のみ）。
type promResponse struct {
	Status string         `json:"status"`
	Data   promResultData `json:"data"`
	Error  string         `json:"error"`
}

type promResultData struct {
	ResultType string         `json:"resultType"`
	Result     []promVecEntry `json:"result"`
}

// promVecEntry は instant vector / scalar 共通の応答エントリ。
// instant vector: [<unix_ts>, "<value>"]、scalar: [<unix_ts>, "<value>"]。
type promVecEntry struct {
	// ベクタの場合のラベル集合（解釈不要）。
	Metric map[string]string `json:"metric,omitempty"`
	// [unix_ts (float seconds), value (string)] の 2 要素タプル。
	Value [2]json.RawMessage `json:"value"`
}

// Evaluate は rule.PromQL を Prometheus に投げて先頭ベクタの値を float64 で返す。
//
// 動作:
//   - resultType = "vector" の場合、先頭エントリの value[1] を float に parse して返す
//   - resultType = "scalar" の場合、value[1] を float に parse して返す
//   - resultType = "matrix" / "string" は未対応（error）
//   - 結果が空（no data）の場合は 0 を返す（rule.Threshold との比較は呼出側）
func (p *PrometheusMetricSource) Evaluate(ctx context.Context, rule FeatureCBRule) (float64, error) {
	if p.BaseURL == "" {
		return 0, fmt.Errorf("tier1/feature: prometheus base url is empty")
	}
	if rule.PromQL == "" {
		return 0, fmt.Errorf("tier1/feature: rule %q has empty PromQL", rule.FlagKey)
	}
	// query string を組み立てる。url.Values で URL エンコードを適用する。
	v := url.Values{}
	v.Set("query", rule.PromQL)
	endpoint := p.BaseURL + "/api/v1/query?" + v.Encode()
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, endpoint, nil)
	if err != nil {
		return 0, fmt.Errorf("tier1/feature: build request: %w", err)
	}
	resp, err := p.HTTPClient.Do(req)
	if err != nil {
		return 0, fmt.Errorf("tier1/feature: prom request: %w", err)
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(io.LimitReader(resp.Body, 1<<20))
	if err != nil {
		return 0, fmt.Errorf("tier1/feature: read response: %w", err)
	}
	if resp.StatusCode/100 != 2 {
		return 0, fmt.Errorf("tier1/feature: prom http %d: %s", resp.StatusCode, string(body))
	}
	var pr promResponse
	if err := json.Unmarshal(body, &pr); err != nil {
		return 0, fmt.Errorf("tier1/feature: parse response: %w", err)
	}
	if pr.Status != "success" {
		return 0, fmt.Errorf("tier1/feature: prom error: %s", pr.Error)
	}
	switch pr.Data.ResultType {
	case "vector":
		if len(pr.Data.Result) == 0 {
			// 結果なし。閾値判定では threshold より小さい値（0）として扱う。
			return 0, nil
		}
		return parsePromValue(pr.Data.Result[0].Value[1])
	case "scalar":
		if len(pr.Data.Result) == 0 {
			return 0, nil
		}
		return parsePromValue(pr.Data.Result[0].Value[1])
	default:
		return 0, fmt.Errorf("tier1/feature: unsupported resultType %q", pr.Data.ResultType)
	}
}

// parsePromValue は Prometheus 応答の value[1] (json.RawMessage、文字列 or null) を float64 に変換する。
func parsePromValue(raw json.RawMessage) (float64, error) {
	if len(raw) == 0 {
		return 0, fmt.Errorf("tier1/feature: empty value")
	}
	// "null" は値なし扱い。
	if string(raw) == "null" {
		return 0, nil
	}
	// 文字列として unquote する（Prometheus は数値も string で返す）。
	var s string
	if err := json.Unmarshal(raw, &s); err != nil {
		return 0, fmt.Errorf("tier1/feature: unmarshal value: %w", err)
	}
	switch s {
	case "+Inf", "Inf":
		return 1e18, nil // 比較用の十分大きな値（threshold 超過判定で必ず true）
	case "-Inf":
		return -1e18, nil
	case "NaN":
		return 0, fmt.Errorf("tier1/feature: prom returned NaN")
	}
	f, err := strconv.ParseFloat(s, 64)
	if err != nil {
		return 0, fmt.Errorf("tier1/feature: parse float %q: %w", s, err)
	}
	return f, nil
}
