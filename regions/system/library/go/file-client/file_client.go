package fileclient

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"sync"
	"time"
)

// FileMetadata はファイルメタデータ。
type FileMetadata struct {
	Path         string            `json:"path"`
	SizeBytes    int64             `json:"size_bytes"`
	ContentType  string            `json:"content_type"`
	ETag         string            `json:"etag"`
	LastModified time.Time         `json:"last_modified"`
	Tags         map[string]string `json:"tags"`
}

// PresignedURL はプリサインドURL。
type PresignedURL struct {
	URL       string            `json:"url"`
	Method    string            `json:"method"`
	ExpiresAt time.Time         `json:"expires_at"`
	Headers   map[string]string `json:"headers"`
}

// FileClient はファイルストレージ操作のインターフェース。
type FileClient interface {
	GenerateUploadURL(ctx context.Context, path, contentType string, expiresIn time.Duration) (*PresignedURL, error)
	GenerateDownloadURL(ctx context.Context, path string, expiresIn time.Duration) (*PresignedURL, error)
	Delete(ctx context.Context, path string) error
	GetMetadata(ctx context.Context, path string) (*FileMetadata, error)
	List(ctx context.Context, prefix string) ([]*FileMetadata, error)
	Copy(ctx context.Context, src, dst string) error
}

// InMemoryFileClient はメモリ内のファイルクライアント。
type InMemoryFileClient struct {
	mu    sync.Mutex
	files map[string]*FileMetadata
}

// NewInMemoryFileClient は新しい InMemoryFileClient を生成する。
func NewInMemoryFileClient() *InMemoryFileClient {
	return &InMemoryFileClient{
		files: make(map[string]*FileMetadata),
	}
}

func (c *InMemoryFileClient) GenerateUploadURL(_ context.Context, path, contentType string, expiresIn time.Duration) (*PresignedURL, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.files[path] = &FileMetadata{
		Path:         path,
		SizeBytes:    0,
		ContentType:  contentType,
		ETag:         "",
		LastModified: time.Now(),
		Tags:         make(map[string]string),
	}
	return &PresignedURL{
		URL:       fmt.Sprintf("https://storage.example.com/upload/%s", path),
		Method:    "PUT",
		ExpiresAt: time.Now().Add(expiresIn),
		Headers:   make(map[string]string),
	}, nil
}

func (c *InMemoryFileClient) GenerateDownloadURL(_ context.Context, path string, expiresIn time.Duration) (*PresignedURL, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if _, ok := c.files[path]; !ok {
		return nil, fmt.Errorf("file not found: %s", path)
	}
	return &PresignedURL{
		URL:       fmt.Sprintf("https://storage.example.com/download/%s", path),
		Method:    "GET",
		ExpiresAt: time.Now().Add(expiresIn),
		Headers:   make(map[string]string),
	}, nil
}

func (c *InMemoryFileClient) Delete(_ context.Context, path string) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if _, ok := c.files[path]; !ok {
		return fmt.Errorf("file not found: %s", path)
	}
	delete(c.files, path)
	return nil
}

func (c *InMemoryFileClient) GetMetadata(_ context.Context, path string) (*FileMetadata, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	meta, ok := c.files[path]
	if !ok {
		return nil, fmt.Errorf("file not found: %s", path)
	}
	copied := *meta
	return &copied, nil
}

func (c *InMemoryFileClient) List(_ context.Context, prefix string) ([]*FileMetadata, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	var result []*FileMetadata
	for _, meta := range c.files {
		if strings.HasPrefix(meta.Path, prefix) {
			copied := *meta
			result = append(result, &copied)
		}
	}
	return result, nil
}

func (c *InMemoryFileClient) Copy(_ context.Context, src, dst string) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	source, ok := c.files[src]
	if !ok {
		return fmt.Errorf("file not found: %s", src)
	}
	copied := *source
	copied.Path = dst
	c.files[dst] = &copied
	return nil
}

// StoredFiles は保存されているファイル一覧を返す。
func (c *InMemoryFileClient) StoredFiles() []*FileMetadata {
	c.mu.Lock()
	defer c.mu.Unlock()
	var result []*FileMetadata
	for _, meta := range c.files {
		copied := *meta
		result = append(result, &copied)
	}
	return result
}

// ---------------------------------------------------------------------------
// FileClientConfig — バックエンド設定
// ---------------------------------------------------------------------------

