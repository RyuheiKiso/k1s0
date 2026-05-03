// tests/e2e/owner/helpers/prometheus_client.go
//
// observability/ 検証 2 (Prometheus cardinality regression) で使う Prometheus HTTP API client。
//
// 設計正典: ADR-TEST-009 §1 検証 2
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

// PrometheusClient は Prometheus /api/v1/* 系の薄い wrapper
type PrometheusClient struct {
	BaseURL    string
	HTTPClient *http.Client
}

// NewPrometheusClient は base URL から PrometheusClient を生成する
func NewPrometheusClient(baseURL string) *PrometheusClient {
	return &PrometheusClient{
		BaseURL:    baseURL,
		HTTPClient: &http.Client{Timeout: 30 * time.Second},
	}
}

// seriesResponse は /api/v1/series の response 構造
type seriesResponse struct {
	Status string              `json:"status"`
	Data   []map[string]string `json:"data"`
}

// SeriesCount は match[] 句にマッチする series の cardinality（series 数）を返す。
// 検証 2 で baseline JSON との比較に使う。
func (c *PrometheusClient) SeriesCount(ctx context.Context, matchExpr string) (int, error) {
	// /api/v1/series?match[]=<expr>
	q := url.Values{}
	q.Set("match[]", matchExpr)
	reqURL := fmt.Sprintf("%s/api/v1/series?%s", c.BaseURL, q.Encode())
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, reqURL, nil)
	if err != nil {
		return 0, fmt.Errorf("Prometheus request 構築失敗: %w", err)
	}
	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return 0, fmt.Errorf("Prometheus HTTP 失敗: %w", err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return 0, fmt.Errorf("Prometheus unexpected status: %d", resp.StatusCode)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return 0, err
	}
	var result seriesResponse
	if err := json.Unmarshal(body, &result); err != nil {
		return 0, fmt.Errorf("Prometheus response decode 失敗: %w", err)
	}
	if result.Status != "success" {
		return 0, fmt.Errorf("Prometheus status != success: %s", result.Status)
	}
	return len(result.Data), nil
}

// labelsResponse は /api/v1/labels の response 構造
type labelsResponse struct {
	Status string   `json:"status"`
	Data   []string `json:"data"`
}

// LabelNames は cluster の全 metric から取得できる label name の集合を返す。
// 検証 2 の sanity check で使う（label 数が baseline 1.2 倍を超えないか確認の補助）。
func (c *PrometheusClient) LabelNames(ctx context.Context) ([]string, error) {
	reqURL := fmt.Sprintf("%s/api/v1/labels", c.BaseURL)
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, reqURL, nil)
	if err != nil {
		return nil, err
	}
	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("Prometheus labels unexpected status: %d", resp.StatusCode)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	var result labelsResponse
	if err := json.Unmarshal(body, &result); err != nil {
		return nil, err
	}
	return result.Data, nil
}
