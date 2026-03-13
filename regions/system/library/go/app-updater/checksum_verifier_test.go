package appupdater_test

import (
	"crypto/sha256"
	"encoding/hex"
	"os"
	"path/filepath"
	"testing"

	appupdater "github.com/k1s0-platform/system-library-go-app-updater"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestCalculateChecksum(t *testing.T) {
	dir := t.TempDir()
	filePath := filepath.Join(dir, "test.txt")
	content := []byte("hello world")
	err := os.WriteFile(filePath, content, 0644)
	require.NoError(t, err)

	// Expected SHA-256 of "hello world".
	h := sha256.Sum256(content)
	expected := hex.EncodeToString(h[:])

	actual, err := appupdater.CalculateChecksum(filePath)
	require.NoError(t, err)
	assert.Equal(t, expected, actual)
}

func TestCalculateChecksum_FileNotFound(t *testing.T) {
	_, err := appupdater.CalculateChecksum("/nonexistent/file.txt")
	require.Error(t, err)
}

func TestVerifyChecksum_Valid(t *testing.T) {
	dir := t.TempDir()
	filePath := filepath.Join(dir, "test.txt")
	content := []byte("hello world")
	err := os.WriteFile(filePath, content, 0644)
	require.NoError(t, err)

	h := sha256.Sum256(content)
	checksum := hex.EncodeToString(h[:])

	ok, err := appupdater.VerifyChecksum(filePath, checksum)
	require.NoError(t, err)
	assert.True(t, ok)
}

func TestVerifyChecksum_CaseInsensitive(t *testing.T) {
	dir := t.TempDir()
	filePath := filepath.Join(dir, "test.txt")
	content := []byte("hello world")
	err := os.WriteFile(filePath, content, 0644)
	require.NoError(t, err)

	h := sha256.Sum256(content)
	upper := hex.EncodeToString(h[:])
	// Use uppercase to verify case-insensitive comparison.
	ok, err := appupdater.VerifyChecksum(filePath, strings_ToUpper(upper))
	require.NoError(t, err)
	assert.True(t, ok)
}

func TestVerifyChecksum_Invalid(t *testing.T) {
	dir := t.TempDir()
	filePath := filepath.Join(dir, "test.txt")
	content := []byte("hello world")
	err := os.WriteFile(filePath, content, 0644)
	require.NoError(t, err)

	ok, err := appupdater.VerifyChecksum(filePath, "0000000000000000000000000000000000000000000000000000000000000000")
	require.NoError(t, err)
	assert.False(t, ok)
}

func TestVerifyChecksumOrError_Match(t *testing.T) {
	dir := t.TempDir()
	filePath := filepath.Join(dir, "test.txt")
	content := []byte("test content")
	err := os.WriteFile(filePath, content, 0644)
	require.NoError(t, err)

	h := sha256.Sum256(content)
	checksum := hex.EncodeToString(h[:])

	err = appupdater.VerifyChecksumOrError(filePath, checksum)
	assert.NoError(t, err)
}

func TestVerifyChecksumOrError_Mismatch(t *testing.T) {
	dir := t.TempDir()
	filePath := filepath.Join(dir, "test.txt")
	content := []byte("test content")
	err := os.WriteFile(filePath, content, 0644)
	require.NoError(t, err)

	err = appupdater.VerifyChecksumOrError(filePath, "invalid_checksum")
	require.Error(t, err)

	var checksumErr *appupdater.ChecksumError
	assert.ErrorAs(t, err, &checksumErr)
}

// strings_ToUpper is a helper to uppercase a string without importing "strings" in the test.
func strings_ToUpper(s string) string {
	result := make([]byte, len(s))
	for i := 0; i < len(s); i++ {
		c := s[i]
		if c >= 'a' && c <= 'z' {
			c -= 32
		}
		result[i] = c
	}
	return string(result)
}
