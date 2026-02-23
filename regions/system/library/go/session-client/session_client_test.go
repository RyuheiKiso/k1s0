package sessionclient_test

import (
	"context"
	"testing"
	"time"

	sessionclient "github.com/k1s0-platform/system-library-go-session-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

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

func TestGet_NotFound(t *testing.T) {
	c := sessionclient.NewInMemorySessionClient()
	ctx := context.Background()

	_, err := c.Get(ctx, "nonexistent")
	require.Error(t, err)
}

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

func TestRevoke_NotFound(t *testing.T) {
	c := sessionclient.NewInMemorySessionClient()
	ctx := context.Background()

	err := c.Revoke(ctx, "nonexistent")
	require.Error(t, err)
}

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
