package session

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/redis/go-redis/v9"
)

// Store is the interface for session persistence.
type Store interface {
	// Create persists a new session and returns its ID.
	Create(ctx context.Context, data *SessionData, ttl time.Duration) (string, error)

	// Get retrieves a session by ID. Returns nil if not found.
	Get(ctx context.Context, id string) (*SessionData, error)

	// Update replaces session data while preserving the TTL.
	Update(ctx context.Context, id string, data *SessionData, ttl time.Duration) error

	// Delete removes a session by ID.
	Delete(ctx context.Context, id string) error

	// Touch extends the session TTL (sliding expiration).
	Touch(ctx context.Context, id string, ttl time.Duration) error
}

// RedisStore implements Store backed by Redis (standalone or Sentinel).
type RedisStore struct {
	client redis.Cmdable
	prefix string
}

// NewRedisStore creates a Redis-backed session store.
func NewRedisStore(client redis.Cmdable, prefix string) *RedisStore {
	if prefix == "" {
		prefix = "bff:session:"
	}
	return &RedisStore{client: client, prefix: prefix}
}

func (s *RedisStore) key(id string) string {
	return s.prefix + id
}

// Create persists a new session and returns its ID.
func (s *RedisStore) Create(ctx context.Context, data *SessionData, ttl time.Duration) (string, error) {
	id := uuid.New().String()
	data.CreatedAt = time.Now().Unix()

	b, err := json.Marshal(data)
	if err != nil {
		return "", fmt.Errorf("failed to marshal session: %w", err)
	}

	if err := s.client.Set(ctx, s.key(id), b, ttl).Err(); err != nil {
		return "", fmt.Errorf("failed to store session: %w", err)
	}

	return id, nil
}

// Get retrieves a session by ID. Returns nil if not found.
func (s *RedisStore) Get(ctx context.Context, id string) (*SessionData, error) {
	val, err := s.client.Get(ctx, s.key(id)).Result()
	if err == redis.Nil {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get session: %w", err)
	}

	var data SessionData
	if err := json.Unmarshal([]byte(val), &data); err != nil {
		return nil, fmt.Errorf("failed to unmarshal session: %w", err)
	}

	return &data, nil
}

// Update replaces session data while preserving the TTL.
func (s *RedisStore) Update(ctx context.Context, id string, data *SessionData, ttl time.Duration) error {
	b, err := json.Marshal(data)
	if err != nil {
		return fmt.Errorf("failed to marshal session: %w", err)
	}

	if err := s.client.Set(ctx, s.key(id), b, ttl).Err(); err != nil {
		return fmt.Errorf("failed to update session: %w", err)
	}

	return nil
}

// Delete removes a session by ID.
func (s *RedisStore) Delete(ctx context.Context, id string) error {
	if err := s.client.Del(ctx, s.key(id)).Err(); err != nil {
		return fmt.Errorf("failed to delete session: %w", err)
	}
	return nil
}

// Touch extends the session TTL (sliding expiration).
func (s *RedisStore) Touch(ctx context.Context, id string, ttl time.Duration) error {
	if err := s.client.Expire(ctx, s.key(id), ttl).Err(); err != nil {
		return fmt.Errorf("failed to touch session: %w", err)
	}
	return nil
}
