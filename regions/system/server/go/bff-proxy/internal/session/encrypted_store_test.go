// EncryptedStore の単体テスト。
// Redis クライアントをインメモリモックに差し替えて、AES-GCM 暗号化ロジックを検証する。
// 依存: go-redis/v9 の Cmdable インターフェースを最小実装でモックする。
package session

import (
	"context"
	"fmt"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// minRedis は EncryptedStore が使用するメソッドのみを実装した Redis インメモリモック。
// redis.Cmdable を embed して未使用メソッドのコンパイルエラーを回避する。
type minRedis struct {
	redis.Cmdable // 未実装メソッドは呼ばれないため embed のみ
	mu            sync.Mutex
	data          map[string]string
	ttls          map[string]time.Time
}

// newMinRedis はテスト用のインメモリ Redis クライアントを生成する。
func newMinRedis() *minRedis {
	return &minRedis{
		data: make(map[string]string),
		ttls: make(map[string]time.Time),
	}
}

// Set はキーと値を保存する。
func (m *minRedis) Set(_ context.Context, key string, value any, expiration time.Duration) *redis.StatusCmd {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.data[key] = fmt.Sprintf("%v", value)
	if expiration > 0 {
		m.ttls[key] = time.Now().Add(expiration)
	}
	cmd := redis.NewStatusCmd(context.Background())
	cmd.SetVal("OK")
	return cmd
}

// Get はキーに対応する値を返す。存在しない場合は redis.Nil エラーを返す。
func (m *minRedis) Get(_ context.Context, key string) *redis.StringCmd {
	m.mu.Lock()
	defer m.mu.Unlock()
	val, ok := m.data[key]
	if !ok {
		cmd := redis.NewStringCmd(context.Background())
		cmd.SetErr(redis.Nil)
		return cmd
	}
	if exp, hasExp := m.ttls[key]; hasExp && time.Now().After(exp) {
		delete(m.data, key)
		delete(m.ttls, key)
		cmd := redis.NewStringCmd(context.Background())
		cmd.SetErr(redis.Nil)
		return cmd
	}
	cmd := redis.NewStringCmd(context.Background())
	cmd.SetVal(val)
	return cmd
}

// Del はキーを削除する。
func (m *minRedis) Del(_ context.Context, keys ...string) *redis.IntCmd {
	m.mu.Lock()
	defer m.mu.Unlock()
	var deleted int64
	for _, k := range keys {
		if _, ok := m.data[k]; ok {
			delete(m.data, k)
			delete(m.ttls, k)
			deleted++
		}
	}
	cmd := redis.NewIntCmd(context.Background())
	cmd.SetVal(deleted)
	return cmd
}

// Expire はキーの TTL を延長する。
func (m *minRedis) Expire(_ context.Context, key string, expiration time.Duration) *redis.BoolCmd {
	m.mu.Lock()
	defer m.mu.Unlock()
	cmd := redis.NewBoolCmd(context.Background())
	if _, ok := m.data[key]; ok {
		m.ttls[key] = time.Now().Add(expiration)
		cmd.SetVal(true)
	} else {
		cmd.SetVal(false)
	}
	return cmd
}

// valid32ByteKey はテスト用の 32 バイト固定キーを返す。
func valid32ByteKey() []byte {
	return []byte("01234567890123456789012345678901") // 32 bytes
}

// TestNewEncryptedStore_InvalidKey は鍵長不正の場合にエラーを返すことを検証する。
func TestNewEncryptedStore_InvalidKey(t *testing.T) {
	client := newMinRedis()

	_, err := NewEncryptedStore(client, "", []byte("short"))
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "32 バイト")
}

// TestNewEncryptedStore_ValidKey は正常な鍵でストアが作成されることを検証する。
func TestNewEncryptedStore_ValidKey(t *testing.T) {
	client := newMinRedis()
	store, err := NewEncryptedStore(client, "", valid32ByteKey())
	require.NoError(t, err)
	assert.NotNil(t, store)
}

// TestEncryptedStore_CreateAndGet は Create→Get の暗号化ラウンドトリップを検証する。
func TestEncryptedStore_CreateAndGet(t *testing.T) {
	client := newMinRedis()
	store, err := NewEncryptedStore(client, "test:", valid32ByteKey())
	require.NoError(t, err)

	ctx := context.Background()
	data := &SessionData{
		AccessToken:  "access-token-secret",
		RefreshToken: "refresh-token-secret",
		Subject:      "user-sub-001",
		CSRFToken:    "csrf-123",
		ExpiresAt:    time.Now().Add(1 * time.Hour).Unix(),
	}

	id, err := store.Create(ctx, data, 30*time.Minute)
	require.NoError(t, err)
	assert.NotEmpty(t, id)

	// Redis に保存されている値が平文でないことを確認する（AES-GCM 暗号化済み）
	rawVal, found := client.data["test:"+id]
	assert.True(t, found, "key should exist in redis")
	assert.NotContains(t, rawVal, "access-token-secret", "stored value should be encrypted, not plaintext")

	// Get で正しく復号されること
	got, err := store.Get(ctx, id)
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, "access-token-secret", got.AccessToken)
	assert.Equal(t, "refresh-token-secret", got.RefreshToken)
	assert.Equal(t, "user-sub-001", got.Subject)
	assert.Equal(t, "csrf-123", got.CSRFToken)
}