// FileClientConfig はファイルクライアントの設定。
type FileClientConfig struct {
	ServerURL       string
	S3Endpoint      string
	Bucket          string
	Region          string
	AccessKeyID     string
	SecretAccessKey string
	Timeout         time.Duration
}

// Option は FileClientConfig を変更するオプション関数。
type Option func(*FileClientConfig)

// WithTimeout はタイムアウトを設定する。
func WithTimeout(d time.Duration) Option {
	return func(c *FileClientConfig) { c.Timeout = d }
}

// ---------------------------------------------------------------------------
// ServerFileClient — file-server 経由の HTTP 実装
// ---------------------------------------------------------------------------

// ServerFileClient は file-server に HTTP で委譲する FileClient 実装。
type ServerFileClient struct {
	baseURL string
	http    *http.Client
}

// NewServerFileClient は ServerFileClient を生成する。
func NewServerFileClient(serverURL string, opts ...Option) FileClient {
	cfg := &FileClientConfig{
		ServerURL: serverURL,
		Timeout:   30 * time.Second,
	}
	for _, o := range opts {
		o(cfg)
	}
	return &ServerFileClient{
		baseURL: strings.TrimRight(serverURL, "/"),
		http:    &http.Client{Timeout: cfg.Timeout},
	}
}

// generateUrlRequest は upload/download URL 生成リクエストの DTO。
type generateUrlRequest struct {
	Path          string `json:"path"`
	ContentType   string `json:"content_type,omitempty"`
	ExpiresInSecs uint64 `json:"expires_in_secs"`
}

// generateUrlResponse は upload/download URL 生成レスポンスの DTO。
type generateUrlResponse struct {
	URL       string            `json:"url"`
	Method    string            `json:"method"`
	ExpiresAt time.Time         `json:"expires_at"`
	Headers   map[string]string `json:"headers"`
}

// copyRequest はコピーリクエストの DTO。
type copyRequest struct {
	Src string `json:"src"`
	Dst string `json:"dst"`
}

func (c *ServerFileClient) doJSON(ctx context.Context, method, path string, body any, out any) error {
	var r io.Reader
	if body != nil {
		b, err := json.Marshal(body)
		if err != nil {
			return fmt.Errorf("marshal: %w", err)
		}
		r = bytes.NewReader(b)
	}
	req, err := http.NewRequestWithContext(ctx, method, c.baseURL+path, r)
	if err != nil {
		return err
	}
	if body != nil {
		req.Header.Set("Content-Type", "application/json")
	}
	resp, err := c.http.Do(req)
	if err != nil {
		return fmt.Errorf("connection error: %w", err)
	}
	defer resp.Body.Close()
	respBody, _ := io.ReadAll(resp.Body)
	if resp.StatusCode == http.StatusNotFound {
		return fmt.Errorf("not found: %s", string(respBody))
	}
	if resp.StatusCode == http.StatusUnauthorized || resp.StatusCode == http.StatusForbidden {
		return fmt.Errorf("unauthorized: %s", string(respBody))
	}
	if resp.StatusCode >= 300 {
		return fmt.Errorf("HTTP %d: %s", resp.StatusCode, string(respBody))
	}
	if out != nil {
		return json.Unmarshal(respBody, out)
	}
	return nil
}

func (c *ServerFileClient) GenerateUploadURL(ctx context.Context, path, contentType string, expiresIn time.Duration) (*PresignedURL, error) {
	var res generateUrlResponse
	err := c.doJSON(ctx, http.MethodPost, "/api/v1/files/upload-url", generateUrlRequest{
		Path:          path,
		ContentType:   contentType,
		ExpiresInSecs: uint64(expiresIn.Seconds()),
	}, &res)
	if err != nil {
		return nil, err
	}
	return &PresignedURL{URL: res.URL, Method: res.Method, ExpiresAt: res.ExpiresAt, Headers: res.Headers}, nil
}

func (c *ServerFileClient) GenerateDownloadURL(ctx context.Context, path string, expiresIn time.Duration) (*PresignedURL, error) {
	var res generateUrlResponse
	err := c.doJSON(ctx, http.MethodPost, "/api/v1/files/download-url", generateUrlRequest{
		Path:          path,
		ExpiresInSecs: uint64(expiresIn.Seconds()),
	}, &res)
	if err != nil {
		return nil, err
	}
	return &PresignedURL{URL: res.URL, Method: res.Method, ExpiresAt: res.ExpiresAt, Headers: res.Headers}, nil
}

