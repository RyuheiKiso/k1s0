package fileclient_test

import (
	"context"
	"testing"
	"time"

	fileclient "github.com/k1s0-platform/system-library-go-file-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGenerateUploadURL(t *testing.T) {
	c := fileclient.NewInMemoryFileClient()
	url, err := c.GenerateUploadURL(context.Background(), "uploads/test.png", "image/png", time.Hour)
	require.NoError(t, err)
	assert.Contains(t, url.URL, "uploads/test.png")
	assert.Equal(t, "PUT", url.Method)
}

func TestGenerateDownloadURL(t *testing.T) {
	c := fileclient.NewInMemoryFileClient()
	ctx := context.Background()
	_, _ = c.GenerateUploadURL(ctx, "uploads/test.png", "image/png", time.Hour)
	url, err := c.GenerateDownloadURL(ctx, "uploads/test.png", 5*time.Minute)
	require.NoError(t, err)
	assert.Contains(t, url.URL, "uploads/test.png")
	assert.Equal(t, "GET", url.Method)
}

func TestGenerateDownloadURL_NotFound(t *testing.T) {
	c := fileclient.NewInMemoryFileClient()
	_, err := c.GenerateDownloadURL(context.Background(), "nonexistent.txt", 5*time.Minute)
	assert.Error(t, err)
}

func TestDelete(t *testing.T) {
	c := fileclient.NewInMemoryFileClient()
	ctx := context.Background()
	_, _ = c.GenerateUploadURL(ctx, "uploads/test.png", "image/png", time.Hour)
	err := c.Delete(ctx, "uploads/test.png")
	require.NoError(t, err)
	_, err = c.GetMetadata(ctx, "uploads/test.png")
	assert.Error(t, err)
}

func TestGetMetadata(t *testing.T) {
	c := fileclient.NewInMemoryFileClient()
	ctx := context.Background()
	_, _ = c.GenerateUploadURL(ctx, "uploads/test.png", "image/png", time.Hour)
	meta, err := c.GetMetadata(ctx, "uploads/test.png")
	require.NoError(t, err)
	assert.Equal(t, "uploads/test.png", meta.Path)
	assert.Equal(t, "image/png", meta.ContentType)
}

func TestList(t *testing.T) {
	c := fileclient.NewInMemoryFileClient()
	ctx := context.Background()
	_, _ = c.GenerateUploadURL(ctx, "uploads/a.png", "image/png", time.Hour)
	_, _ = c.GenerateUploadURL(ctx, "uploads/b.jpg", "image/jpeg", time.Hour)
	_, _ = c.GenerateUploadURL(ctx, "other/c.txt", "text/plain", time.Hour)
	files, err := c.List(ctx, "uploads/")
	require.NoError(t, err)
	assert.Len(t, files, 2)
}

func TestCopy(t *testing.T) {
	c := fileclient.NewInMemoryFileClient()
	ctx := context.Background()
	_, _ = c.GenerateUploadURL(ctx, "uploads/test.png", "image/png", time.Hour)
	err := c.Copy(ctx, "uploads/test.png", "archive/test.png")
	require.NoError(t, err)
	meta, err := c.GetMetadata(ctx, "archive/test.png")
	require.NoError(t, err)
	assert.Equal(t, "image/png", meta.ContentType)
}

func TestCopy_NotFound(t *testing.T) {
	c := fileclient.NewInMemoryFileClient()
	err := c.Copy(context.Background(), "nonexistent.txt", "dest.txt")
	assert.Error(t, err)
}

func TestStoredFiles_Empty(t *testing.T) {
	c := fileclient.NewInMemoryFileClient()
	assert.Empty(t, c.StoredFiles())
}
