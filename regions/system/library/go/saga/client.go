package saga

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
)

// SagaClient は Saga サーバーへの REST クライアント。
type SagaClient struct {
	endpoint   string
	httpClient *http.Client
}

// NewSagaClient は新しい SagaClient を生成する。
// endpoint の末尾スラッシュは削除される。
func NewSagaClient(endpoint string) *SagaClient {
	return &SagaClient{
		endpoint:   strings.TrimRight(endpoint, "/"),
		httpClient: &http.Client{},
	}
}

// NewSagaClientWithHTTPClient はカスタム http.Client を使う SagaClient を生成する（テスト用）。
func NewSagaClientWithHTTPClient(endpoint string, httpClient *http.Client) *SagaClient {
	return &SagaClient{
		endpoint:   strings.TrimRight(endpoint, "/"),
		httpClient: httpClient,
	}
}

// StartSaga は新しい Saga を開始する。
// POST /api/v1/sagas
func (c *SagaClient) StartSaga(ctx context.Context, req *StartSagaRequest) (*StartSagaResponse, error) {
	body, err := json.Marshal(req)
	if err != nil {
		return nil, &SagaError{Op: "start_saga", Err: fmt.Errorf("failed to marshal request: %w", err)}
	}

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, c.endpoint+"/api/v1/sagas", bytes.NewReader(body))
	if err != nil {
		return nil, &SagaError{Op: "start_saga", Err: fmt.Errorf("failed to create request: %w", err)}
	}
	httpReq.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return nil, &SagaError{Op: "start_saga", Err: fmt.Errorf("request failed: %w", err)}
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		respBody, _ := io.ReadAll(resp.Body)
		return nil, &SagaError{
			Op:         "start_saga",
			StatusCode: resp.StatusCode,
			Err:        fmt.Errorf("%s", string(respBody)),
		}
	}

	var result StartSagaResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, &SagaError{Op: "start_saga", Err: fmt.Errorf("failed to decode response: %w", err)}
	}

	return &result, nil
}

// GetSaga は Saga の現在状態を取得する。
// GET /api/v1/sagas/{sagaID}
func (c *SagaClient) GetSaga(ctx context.Context, sagaID string) (*SagaState, error) {
	httpReq, err := http.NewRequestWithContext(ctx, http.MethodGet, fmt.Sprintf("%s/api/v1/sagas/%s", c.endpoint, sagaID), nil)
	if err != nil {
		return nil, &SagaError{Op: "get_saga", Err: fmt.Errorf("failed to create request: %w", err)}
	}

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return nil, &SagaError{Op: "get_saga", Err: fmt.Errorf("request failed: %w", err)}
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		respBody, _ := io.ReadAll(resp.Body)
		return nil, &SagaError{
			Op:         "get_saga",
			StatusCode: resp.StatusCode,
			Err:        fmt.Errorf("%s", string(respBody)),
		}
	}

	// レスポンスは {"saga": {...}} の形式
	var wrapper struct {
		Saga SagaState `json:"saga"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&wrapper); err != nil {
		return nil, &SagaError{Op: "get_saga", Err: fmt.Errorf("failed to decode response: %w", err)}
	}

	return &wrapper.Saga, nil
}

// CancelSaga は Saga をキャンセルする。
// POST /api/v1/sagas/{sagaID}/cancel
func (c *SagaClient) CancelSaga(ctx context.Context, sagaID string) error {
	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, fmt.Sprintf("%s/api/v1/sagas/%s/cancel", c.endpoint, sagaID), nil)
	if err != nil {
		return &SagaError{Op: "cancel_saga", Err: fmt.Errorf("failed to create request: %w", err)}
	}

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return &SagaError{Op: "cancel_saga", Err: fmt.Errorf("request failed: %w", err)}
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		respBody, _ := io.ReadAll(resp.Body)
		return &SagaError{
			Op:         "cancel_saga",
			StatusCode: resp.StatusCode,
			Err:        fmt.Errorf("%s", string(respBody)),
		}
	}

	return nil
}
