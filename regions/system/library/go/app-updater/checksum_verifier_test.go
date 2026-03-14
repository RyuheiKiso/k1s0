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

// CalculateChecksum がファイルの SHA-256 チェックサムを正しく計算することを確認する。
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

// CalculateChecksum が存在しないファイルを指定するとエラーを返すことを確認する。
func TestCalculateChecksum_FileNotFound(t *testing.T) {
	_, err := appupdater.CalculateChecksum("/nonexistent/file.txt")
	require.Error(t, err)
}

// VerifyChecksum が正しいチェックサムに対して true を返すことを確認する。
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

// VerifyChecksum がチェックサムの大文字・小文字を区別せず比較することを確認する。
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

// VerifyChecksum が一致しないチェックサムに対して false を返すことを確認する。
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

// VerifyChecksumOrError がチェックサムが一致する場合にエラーなしで完了することを確認する。
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

// VerifyChecksumOrError がチェックサムが不一致の場合に ChecksumError を返すことを確認する。
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
