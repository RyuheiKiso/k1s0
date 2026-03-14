package bbaiclient

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

// HTTPClient は HTTP 経由で AI ゲートウェイと通信する AiClient の実装。
type HTTPClient struct {
	baseURL    string
	apiKey     string
	httpClient *http.Client
}

// HTTPClientConfig は HTTPClient の設定を保持する。
type HTTPClientConfig struct {
	BaseURL string
	APIKey  string
	Timeout time.Duration
}

// NewHTTPClient は HTTPClient を生成する。
// Timeout が 0 の場合は 30 秒がデフォルトとなる。
func NewHTTPClient(cfg HTTPClientConfig) *HTTPClient {
	timeout := cfg.Timeout
	if timeout == 0 {
		timeout = 30 * time.Second
	}
	return &HTTPClient{
		baseURL:    cfg.BaseURL,
		apiKey:     cfg.APIKey,
		httpClient: &http.Client{Timeout: timeout},
	}
}

// Complete は AI ゲートウェイの /v1/complete エンドポイントを呼び出す。
func (c *HTTPClient) Complete(ctx context.Context, req CompleteRequest) (*CompleteResponse, error) {
	body, err := json.Marshal(req)
	if err != nil {
		return nil, fmt.Errorf("bbaiclient: marshal request: %w", err)
	}

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, c.baseURL+"/v1/complete", bytes.NewReader(body))
	if err != nil {
		return nil, fmt.Errorf("bbaiclient: create request: %w", err)
	}
	httpReq.Header.Set("Content-Type", "application/json")
	if c.apiKey != "" {
		httpReq.Header.Set("Authorization", "Bearer "+c.apiKey)
	}

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("bbaiclient: do request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, &APIError{StatusCode: resp.StatusCode}
	}

	var result CompleteResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("bbaiclient: decode response: %w", err)
	}
	return &result, nil
}

// Embed は AI ゲートウェイの /v1/embed エンドポイントを呼び出す。
func (c *HTTPClient) Embed(ctx context.Context, req EmbedRequest) (*EmbedResponse, error) {
	body, err := json.Marshal(req)
	if err != nil {
		return nil, fmt.Errorf("bbaiclient: marshal request: %w", err)
	}

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, c.baseURL+"/v1/embed", bytes.NewReader(body))
	if err != nil {
		return nil, fmt.Errorf("bbaiclient: create request: %w", err)
	}
	httpReq.Header.Set("Content-Type", "application/json")
	if c.apiKey != "" {
		httpReq.Header.Set("Authorization", "Bearer "+c.apiKey)
	}

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("bbaiclient: do request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, &APIError{StatusCode: resp.StatusCode}
	}

	var result EmbedResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("bbaiclient: decode response: %w", err)
	}
	return &result, nil
}

// ListModels は AI ゲートウェイの /v1/models エンドポイントを呼び出す。
func (c *HTTPClient) ListModels(ctx context.Context) ([]ModelInfo, error) {
	httpReq, err := http.NewRequestWithContext(ctx, http.MethodGet, c.baseURL+"/v1/models", nil)
	if err != nil {
		return nil, fmt.Errorf("bbaiclient: create request: %w", err)
	}
	if c.apiKey != "" {
		httpReq.Header.Set("Authorization", "Bearer "+c.apiKey)
	}

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("bbaiclient: do request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, &APIError{StatusCode: resp.StatusCode}
	}

	var result []ModelInfo
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("bbaiclient: decode response: %w", err)
	}
	return result, nil
}
