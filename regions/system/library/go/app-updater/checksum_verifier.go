package appupdater

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	"strings"
)

// CalculateChecksum はファイルの SHA-256 チェックサムを計算する。
func CalculateChecksum(filePath string) (string, error) {
	f, err := os.Open(filePath)
	if err != nil {
		return "", fmt.Errorf("failed to open file: %w", err)
	}
	defer f.Close()

	h := sha256.New()
	if _, err := io.Copy(h, f); err != nil {
		return "", fmt.Errorf("failed to read file: %w", err)
	}

	return hex.EncodeToString(h.Sum(nil)), nil
}

// VerifyChecksum はファイルのチェックサムを検証する。
func VerifyChecksum(filePath, expectedChecksum string) (bool, error) {
	actual, err := CalculateChecksum(filePath)
	if err != nil {
		return false, err
	}
	return strings.EqualFold(actual, expectedChecksum), nil
}

// VerifyChecksumOrError はファイルのチェックサムを検証し、不一致の場合エラーを返す。
func VerifyChecksumOrError(filePath, expectedChecksum string) error {
	ok, err := VerifyChecksum(filePath, expectedChecksum)
	if err != nil {
		return err
	}
	if !ok {
		return NewChecksumError("file checksum did not match")
	}
	return nil
}
