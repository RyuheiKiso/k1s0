// event_codegen_test.go: event_codegen.go のユニットテスト。
// 一時ディレクトリを使用してファイル生成の正確性・冪等性・エラー処理を検証する。
package codegen_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/k1s0-platform/system-library-go-codegen"
)

// newBasicConfig はテスト用のシンプルな EventCodegenConfig を生成する。
func newBasicConfig(t *testing.T) codegen.EventCodegenConfig {
	t.Helper()
	return codegen.EventCodegenConfig{
		OutputDir:   t.TempDir(),
		PackageName: "events",
		Events: []codegen.EventDef{
			{
				Name:  "UserCreated",
				Topic: "user.created",
				Fields: []codegen.EventField{
					{Name: "user_id", Type: "string"},
					{Name: "email", Type: "string", Optional: true},
				},
			},
		},
	}
}

// TestGenerateEventCode_Basic はシンプルなイベント定義で3ファイルが生成されることを確認する。
func TestGenerateEventCode_Basic(t *testing.T) {
	cfg := newBasicConfig(t)
	result, err := codegen.GenerateEventCode(cfg)
	require.NoError(t, err)
	require.NotNil(t, result)

	// events.go, producer.go, consumer.go の3ファイルが生成されること。
	assert.Len(t, result.CreatedFiles, 3)

	expectedFiles := []string{"events.go", "producer.go", "consumer.go"}
	for _, name := range expectedFiles {
		fullPath := filepath.Join(cfg.OutputDir, name)
		_, statErr := os.Stat(fullPath)
		assert.NoError(t, statErr, "ファイルが存在すること: %s", name)
	}
}

// TestGenerateEventCode_WithOutbox は EnableOutbox=true でSQLマイグレーションも生成されることを確認する。
func TestGenerateEventCode_WithOutbox(t *testing.T) {
	dir := t.TempDir()
	cfg := codegen.EventCodegenConfig{
		OutputDir:   dir,
		PackageName: "events",
		Events: []codegen.EventDef{
			{
				Name:         "OrderPlaced",
				Topic:        "order.placed",
				EnableOutbox: true,
				Fields: []codegen.EventField{
					{Name: "order_id", Type: "string"},
				},
			},
		},
	}
	result, err := codegen.GenerateEventCode(cfg)
	require.NoError(t, err)

	// 3ファイル + Outbox SQLマイグレーション = 4ファイルが生成されること。
	assert.Len(t, result.CreatedFiles, 4)

	sqlPath := filepath.Join(dir, "migrations", "add_outbox_orderplaced.sql")
	_, statErr := os.Stat(sqlPath)
	assert.NoError(t, statErr, "Outbox マイグレーションファイルが存在すること")
}

// TestGenerateEventCode_EmptyEvents はイベント定義なしでエラーが返ることを確認する。
func TestGenerateEventCode_EmptyEvents(t *testing.T) {
	cfg := codegen.EventCodegenConfig{
		OutputDir:   t.TempDir(),
		PackageName: "events",
		Events:      []codegen.EventDef{},
	}
	_, err := codegen.GenerateEventCode(cfg)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "at least one event is required")
}

// TestGenerateEventCode_Idempotent は2回呼んでも同じ結果になることを確認する（冪等性）。
func TestGenerateEventCode_Idempotent(t *testing.T) {
	cfg := newBasicConfig(t)

	// 1回目の生成。
	result1, err := codegen.GenerateEventCode(cfg)
	require.NoError(t, err)
	assert.Len(t, result1.CreatedFiles, 3)

	// 2回目は既存ファイルをスキップするため CreatedFiles が空になること。
	result2, err := codegen.GenerateEventCode(cfg)
	require.NoError(t, err)
	assert.Empty(t, result2.CreatedFiles, "既存ファイルはスキップされること")
}

// TestGenerateEventCode_OutputContent は生成されたevents.goが正しい構造体を含むことを確認する。
func TestGenerateEventCode_OutputContent(t *testing.T) {
	cfg := newBasicConfig(t)
	_, err := codegen.GenerateEventCode(cfg)
	require.NoError(t, err)

	// events.go の内容を読み込んで検証する。
	content, readErr := os.ReadFile(filepath.Join(cfg.OutputDir, "events.go"))
	require.NoError(t, readErr)

	body := string(content)
	// パッケージ宣言が正しいこと。
	assert.True(t, strings.Contains(body, "package events"), "パッケージ名が正しいこと")
	// UserCreated 構造体が定義されていること。
	assert.True(t, strings.Contains(body, "type UserCreated struct"), "イベント構造体が定義されていること")
	// フィールドが生成されていること。
	assert.True(t, strings.Contains(body, "UserId"), "UserId フィールドが存在すること")
	// OccurredAt フィールドが追加されていること。
	assert.True(t, strings.Contains(body, "OccurredAt"), "OccurredAt フィールドが存在すること")
	// Topic メソッドが生成されていること。
	assert.True(t, strings.Contains(body, `func (e *UserCreated) Topic() string`), "Topic メソッドが存在すること")
	// omitempty が Optional フィールドに付与されていること。
	assert.True(t, strings.Contains(body, "omitempty"), "Optional フィールドに omitempty が付与されていること")
}
