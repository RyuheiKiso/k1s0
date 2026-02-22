package session

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// mockRedis implements redis.Cmdable subset for testing via the Store interface.
// We test through a MemoryStore to avoid Redis dependency in unit tests.

// MemoryStore is an in-memory Store implementation for testing.
type MemoryStore struct {
	data    map[string]*SessionData
	expires map[string]time.Time
}

func NewMemoryStore() *MemoryStore {
	return &MemoryStore{
		data:    make(map[string]*SessionData),
		expires: make(map[string]time.Time),
	}
}

func (m *MemoryStore) Create(_ context.Context, data *SessionData, ttl time.Duration) (string, error) {
	id := "test-session-" + time.Now().Format("150405.000000")
	data.CreatedAt = time.Now().Unix()
	copied := *data
	m.data[id] = &copied
	m.expires[id] = time.Now().Add(ttl)
	return id, nil
}

func (m *MemoryStore) Get(_ context.Context, id string) (*SessionData, error) {
	d, ok := m.data[id]
	if !ok {
		return nil, nil
	}
	if exp, exists := m.expires[id]; exists && time.Now().After(exp) {
		delete(m.data, id)
		delete(m.expires, id)
		return nil, nil
	}
	return d, nil
}

func (m *MemoryStore) Update(_ context.Context, id string, data *SessionData, ttl time.Duration) error {
	copied := *data
	m.data[id] = &copied
	m.expires[id] = time.Now().Add(ttl)
	return nil
}

func (m *MemoryStore) Delete(_ context.Context, id string) error {
	delete(m.data, id)
	delete(m.expires, id)
	return nil
}

func (m *MemoryStore) Touch(_ context.Context, id string, ttl time.Duration) error {
	if _, ok := m.data[id]; ok {
		m.expires[id] = time.Now().Add(ttl)
	}
	return nil
}

func TestMemoryStore_CreateAndGet(t *testing.T) {
	store := NewMemoryStore()
	ctx := context.Background()

	data := &SessionData{
		AccessToken: "test-token",
		CSRFToken:   "csrf-123",
		Subject:     "user-1",
	}

	id, err := store.Create(ctx, data, 10*time.Minute)
	require.NoError(t, err)
	assert.NotEmpty(t, id)

	got, err := store.Get(ctx, id)
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, "test-token", got.AccessToken)
	assert.Equal(t, "csrf-123", got.CSRFToken)
	assert.Equal(t, "user-1", got.Subject)
	assert.NotZero(t, got.CreatedAt)
}

func TestMemoryStore_GetNotFound(t *testing.T) {
	store := NewMemoryStore()
	ctx := context.Background()

	got, err := store.Get(ctx, "nonexistent")
	require.NoError(t, err)
	assert.Nil(t, got)
}

func TestMemoryStore_Update(t *testing.T) {
	store := NewMemoryStore()
	ctx := context.Background()

	data := &SessionData{AccessToken: "old-token"}
	id, err := store.Create(ctx, data, 10*time.Minute)
	require.NoError(t, err)

	updated := &SessionData{AccessToken: "new-token"}
	err = store.Update(ctx, id, updated, 10*time.Minute)
	require.NoError(t, err)

	got, err := store.Get(ctx, id)
	require.NoError(t, err)
	assert.Equal(t, "new-token", got.AccessToken)
}

func TestMemoryStore_Delete(t *testing.T) {
	store := NewMemoryStore()
	ctx := context.Background()

	data := &SessionData{AccessToken: "token"}
	id, err := store.Create(ctx, data, 10*time.Minute)
	require.NoError(t, err)

	err = store.Delete(ctx, id)
	require.NoError(t, err)

	got, err := store.Get(ctx, id)
	require.NoError(t, err)
	assert.Nil(t, got)
}

func TestSessionData_IsExpired(t *testing.T) {
	tests := []struct {
		name      string
		expiresAt int64
		expired   bool
	}{
		{
			name:      "not expired",
			expiresAt: time.Now().Add(10 * time.Minute).Unix(),
			expired:   false,
		},
		{
			name:      "expired",
			expiresAt: time.Now().Add(-10 * time.Minute).Unix(),
			expired:   true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			s := &SessionData{ExpiresAt: tt.expiresAt}
			assert.Equal(t, tt.expired, s.IsExpired())
		})
	}
}
