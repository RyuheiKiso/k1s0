package idempotency

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"errors"
	"strings"
	"time"

	"github.com/redis/go-redis/v9"
)

// markCompletedScript は MarkCompleted をアトミックに実行する Lua スクリプト。
// Get → フィールド更新 → SET を1回のアトミック操作として行い TOCTOU 競合を防ぐ。
// KEYS[1]: プレフィックス付きキー
// ARGV[1]: 新しいレスポンス (JSON バイト列を base64 ではなく raw 文字列として渡す)
// ARGV[2]: HTTP ステータスコード (文字列)
var markCompletedScript = redis.NewScript(`
local raw = redis.call('GET', KEYS[1])
if raw == false then
  return redis.error_reply('not_found')
end
local record = cjson.decode(raw)
record['status'] = 'completed'
if ARGV[1] ~= '' then
  record['response'] = ARGV[1]
else
  record['response'] = nil
end
record['status_code'] = tonumber(ARGV[2])
record['error'] = nil
redis.call('SET', KEYS[1], cjson.encode(record), 'KEEPTTL')
return 1
`)

// markFailedScript は MarkFailed をアトミックに実行する Lua スクリプト。
// Get → フィールド更新 → SET を1回のアトミック操作として行い TOCTOU 競合を防ぐ。
// KEYS[1]: プレフィックス付きキー
// ARGV[1]: エラーメッセージ文字列
var markFailedScript = redis.NewScript(`
local raw = redis.call('GET', KEYS[1])
if raw == false then
  return redis.error_reply('not_found')
end
local record = cjson.decode(raw)
record['status'] = 'failed'
record['response'] = nil
record['status_code'] = 0
if ARGV[1] ~= '' then
  record['error'] = ARGV[1]
else
  record['error'] = nil
end
redis.call('SET', KEYS[1], cjson.encode(record), 'KEEPTTL')
return 1
`)

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

// prefixedKey はプレフィックスを付与した完全キーを返す。
func (s *RedisIdempotencyStore) prefixedKey(key string) string {
	if s.keyPrefix == "" {
		return key
	}
	return s.keyPrefix + ":" + key
}

// Get は指定キーのレコードを取得する。存在しない場合は nil を返す。
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

// Set は新規レコードを SetNX でアトミックに登録する。重複キーの場合は DuplicateError を返す。
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

// MarkCompleted はレコードを Completed 状態へアトミックに更新する。
// Lua スクリプトにより GET → フィールド更新 → SET が単一アトミック操作として実行され、
// TOCTOU 競合状態（Lost Update）を防止する。
func (s *RedisIdempotencyStore) MarkCompleted(
	ctx context.Context,
	key string,
	response []byte,
	statusCode int,
) error {
	// Go の encoding/json は []byte フィールドを base64 エンコードして保存するため、
	// Lua スクリプトへ渡す際も base64 エンコード済み文字列を使用する。
	responseArg := ""
	if len(response) > 0 {
		responseArg = base64.StdEncoding.EncodeToString(response)
	}

	err := markCompletedScript.Run(
		ctx,
		s.client,
		[]string{s.prefixedKey(key)},
		responseArg,
		statusCode,
	).Err()

	if err != nil {
		// Lua スクリプトが返す "not_found" エラーを NotFoundError に変換する
		if isLuaError(err, "not_found") {
			return NewNotFoundError(key)
		}
		return err
	}
	return nil
}

// MarkFailed はレコードを Failed 状態へアトミックに更新する。
// Lua スクリプトにより GET → フィールド更新 → SET が単一アトミック操作として実行され、
// TOCTOU 競合状態（Lost Update）を防止する。
func (s *RedisIdempotencyStore) MarkFailed(ctx context.Context, key string, err error) error {
	// エラーメッセージを文字列として Lua スクリプトへ渡す（nil の場合は空文字列）
	errMsg := ""
	if err != nil {
		errMsg = err.Error()
	}

	runErr := markFailedScript.Run(
		ctx,
		s.client,
		[]string{s.prefixedKey(key)},
		errMsg,
	).Err()

	if runErr != nil {
		// Lua スクリプトが返す "not_found" エラーを NotFoundError に変換する
		if isLuaError(runErr, "not_found") {
			return NewNotFoundError(key)
		}
		return runErr
	}
	return nil
}

// ttlForRecord はレコードの残り TTL を計算する。ExpiresAt が未設定の場合は 0 を返す。
func (s *RedisIdempotencyStore) ttlForRecord(record *IdempotencyRecord) time.Duration {
	if record.ExpiresAt.IsZero() {
		return 0
	}
	return time.Until(record.ExpiresAt)
}

// isLuaError は Redis Lua スクリプトが返すエラーメッセージに指定文字列が含まれるか判定する。
// go-redis は Lua の error_reply を "ERR <message>" 形式のエラーとして返す。
func isLuaError(err error, substr string) bool {
	if err == nil {
		return false
	}
	return strings.Contains(err.Error(), substr)
}
