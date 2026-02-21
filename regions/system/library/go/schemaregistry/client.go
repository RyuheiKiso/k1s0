package schemaregistry

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
)

// httpSchemaRegistryClient は net/http を使った SchemaRegistryClient 実装。
type httpSchemaRegistryClient struct {
	config     *SchemaRegistryConfig
	httpClient *http.Client
}

// NewClient は新しい SchemaRegistryClient を生成する。
func NewClient(config *SchemaRegistryConfig) (SchemaRegistryClient, error) {
	if err := config.Validate(); err != nil {
		return nil, err
	}
	return &httpSchemaRegistryClient{
		config:     config,
		httpClient: &http.Client{},
	}, nil
}

// NewClientWithHTTPClient はカスタム http.Client を使う SchemaRegistryClient を生成する（テスト用）。
func NewClientWithHTTPClient(config *SchemaRegistryConfig, httpClient *http.Client) (SchemaRegistryClient, error) {
	if err := config.Validate(); err != nil {
		return nil, err
	}
	return &httpSchemaRegistryClient{
		config:     config,
		httpClient: httpClient,
	}, nil
}

func (c *httpSchemaRegistryClient) doRequest(ctx context.Context, method, path string, body interface{}) (*http.Response, error) {
	var reqBody io.Reader
	if body != nil {
		data, err := json.Marshal(body)
		if err != nil {
			return nil, fmt.Errorf("marshal request body: %w", err)
		}
		reqBody = bytes.NewReader(data)
	}

	req, err := http.NewRequestWithContext(ctx, method, c.config.URL+path, reqBody)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/vnd.schemaregistry.v1+json")

	if c.config.Username != "" {
		req.SetBasicAuth(c.config.Username, c.config.Password)
	}

	return c.httpClient.Do(req)
}

// RegisterSchema はスキーマを登録し、スキーマ ID を返す。
func (c *httpSchemaRegistryClient) RegisterSchema(ctx context.Context, subject, schema, schemaType string) (int, error) {
	reqBody := map[string]string{
		"schema":     schema,
		"schemaType": schemaType,
	}
	resp, err := c.doRequest(ctx, http.MethodPost, fmt.Sprintf("/subjects/%s/versions", subject), reqBody)
	if err != nil {
		return 0, &SchemaRegistryError{StatusCode: 0, Message: err.Error()}
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return 0, &NotFoundError{Resource: subject}
	}
	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return 0, &SchemaRegistryError{StatusCode: resp.StatusCode, Message: string(body)}
	}

	var result struct {
		ID int `json:"id"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return 0, fmt.Errorf("decode response: %w", err)
	}
	return result.ID, nil
}

// GetSchemaByID はスキーマ ID でスキーマを取得する。
func (c *httpSchemaRegistryClient) GetSchemaByID(ctx context.Context, id int) (*RegisteredSchema, error) {
	resp, err := c.doRequest(ctx, http.MethodGet, fmt.Sprintf("/schemas/ids/%d", id), nil)
	if err != nil {
		return nil, &SchemaRegistryError{StatusCode: 0, Message: err.Error()}
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return nil, &NotFoundError{Resource: fmt.Sprintf("schema id=%d", id)}
	}
	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, &SchemaRegistryError{StatusCode: resp.StatusCode, Message: string(body)}
	}

	var result RegisteredSchema
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("decode response: %w", err)
	}
	result.ID = id
	return &result, nil
}

// GetLatestSchema はサブジェクトの最新スキーマを取得する。
func (c *httpSchemaRegistryClient) GetLatestSchema(ctx context.Context, subject string) (*RegisteredSchema, error) {
	return c.getSchemaVersion(ctx, subject, "latest")
}

// GetSchemaVersion はサブジェクトの特定バージョンのスキーマを取得する。
func (c *httpSchemaRegistryClient) GetSchemaVersion(ctx context.Context, subject string, version int) (*RegisteredSchema, error) {
	return c.getSchemaVersion(ctx, subject, fmt.Sprintf("%d", version))
}

func (c *httpSchemaRegistryClient) getSchemaVersion(ctx context.Context, subject, version string) (*RegisteredSchema, error) {
	resp, err := c.doRequest(ctx, http.MethodGet, fmt.Sprintf("/subjects/%s/versions/%s", subject, version), nil)
	if err != nil {
		return nil, &SchemaRegistryError{StatusCode: 0, Message: err.Error()}
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return nil, &NotFoundError{Resource: fmt.Sprintf("%s/versions/%s", subject, version)}
	}
	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, &SchemaRegistryError{StatusCode: resp.StatusCode, Message: string(body)}
	}

	var result RegisteredSchema
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("decode response: %w", err)
	}
	return &result, nil
}

// ListSubjects は全サブジェクトの一覧を返す。
func (c *httpSchemaRegistryClient) ListSubjects(ctx context.Context) ([]string, error) {
	resp, err := c.doRequest(ctx, http.MethodGet, "/subjects", nil)
	if err != nil {
		return nil, &SchemaRegistryError{StatusCode: 0, Message: err.Error()}
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, &SchemaRegistryError{StatusCode: resp.StatusCode, Message: string(body)}
	}

	var subjects []string
	if err := json.NewDecoder(resp.Body).Decode(&subjects); err != nil {
		return nil, fmt.Errorf("decode response: %w", err)
	}
	return subjects, nil
}

// CheckCompatibility はスキーマの後方互換性を検証する。
func (c *httpSchemaRegistryClient) CheckCompatibility(ctx context.Context, subject, schema string) (bool, error) {
	reqBody := map[string]string{"schema": schema}
	resp, err := c.doRequest(ctx, http.MethodPost, fmt.Sprintf("/compatibility/subjects/%s/versions/latest", subject), reqBody)
	if err != nil {
		return false, &SchemaRegistryError{StatusCode: 0, Message: err.Error()}
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return false, &NotFoundError{Resource: subject}
	}
	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return false, &SchemaRegistryError{StatusCode: resp.StatusCode, Message: string(body)}
	}

	var result struct {
		IsCompatible bool `json:"is_compatible"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return false, fmt.Errorf("decode response: %w", err)
	}
	return result.IsCompatible, nil
}

// HealthCheck は Schema Registry の疎通確認を行う。
func (c *httpSchemaRegistryClient) HealthCheck(ctx context.Context) error {
	resp, err := c.doRequest(ctx, http.MethodGet, "/", nil)
	if err != nil {
		return &SchemaRegistryError{StatusCode: 0, Message: err.Error()}
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return &SchemaRegistryError{StatusCode: resp.StatusCode, Message: "health check failed"}
	}
	return nil
}
