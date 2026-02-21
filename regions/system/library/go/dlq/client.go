package dlq

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
)

// DlqClient は DLQ 管理サーバーへの REST クライアント。
type DlqClient struct {
	endpoint   string
	httpClient *http.Client
}

// NewDlqClient は新しい DlqClient を生成する。
func NewDlqClient(endpoint string) *DlqClient {
	return &DlqClient{
		endpoint:   strings.TrimRight(endpoint, "/"),
		httpClient: &http.Client{},
	}
}

// NewDlqClientWithHTTPClient はカスタム http.Client を使う DlqClient を生成する（テスト用）。
func NewDlqClientWithHTTPClient(endpoint string, httpClient *http.Client) *DlqClient {
	return &DlqClient{
		endpoint:   strings.TrimRight(endpoint, "/"),
		httpClient: httpClient,
	}
}

// ListMessages はトピック別 DLQ メッセージ一覧を取得する。
// GET /api/v1/dlq/:topic?page=:page&page_size=:page_size
func (c *DlqClient) ListMessages(ctx context.Context, req *ListDlqMessagesRequest) (*ListDlqMessagesResponse, error) {
	url := fmt.Sprintf("%s/api/v1/dlq/%s?page=%d&page_size=%d",
		c.endpoint, req.Topic, req.Page, req.PageSize)

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, &DlqError{Op: "list_messages", Err: fmt.Errorf("failed to create request: %w", err)}
	}

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return nil, &DlqError{Op: "list_messages", Err: fmt.Errorf("request failed: %w", err)}
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		respBody, _ := io.ReadAll(resp.Body)
		return nil, &DlqError{
			Op:         "list_messages",
			StatusCode: resp.StatusCode,
			Err:        fmt.Errorf("%s", string(respBody)),
		}
	}

	var result ListDlqMessagesResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, &DlqError{Op: "list_messages", Err: fmt.Errorf("failed to decode response: %w", err)}
	}

	return &result, nil
}

// GetMessage は DLQ メッセージの詳細を取得する。
// GET /api/v1/dlq/messages/:id
func (c *DlqClient) GetMessage(ctx context.Context, messageID string) (*DlqMessage, error) {
	url := fmt.Sprintf("%s/api/v1/dlq/messages/%s", c.endpoint, messageID)

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, &DlqError{Op: "get_message", Err: fmt.Errorf("failed to create request: %w", err)}
	}

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return nil, &DlqError{Op: "get_message", Err: fmt.Errorf("request failed: %w", err)}
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		respBody, _ := io.ReadAll(resp.Body)
		return nil, &DlqError{
			Op:         "get_message",
			StatusCode: resp.StatusCode,
			Err:        fmt.Errorf("%s", string(respBody)),
		}
	}

	var result DlqMessage
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, &DlqError{Op: "get_message", Err: fmt.Errorf("failed to decode response: %w", err)}
	}

	return &result, nil
}

// RetryMessage は DLQ メッセージを再処理する。
// POST /api/v1/dlq/messages/:id/retry
func (c *DlqClient) RetryMessage(ctx context.Context, messageID string) (*RetryDlqMessageResponse, error) {
	url := fmt.Sprintf("%s/api/v1/dlq/messages/%s/retry", c.endpoint, messageID)

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader([]byte("{}")))
	if err != nil {
		return nil, &DlqError{Op: "retry_message", Err: fmt.Errorf("failed to create request: %w", err)}
	}
	httpReq.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return nil, &DlqError{Op: "retry_message", Err: fmt.Errorf("request failed: %w", err)}
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		respBody, _ := io.ReadAll(resp.Body)
		return nil, &DlqError{
			Op:         "retry_message",
			StatusCode: resp.StatusCode,
			Err:        fmt.Errorf("%s", string(respBody)),
		}
	}

	var result RetryDlqMessageResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, &DlqError{Op: "retry_message", Err: fmt.Errorf("failed to decode response: %w", err)}
	}

	return &result, nil
}

// DeleteMessage は DLQ メッセージを削除する。
// DELETE /api/v1/dlq/messages/:id
func (c *DlqClient) DeleteMessage(ctx context.Context, messageID string) error {
	url := fmt.Sprintf("%s/api/v1/dlq/messages/%s", c.endpoint, messageID)

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodDelete, url, nil)
	if err != nil {
		return &DlqError{Op: "delete_message", Err: fmt.Errorf("failed to create request: %w", err)}
	}

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return &DlqError{Op: "delete_message", Err: fmt.Errorf("request failed: %w", err)}
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusNoContent {
		respBody, _ := io.ReadAll(resp.Body)
		return &DlqError{
			Op:         "delete_message",
			StatusCode: resp.StatusCode,
			Err:        fmt.Errorf("%s", string(respBody)),
		}
	}

	return nil
}

// RetryAll はトピック内全メッセージを一括再処理する。
// POST /api/v1/dlq/:topic/retry-all
func (c *DlqClient) RetryAll(ctx context.Context, topic string) error {
	url := fmt.Sprintf("%s/api/v1/dlq/%s/retry-all", c.endpoint, topic)

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader([]byte("{}")))
	if err != nil {
		return &DlqError{Op: "retry_all", Err: fmt.Errorf("failed to create request: %w", err)}
	}
	httpReq.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return &DlqError{Op: "retry_all", Err: fmt.Errorf("request failed: %w", err)}
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusNoContent {
		respBody, _ := io.ReadAll(resp.Body)
		return &DlqError{
			Op:         "retry_all",
			StatusCode: resp.StatusCode,
			Err:        fmt.Errorf("%s", string(respBody)),
		}
	}

	return nil
}
