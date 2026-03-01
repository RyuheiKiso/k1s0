package searchclient

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
)

// GrpcSearchClient は search-server への HTTP クライアント実装。
// 名称は gRPC クライアントだが、HTTP/JSON API 経由で search-server と通信する。
type GrpcSearchClient struct {
	endpoint   string
	httpClient *http.Client
}

// NewGrpcSearchClient は新しい GrpcSearchClient を生成する。
// addr は "host:port" または "http://host:port" 形式。
func NewGrpcSearchClient(addr string) (*GrpcSearchClient, error) {
	endpoint := addr
	if !strings.HasPrefix(endpoint, "http://") && !strings.HasPrefix(endpoint, "https://") {
		endpoint = "http://" + endpoint
	}
	endpoint = strings.TrimRight(endpoint, "/")
	return &GrpcSearchClient{
		endpoint:   endpoint,
		httpClient: &http.Client{},
	}, nil
}

// NewGrpcSearchClientWithHTTPClient はカスタム http.Client を使う GrpcSearchClient を生成する（テスト用）。
func NewGrpcSearchClientWithHTTPClient(addr string, httpClient *http.Client) (*GrpcSearchClient, error) {
	endpoint := addr
	if !strings.HasPrefix(endpoint, "http://") && !strings.HasPrefix(endpoint, "https://") {
		endpoint = "http://" + endpoint
	}
	endpoint = strings.TrimRight(endpoint, "/")
	return &GrpcSearchClient{
		endpoint:   endpoint,
		httpClient: httpClient,
	}, nil
}

// Close はクライアントのリソースを解放する。
func (c *GrpcSearchClient) Close() {}

// CreateIndex は指定名とマッピングでインデックスを作成する。
// PUT /api/v1/indexes/:name
func (c *GrpcSearchClient) CreateIndex(ctx context.Context, name string, mapping IndexMapping) error {
	body, err := json.Marshal(map[string]interface{}{
		"name":    name,
		"mapping": mapping,
	})
	if err != nil {
		return fmt.Errorf("search: create_index marshal: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPut,
		fmt.Sprintf("%s/api/v1/indexes/%s", c.endpoint, name),
		bytes.NewReader(body))
	if err != nil {
		return fmt.Errorf("search: create_index request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("search: create_index: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusCreated && resp.StatusCode != http.StatusNoContent {
		respBody, _ := io.ReadAll(resp.Body)
		return parseSearchError("create_index", resp.StatusCode, respBody)
	}
	return nil
}

// IndexDocument はドキュメントをインデックスに登録する。
// POST /api/v1/indexes/:index/documents
func (c *GrpcSearchClient) IndexDocument(ctx context.Context, index string, doc IndexDocument) (IndexResult, error) {
	body, err := json.Marshal(doc)
	if err != nil {
		return IndexResult{}, fmt.Errorf("search: index_document marshal: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost,
		fmt.Sprintf("%s/api/v1/indexes/%s/documents", c.endpoint, index),
		bytes.NewReader(body))
	if err != nil {
		return IndexResult{}, fmt.Errorf("search: index_document request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return IndexResult{}, fmt.Errorf("search: index_document: %w", err)
	}
	defer resp.Body.Close()

	respBody, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusCreated {
		return IndexResult{}, parseSearchError("index_document", resp.StatusCode, respBody)
	}

	var result IndexResult
	if err := json.Unmarshal(respBody, &result); err != nil {
		return IndexResult{}, fmt.Errorf("search: index_document decode: %w", err)
	}
	return result, nil
}

// BulkIndex は複数ドキュメントをまとめてインデックス登録する。
// POST /api/v1/indexes/:index/documents/_bulk
func (c *GrpcSearchClient) BulkIndex(ctx context.Context, index string, docs []IndexDocument) (BulkResult, error) {
	body, err := json.Marshal(map[string]interface{}{"documents": docs})
	if err != nil {
		return BulkResult{}, fmt.Errorf("search: bulk_index marshal: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost,
		fmt.Sprintf("%s/api/v1/indexes/%s/documents/_bulk", c.endpoint, index),
		bytes.NewReader(body))
	if err != nil {
		return BulkResult{}, fmt.Errorf("search: bulk_index request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return BulkResult{}, fmt.Errorf("search: bulk_index: %w", err)
	}
	defer resp.Body.Close()

	respBody, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusCreated {
		return BulkResult{}, parseSearchError("bulk_index", resp.StatusCode, respBody)
	}

	var result BulkResult
	if err := json.Unmarshal(respBody, &result); err != nil {
		return BulkResult{}, fmt.Errorf("search: bulk_index decode: %w", err)
	}
	return result, nil
}

// Search はインデックスに対して検索を実行する。
// POST /api/v1/indexes/:index/_search
func (c *GrpcSearchClient) Search(ctx context.Context, index string, query SearchQuery) (SearchResult, error) {
	body, err := json.Marshal(query)
	if err != nil {
		return SearchResult{}, fmt.Errorf("search: search marshal: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost,
		fmt.Sprintf("%s/api/v1/indexes/%s/_search", c.endpoint, index),
		bytes.NewReader(body))
	if err != nil {
		return SearchResult{}, fmt.Errorf("search: search request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return SearchResult{}, fmt.Errorf("search: search: %w", err)
	}
	defer resp.Body.Close()

	respBody, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusOK {
		return SearchResult{}, parseSearchError("search", resp.StatusCode, respBody)
	}

	var result SearchResult
	if err := json.Unmarshal(respBody, &result); err != nil {
		return SearchResult{}, fmt.Errorf("search: search decode: %w", err)
	}
	return result, nil
}

// DeleteDocument は指定ドキュメントをインデックスから削除する。
// DELETE /api/v1/indexes/:index/documents/:id
func (c *GrpcSearchClient) DeleteDocument(ctx context.Context, index, id string) error {
	req, err := http.NewRequestWithContext(ctx, http.MethodDelete,
		fmt.Sprintf("%s/api/v1/indexes/%s/documents/%s", c.endpoint, index, id),
		nil)
	if err != nil {
		return fmt.Errorf("search: delete_document request: %w", err)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("search: delete_document: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusNoContent {
		respBody, _ := io.ReadAll(resp.Body)
		return parseSearchError("delete_document", resp.StatusCode, respBody)
	}
	return nil
}

// parseSearchError はHTTPステータスコードとレスポンスボディからエラーを生成する。
func parseSearchError(op string, statusCode int, body []byte) error {
	msg := string(body)
	switch statusCode {
	case http.StatusNotFound:
		return fmt.Errorf("search: %s: index not found: %s", op, msg)
	case http.StatusBadRequest:
		return fmt.Errorf("search: %s: invalid query: %s", op, msg)
	case http.StatusGatewayTimeout, http.StatusRequestTimeout:
		return fmt.Errorf("search: %s: timeout", op)
	default:
		return fmt.Errorf("search: %s: server error (status=%d): %s", op, statusCode, msg)
	}
}
