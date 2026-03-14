package sessionclient_test

import (
	"context"
	"testing"
	"time"

	sessionclient "github.com/k1s0-platform/system-library-go-session-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Createが新しいセッションを正常に作成し、ID・トークン・有効期限などを返すことを確認する。
func TestCreate(t *testing.T) {
	c := sessionclient.NewInMemorySessionClient()
	ctx := context.Background()

	session, err := c.Create(ctx, sessionclient.CreateSessionRequest{
		UserID:     "user-1",
		TTLSeconds: 3600,
		Metadata:   map[string]string{"device": "mobile"},
	})
	require.NoError(t, err)
	assert.NotEmpty(t, session.ID)
	assert.NotEmpty(t, session.Token)
	assert.Equal(t, "user-1", session.UserID)
	assert.False(t, session.Revoked)
	assert.Equal(t, "mobile", session.Metadata["device"])
	assert.True(t, session.ExpiresAt.After(time.Now()))
}

// GetがセッションIDに対応するセッションを正常に取得することを確認する。
func TestGet(t *testing.T) {
	c := sessionclient.NewInMemorySessionClient()
	ctx := context.Background()

	created, _ := c.Create(ctx, sessionclient.CreateSessionRequest{
		UserID: "user-1", TTLSeconds: 3600,
	})

	got, err := c.Get(ctx, created.ID)
	require.NoError(t, err)
	assert.Equal(t, created.ID, got.ID)
}

// 存在しないセッションIDでGetを呼び出した際にエラーが返ることを確認する。
func TestGet_NotFound(t *testing.T) {
	c := sessionclient.NewInMemorySessionClient()
	ctx := context.Background()

	_, err := c.Get(ctx, "nonexistent")
	require.Error(t, err)
}

// Refreshがセッションのトークンを更新し、新しいトークンと延長された有効期限を返すことを確認する。
func TestRefresh(t *testing.T) {
	c := sessionclient.NewInMemorySessionClient()
	ctx := context.Background()

	created, _ := c.Create(ctx, sessionclient.CreateSessionRequest{
		UserID: "user-1", TTLSeconds: 60,
	})
	oldToken := created.Token

	refreshed, err := c.Refresh(ctx, sessionclient.RefreshSessionRequest{
		ID: created.ID, TTLSeconds: 7200,
	})
	require.NoError(t, err)
	assert.NotEqual(t, oldToken, refreshed.Token)
	assert.True(t, refreshed.ExpiresAt.After(time.Now().Add(3600*time.Second)))
}

// Revokeがセッションを無効化し、Revokedフラグがtrueに設定されることを確認する。
func TestRevoke(t *testing.T) {
	c := sessionclient.NewInMemorySessionClient()
	ctx := context.Background()

	created, _ := c.Create(ctx, sessionclient.CreateSessionRequest{
		UserID: "user-1", TTLSeconds: 3600,
	})

	err := c.Revoke(ctx, created.ID)
	require.NoError(t, err)

	got, _ := c.Get(ctx, created.ID)
	assert.True(t, got.Revoked)
}

// 存在しないセッションIDでRevokeを呼び出した際にエラーが返ることを確認する。
func TestRevoke_NotFound(t *testing.T) {
	c := sessionclient.NewInMemorySessionClient()
	ctx := context.Background()

	err := c.Revoke(ctx, "nonexistent")
	require.Error(t, err)
}

// ListUserSessionsが指定ユーザーのセッションのみを返すことを確認する。
func TestListUserSessions(t *testing.T) {
	c := sessionclient.NewInMemorySessionClient()
	ctx := context.Background()

	_, _ = c.Create(ctx, sessionclient.CreateSessionRequest{UserID: "user-1", TTLSeconds: 3600})
	_, _ = c.Create(ctx, sessionclient.CreateSessionRequest{UserID: "user-1", TTLSeconds: 3600})
	_, _ = c.Create(ctx, sessionclient.CreateSessionRequest{UserID: "user-2", TTLSeconds: 3600})

	sessions, err := c.ListUserSessions(ctx, "user-1")
	require.NoError(t, err)
	assert.Len(t, sessions, 2)
}

// RevokeAllが指定ユーザーの全セッションを無効化し、無効化件数を返すことを確認する。
func TestRevokeAll(t *testing.T) {
	c := sessionclient.NewInMemorySessionClient()
	ctx := context.Background()

	_, _ = c.Create(ctx, sessionclient.CreateSessionRequest{UserID: "user-1", TTLSeconds: 3600})
	_, _ = c.Create(ctx, sessionclient.CreateSessionRequest{UserID: "user-1", TTLSeconds: 3600})
	_, _ = c.Create(ctx, sessionclient.CreateSessionRequest{UserID: "user-2", TTLSeconds: 3600})

	count, err := c.RevokeAll(ctx, "user-1")
	require.NoError(t, err)
	assert.Equal(t, 2, count)

	sessions, _ := c.ListUserSessions(ctx, "user-1")
	for _, s := range sessions {
		assert.True(t, s.Revoked)
	}
}
