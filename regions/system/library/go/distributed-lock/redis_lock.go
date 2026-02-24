package distributedlock

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"time"

	"github.com/redis/go-redis/v9"
)

// releaseScript is a Lua script for safe lock release.
// Only deletes the key if the stored value matches the token.
const releaseScript = `
if redis.call("get", KEYS[1]) == ARGV[1] then
    return redis.call("del", KEYS[1])
else
    return 0
end
`

// extendScript is a Lua script for safe lock extension.
// Only extends TTL if the stored value matches the token.
const extendScript = `
if redis.call("get", KEYS[1]) == ARGV[1] then
    return redis.call("pexpire", KEYS[1], ARGV[2])
else
    return 0
end
`

// RedisLock は Redis を使用した分散ロック実装。
// SET NX PX によるアトミックなロック取得と、Lua スクリプトによる安全な解放を実現する。
type RedisLock struct {
	client    redis.Cmdable
	keyPrefix string
}

// RedisLockOption は RedisLock の設定オプション。
type RedisLockOption func(*RedisLock)

// WithLockPrefix はロックキーのプレフィックスを設定する。
func WithLockPrefix(prefix string) RedisLockOption {
	return func(l *RedisLock) {
		l.keyPrefix = prefix
	}
}

// NewRedisLock は新しい RedisLock を生成する。
func NewRedisLock(client redis.Cmdable, opts ...RedisLockOption) *RedisLock {
	l := &RedisLock{
		client:    client,
		keyPrefix: "lock",
	}
	for _, opt := range opts {
		opt(l)
	}
	return l
}

// NewRedisLockFromURL は Redis URL から新しい RedisLock を生成する。
func NewRedisLockFromURL(url string, opts ...RedisLockOption) (*RedisLock, error) {
	options, err := redis.ParseURL(url)
	if err != nil {
		return nil, err
	}
	client := redis.NewClient(options)
	return NewRedisLock(client, opts...), nil
}

func (l *RedisLock) lockKey(key string) string {
	return l.keyPrefix + ":" + key
}

func (l *RedisLock) Acquire(ctx context.Context, key string, ttl time.Duration) (*LockGuard, error) {
	fullKey := l.lockKey(key)
	token := generateRedisToken()

	ok, err := l.client.SetNX(ctx, fullKey, token, ttl).Result()
	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, ErrAlreadyLocked
	}

	return &LockGuard{Key: key, Token: token}, nil
}

func (l *RedisLock) Release(ctx context.Context, guard *LockGuard) error {
	fullKey := l.lockKey(guard.Key)

	result, err := l.client.Eval(ctx, releaseScript, []string{fullKey}, guard.Token).Int64()
	if err != nil {
		return err
	}
	if result == 0 {
		return ErrTokenMismatch
	}
	return nil
}

// Extend はロックの有効期限を延長する。ガードのトークンが一致する場合のみ延長する。
func (l *RedisLock) Extend(ctx context.Context, guard *LockGuard, ttl time.Duration) error {
	fullKey := l.lockKey(guard.Key)
	millis := ttl.Milliseconds()

	result, err := l.client.Eval(ctx, extendScript, []string{fullKey}, guard.Token, millis).Int64()
	if err != nil {
		return err
	}
	if result == 0 {
		return ErrTokenMismatch
	}
	return nil
}

func (l *RedisLock) IsLocked(ctx context.Context, key string) (bool, error) {
	fullKey := l.lockKey(key)
	count, err := l.client.Exists(ctx, fullKey).Result()
	if err != nil {
		return false, err
	}
	return count > 0, nil
}

func generateRedisToken() string {
	b := make([]byte, 16)
	_, _ = rand.Read(b)
	return hex.EncodeToString(b)
}
