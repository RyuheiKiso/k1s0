package quotaclient

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

// QuotaClientConfig は HTTP クライアントの設定。
type QuotaClientConfig struct {
	// BaseURL は quota-server のベース URL (例: "http://quota-server:8080")。
	BaseURL string
	// Timeout は HTTP リクエストのタイムアウト。デフォルト: 5s。
	Timeout time.Duration
	// PolicyCacheTTL はポリシーキャッシュの TTL。デフォルト: 60s。
	PolicyCacheTTL time.Duration
}

// NewQuotaClientConfig はデフォルト値付きの QuotaClientConfig を生成する。
func NewQuotaClientConfig(baseURL string) QuotaClientConfig {
	return QuotaClientConfig{
		BaseURL:        baseURL,
		Timeout:        5 * time.Second,
		PolicyCacheTTL: 60 * time.Second,
	}
}

// QuotaClientError は quota-client のエラー型。
type QuotaClientError struct {
	Code    string
	Message string
}

func (e *QuotaClientError) Error() string {
	return fmt.Sprintf("quota-client error [%s]: %s", e.Code, e.Message)
}

// QuotaNotFoundError はクォータ ID が見つからない場合のエラー。
type QuotaNotFoundError struct {
	QuotaID string
}

func (e *QuotaNotFoundError) Error() string {
	return fmt.Sprintf("quota not found: %s", e.QuotaID)
}

// QuotaConnectionError は quota-server への接続失敗エラー。
type QuotaConnectionError struct {
	Message string
}

func (e *QuotaConnectionError) Error() string {
	return fmt.Sprintf("quota connection error: %s", e.Message)
}

// HttpQuotaClient は quota-server への HTTP クライアント実装。
type HttpQuotaClient struct {
	httpClient *http.Client
	baseURL    string
}

// NewHttpQuotaClient は新しい HttpQuotaClient を生成する。
func NewHttpQuotaClient(baseURL string, config QuotaClientConfig) *HttpQuotaClient {
	timeout := config.Timeout
	if timeout == 0 {
		timeout = 5 * time.Second
	}
	return &HttpQuotaClient{
		httpClient: &http.Client{Timeout: timeout},
		baseURL:    baseURL,
	}
}

// Check はクォータの残量を quota-server に問い合わせる。
func (c *HttpQuotaClient) Check(ctx context.Context, quotaID string, amount uint64) (*QuotaStatus, error) {
	url := fmt.Sprintf("%s/api/v1/quotas/%s/check", c.baseURL, quotaID)
	body, err := json.Marshal(map[string]uint64{"amount": amount})
	if err != nil {
		return nil, &QuotaClientError{Code: "INTERNAL", Message: err.Error()}
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return nil, &QuotaClientError{Code: "INTERNAL", Message: err.Error()}
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, &QuotaConnectionError{Message: err.Error()}
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return nil, &QuotaNotFoundError{QuotaID: quotaID}
	}
	if resp.StatusCode != http.StatusOK {
		return nil, &QuotaClientError{Code: "INVALID_RESPONSE", Message: fmt.Sprintf("unexpected status: %d", resp.StatusCode)}
	}

	var status QuotaStatus
	if err := json.NewDecoder(resp.Body).Decode(&status); err != nil {
		return nil, &QuotaClientError{Code: "INVALID_RESPONSE", Message: err.Error()}
	}
	return &status, nil
}

// Increment はクォータ使用量を quota-server に加算する。
func (c *HttpQuotaClient) Increment(ctx context.Context, quotaID string, amount uint64) (*QuotaUsage, error) {
	url := fmt.Sprintf("%s/api/v1/quotas/%s/increment", c.baseURL, quotaID)
	body, err := json.Marshal(map[string]uint64{"amount": amount})
	if err != nil {
		return nil, &QuotaClientError{Code: "INTERNAL", Message: err.Error()}
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return nil, &QuotaClientError{Code: "INTERNAL", Message: err.Error()}
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, &QuotaConnectionError{Message: err.Error()}
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return nil, &QuotaNotFoundError{QuotaID: quotaID}
	}
	if resp.StatusCode != http.StatusOK {
		return nil, &QuotaClientError{Code: "INVALID_RESPONSE", Message: fmt.Sprintf("unexpected status: %d", resp.StatusCode)}
	}

	var usage QuotaUsage
	if err := json.NewDecoder(resp.Body).Decode(&usage); err != nil {
		return nil, &QuotaClientError{Code: "INVALID_RESPONSE", Message: err.Error()}
	}
	return &usage, nil
}

// GetUsage はクォータ使用量を quota-server から取得する。
func (c *HttpQuotaClient) GetUsage(ctx context.Context, quotaID string) (*QuotaUsage, error) {
	url := fmt.Sprintf("%s/api/v1/quotas/%s/usage", c.baseURL, quotaID)

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, &QuotaClientError{Code: "INTERNAL", Message: err.Error()}
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, &QuotaConnectionError{Message: err.Error()}
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return nil, &QuotaNotFoundError{QuotaID: quotaID}
	}
	if resp.StatusCode != http.StatusOK {
		return nil, &QuotaClientError{Code: "INVALID_RESPONSE", Message: fmt.Sprintf("unexpected status: %d", resp.StatusCode)}
	}

	var usage QuotaUsage
	if err := json.NewDecoder(resp.Body).Decode(&usage); err != nil {
		return nil, &QuotaClientError{Code: "INVALID_RESPONSE", Message: err.Error()}
	}
	return &usage, nil
}

// GetPolicy はクォータポリシーを quota-server から取得する。
func (c *HttpQuotaClient) GetPolicy(ctx context.Context, quotaID string) (*QuotaPolicy, error) {
	url := fmt.Sprintf("%s/api/v1/quotas/%s/policy", c.baseURL, quotaID)

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, &QuotaClientError{Code: "INTERNAL", Message: err.Error()}
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, &QuotaConnectionError{Message: err.Error()}
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return nil, &QuotaNotFoundError{QuotaID: quotaID}
	}
	if resp.StatusCode != http.StatusOK {
		return nil, &QuotaClientError{Code: "INVALID_RESPONSE", Message: fmt.Sprintf("unexpected status: %d", resp.StatusCode)}
	}

	var policy QuotaPolicy
	if err := json.NewDecoder(resp.Body).Decode(&policy); err != nil {
		return nil, &QuotaClientError{Code: "INVALID_RESPONSE", Message: err.Error()}
	}
	return &policy, nil
}