// TestEncryptedStore_GetNotFound は存在しないセッション ID で nil が返ることを検証する。
func TestEncryptedStore_GetNotFound(t *testing.T) {
	client := newMinRedis()
	store, err := NewEncryptedStore(client, "test:", valid32ByteKey())
	require.NoError(t, err)

	got, err := store.Get(context.Background(), "nonexistent-id")
	require.NoError(t, err)
	assert.Nil(t, got)
}

// TestEncryptedStore_Update は Update 後に Get で更新済みデータが取得できることを検証する。
func TestEncryptedStore_Update(t *testing.T) {
	client := newMinRedis()
	store, err := NewEncryptedStore(client, "test:", valid32ByteKey())
	require.NoError(t, err)

	ctx := context.Background()
	original := &SessionData{AccessToken: "old-token", Subject: "user-001"}
	id, err := store.Create(ctx, original, 30*time.Minute)
	require.NoError(t, err)

	updated := &SessionData{AccessToken: "new-token", Subject: "user-001"}
	err = store.Update(ctx, id, updated, 30*time.Minute)
	require.NoError(t, err)

	got, err := store.Get(ctx, id)
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, "new-token", got.AccessToken)
}

// TestEncryptedStore_Delete はセッション削除後に Get で nil が返ることを検証する。
func TestEncryptedStore_Delete(t *testing.T) {
	client := newMinRedis()
	store, err := NewEncryptedStore(client, "test:", valid32ByteKey())
	require.NoError(t, err)

	ctx := context.Background()
	data := &SessionData{AccessToken: "token-to-delete"}
	id, err := store.Create(ctx, data, 30*time.Minute)
	require.NoError(t, err)

	err = store.Delete(ctx, id)
	require.NoError(t, err)

	// 削除後は nil が返ること
	got, err := store.Get(ctx, id)
	require.NoError(t, err)
	assert.Nil(t, got)
}

// TestEncryptedStore_Touch は Touch 後に TTL が延長されることを検証する。
func TestEncryptedStore_Touch(t *testing.T) {
	client := newMinRedis()
	store, err := NewEncryptedStore(client, "test:", valid32ByteKey())
	require.NoError(t, err)

	ctx := context.Background()
	data := &SessionData{AccessToken: "token-to-touch"}
	id, err := store.Create(ctx, data, 5*time.Minute)
	require.NoError(t, err)

	// Touch で TTL を延長する
	err = store.Touch(ctx, id, 30*time.Minute)
	require.NoError(t, err)

	// Touch 後もセッションが取得できること
	got, err := store.Get(ctx, id)
	require.NoError(t, err)
	assert.NotNil(t, got, "session should still be accessible after touch")
}

// TestEncryptedStore_TamperedCiphertext は改ざんされた暗号文で Get がエラーを返すことを検証する。
func TestEncryptedStore_TamperedCiphertext(t *testing.T) {
	client := newMinRedis()
	store, err := NewEncryptedStore(client, "test:", valid32ByteKey())
	require.NoError(t, err)

	ctx := context.Background()
	data := &SessionData{AccessToken: "secret-token"}
	id, err := store.Create(ctx, data, 30*time.Minute)
	require.NoError(t, err)

	// Redis に保存されている暗号文を改ざんする
	key := "test:" + id
	original := client.data[key]
	// 最後の文字を変更して認証タグを破壊する
	tampered := original[:len(original)-2] + strings.ToUpper(original[len(original)-2:])
	if tampered == original {
		// 変更がなかった場合の代替改ざん
		tampered = "TAMPERED_INVALID_BASE64_CIPHERTEXT"
	}
	client.data[key] = tampered

	// 改ざんされた暗号文を復号しようとするとエラーが返ること
	got, err := store.Get(ctx, id)
	assert.Error(t, err, "tampered ciphertext should cause decryption error")
	assert.Nil(t, got)
}

// TestEncryptedStore_KeyIsolation は異なる鍵で暗号化されたデータが復号できないことを検証する。
// AES-GCM の認証タグで鍵不一致が検出されることを確認する。
func TestEncryptedStore_KeyIsolation(t *testing.T) {
	client := newMinRedis()
	key1 := []byte("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA") // 32 bytes
	key2 := []byte("BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB") // 32 bytes

	store1, err := NewEncryptedStore(client, "test:", key1)
	require.NoError(t, err)
	store2, err := NewEncryptedStore(client, "test:", key2)
	require.NoError(t, err)

	ctx := context.Background()
	data := &SessionData{AccessToken: "secret-with-key1"}
	id, err := store1.Create(ctx, data, 30*time.Minute)
	require.NoError(t, err)

	// 別の鍵で復号しようとするとエラーになること（鍵不一致の認証タグ検証失敗）
	got, err := store2.Get(ctx, id)
	assert.Error(t, err, "decryption with wrong key should fail")
	assert.Nil(t, got)
}

// TestEncryptedStore_DefaultPrefix は prefix 未指定時にデフォルト値が使われることを検証する。
func TestEncryptedStore_DefaultPrefix(t *testing.T) {
	client := newMinRedis()
	store, err := NewEncryptedStore(client, "", valid32ByteKey())
	require.NoError(t, err)

	ctx := context.Background()
	data := &SessionData{AccessToken: "token"}
	id, err := store.Create(ctx, data, 30*time.Minute)
	require.NoError(t, err)

	// デフォルト prefix "bff:session:" でキーが保存されていること
	defaultKey := "bff:session:" + id
	_, found := client.data[defaultKey]
	assert.True(t, found, "should use default prefix 'bff:session:'")
}
