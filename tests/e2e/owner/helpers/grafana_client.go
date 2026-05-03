// tests/e2e/owner/helpers/grafana_client.go
//
// observability/ 検証 5 (Grafana dashboard goldenfile) で使う Grafana HTTP API client。
//
// 設計正典: ADR-TEST-009 §1 検証 5
package helpers

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

// GrafanaClient は Grafana /api/dashboards/* HTTP API client
type GrafanaClient struct {
	BaseURL    string
	APIKey     string
	HTTPClient *http.Client
}

// NewGrafanaClient は base URL + API key から client を生成
func NewGrafanaClient(baseURL, apiKey string) *GrafanaClient {
	return &GrafanaClient{
		BaseURL:    baseURL,
		APIKey:     apiKey,
		HTTPClient: &http.Client{Timeout: 30 * time.Second},
	}
}

// GetDashboardJSON は /api/dashboards/uid/<uid> を叩いて dashboard JSON を取得する
// 戻り値は raw JSON (json.RawMessage)。検証 5 で baseline と canonical diff する。
func (c *GrafanaClient) GetDashboardJSON(ctx context.Context, uid string) (json.RawMessage, error) {
	reqURL := fmt.Sprintf("%s/api/dashboards/uid/%s", c.BaseURL, uid)
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, reqURL, nil)
	if err != nil {
		return nil, err
	}
	if c.APIKey != "" {
		req.Header.Set("Authorization", "Bearer "+c.APIKey)
	}
	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("Grafana unexpected status: %d", resp.StatusCode)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	// response は { "dashboard": { ... }, "meta": { ... } } の構造、
	// dashboard field のみを取り出して baseline と比較する
	var wrapper struct {
		Dashboard json.RawMessage `json:"dashboard"`
	}
	if err := json.Unmarshal(body, &wrapper); err != nil {
		return nil, fmt.Errorf("Grafana response decode 失敗: %w", err)
	}
	return wrapper.Dashboard, nil
}
