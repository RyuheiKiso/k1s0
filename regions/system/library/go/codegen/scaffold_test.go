// scaffold_test.go: Generate 関数のユニットテスト。
// Go/Rust スキャフォールド生成の正常系・異常系・バリデーションを検証する。
package codegen_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	codegen "github.com/k1s0-platform/system-library-go-codegen"
)

// newScaffoldConfig はテスト用のデフォルト ScaffoldConfig を生成する。
func newScaffoldConfig(t *testing.T, lang string) codegen.ScaffoldConfig {
	t.Helper()
	return codegen.ScaffoldConfig{
		Domain:      "k1s0",
		Tier:        "system",
		ServiceName: "myservice",
		Language:    lang,
		OutputDir:   t.TempDir(),
	}
}

// TestGenerate_Go は Go スキャフォールドが3ファイル生成されることを確認する。
func TestGenerate_Go(t *testing.T) {
	cfg := newScaffoldConfig(t, "go")
	result, err := codegen.Generate(cfg)
	require.NoError(t, err)
	require.NotNil(t, result)

	// main.go, go.mod, internal/app/app.go の3ファイルが生成されること。
	assert.Len(t, result.CreatedFiles, 3)

	expectedFiles := []string{"main.go", "go.mod", filepath.Join("internal", "app", "app.go")}
	for _, rel := range expectedFiles {
		fullPath := filepath.Join(cfg.OutputDir, rel)
		_, statErr := os.Stat(fullPath)
		assert.NoError(t, statErr, "ファイルが存在すること: %s", rel)
	}
}

// TestGenerate_Go_Content は生成された go.mod がサービス名を含むことを確認する。
func TestGenerate_Go_Content(t *testing.T) {
	cfg := newScaffoldConfig(t, "go")
	_, err := codegen.Generate(cfg)
	require.NoError(t, err)

	// go.mod にサービス名が埋め込まれていること。
	content, readErr := os.ReadFile(filepath.Join(cfg.OutputDir, "go.mod"))
	require.NoError(t, readErr)
	assert.True(t, strings.Contains(string(content), cfg.ServiceName), "go.mod にサービス名が含まれること")
	assert.True(t, strings.Contains(string(content), cfg.Tier), "go.mod にティア名が含まれること")
}

// TestGenerate_Rust は Rust スキャフォールドが2ファイル生成されることを確認する。
func TestGenerate_Rust(t *testing.T) {
	cfg := newScaffoldConfig(t, "rust")
	result, err := codegen.Generate(cfg)
	require.NoError(t, err)
	require.NotNil(t, result)

	// src/main.rs, Cargo.toml の2ファイルが生成されること。
	assert.Len(t, result.CreatedFiles, 2)

	expectedFiles := []string{filepath.Join("src", "main.rs"), "Cargo.toml"}
	for _, rel := range expectedFiles {
		fullPath := filepath.Join(cfg.OutputDir, rel)
		_, statErr := os.Stat(fullPath)
		assert.NoError(t, statErr, "ファイルが存在すること: %s", rel)
	}
}

// TestGenerate_Rust_Content は生成された Cargo.toml がサービス名を含むことを確認する。
func TestGenerate_Rust_Content(t *testing.T) {
	cfg := newScaffoldConfig(t, "rust")
	_, err := codegen.Generate(cfg)
	require.NoError(t, err)

	// Cargo.toml にサービス名が埋め込まれていること。
	content, readErr := os.ReadFile(filepath.Join(cfg.OutputDir, "Cargo.toml"))
	require.NoError(t, readErr)
	assert.True(t, strings.Contains(string(content), cfg.ServiceName), "Cargo.toml にサービス名が含まれること")
}

// TestGenerate_UnsupportedLanguage は未対応の言語が指定された場合にエラーが返ることを確認する。
func TestGenerate_UnsupportedLanguage(t *testing.T) {
	cfg := newScaffoldConfig(t, "python")
	_, err := codegen.Generate(cfg)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "unsupported language")
}

// TestGenerate_Idempotent は2回実行しても既存ファイルがスキップされることを確認する（冪等性）。
func TestGenerate_Idempotent(t *testing.T) {
	cfg := newScaffoldConfig(t, "go")

	// 1回目の生成。
	result1, err := codegen.Generate(cfg)
	require.NoError(t, err)
	assert.Len(t, result1.CreatedFiles, 3)

	// 2回目は既存ファイルをスキップするため CreatedFiles が空になること。
	result2, err := codegen.Generate(cfg)
	require.NoError(t, err)
	assert.Empty(t, result2.CreatedFiles, "既存ファイルはスキップされること")
}

// TestGenerate_ValidationErrors は必須フィールドが欠落している場合にバリデーションエラーが返ることを確認する。
func TestGenerate_ValidationErrors(t *testing.T) {
	// テーブルドリブンで各必須フィールド欠落を検証する。
	tests := []struct {
		name   string
		cfg    codegen.ScaffoldConfig
		errMsg string
	}{
		{
			name: "domain が未設定",
			cfg: codegen.ScaffoldConfig{
				Tier: "system", ServiceName: "svc", Language: "go", OutputDir: t.TempDir(),
			},
			errMsg: "domain",
		},
		{
			name: "tier が未設定",
			cfg: codegen.ScaffoldConfig{
				Domain: "k1s0", ServiceName: "svc", Language: "go", OutputDir: t.TempDir(),
			},
			errMsg: "tier",
		},
		{
			name: "service_name が未設定",
			cfg: codegen.ScaffoldConfig{
				Domain: "k1s0", Tier: "system", Language: "go", OutputDir: t.TempDir(),
			},
			errMsg: "service_name",
		},
		{
			name: "language が未設定",
			cfg: codegen.ScaffoldConfig{
				Domain: "k1s0", Tier: "system", ServiceName: "svc", OutputDir: t.TempDir(),
			},
			errMsg: "language",
		},
		{
			name: "output_dir が未設定",
			cfg: codegen.ScaffoldConfig{
				Domain: "k1s0", Tier: "system", ServiceName: "svc", Language: "go",
			},
			errMsg: "output_dir",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			_, err := codegen.Generate(tc.cfg)
			require.Error(t, err)
			assert.Contains(t, err.Error(), tc.errMsg, "エラーメッセージにフィールド名が含まれること")
		})
	}
}
