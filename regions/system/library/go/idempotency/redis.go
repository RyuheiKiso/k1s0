package idempotency

import (
	"context"
	"encoding/json"
	"errors"
	"time"

	"github.com/redis/go-redis/v9"
)

// RedisStoreOption は RedisIdempotencyStore の設定オプション。
type RedisStoreOption func(*RedisIdempotencyStore)

// WithRedisKeyPrefix はキーのプレフィックスを設定する。
func WithRedisKeyPrefix(prefix string) RedisStoreOption {
	return func(s *RedisIdempotencyStore) {
		s.keyPrefix = prefix
	}
}

// WithRedisDefaultTTL は ExpiresAt 未指定時に使う TTL を設定する。
func WithRedisDefaultTTL(ttl time.Duration) RedisStoreOption {
	return func(s *RedisIdempotencyStore) {
		s.defaultTTL = ttl
	}
}

// RedisIdempotencyStore は Redis バックエンド実装。
type RedisIdempotencyStore struct {
	client     redis.Cmdable
	keyPrefix  string
	defaultTTL time.Duration
}

// NewRedisIdempotencyStore は Redis クライアントからストアを生成する。
func NewRedisIdempotencyStore(client redis.Cmdable, opts ...RedisStoreOption) *RedisIdempotencyStore {
	store := &RedisIdempotencyStore{
		client:     client,
		defaultTTL: 24 * time.Hour,
	}
	for _, opt := range opts {
		opt(store)
	}
	return store
}

// NewRedisIdempotencyStoreFromURL は Redis URL からストアを生成する。
func NewRedisIdempotencyStoreFromURL(url string, opts ...RedisStoreOption) (*RedisIdempotencyStore, error) {
	options, err := redis.ParseURL(url)
	if err != nil {
		return nil, err
	}
	client := redis.NewClient(options)
	return NewRedisIdempotencyStore(client, opts...), nil
}

func (s *RedisIdempotencyStore) prefixedKey(key string) string {
	if s.keyPrefix == "" {
		return key
	}
	return s.keyPrefix + ":" + key
}

func (s *RedisIdempotencyStore) Get(ctx context.Context, key string) (*IdempotencyRecord, error) {
	fullKey := s.prefixedKey(key)
	raw, err := s.client.Get(ctx, fullKey).Bytes()
	if err != nil {
		if errors.Is(err, redis.Nil) {
			return nil, nil
		}
		return nil, err
	}
	var record IdempotencyRecord
	if err := json.Unmarshal(raw, &record); err != nil {
		return nil, err
	}
	if record.Key == "" {
		record.Key = key
	}
	if record.IsExpired() {
		_ = s.client.Del(ctx, fullKey).Err()
		return nil, nil
	}
	return &record, nil
}

func (s *RedisIdempotencyStore) Set(ctx context.Context, key string, record *IdempotencyRecord) error {
	copy := *record
	copy.Key = key
	if copy.CreatedAt.IsZero() {
		copy.CreatedAt = time.Now().UTC()
	}
	if copy.Status == "" {
		copy.Status = StatusPending
	}
	if copy.ExpiresAt.IsZero() && s.defaultTTL > 0 {
		copy.ExpiresAt = copy.CreatedAt.Add(s.defaultTTL)
	}

	ttl := s.ttlForRecord(&copy)
	if ttl <= 0 && !copy.ExpiresAt.IsZero() {
		return NewExpiredError(key)
	}

	payload, err := json.Marshal(copy)
	if err != nil {
		return err
	}

	ok, err := s.client.SetNX(ctx, s.prefixedKey(key), payload, ttl).Result()
	if err != nil {
		return err
	}
	if !ok {
		return NewDuplicateError(key)
	}
	return nil
}

func (s *RedisIdempotencyStore) MarkCompleted(
	ctx context.Context,
	key string,
	response []byte,
	statusCode int,
) error {
	record, err := s.Get(ctx, key)
	if err != nil {
		return err
	}
	if record == nil {
		return NewNotFoundError(key)
	}

	record.Status = StatusCompleted
	record.Response = append([]byte(nil), response...)
	record.StatusCode = statusCode
	record.Error = ""

	return s.saveUpdatedRecord(ctx, key, record)
}

func (s *RedisIdempotencyStore) MarkFailed(ctx context.Context, key string, err error) error {
	record, getErr := s.Get(ctx, key)
	if getErr != nil {
		return getErr
	}
	if record == nil {
		return NewNotFoundError(key)
	}

	record.Status = StatusFailed
	record.Response = nil
	record.StatusCode = 0
	if err != nil {
		record.Error = err.Error()
	} else {
		record.Error = ""
	}

	return s.saveUpdatedRecord(ctx, key, record)
}

func (s *RedisIdempotencyStore) saveUpdatedRecord(ctx context.Context, key string, record *IdempotencyRecord) error {
	ttl := s.ttlForRecord(record)
	if ttl <= 0 && !record.ExpiresAt.IsZero() {
		_ = s.client.Del(ctx, s.prefixedKey(key)).Err()
		return NewExpiredError(key)
	}

	payload, err := json.Marshal(record)
	if err != nil {
		return err
	}

	return s.client.Set(ctx, s.prefixedKey(key), payload, ttl).Err()
}

func (s *RedisIdempotencyStore) ttlForRecord(record *IdempotencyRecord) time.Duration {
	if record.ExpiresAt.IsZero() {
		return 0
	}
	return time.Until(record.ExpiresAt)
}
