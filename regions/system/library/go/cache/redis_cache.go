package cache

import (
	"context"
	"errors"
	"time"

	"github.com/redis/go-redis/v9"
)

// RedisCacheClient は Redis を使用したキャッシュクライアント実装。
type RedisCacheClient struct {
	client    redis.Cmdable
	keyPrefix string
}

// RedisCacheOption は RedisCacheClient の設定オプション。
type RedisCacheOption func(*RedisCacheClient)

// WithKeyPrefix はキーのプレフィックスを設定する。
func WithKeyPrefix(prefix string) RedisCacheOption {
	return func(c *RedisCacheClient) {
		c.keyPrefix = prefix
	}
}

// NewRedisCacheClient は新しい RedisCacheClient を生成する。
func NewRedisCacheClient(client redis.Cmdable, opts ...RedisCacheOption) *RedisCacheClient {
	c := &RedisCacheClient{
		client: client,
	}
	for _, opt := range opts {
		opt(c)
	}
	return c
}

// NewRedisCacheClientFromURL は Redis URL から新しい RedisCacheClient を生成する。
func NewRedisCacheClientFromURL(url string, opts ...RedisCacheOption) (*RedisCacheClient, error) {
	options, err := redis.ParseURL(url)
	if err != nil {
		return nil, NewConnectionError(err.Error())
	}
	client := redis.NewClient(options)
	return NewRedisCacheClient(client, opts...), nil
}

func (c *RedisCacheClient) prefixedKey(key string) string {
	if c.keyPrefix == "" {
		return key
	}
	return c.keyPrefix + ":" + key
}

func (c *RedisCacheClient) Get(ctx context.Context, key string) (*string, error) {
	fullKey := c.prefixedKey(key)
	val, err := c.client.Get(ctx, fullKey).Result()
	if err != nil {
		if errors.Is(err, redis.Nil) {
			return nil, nil
		}
		return nil, NewConnectionError(err.Error())
	}
	return &val, nil
}

func (c *RedisCacheClient) Set(ctx context.Context, key string, value string, ttl *time.Duration) error {
	fullKey := c.prefixedKey(key)
	var expiration time.Duration
	if ttl != nil {
		expiration = *ttl
	}
	err := c.client.Set(ctx, fullKey, value, expiration).Err()
	if err != nil {
		return NewConnectionError(err.Error())
	}
	return nil
}

func (c *RedisCacheClient) Delete(ctx context.Context, key string) (bool, error) {
	fullKey := c.prefixedKey(key)
	count, err := c.client.Del(ctx, fullKey).Result()
	if err != nil {
		return false, NewConnectionError(err.Error())
	}
	return count > 0, nil
}

func (c *RedisCacheClient) Exists(ctx context.Context, key string) (bool, error) {
	fullKey := c.prefixedKey(key)
	count, err := c.client.Exists(ctx, fullKey).Result()
	if err != nil {
		return false, NewConnectionError(err.Error())
	}
	return count > 0, nil
}

func (c *RedisCacheClient) SetNX(ctx context.Context, key string, value string, ttl time.Duration) (bool, error) {
	fullKey := c.prefixedKey(key)
	ok, err := c.client.SetNX(ctx, fullKey, value, ttl).Result()
	if err != nil {
		return false, NewConnectionError(err.Error())
	}
	return ok, nil
}

func (c *RedisCacheClient) Expire(ctx context.Context, key string, ttl time.Duration) (bool, error) {
	fullKey := c.prefixedKey(key)
	ok, err := c.client.Expire(ctx, fullKey, ttl).Result()
	if err != nil {
		return false, NewConnectionError(err.Error())
	}
	return ok, nil
}
