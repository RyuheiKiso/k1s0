// client_sdk_test.go: GenerateClientSDK 関数のユニットテスト。
// proto ファイルから各言語のクライアント SDK を生成する正常系・異常系を検証する。
package codegen_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	codegen "github.com/k1s0-platform/system-library-go-codegen"
)

// sampleAuthProto は client SDK テスト用のシンプルな proto 文字列。
const sampleAuthProto = `
syntax = "proto3";

package k1s0.system.auth.v1;

service Auth {
  rpc Login(LoginRequest) returns (LoginResponse);
}

message LoginRequest {
  string email = 1;
}

message LoginResponse {
  string token = 1;
}
`

// writeProtoFile は一時ディレクトリに proto ファイルを書き込み、そのパスを返す。
func writeProtoFile(t *testing.T, content string) string {
	t.Helper()
	dir := t.TempDir()
	path := filepath.Join(dir, "service.proto")
	err := os.WriteFile(path, []byte(content), 0644)
	require.NoError(t, err)
	return path
}

// TestGenerateClientSDK_Go は Go 向けに client.go / grpc.go / mock.go が生成されることを確認する。
func TestGenerateClientSDK_Go(t *testing.T) {
	protoPath := writeProtoFile(t, sampleAuthProto)
	outDir := t.TempDir()

	result, err := codegen.GenerateClientSDK(codegen.ClientSDKConfig{
		ProtoPath:  protoPath,
		OutputDir:  outDir,
		Language:   "go",
		PackageURL: "github.com/k1s0-platform/auth-client",
	})
	require.NoError(t, err)
	require.NotNil(t, result)

	// client.go, grpc.go, mock.go の3ファイルが生成されること。
	assert.Len(t, result.CreatedFiles, 3)

	for _, name := range []string{"client.go", "grpc.go", "mock.go"} {
		fullPath := filepath.Join(outDir, name)
		_, statErr := os.Stat(fullPath)
		assert.NoError(t, statErr, "ファイルが存在すること: %s", name)
	}
}

// TestGenerateClientSDK_UnsupportedLanguage は未対応言語でエラーが返ることを確認する。
func TestGenerateClientSDK_UnsupportedLanguage(t *testing.T) {
	protoPath := writeProtoFile(t, sampleAuthProto)

	_, err := codegen.GenerateClientSDK(codegen.ClientSDKConfig{
		ProtoPath: protoPath,
		OutputDir: t.TempDir(),
		Language:  "typescript",
	})
	require.Error(t, err)
	assert.Contains(t, err.Error(), "unsupported")
}

// TestGenerateClientSDK_MissingProtoPath は proto_path 未指定でエラーが返ることを確認する。
func TestGenerateClientSDK_MissingProtoPath(t *testing.T) {
	_, err := codegen.GenerateClientSDK(codegen.ClientSDKConfig{
		OutputDir: t.TempDir(),
		Language:  "go",
	})
	require.Error(t, err)
	assert.Contains(t, err.Error(), "proto_path")
}

// TestGenerateClientSDK_MissingOutputDir は output_dir 未指定でエラーが返ることを確認する。
func TestGenerateClientSDK_MissingOutputDir(t *testing.T) {
	protoPath := writeProtoFile(t, sampleAuthProto)

	_, err := codegen.GenerateClientSDK(codegen.ClientSDKConfig{
		ProtoPath: protoPath,
		Language:  "go",
	})
	require.Error(t, err)
	assert.Contains(t, err.Error(), "output_dir")
}

// TestGenerateClientSDK_Idempotent は2回実行しても既存ファイルがスキップされることを確認する。
func TestGenerateClientSDK_Idempotent(t *testing.T) {
	protoPath := writeProtoFile(t, sampleAuthProto)
	outDir := t.TempDir()
	cfg := codegen.ClientSDKConfig{
		ProtoPath: protoPath,
		OutputDir: outDir,
		Language:  "go",
	}

	// 1回目は3ファイル生成。
	result1, err := codegen.GenerateClientSDK(cfg)
	require.NoError(t, err)
	assert.Len(t, result1.CreatedFiles, 3)

	// 2回目は既存ファイルをスキップして CreatedFiles が空になること。
	result2, err := codegen.GenerateClientSDK(cfg)
	require.NoError(t, err)
	assert.Empty(t, result2.CreatedFiles, "既存ファイルはスキップされること")
}
