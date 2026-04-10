// util パッケージはアプリケーション全体で再利用される汎用ユーティリティを提供する。
// このファイルはセキュアなランダム値生成機能を提供する。
package util

import (
	"crypto/rand"
	"encoding/hex"
)

// GenerateRandomHex はバイト長 n のランダムな hex エンコード文字列を生成する。
// MED-013 監査対応: auth_usecase.go と proxy_usecase.go の重複実装を統一する。
// crypto/rand を使用することで CSPRNG（暗号論的に安全な擬似乱数生成器）から値を取得する。
// CSRF トークン・state パラメータ等のセキュリティ用途で使用すること。
func GenerateRandomHex(n int) (string, error) {
	// n バイトの乱数を生成してから hex エンコードする（結果は 2n 文字の hex 文字列になる）
	b := make([]byte, n)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return hex.EncodeToString(b), nil
}
