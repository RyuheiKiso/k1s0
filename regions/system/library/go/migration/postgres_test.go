package migration

import (
	"context"
	"database/sql"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// newTestDB opens a test PostgreSQL database from the DATABASE_URL env var.
// Tests are skipped if DATABASE_URL is not set.
func newTestDB(t *testing.T) *sql.DB {
	t.Helper()
	dsn := os.Getenv("DATABASE_URL")
	if dsn == "" {
		t.Skip("DATABASE_URL not set, skipping PostgreSQL tests")
	}
	db, err := sql.Open("postgres", dsn)
	require.NoError(t, err)
	require.NoError(t, db.Ping())
	t.Cleanup(func() {
		db.Exec("DROP TABLE IF EXISTS _migrations, test_postgres_migration_table")
		db.Close()
	})
	return db
}

// writeMigrationFiles writes temporary migration files and returns the directory.
func writeMigrationFiles(t *testing.T, files map[string]string) string {
	t.Helper()
	dir := t.TempDir()
	for name, content := range files {
		require.NoError(t, os.WriteFile(filepath.Join(dir, name), []byte(content), 0644))
	}
	return dir
}

// PostgressMigrationRunnerの初期化時に_migrationsテーブルが作成されることを確認する。
func TestPostgresMigrationRunner_EnsureTableCreated(t *testing.T) {
	db := newTestDB(t)
	dir := writeMigrationFiles(t, map[string]string{
		"20240101000001_init.up.sql":   "SELECT 1;",
		"20240101000001_init.down.sql": "SELECT 1;",
	})

	cfg := NewMigrationConfig(dir, "")
	// ctx を渡して deadline のキャンセル伝播を有効にする
	runner, err := NewPostgresMigrationRunner(context.Background(), db, cfg)
	require.NoError(t, err)
	assert.NotNil(t, runner)

	// _migrations table should exist after construction
	var count int
	err = db.QueryRow("SELECT COUNT(*) FROM _migrations").Scan(&count)
	require.NoError(t, err)
	assert.Equal(t, 0, count)
}

// PostgresMigrationRunnerのRunUpが全マイグレーションを正しく適用することを確認する。
func TestPostgresMigrationRunner_RunUp(t *testing.T) {
	db := newTestDB(t)
	dir := writeMigrationFiles(t, map[string]string{
		"20240101000001_create_test.up.sql":   "CREATE TABLE IF NOT EXISTS test_postgres_migration_table (id SERIAL PRIMARY KEY);",
		"20240101000001_create_test.down.sql": "DROP TABLE IF EXISTS test_postgres_migration_table;",
		"20240101000002_add_col.up.sql":       "ALTER TABLE test_postgres_migration_table ADD COLUMN IF NOT EXISTS name TEXT;",
		"20240101000002_add_col.down.sql":     "ALTER TABLE test_postgres_migration_table DROP COLUMN IF EXISTS name;",
	})

	cfg := NewMigrationConfig(dir, "")
	// ctx を渡して deadline のキャンセル伝播を有効にする
	runner, err := NewPostgresMigrationRunner(context.Background(), db, cfg)
	require.NoError(t, err)

	report, err := runner.RunUp(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 2, report.AppliedCount)
	assert.Empty(t, report.Errors)
}

// PostgresMigrationRunnerのRunUpが冪等に動作し、2回目の実行では適用数が0になることを確認する。
func TestPostgresMigrationRunner_RunUpIdempotent(t *testing.T) {
	db := newTestDB(t)
	dir := writeMigrationFiles(t, map[string]string{
		"20240101000001_create_test.up.sql":   "CREATE TABLE IF NOT EXISTS test_postgres_migration_table (id SERIAL PRIMARY KEY);",
		"20240101000001_create_test.down.sql": "DROP TABLE IF EXISTS test_postgres_migration_table;",
	})

	cfg := NewMigrationConfig(dir, "")
	// ctx を渡して deadline のキャンセル伝播を有効にする
	runner, err := NewPostgresMigrationRunner(context.Background(), db, cfg)
	require.NoError(t, err)

	_, err = runner.RunUp(context.Background())
	require.NoError(t, err)

	report, err := runner.RunUp(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 0, report.AppliedCount)
}

// PostgresMigrationRunnerのStatusがRunUp前後で正しいAppliedAt状態を返すことを確認する。
func TestPostgresMigrationRunner_Status(t *testing.T) {
	db := newTestDB(t)
	dir := writeMigrationFiles(t, map[string]string{
		"20240101000001_create_test.up.sql":   "CREATE TABLE IF NOT EXISTS test_postgres_migration_table (id SERIAL PRIMARY KEY);",
		"20240101000001_create_test.down.sql": "DROP TABLE IF EXISTS test_postgres_migration_table;",
	})

	cfg := NewMigrationConfig(dir, "")
	// ctx を渡して deadline のキャンセル伝播を有効にする
	runner, err := NewPostgresMigrationRunner(context.Background(), db, cfg)
	require.NoError(t, err)

	// Before RunUp: AppliedAt is nil
	statuses, err := runner.Status(context.Background())
	require.NoError(t, err)
	require.Len(t, statuses, 1)
	assert.Nil(t, statuses[0].AppliedAt)
	assert.Equal(t, "20240101000001", statuses[0].Version)

	// After RunUp: AppliedAt is set
	_, err = runner.RunUp(context.Background())
	require.NoError(t, err)

	statuses, err = runner.Status(context.Background())
	require.NoError(t, err)
	require.Len(t, statuses, 1)
	assert.NotNil(t, statuses[0].AppliedAt)
}

// PostgresMigrationRunnerのPendingがRunUp前後で正しい未適用マイグレーション一覧を返すことを確認する。
func TestPostgresMigrationRunner_Pending(t *testing.T) {
	db := newTestDB(t)
	dir := writeMigrationFiles(t, map[string]string{
		"20240101000001_create_test.up.sql":   "CREATE TABLE IF NOT EXISTS test_postgres_migration_table (id SERIAL PRIMARY KEY);",
		"20240101000001_create_test.down.sql": "DROP TABLE IF EXISTS test_postgres_migration_table;",
		"20240101000002_add_col.up.sql":       "ALTER TABLE test_postgres_migration_table ADD COLUMN IF NOT EXISTS name TEXT;",
		"20240101000002_add_col.down.sql":     "ALTER TABLE test_postgres_migration_table DROP COLUMN IF EXISTS name;",
	})

	cfg := NewMigrationConfig(dir, "")
	// ctx を渡して deadline のキャンセル伝播を有効にする
	runner, err := NewPostgresMigrationRunner(context.Background(), db, cfg)
	require.NoError(t, err)

	pending, err := runner.Pending(context.Background())
	require.NoError(t, err)
	assert.Len(t, pending, 2)

	_, err = runner.RunUp(context.Background())
	require.NoError(t, err)

	pending, err = runner.Pending(context.Background())
	require.NoError(t, err)
	assert.Empty(t, pending)
}

// 存在しないマイグレーションディレクトリを指定した場合にエラーが返ることを確認する。
func TestPostgresMigrationRunner_InvalidDir(t *testing.T) {
	db := newTestDB(t)
	cfg := NewMigrationConfig("/nonexistent/path", "")
	// ctx を渡して deadline のキャンセル伝播を有効にする
	_, err := NewPostgresMigrationRunner(context.Background(), db, cfg)
	assert.Error(t, err)
}
