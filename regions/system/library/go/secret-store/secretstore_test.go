// secretstore_test パッケージは secretstore パッケージの各実装に対するユニットテストを提供する。
// InMemorySecretStore、EnvSecretStore、FileSecretStore、VaultSecretStore の動作を検証する。
package secretstore_test

import (
	"context"
	"errors"
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

// TestFileSecretStore_PathTraversal_EtcPasswd は "../../../etc/passwd" 形式のキーでエラーが返ることを確認する。
// パストラバーサル攻撃による任意ファイル読み取りを防止するための検証。
func TestFileSecretStore_PathTraversal_EtcPasswd(t *testing.T) {
	ctx := context.Background()
	// 一時ディレクトリを基底ディレクトリとして使用する。
	dir := t.TempDir()
	store := secretstore.NewFileSecretStore(dir)

	// "../../etc/passwd" 形式のパストラバーサル試行はエラーを返す必要がある。
	_, err := store.Get(ctx, "../../etc/passwd")
	require.Error(t, err)
	assert.Contains(t, err.Error(), "path traversal detected")
}

// TestFileSecretStore_PathTraversal_SiblingDir は "../sibling_dir/secret" 形式のキーでエラーが返ることを確認する。
// 隣接ディレクトリへのパストラバーサル攻撃を防止するための検証。
func TestFileSecretStore_PathTraversal_SiblingDir(t *testing.T) {
	ctx := context.Background()
	// 一時ディレクトリを基底ディレクトリとして使用する。
	dir := t.TempDir()
	store := secretstore.NewFileSecretStore(dir)

	// "../sibling/secret" 形式のパストラバーサル試行はエラーを返す必要がある。
	_, err := store.Get(ctx, "../sibling/secret")
	require.Error(t, err)
	assert.Contains(t, err.Error(), "path traversal detected")
}

// TestFileSecretStore_ValidKey は正常なキー名でファイルが読み取れることを確認する。
// パストラバーサル防止追加後も正常なキーが動作することを検証する。
func TestFileSecretStore_ValidKey(t *testing.T) {
	ctx := context.Background()
	// 一時ディレクトリにテスト用シークレットファイルを作成する。
	dir := t.TempDir()
	err := os.WriteFile(filepath.Join(dir, "valid-secret"), []byte("secret-value\n"), 0600)
	require.NoError(t, err)

	store := secretstore.NewFileSecretStore(dir)

	// 正常なキー名でシークレットが取得できることを確認する。
	secret, err := store.Get(ctx, "valid-secret")
	require.NoError(t, err)
	assert.Equal(t, "valid-secret", secret.Key)
	assert.Equal(t, "secret-value", secret.Value)
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

// ─────────────────────────────────────────
// M-15 監査対応: カバレッジ向上のための追加テスト
// ─────────────────────────────────────────

// TestComponentError_Unwrap は Unwrap() が内包エラーを正しく返すことを確認する。
// errors.Is/As によるエラーチェーン探索が機能することを検証する。
func TestComponentError_Unwrap(t *testing.T) {
	inner := fmt.Errorf("original error")
	cerr := secretstore.NewComponentError("comp", "op", "message", inner)

	// errors.Is でラップされたエラーを検出できることを確認する。
	assert.ErrorIs(t, cerr, inner)
	// errors.Unwrap で内包エラーを直接取得できることを確認する（Unwrap メソッドのカバレッジ）。
	unwrapped := errors.Unwrap(cerr)
	assert.Equal(t, inner, unwrapped)

	// Err が nil の場合は Unwrap が nil を返すことを確認する。
	cerrNoWrap := secretstore.NewComponentError("comp", "op", "message", nil)
	assert.Nil(t, errors.Unwrap(cerrNoWrap))
}

// TestInMemorySecretStore_Lifecycle は Init/Status/Close のライフサイクルを検証する。
func TestInMemorySecretStore_Lifecycle(t *testing.T) {
	ctx := context.Background()
	store := secretstore.NewInMemorySecretStore()

	// Name/Version が期待通りの文字列を返すことを確認する。
	assert.Equal(t, "inmemory-secretstore", store.Name())
	assert.Equal(t, "1.0.0", store.Version())

	// 初期化前は Uninitialized 状態であることを確認する。
	assert.Equal(t, secretstore.StatusUninitialized, store.Status(ctx))

	// Init 後は Ready 状態になることを確認する。
	require.NoError(t, store.Init(ctx, secretstore.Metadata{Name: "test"}))
	assert.Equal(t, secretstore.StatusReady, store.Status(ctx))

	// Close 後は Closed 状態になることを確認する。
	require.NoError(t, store.Close(ctx))
	assert.Equal(t, secretstore.StatusClosed, store.Status(ctx))
}

// TestInMemorySecretStore_BulkGet_Error は BulkGet でエラーが発生した場合に即座に返ることを確認する。
func TestInMemorySecretStore_BulkGet_Error(t *testing.T) {
	ctx := context.Background()
	store := secretstore.NewInMemorySecretStore()
	store.Put("key1", "val1")
	// key2 は存在しないため BulkGet はエラーを返す。
	_, err := store.BulkGet(ctx, []string{"key1", "key2_missing"})
	require.Error(t, err)
	assert.Contains(t, err.Error(), "not found")
}

// TestEnvSecretStore_Lifecycle は EnvSecretStore の Init/Status/Close と BulkGet を検証する。
func TestEnvSecretStore_Lifecycle(t *testing.T) {
	ctx := context.Background()
	store := secretstore.NewEnvSecretStore("TEST_ENV_STORE_")

	// Name/Version を確認する。
	assert.Equal(t, "env-secretstore", store.Name())
	assert.Equal(t, "1.0.0", store.Version())

	// 初期化前は Uninitialized 状態であることを確認する。
	assert.Equal(t, secretstore.StatusUninitialized, store.Status(ctx))

	// Init 後は Ready 状態になることを確認する。
	require.NoError(t, store.Init(ctx, secretstore.Metadata{Name: "test"}))
	assert.Equal(t, secretstore.StatusReady, store.Status(ctx))

	// Close 後は Closed 状態になることを確認する。
	require.NoError(t, store.Close(ctx))
	assert.Equal(t, secretstore.StatusClosed, store.Status(ctx))
}

// TestEnvSecretStore_BulkGet は BulkGet が複数の環境変数をまとめて取得できることを確認する。
func TestEnvSecretStore_BulkGet(t *testing.T) {
	ctx := context.Background()
	t.Setenv("BULK_TEST_KEY1", "val1")
	t.Setenv("BULK_TEST_KEY2", "val2")
	store := secretstore.NewEnvSecretStore("BULK_TEST_")

	// 複数の環境変数を一括取得できることを検証する。
	secrets, err := store.BulkGet(ctx, []string{"KEY1", "KEY2"})
	require.NoError(t, err)
	require.Len(t, secrets, 2)
}

// TestEnvSecretStore_BulkGet_Error は BulkGet で存在しないキーがある場合にエラーを返すことを確認する。
func TestEnvSecretStore_BulkGet_Error(t *testing.T) {
	ctx := context.Background()
	store := secretstore.NewEnvSecretStore("DEFINITELY_NOT_SET_XYZ_")
	_, err := store.BulkGet(ctx, []string{"MISSING1", "MISSING2"})
	require.Error(t, err)
	assert.Contains(t, err.Error(), "not found")
}

// TestFileSecretStore_Lifecycle は FileSecretStore の Init/Status/Close を検証する。
func TestFileSecretStore_Lifecycle(t *testing.T) {
	ctx := context.Background()
	store := secretstore.NewFileSecretStore(t.TempDir())

	// Name/Version を確認する。
	assert.Equal(t, "file-secretstore", store.Name())
	assert.Equal(t, "1.0.0", store.Version())

	// 初期化前は Uninitialized 状態であることを確認する。
	assert.Equal(t, secretstore.StatusUninitialized, store.Status(ctx))

	// Init 後は Ready 状態になることを確認する。
	require.NoError(t, store.Init(ctx, secretstore.Metadata{Name: "test"}))
	assert.Equal(t, secretstore.StatusReady, store.Status(ctx))

	// Close 後は Closed 状態になることを確認する。
	require.NoError(t, store.Close(ctx))
	assert.Equal(t, secretstore.StatusClosed, store.Status(ctx))
}

// TestFileSecretStore_BulkGet は複数のファイルからシークレットをまとめて取得できることを確認する。
func TestFileSecretStore_BulkGet(t *testing.T) {
	ctx := context.Background()
	dir := t.TempDir()
	require.NoError(t, os.WriteFile(filepath.Join(dir, "secret1"), []byte("value1"), 0600))
	require.NoError(t, os.WriteFile(filepath.Join(dir, "secret2"), []byte("value2"), 0600))
	store := secretstore.NewFileSecretStore(dir)

	// 複数ファイルを一括取得できることを検証する。
	secrets, err := store.BulkGet(ctx, []string{"secret1", "secret2"})
	require.NoError(t, err)
	require.Len(t, secrets, 2)
}

// TestFileSecretStore_BulkGet_Error は BulkGet でエラーが発生した場合に即座に返ることを確認する。
func TestFileSecretStore_BulkGet_Error(t *testing.T) {
	ctx := context.Background()
	store := secretstore.NewFileSecretStore(t.TempDir())
	_, err := store.BulkGet(ctx, []string{"nonexistent"})
	require.Error(t, err)
	assert.Contains(t, err.Error(), "not found")
}

// TestVaultSecretStore_Lifecycle は VaultSecretStore の Init/Status/Close を検証する。
func TestVaultSecretStore_Lifecycle(t *testing.T) {
	ctx := context.Background()
	store := secretstore.NewVaultSecretStore("my-vault", &mockVaultClient{secrets: map[string]secretstore.VaultSecret{}})

	// Name/Version を確認する。
	assert.Equal(t, "my-vault", store.Name())
	assert.Equal(t, "1.0.0", store.Version())

	// 初期化前は Uninitialized 状態であることを確認する。
	assert.Equal(t, secretstore.StatusUninitialized, store.Status(ctx))

	// Init 後は Ready 状態になることを確認する。
	require.NoError(t, store.Init(ctx, secretstore.Metadata{Name: "test"}))
	assert.Equal(t, secretstore.StatusReady, store.Status(ctx))

	// Close 後は Closed 状態になることを確認する。
	require.NoError(t, store.Close(ctx))
	assert.Equal(t, secretstore.StatusClosed, store.Status(ctx))
}

// TestVaultSecretStore_Get_MultipleKeys は複数キーを持つシークレットが "key=value" 形式で返ることを確認する。
func TestVaultSecretStore_Get_MultipleKeys(t *testing.T) {
	ctx := context.Background()
	client := &mockVaultClient{
		secrets: map[string]secretstore.VaultSecret{
			"secret/multi": {
				Path:    "secret/multi",
				Data:    map[string]string{"user": "admin", "pass": "s3cret"},
				Version: 1,
			},
		},
	}
	store := secretstore.NewVaultSecretStore("vault-store", client)
	secret, err := store.Get(ctx, "secret/multi")
	require.NoError(t, err)
	// 複数キーの場合は "key=value;key=value" 形式になることを確認する。
	assert.Contains(t, secret.Value, "=")
	assert.Contains(t, secret.Value, ";")
}

// TestVaultSecretStore_Get_Error は Vault エラー時に ComponentError が返ることを確認する。
func TestVaultSecretStore_Get_Error(t *testing.T) {
	ctx := context.Background()
	client := &mockVaultClient{err: fmt.Errorf("vault unavailable")}
	store := secretstore.NewVaultSecretStore("vault-store", client)
	_, err := store.Get(ctx, "any/path")
	require.Error(t, err)
	assert.Contains(t, err.Error(), "vault unavailable")
}

// TestVaultSecretStore_BulkGet は複数パスのシークレットをまとめて取得できることを確認する。
func TestVaultSecretStore_BulkGet(t *testing.T) {
	ctx := context.Background()
	client := &mockVaultClient{
		secrets: map[string]secretstore.VaultSecret{
			"secret/a": {Data: map[string]string{"val": "aaa"}, Version: 1},
			"secret/b": {Data: map[string]string{"val": "bbb"}, Version: 2},
		},
	}
	store := secretstore.NewVaultSecretStore("vault-store", client)
	secrets, err := store.BulkGet(ctx, []string{"secret/a", "secret/b"})
	require.NoError(t, err)
	require.Len(t, secrets, 2)
}

// TestVaultSecretStore_BulkGet_Error は BulkGet でエラーが発生した場合に即座に返ることを確認する。
func TestVaultSecretStore_BulkGet_Error(t *testing.T) {
	ctx := context.Background()
	client := &mockVaultClient{err: fmt.Errorf("vault unavailable")}
	store := secretstore.NewVaultSecretStore("vault-store", client)
	_, err := store.BulkGet(ctx, []string{"secret/a"})
	require.Error(t, err)
}
