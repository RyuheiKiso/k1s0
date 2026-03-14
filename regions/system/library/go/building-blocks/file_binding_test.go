package buildingblocks

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
	"time"
)

// mockFileClient は FileClientIface のテスト用モック実装。
type mockFileClient struct {
	uploadURL   *FilePresignedURL
	downloadURL *FilePresignedURL
	files       []*FileInfo
	err         error
}

func (m *mockFileClient) GenerateUploadURL(_ context.Context, path, _ string, _ time.Duration) (*FilePresignedURL, error) {
	if m.err != nil {
		return nil, m.err
	}
	if m.uploadURL != nil {
		return m.uploadURL, nil
	}
	return &FilePresignedURL{URL: "https://upload.example.com/" + path, Method: "PUT"}, nil
}

func (m *mockFileClient) GenerateDownloadURL(_ context.Context, path string, _ time.Duration) (*FilePresignedURL, error) {
	if m.err != nil {
		return nil, m.err
	}
	if m.downloadURL != nil {
		return m.downloadURL, nil
	}
	return &FilePresignedURL{URL: "https://download.example.com/" + path, Method: "GET"}, nil
}

func (m *mockFileClient) Delete(_ context.Context, _ string) error {
	return m.err
}

func (m *mockFileClient) List(_ context.Context, _ string) ([]*FileInfo, error) {
	if m.err != nil {
		return nil, m.err
	}
	return m.files, nil
}

func (m *mockFileClient) Copy(_ context.Context, _, _ string) error {
	return m.err
}

// TestFileOutputBinding_InitAndStatus は Init 前後でステータスが Uninitialized → Ready に遷移することを検証する。
func TestFileOutputBinding_InitAndStatus(t *testing.T) {
	b := NewFileOutputBinding("file-binding", &mockFileClient{})
	ctx := context.Background()

	if b.Status(ctx) != StatusUninitialized {
		t.Errorf("expected StatusUninitialized, got %s", b.Status(ctx))
	}
	if err := b.Init(ctx, Metadata{}); err != nil {
		t.Fatalf("Init failed: %v", err)
	}
	if b.Status(ctx) != StatusReady {
		t.Errorf("expected StatusReady, got %s", b.Status(ctx))
	}
}

// TestFileOutputBinding_NameVersion は Name と Version が正しい値を返すことを検証する。
func TestFileOutputBinding_NameVersion(t *testing.T) {
	b := NewFileOutputBinding("my-files", &mockFileClient{})
	if b.Name() != "my-files" {
		t.Errorf("unexpected Name: %q", b.Name())
	}
	if b.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", b.Version())
	}
}

// TestFileOutputBinding_UploadURL は upload-url オペレーションで PUT メソッドの署名付き URL が返ることを検証する。
func TestFileOutputBinding_UploadURL(t *testing.T) {
	b := NewFileOutputBinding("files", &mockFileClient{})
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	resp, err := b.Invoke(ctx, "upload-url", nil, map[string]string{
		"path":         "images/photo.jpg",
		"content-type": "image/jpeg",
	})
	if err != nil {
		t.Fatalf("Invoke upload-url failed: %v", err)
	}

	var info FilePresignedURL
	if err := json.Unmarshal(resp.Data, &info); err != nil {
		t.Fatalf("failed to unmarshal response: %v", err)
	}
	if info.Method != "PUT" {
		t.Errorf("expected Method 'PUT', got %q", info.Method)
	}
}

// TestFileOutputBinding_UploadURLMissingPath は upload-url オペレーションで path が未指定の場合にエラーになることを検証する。
func TestFileOutputBinding_UploadURLMissingPath(t *testing.T) {
	b := NewFileOutputBinding("files", &mockFileClient{})
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	_, err := b.Invoke(ctx, "upload-url", nil, map[string]string{})
	if err == nil {
		t.Fatal("expected error when path is missing")
	}
}

// TestFileOutputBinding_DownloadURL は download-url オペレーションで GET メソッドの署名付き URL が返ることを検証する。
func TestFileOutputBinding_DownloadURL(t *testing.T) {
	b := NewFileOutputBinding("files", &mockFileClient{})
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	resp, err := b.Invoke(ctx, "download-url", nil, map[string]string{"path": "docs/readme.pdf"})
	if err != nil {
		t.Fatalf("Invoke download-url failed: %v", err)
	}

	var info FilePresignedURL
	if err := json.Unmarshal(resp.Data, &info); err != nil {
		t.Fatalf("failed to unmarshal response: %v", err)
	}
	if info.Method != "GET" {
		t.Errorf("expected Method 'GET', got %q", info.Method)
	}
}

