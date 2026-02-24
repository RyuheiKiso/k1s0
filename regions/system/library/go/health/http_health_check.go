package health

import (
	"context"
	"fmt"
	"net/http"
	"time"
)

// HttpHealthCheck はHTTP GETリクエストでヘルスを確認する。
type HttpHealthCheck struct {
	name    string
	url     string
	timeout time.Duration
	client  *http.Client
}

// HttpHealthCheckOption はHttpHealthCheckの設定オプション。
type HttpHealthCheckOption func(*HttpHealthCheck)

// WithTimeout はタイムアウトを設定する。
func WithTimeout(d time.Duration) HttpHealthCheckOption {
	return func(h *HttpHealthCheck) {
		h.timeout = d
		h.client.Timeout = d
	}
}

// WithName はヘルスチェック名を設定する。
func WithName(name string) HttpHealthCheckOption {
	return func(h *HttpHealthCheck) {
		h.name = name
	}
}

// NewHttpHealthCheck は新しいHttpHealthCheckを生成する。
func NewHttpHealthCheck(url string, opts ...HttpHealthCheckOption) *HttpHealthCheck {
	h := &HttpHealthCheck{
		name:    "http",
		url:     url,
		timeout: 5 * time.Second,
		client:  &http.Client{Timeout: 5 * time.Second},
	}
	for _, opt := range opts {
		opt(h)
	}
	return h
}

// Name はヘルスチェック名を返す。
func (h *HttpHealthCheck) Name() string {
	return h.name
}

// Check はHTTP GETリクエストを送信し、2xxレスポンスであればnilを返す。
func (h *HttpHealthCheck) Check(ctx context.Context) error {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, h.url, nil)
	if err != nil {
		return fmt.Errorf("HTTP check failed: %w", err)
	}

	resp, err := h.client.Do(req)
	if err != nil {
		return fmt.Errorf("HTTP check failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return fmt.Errorf("HTTP %s returned status %d", h.url, resp.StatusCode)
	}

	return nil
}
