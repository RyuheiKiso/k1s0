// tests/e2e/owner/helpers/loki_client.go
//
// observability/ 検証 3 (log↔trace 結合) で使う Loki LogQL client。
//
// 設計正典: ADR-TEST-009 §1 検証 3
package helpers

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"time"
)

// LokiClient は Loki /loki/api/v1/* 系 HTTP API client
type LokiClient struct {
	BaseURL    string
	HTTPClient *http.Client
}

// NewLokiClient は base URL から LokiClient を生成する
func NewLokiClient(baseURL string) *LokiClient {
	return &LokiClient{
		BaseURL:    baseURL,
		HTTPClient: &http.Client{Timeout: 30 * time.Second},
	}
}

// queryRangeResponse は /loki/api/v1/query_range の response 構造（簡略版）
type queryRangeResponse struct {
	Status string `json:"status"`
	Data   struct {
		ResultType string `json:"resultType"`
		Result     []struct {
			Stream map[string]string `json:"stream"`
			Values [][]string        `json:"values"`
		} `json:"result"`
	} `json:"data"`
}

// QueryRangeCount は LogQL query_range で取得した log line 数を返す。
// 検証 3 で「全 log と trace_id 付き log の比率」を計算するために使う。
func (c *LokiClient) QueryRangeCount(ctx context.Context, query string, startUnix, endUnix int64) (int, error) {
	q := url.Values{}
	q.Set("query", query)
	q.Set("start", fmt.Sprintf("%d", startUnix*1_000_000_000))
	q.Set("end", fmt.Sprintf("%d", endUnix*1_000_000_000))
	q.Set("limit", "5000")
	reqURL := fmt.Sprintf("%s/loki/api/v1/query_range?%s", c.BaseURL, q.Encode())
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, reqURL, nil)
	if err != nil {
		return 0, err
	}
	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return 0, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return 0, fmt.Errorf("Loki unexpected status: %d", resp.StatusCode)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return 0, err
	}
	var result queryRangeResponse
	if err := json.Unmarshal(body, &result); err != nil {
		return 0, err
	}
	if result.Status != "success" {
		return 0, fmt.Errorf("Loki status != success: %s", result.Status)
	}
	// stream ごとの values 件数を合計
	count := 0
	for _, stream := range result.Data.Result {
		count += len(stream.Values)
	}
	return count, nil
}
