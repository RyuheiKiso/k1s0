// noop_store_test.go は NoOpStore の全メソッドが安全に no-op 動作することを検証する。
// エラーが返されず、かつ Get/GetExchangeCode が nil を返すことを確認する（H-002 監査対応）。
package session

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestNoOpStore_Create は Create がエラーなしにダミーセッション ID を返すことを確認する。
func TestNoOpStore_Create(t *testing.T) {
	store := NewNoOpStore()
	ctx := context.Background()

	// 任意の SessionData で Create を呼んでもエラーにならない
	id, err := store.Create(ctx, &SessionData{AccessToken: "token"}, 10*time.Minute)
	require.NoError(t, err)
	assert.NotEmpty(t, id)
}

// TestNoOpStore_Get は Get が常に nil を返すことを確認する。
func TestNoOpStore_Get(t *testing.T) {
	store := NewNoOpStore()
	ctx := context.Background()

	// Create した後でも Get は nil を返す（データを保持しない）
	_, err := store.Create(ctx, &SessionData{}, time.Minute)
	require.NoError(t, err)

	got, err := store.Get(ctx, "any-session-id")
	require.NoError(t, err)
	assert.Nil(t, got, "NoOpStore.Get は常に nil を返す")
}

// TestNoOpStore_Update は Update がエラーなしに処理されることを確認する。
func TestNoOpStore_Update(t *testing.T) {
	store := NewNoOpStore()
	ctx := context.Background()

	err := store.Update(ctx, "any-id", &SessionData{AccessToken: "new"}, time.Minute)
	require.NoError(t, err)
}

// TestNoOpStore_Delete は Delete がエラーなしに処理されることを確認する。
func TestNoOpStore_Delete(t *testing.T) {
	store := NewNoOpStore()
	ctx := context.Background()

	err := store.Delete(ctx, "any-id")
	require.NoError(t, err)
}

// TestNoOpStore_Touch は Touch がエラーなしに処理されることを確認する。
func TestNoOpStore_Touch(t *testing.T) {
	store := NewNoOpStore()
	ctx := context.Background()

	err := store.Touch(ctx, "any-id", time.Minute)
	require.NoError(t, err)
}

// TestNoOpStore_ExchangeCode は ExchangeCode 関連メソッドが安全に動作することを確認する。
func TestNoOpStore_ExchangeCode(t *testing.T) {
	store := NewNoOpStore()
	ctx := context.Background()

	// CreateExchangeCode はダミーコードを返す
	code, err := store.CreateExchangeCode(ctx, &ExchangeCodeData{SessionID: "sess"}, time.Minute)
	require.NoError(t, err)
	assert.NotEmpty(t, code)

	// GetExchangeCode は常に nil を返す（データを保持しない）
	got, err := store.GetExchangeCode(ctx, code)
	require.NoError(t, err)
	assert.Nil(t, got, "NoOpStore.GetExchangeCode は常に nil を返す")

	// DeleteExchangeCode はエラーなしに処理される
	err = store.DeleteExchangeCode(ctx, code)
	require.NoError(t, err)
}

// TestNoOpStore_ImplementsFullStore は NoOpStore が FullStore インターフェースを実装していることをコンパイル時に検証する。
func TestNoOpStore_ImplementsFullStore(t *testing.T) {
	var _ FullStore = (*NoOpStore)(nil)
}
