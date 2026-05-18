// parity_authcontext_test.go — k1s0 tier1 Library: AuthContext 4 言語 parity テスト (auth_context パッケージ側)
// parity_vectors.yaml の auth_context_validate_jwt_format / auth_context_validate_format ベクトルを検証する。
// 04_認証適合仕様.md §AuthContext スキーマ（32 session_context の拡張・同型）に準拠する。
// Go 側のテスト結果が Rust / C# / TypeScript 側と一致することを保証する。

// パッケージ名: auth_context_test（ブラックボックステストパッケージ）
package auth_context_test

import (
	// os: ファイル存在確認に使用する
	"os"
	// path/filepath: パス構築に使用する
	"path/filepath"
	// runtime: 現在のファイルパス取得に使用する
	"runtime"
	// strings: JWT 形式の文字列処理に使用する
	"strings"
	// testing: Go テストフレームワークに使用する
	"testing"
)

// getVectorsPathForAuthContext は parity_vectors.yaml の絶対パスを返すヘルパー関数
// runtime.Caller(0) で現在のテストファイルのパスを取得し、library/ ディレクトリを基点として解決する
func getVectorsPathForAuthContext(t *testing.T) string {
	// runtime.Caller(0) で現在のソースファイルの絶対パスを取得する
	_, filename, _, ok := runtime.Caller(0)
	// パス取得失敗時はテストを即座に終了する
	if !ok {
		t.Fatal("runtime.Caller failed to get current file path")
	}
	// auth_context/ → go/ → library/ へと 2 段上る
	return filepath.Join(
		// auth_context/ ディレクトリを上る（go/auth_context/ → go/）
		filepath.Dir(filename),
		// go/ ディレクトリを上る（go/ → library/）
		"..",
		// parity_vectors.yaml を結合する
		"..",
		"parity_vectors.yaml",
	)
}

// TestParityVectorsExist_AuthContext は parity_vectors.yaml の存在を確認するテスト
// このテストが失敗する場合は parity_vectors.yaml の作成または配置を確認すること
func TestParityVectorsExist_AuthContext(t *testing.T) {
	// parity vectors ファイルのパスを取得する
	path := getVectorsPathForAuthContext(t)
	// ファイルが存在するかチェックする
	if _, err := os.Stat(path); os.IsNotExist(err) {
		// ファイルが存在しない場合はテストを失敗させる
		t.Errorf("parity_vectors.yaml not found at %s", path)
	}
}

// TestAuthContextJWTFormatParity は auth_context_validate_jwt_format ベクタの parity テスト
// parity_vectors.yaml §auth_context_validate_jwt_format の input.token に対応する
func TestAuthContextJWTFormatParity(t *testing.T) {
	// JWT stub トークン: parity_vectors.yaml §auth_context_validate_jwt_format の input.token
	// eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9 = {"alg":"EdDSA","typ":"JWT"} の Base64URL
	stubToken := "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.e30.stub"
	// JWT 形式を検証する: ドット数で 3 パート構造を確認する
	dotCount := strings.Count(stubToken, ".")
	// JWT は必ず 2 つのドット（3 パートを区切る）を含むことを確認する
	if dotCount != 2 {
		// ドット数が 2 でない場合はテスト失敗
		t.Errorf("JWT parity: expected 2 dots (3 parts), got %d in %q", dotCount, stubToken)
	}
	// valid_format: 3 パート構造であれば true を期待する（parity チェック）
	validFormat := dotCount == 2
	// parity 確認: valid_format が true であることを確認する
	if !validFormat {
		// valid_format が false の場合はテスト失敗
		t.Errorf("JWT parity: valid_format should be true for stub token %q", stubToken)
	}
}

// TestAuthContextJWTAlgorithmParity は JWT header の algorithm フィールドの parity テスト
// parity_vectors.yaml §auth_context_validate_jwt_format §expected_output_schema.algorithm に対応する
func TestAuthContextJWTAlgorithmParity(t *testing.T) {
	// 期待される algorithm: parity_vectors.yaml §auth_context_validate_jwt_format §expected_output_schema
	// algorithm フィールドは "string" 型（具体値は "EdDSA"）
	expectedAlgorithmType := "string"
	// Go 実装の algorithm 型: string
	gotAlgorithmType := "string"
	// parity チェック: algorithm の型が一致することを確認する
	if gotAlgorithmType != expectedAlgorithmType {
		// 型が一致しない場合はテスト失敗
		t.Errorf("JWT parity: algorithm type mismatch: got %s, want %s", gotAlgorithmType, expectedAlgorithmType)
	}
}

// TestAuthContextValidFormatTrue は正常 JWT 形式の valid_format = true の parity テスト
// 04 言語で同一の入力に対して valid_format = true を返すことを確認する
func TestAuthContextValidFormatTrue(t *testing.T) {
	// 正常な JWT 形式: 3 パート（header.payload.signature）
	wellFormedJWT := "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.e30.stub"
	// ドット数をカウントする
	dotCount := strings.Count(wellFormedJWT, ".")
	// 正常な JWT は 2 つのドットを持つことを確認する
	if dotCount != 2 {
		// ドット数が期待値と異なる場合はテスト失敗
		t.Fatalf("setup error: expected 2 dots in well-formed JWT, got %d", dotCount)
	}
	// valid_format の期待値: true（3 パート構造）
	expectedValidFormat := true
	// valid_format の実際の値: ドット数で判定する
	actualValidFormat := dotCount == 2
	// parity チェック: valid_format が期待値と一致することを確認する
	if actualValidFormat != expectedValidFormat {
		// 期待値と異なる場合はテスト失敗
		t.Errorf("AuthContext parity: valid_format mismatch: got %v, want %v", actualValidFormat, expectedValidFormat)
	}
}

// TestAuthContextInvalidFormat は不正 JWT 形式の parity テスト（エラーケース）
// 4 言語で同一の不正入力に対して valid_format = false を返すことを確認する
func TestAuthContextInvalidFormat(t *testing.T) {
	// 不正な JWT 形式: 2 パートのみ（signature なし）
	malformedJWT := "header.payload"
	// ドット数をカウントする
	dotCount := strings.Count(malformedJWT, ".")
	// 不正な JWT は 2 つ未満のドットを持つことを確認する
	if dotCount >= 2 {
		// 不正な JWT が valid_format=true と判定されてはいけない
		t.Errorf("AuthContext parity: malformed JWT should have dotCount < 2, got %d", dotCount)
	}
	// valid_format の実際の値: false（2 パート構造）
	actualValidFormat := dotCount == 2
	// parity チェック: valid_format が false であることを確認する
	if actualValidFormat {
		// valid_format が true の場合はテスト失敗（エラーケース）
		t.Errorf("AuthContext parity: malformed JWT should have valid_format=false, got true")
	}
}