// TestFileOutputBinding_DownloadURLMissingPath は download-url オペレーションで path が未指定の場合にエラーになることを検証する。
func TestFileOutputBinding_DownloadURLMissingPath(t *testing.T) {
	b := NewFileOutputBinding("files", &mockFileClient{})
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	_, err := b.Invoke(ctx, "download-url", nil, map[string]string{})
	if err == nil {
		t.Fatal("expected error when path is missing")
	}
}

// TestFileOutputBinding_Delete は delete オペレーションでファイルを削除しレスポンスデータが nil であることを検証する。
func TestFileOutputBinding_Delete(t *testing.T) {
	b := NewFileOutputBinding("files", &mockFileClient{})
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	resp, err := b.Invoke(ctx, "delete", nil, map[string]string{"path": "old/file.txt"})
	if err != nil {
		t.Fatalf("Invoke delete failed: %v", err)
	}
	if resp.Data != nil {
		t.Errorf("expected nil Data for delete, got %v", resp.Data)
	}
}

// TestFileOutputBinding_DeleteMissingPath は delete オペレーションで path が未指定の場合にエラーになることを検証する。
func TestFileOutputBinding_DeleteMissingPath(t *testing.T) {
	b := NewFileOutputBinding("files", &mockFileClient{})
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	_, err := b.Invoke(ctx, "delete", nil, map[string]string{})
	if err == nil {
		t.Fatal("expected error when path is missing")
	}
}

// TestFileOutputBinding_List は list オペレーションでプレフィックス配下のファイル一覧を取得できることを検証する。
func TestFileOutputBinding_List(t *testing.T) {
	client := &mockFileClient{
		files: []*FileInfo{
			{Path: "images/a.jpg", SizeBytes: 1024, ContentType: "image/jpeg"},
			{Path: "images/b.png", SizeBytes: 2048, ContentType: "image/png"},
		},
	}
	b := NewFileOutputBinding("files", client)
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	resp, err := b.Invoke(ctx, "list", nil, map[string]string{"prefix": "images/"})
	if err != nil {
		t.Fatalf("Invoke list failed: %v", err)
	}

	var files []*FileInfo
	if err := json.Unmarshal(resp.Data, &files); err != nil {
		t.Fatalf("failed to unmarshal response: %v", err)
	}
	if len(files) != 2 {
		t.Fatalf("expected 2 files, got %d", len(files))
	}
	if files[0].Path != "images/a.jpg" {
		t.Errorf("unexpected first file: %q", files[0].Path)
	}
}

// TestFileOutputBinding_Copy は copy オペレーションでファイルをコピーしレスポンスデータが nil であることを検証する。
func TestFileOutputBinding_Copy(t *testing.T) {
	b := NewFileOutputBinding("files", &mockFileClient{})
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	resp, err := b.Invoke(ctx, "copy", nil, map[string]string{
		"src": "originals/photo.jpg",
		"dst": "thumbs/photo.jpg",
	})
	if err != nil {
		t.Fatalf("Invoke copy failed: %v", err)
	}
	if resp.Data != nil {
		t.Errorf("expected nil Data for copy, got %v", resp.Data)
	}
}

// TestFileOutputBinding_CopyMissingParams は copy オペレーションで dst が未指定の場合にエラーになることを検証する。
func TestFileOutputBinding_CopyMissingParams(t *testing.T) {
	b := NewFileOutputBinding("files", &mockFileClient{})
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	_, err := b.Invoke(ctx, "copy", nil, map[string]string{"src": "only-src"})
	if err == nil {
		t.Fatal("expected error when dst is missing")
	}
}

// TestFileOutputBinding_UnsupportedOperation はサポートされていないオペレーションを指定するとエラーになることを検証する。
func TestFileOutputBinding_UnsupportedOperation(t *testing.T) {
	b := NewFileOutputBinding("files", &mockFileClient{})
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	_, err := b.Invoke(ctx, "unknown-op", nil, map[string]string{})
	if err == nil {
		t.Fatal("expected error for unsupported operation")
	}
}

// TestFileOutputBinding_ClientError はクライアントがエラーを返す場合に Invoke がエラーになることを検証する。
func TestFileOutputBinding_ClientError(t *testing.T) {
	client := &mockFileClient{err: errors.New("s3 error")}
	b := NewFileOutputBinding("files", client)
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	_, err := b.Invoke(ctx, "delete", nil, map[string]string{"path": "f"})
	if err == nil {
		t.Fatal("expected error from client")
	}
}

// TestFileOutputBinding_Close は Close 後にステータスが StatusClosed に遷移することを検証する。
func TestFileOutputBinding_Close(t *testing.T) {
	b := NewFileOutputBinding("files", &mockFileClient{})
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	if err := b.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if b.Status(ctx) != StatusClosed {
		t.Errorf("expected StatusClosed, got %s", b.Status(ctx))
	}
}
