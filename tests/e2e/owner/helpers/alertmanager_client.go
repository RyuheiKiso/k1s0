// tests/e2e/owner/helpers/alertmanager_client.go
//
// observability/ 検証 4 (SLO burn rate alert 発火 + runbook_url 必須) で使う Alertmanager client。
//
// 設計正典: ADR-TEST-009 §1 検証 4
package helpers

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

// AlertmanagerClient は Alertmanager /api/v2/* 系 HTTP API client
type AlertmanagerClient struct {
	BaseURL    string
	HTTPClient *http.Client
}

// NewAlertmanagerClient は base URL から client を生成
func NewAlertmanagerClient(baseURL string) *AlertmanagerClient {
	return &AlertmanagerClient{
		BaseURL:    baseURL,
		HTTPClient: &http.Client{Timeout: 30 * time.Second},
	}
}

// Alert は /api/v2/alerts response の単一 alert 構造（検証 4 で必要な field のみ）
type Alert struct {
	Labels      map[string]string `json:"labels"`
	Annotations map[string]string `json:"annotations"`
	Status      AlertStatus       `json:"status"`
}

// AlertStatus は alert の状態（active / suppressed / silenced）
type AlertStatus struct {
	State string `json:"state"`
}

// ActiveAlerts は active 状態の alert 一覧を返す。検証 4 で SLO 違反注入後の発火確認に使う。
func (c *AlertmanagerClient) ActiveAlerts(ctx context.Context) ([]Alert, error) {
	reqURL := fmt.Sprintf("%s/api/v2/alerts?active=true&silenced=false&inhibited=false", c.BaseURL)
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
		return nil, fmt.Errorf("Alertmanager unexpected status: %d", resp.StatusCode)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	var alerts []Alert
	if err := json.Unmarshal(body, &alerts); err != nil {
		return nil, fmt.Errorf("Alertmanager response decode 失敗: %w", err)
	}
	return alerts, nil
}

// FindAlertByName は labels.alertname が指定値の alert を返す。なければ nil。
func FindAlertByName(alerts []Alert, name string) *Alert {
	for i := range alerts {
		if alerts[i].Labels["alertname"] == name {
			return &alerts[i]
		}
	}
	return nil
}
