package fileclient

import (
	"context"
	"fmt"
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