func (c *ServerFileClient) Delete(ctx context.Context, path string) error {
	encoded := url.PathEscape(path)
	return c.doJSON(ctx, http.MethodDelete, "/api/v1/files/"+encoded, nil, nil)
}

func (c *ServerFileClient) GetMetadata(ctx context.Context, path string) (*FileMetadata, error) {
	encoded := url.PathEscape(path)
	var meta FileMetadata
	err := c.doJSON(ctx, http.MethodGet, "/api/v1/files/"+encoded+"/metadata", nil, &meta)
	if err != nil {
		return nil, err
	}
	return &meta, nil
}

func (c *ServerFileClient) List(ctx context.Context, prefix string) ([]*FileMetadata, error) {
	path := "/api/v1/files?prefix=" + url.QueryEscape(prefix)
	var files []*FileMetadata
	err := c.doJSON(ctx, http.MethodGet, path, nil, &files)
	if err != nil {
		return nil, err
	}
	return files, nil
}

func (c *ServerFileClient) Copy(ctx context.Context, src, dst string) error {
	return c.doJSON(ctx, http.MethodPost, "/api/v1/files/copy", copyRequest{Src: src, Dst: dst}, nil)
}

// ---------------------------------------------------------------------------
// S3FileClient — AWS S3 / GCS / Ceph 直接実装
// ---------------------------------------------------------------------------

// S3FileClient は S3 互換ストレージに直接アクセスする FileClient 実装。
// 実際の S3 操作は aws-sdk-go-v2 を使用するが、ここでは依存を追加せずに
// プリサインドURL生成の骨格のみを実装する（SDK 統合は利用側で拡張する）。
type S3FileClient struct {
	endpoint string
	bucket   string
	region   string
}

// NewS3FileClient は S3FileClient を生成する。
// endpoint は S3 互換エンドポイント（例: "https://s3.amazonaws.com"）。
func NewS3FileClient(endpoint, bucket, region string, opts ...Option) FileClient {
	cfg := &FileClientConfig{
		S3Endpoint: endpoint,
		Bucket:     bucket,
		Region:     region,
		Timeout:    30 * time.Second,
	}
	for _, o := range opts {
		o(cfg)
	}
	return &S3FileClient{
		endpoint: strings.TrimRight(endpoint, "/"),
		bucket:   bucket,
		region:   region,
	}
}

func (c *S3FileClient) objectURL(path string) string {
	return fmt.Sprintf("%s/%s/%s", c.endpoint, c.bucket, path)
}

func (c *S3FileClient) GenerateUploadURL(_ context.Context, path, contentType string, expiresIn time.Duration) (*PresignedURL, error) {
	return &PresignedURL{
		URL:       c.objectURL(path),
		Method:    "PUT",
		ExpiresAt: time.Now().Add(expiresIn),
		Headers:   map[string]string{"Content-Type": contentType},
	}, nil
}

func (c *S3FileClient) GenerateDownloadURL(_ context.Context, path string, expiresIn time.Duration) (*PresignedURL, error) {
	return &PresignedURL{
		URL:       c.objectURL(path),
		Method:    "GET",
		ExpiresAt: time.Now().Add(expiresIn),
		Headers:   map[string]string{},
	}, nil
}

func (c *S3FileClient) Delete(_ context.Context, path string) error {
	return fmt.Errorf("S3FileClient.Delete: aws-sdk-go-v2 統合が必要です (path=%s)", path)
}

func (c *S3FileClient) GetMetadata(_ context.Context, path string) (*FileMetadata, error) {
	return nil, fmt.Errorf("S3FileClient.GetMetadata: aws-sdk-go-v2 統合が必要です (path=%s)", path)
}

func (c *S3FileClient) List(_ context.Context, prefix string) ([]*FileMetadata, error) {
	return nil, fmt.Errorf("S3FileClient.List: aws-sdk-go-v2 統合が必要です (prefix=%s)", prefix)
}

func (c *S3FileClient) Copy(_ context.Context, src, dst string) error {
	return fmt.Errorf("S3FileClient.Copy: aws-sdk-go-v2 統合が必要です (src=%s, dst=%s)", src, dst)
}
