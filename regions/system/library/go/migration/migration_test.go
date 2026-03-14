package migration

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newTestRunner() *InMemoryMigrationRunner {
	ups := []struct{ Version, Name, Content string }{
		{"20240101000001", "create_users", "CREATE TABLE users (id INT);"},
		{"20240101000002", "add_email", "ALTER TABLE users ADD COLUMN email TEXT;"},
		{"20240201000001", "create_orders", "CREATE TABLE orders (id INT);"},
	}
	downs := []struct{ Version, Name, Content string }{
		{"20240101000001", "create_users", "DROP TABLE users;"},
		{"20240101000002", "add_email", "ALTER TABLE users DROP COLUMN email;"},
		{"20240201000001", "create_orders", "DROP TABLE orders;"},
	}
	cfg := NewMigrationConfig(".", "memory://")
	return NewInMemoryRunnerFromMigrations(cfg, ups, downs)
}

// RunUpが全マイグレーションを適用し、適用数が3になることを確認する。
func TestRunUpAppliesAll(t *testing.T) {
	runner := newTestRunner()
	report, err := runner.RunUp(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 3, report.AppliedCount)
	assert.Empty(t, report.Errors)
}

// RunUpが冪等に動作し、2回目の実行では適用数が0になることを確認する。
func TestRunUpIdempotent(t *testing.T) {
	runner := newTestRunner()
	_, err := runner.RunUp(context.Background())
	require.NoError(t, err)

	report, err := runner.RunUp(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 0, report.AppliedCount)
}

// RunDownが指定ステップ数だけマイグレーションを巻き戻すことを確認する。
func TestRunDown(t *testing.T) {
	runner := newTestRunner()
	_, err := runner.RunUp(context.Background())
	require.NoError(t, err)

	report, err := runner.RunDown(context.Background(), 1)
	require.NoError(t, err)
	assert.Equal(t, 1, report.AppliedCount)

	pending, err := runner.Pending(context.Background())
	require.NoError(t, err)
	assert.Len(t, pending, 1)
	assert.Equal(t, "20240201000001", pending[0].Version)
}

// RunDownが複数ステップの巻き戻しを正しく処理することを確認する。
func TestRunDownMultipleSteps(t *testing.T) {
	runner := newTestRunner()
	_, err := runner.RunUp(context.Background())
	require.NoError(t, err)

	report, err := runner.RunDown(context.Background(), 2)
	require.NoError(t, err)
	assert.Equal(t, 2, report.AppliedCount)

	pending, err := runner.Pending(context.Background())
	require.NoError(t, err)
	assert.Len(t, pending, 2)
}

// RunDownの指定ステップ数が適用済み数を超えても全件巻き戻せることを確認する。
func TestRunDownMoreThanApplied(t *testing.T) {
	runner := newTestRunner()
	_, err := runner.RunUp(context.Background())
	require.NoError(t, err)

	report, err := runner.RunDown(context.Background(), 10)
	require.NoError(t, err)
	assert.Equal(t, 3, report.AppliedCount)
}

// RunUp前のStatusが全マイグレーションをAppliedAt=nilの未適用として返すことを確認する。
func TestStatusAllPending(t *testing.T) {
	runner := newTestRunner()
	statuses, err := runner.Status(context.Background())
	require.NoError(t, err)
	assert.Len(t, statuses, 3)
	for _, s := range statuses {
		assert.Nil(t, s.AppliedAt)
	}
}

// RunUp後のStatusが全マイグレーションをAppliedAtが設定された適用済みとして返すことを確認する。
func TestStatusAfterApply(t *testing.T) {
	runner := newTestRunner()
	_, err := runner.RunUp(context.Background())
	require.NoError(t, err)

	statuses, err := runner.Status(context.Background())
	require.NoError(t, err)
	assert.Len(t, statuses, 3)
	for _, s := range statuses {
		assert.NotNil(t, s.AppliedAt)
	}
}

// RunUp前のPendingが全未適用マイグレーションをバージョン順に返すことを確認する。
func TestPendingReturnsUnapplied(t *testing.T) {
	runner := newTestRunner()
	pending, err := runner.Pending(context.Background())
	require.NoError(t, err)
	assert.Len(t, pending, 3)
	assert.Equal(t, "20240101000001", pending[0].Version)
	assert.Equal(t, "20240101000002", pending[1].Version)
	assert.Equal(t, "20240201000001", pending[2].Version)
}

// RunUp後のPendingが空スライスを返すことを確認する。
func TestPendingEmptyAfterApply(t *testing.T) {
	runner := newTestRunner()
	_, err := runner.RunUp(context.Background())
	require.NoError(t, err)

	pending, err := runner.Pending(context.Background())
	require.NoError(t, err)
	assert.Empty(t, pending)
}

// ParseFilenameがupマイグレーションファイル名からバージョン・名前・方向を正しく解析することを確認する。
func TestParseFilenameUp(t *testing.T) {
	version, name, dir, ok := ParseFilename("20240101000001_create_users.up.sql")
	assert.True(t, ok)
	assert.Equal(t, "20240101000001", version)
	assert.Equal(t, "create_users", name)
	assert.Equal(t, DirectionUp, dir)
}

// ParseFilenameがdownマイグレーションファイル名からバージョン・名前・方向を正しく解析することを確認する。
func TestParseFilenameDown(t *testing.T) {
	version, name, dir, ok := ParseFilename("20240101000001_create_users.down.sql")
	assert.True(t, ok)
	assert.Equal(t, "20240101000001", version)
	assert.Equal(t, "create_users", name)
	assert.Equal(t, DirectionDown, dir)
}

// 不正なファイル名に対してParseFilenameがokをfalseで返すことを確認する。
func TestParseFilenameInvalid(t *testing.T) {
	_, _, _, ok := ParseFilename("invalid.sql")
	assert.False(t, ok)

	_, _, _, ok = ParseFilename("no_direction.sql")
	assert.False(t, ok)

	_, _, _, ok = ParseFilename("_.up.sql")
	assert.False(t, ok)
}

// 同一内容のSQLに対してChecksumが常に同じ値を返すことを確認する。
func TestChecksumDeterministic(t *testing.T) {
	content := "CREATE TABLE users (id SERIAL PRIMARY KEY);"
	c1 := Checksum(content)
	c2 := Checksum(content)
	assert.Equal(t, c1, c2)
}

// 異なる内容のSQLに対してChecksumが異なる値を返すことを確認する。
func TestChecksumDiffersForDifferentContent(t *testing.T) {
	c1 := Checksum("CREATE TABLE users;")
	c2 := Checksum("CREATE TABLE orders;")
	assert.NotEqual(t, c1, c2)
}

// NewMigrationConfigがデフォルトのテーブル名とドライバーを正しく設定することを確認する。
func TestDefaultConfig(t *testing.T) {
	cfg := NewMigrationConfig("./migrations", "postgres://localhost/test")
	assert.Equal(t, "_migrations", cfg.TableName)
	assert.Equal(t, "postgres", cfg.Driver)
}
