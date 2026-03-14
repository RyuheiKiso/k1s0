// secretstore_test パッケージは secretstore パッケージの各実装に対するユニットテストを提供する。
// InMemorySecretStore、EnvSecretStore、FileSecretStore、VaultSecretStore の動作を検証する。
package secretstore_test

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	secretstore "github.com/k1s0-platform/system-library-go-secret-store"
)

// ─────────────────────────────────────────
// InMemorySecretStore テスト
// ─────────────────────────────────────────

// TestInMemorySecretStore_GetPut は Put 後に Get で値を取得できることを確認する。
func TestInMemorySecretStore_GetPut(t *testing.T) {
	ctx := context.Background()
	store := secretstore.NewInMemorySecretStore()

	// シークレットを格納してから取得できることを検証する。
	store.Put("db-password", "s3cret")
	secret, err := store.Get(ctx, "db-password")
	require.NoError(t, err)
	assert.Equal(t, "db-password", secret.Key)
	assert.Equal(t, "s3cret", secret.Value)
}

// TestInMemorySecretStore_NotFound は未登録キーでエラーが返ることを確認する。
func TestInMemorySecretStore_NotFound(t *testing.T) {
	ctx := context.Background()
	store := secretstore.NewInMemorySecretStore()

	// 存在しないキーを取得した場合はエラーが返ることを検証する。
	_, err := store.Get(ctx, "nonexistent")
	require.Error(t, err)
	assert.Contains(t, err.Error(), "not found")
}

// TestInMemorySecretStore_BulkGet は BulkGet で複数のシークレットをまとめて取得できることを確認する。
func TestInMemorySecretStore_BulkGet(t *testing.T) {
	ctx := context.Background()
	store := secretstore.NewInMemorySecretStore()

	// 複数のシークレットを登録して BulkGet で取得できることを検証する。
	store.Put("key1", "val1")
	store.Put("key2", "val2")

	secrets, err := store.BulkGet(ctx, []string{"key1", "key2"})
	require.NoError(t, err)
	require.Len(t, secrets, 2)

	// 各シークレットの値が正しいことを確認する。
	values := map[string]string{}
	for _, s := range secrets {
		values[s.Key] = s.Value
	}
	assert.Equal(t, "val1", values["key1"])
	assert.Equal(t, "val2", values["key2"])
}

// ─────────────────────────────────────────
// EnvSecretStore テスト
// ─────────────────────────────────────────

// TestEnvSecretStore_Get は環境変数を設定して Get で取得できることを確認する。
func TestEnvSecretStore_Get(t *testing.T) {
	ctx := context.Background()
	// テスト用環境変数を設定してテスト終了後に元に戻す。
	t.Setenv("TEST_SECRET_DB_PASS", "mypassword")

	store := secretstore.NewEnvSecretStore("TEST_SECRET_")
	secret, err := store.Get(ctx, "DB_PASS")
	require.NoError(t, err)
	assert.Equal(t, "DB_PASS", secret.Key)
	assert.Equal(t, "mypassword", secret.Value)
}

// TestEnvSecretStore_NotFound は未設定の環境変数でエラーが返ることを確認する。
func TestEnvSecretStore_NotFound(t *testing.T) {
	ctx := context.Background()
	// 存在しないプレフィックスを使用して未設定環境変数のエラーを検証する。
	store := secretstore.NewEnvSecretStore("DEFINITELY_NOT_SET_PREFIX_XYZ_")
	_, err := store.Get(ctx, "MISSING_KEY")
	require.Error(t, err)
	assert.Contains(t, err.Error(), "not found")
}

// ─────────────────────────────────────────
// FileSecretStore テスト
// ─────────────────────────────────────────

// TestFileSecretStore_Get は一時ディレクトリにファイルを作成して Get で取得できることを確認する。
func TestFileSecretStore_Get(t *testing.T) {
	ctx := context.Background()
	// 一時ディレクトリにテスト用シークレットファイルを作成する。
	dir := t.TempDir()
	err := os.WriteFile(filepath.Join(dir, "api-key"), []byte("abc123\n"), 0600)
	require.NoError(t, err)

	store := secretstore.NewFileSecretStore(dir)
	secret, err := store.Get(ctx, "api-key")
	require.NoError(t, err)
	assert.Equal(t, "api-key", secret.Key)
	// 末尾の改行が除去されていることを確認する。
	assert.Equal(t, "abc123", secret.Value)
}

// TestFileSecretStore_NotFound は存在しないファイルでエラーが返ることを確認する。
func TestFileSecretStore_NotFound(t *testing.T) {
	ctx := context.Background()
	// 空の一時ディレクトリを使用して存在しないファイルのエラーを検証する。
	dir := t.TempDir()
	store := secretstore.NewFileSecretStore(dir)
	_, err := store.Get(ctx, "nonexistent-file")
	require.Error(t, err)
	assert.Contains(t, err.Error(), "not found")
}

// ─────────────────────────────────────────
// VaultSecretStore テスト
// ─────────────────────────────────────────

// mockVaultClient はテスト用の VaultClientIface モック実装。
// 事前に設定したシークレットを返す。
type mockVaultClient struct {
	// secrets はパスからシークレットへのマップ。
	secrets map[string]secretstore.VaultSecret
	// err は GetSecret が返すエラー（nilの場合はエラーなし）。
	err error
}

// GetSecret は事前設定されたシークレットを返すモック実装。
func (m *mockVaultClient) GetSecret(_ context.Context, path string) (secretstore.VaultSecret, error) {
	if m.err != nil {
		return secretstore.VaultSecret{}, m.err
	}
	vs, ok := m.secrets[path]
	if !ok {
		return secretstore.VaultSecret{}, fmt.Errorf("vault: secret %q not found", path)
	}
	return vs, nil
}

// TestVaultSecretStore_Get はモッククライアントで GetSecret が正しく呼ばれることを確認する。
func TestVaultSecretStore_Get(t *testing.T) {
	ctx := context.Background()
	// モッククライアントにシークレットを設定する。
	client := &mockVaultClient{
		secrets: map[string]secretstore.VaultSecret{
			"secret/db": {
				Path:    "secret/db",
				Data:    map[string]string{"password": "vault-s3cret"},
				Version: 3,
			},
		},
	}

	store := secretstore.NewVaultSecretStore("vault-store", client)
	secret, err := store.Get(ctx, "secret/db")
	require.NoError(t, err)
	assert.Equal(t, "secret/db", secret.Key)
	// 単一キーの場合は値がそのまま返されることを確認する。
	assert.Equal(t, "vault-s3cret", secret.Value)
	// バージョン情報がメタデータに含まれることを確認する。
	assert.Equal(t, "3", secret.Metadata["version"])
}
