package buildingblocks

import (
	"context"
	"fmt"
	"strconv"
	"sync"
	"sync/atomic"
	"time"
)

// CacheClient はk1s0-cache の CacheClient と互換性を持つインターフェース。
// *cache.RedisClient を注入することでこのインターフェースを満たせる。
type CacheClient interface {
	// Get はキーに対応する値を取得する。キーが存在しない場合は nil を返す。
	Get(ctx context.Context, key string) (*string, error)
	// Set はキーと値を保存する。ttl が nil の場合は有効期限なしで保存する。
	Set(ctx context.Context, key, value string, ttl *time.Duration) error
	// Delete はキーを削除し、削除が行われたかどうかを返す。
	Delete(ctx context.Context, key string) (bool, error)
	// Exists はキーが存在するかどうかを確認する。
	Exists(ctx context.Context, key string) (bool, error)
}

// コンパイル時にインターフェース準拠を検証する。
var _ StateStore = (*RedisStateStore)(nil)

// RedisStateStore は Redis をバックエンドとする StateStore 実装。
// CacheClient をラップして状態管理機能を提供する。
// ETag は ":__etag" サフィックスを付けた別の Redis キーとして保存する。
type RedisStateStore struct {
	// mu は status フィールドへの並行アクセスを保護する読み書きミューテックス。
	mu sync.RWMutex
	// name はコンポーネントの識別子。
	name string
	// client は Redis へのアクセスを担うキャッシュクライアント実装。
	client CacheClient
	// counter は単調増加する ETag の生成に使用するアトミックカウンター。
	counter atomic.Uint64
	// status はコンポーネントの現在の状態を表す。
	status ComponentStatus
}

// NewRedisStateStore は新しい RedisStateStore を生成して返す。
// name はコンポーネント識別子、client は Redis へのアクセスを担うクライアント実装。
func NewRedisStateStore(name string, client CacheClient) *RedisStateStore {
	return &RedisStateStore{
		name:   name,
		client: client,
		status: StatusUninitialized,
	}
}

// Name はコンポーネント識別子を返す。
func (s *RedisStateStore) Name() string { return s.name }

// Version はコンポーネントのバージョン文字列を返す。
func (s *RedisStateStore) Version() string { return "1.0.0" }

// Init はコンポーネントを初期化し、ステータスを Ready に遷移させる。
func (s *RedisStateStore) Init(_ context.Context, _ Metadata) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusReady
	return nil
}

// Close はコンポーネントを終了し、ステータスを Closed に遷移させる。
func (s *RedisStateStore) Close(_ context.Context) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusClosed
	return nil
}

// Status はコンポーネントの現在のステータスを返す。
func (s *RedisStateStore) Status(_ context.Context) ComponentStatus {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.status
}

// etagKey は指定されたキーに対応する ETag 保存用の Redis キーを返す。
// ETag は元のキーに ":__etag" サフィックスを付けた別キーとして管理する。
func (s *RedisStateStore) etagKey(key string) string { return key + ":__etag" }

// nextETag はアトミックカウンターをインクリメントして新しい ETag 文字列を生成する。
// 単調増加する数値を文字列化することで一意性を保証する。
func (s *RedisStateStore) nextETag() string {
	return strconv.FormatUint(s.counter.Add(1), 10)
}

// Get は指定されたキーの状態エントリを Redis から取得する。
// キーが存在しない場合は nil, nil を返す。ETag も合わせて取得して返す。
func (s *RedisStateStore) Get(ctx context.Context, key string) (*StateEntry, error) {
	val, err := s.client.Get(ctx, key)
	if err != nil {
		return nil, NewComponentError(s.name, "Get",
			fmt.Sprintf("failed to get key %q from Redis", key), err)
	}
	// キーが存在しない場合は nil を返す（エラーではない）。
	if val == nil {
		return nil, nil
	}
	// ETag も別キーから取得する。
	etag, err := s.client.Get(ctx, s.etagKey(key))
	if err != nil {
		return nil, NewComponentError(s.name, "Get",
			fmt.Sprintf("failed to get etag for key %q", key), err)
	}
	etagVal := ""
	if etag != nil {
		etagVal = *etag
	}
	return &StateEntry{Key: key, Value: []byte(*val), ETag: &ETag{Value: etagVal}}, nil
}

// Set は指定されたキーに値を保存する。
// ETag が指定されている場合は楽観的排他制御を行い、不一致の場合は ETagMismatchError を返す。
// 保存成功時は新しい ETag を返す。
func (s *RedisStateStore) Set(ctx context.Context, req *SetRequest) (*ETag, error) {
	// ETag が指定されている場合は現在の ETag と照合して楽観的排他制御を行う。
	if req.ETag != nil {
		current, err := s.client.Get(ctx, s.etagKey(req.Key))
		if err != nil {
			return nil, NewComponentError(s.name, "Set",
				fmt.Sprintf("failed to check etag for key %q", req.Key), err)
		}
		if current == nil {
			return nil, &ETagMismatchError{Key: req.Key, Expected: req.ETag, Actual: nil}
		}
		if *current != req.ETag.Value {
			return nil, &ETagMismatchError{Key: req.Key, Expected: req.ETag, Actual: &ETag{Value: *current}}
		}
	}
	// 新しい ETag を生成して値と ETag を Redis に保存する。
	newETag := s.nextETag()
	if err := s.client.Set(ctx, req.Key, string(req.Value), nil); err != nil {
		return nil, NewComponentError(s.name, "Set",
			fmt.Sprintf("failed to set key %q in Redis", req.Key), err)
	}
	if err := s.client.Set(ctx, s.etagKey(req.Key), newETag, nil); err != nil {
		return nil, NewComponentError(s.name, "Set",
			fmt.Sprintf("failed to set etag for key %q", req.Key), err)
	}
	return &ETag{Value: newETag}, nil
}

// Delete は指定されたキーの状態エントリを Redis から削除する。
// ETag が指定されている場合は現在の ETag と照合し、不一致の場合は ETagMismatchError を返す。
// 値と ETag の両方のキーを削除する。
func (s *RedisStateStore) Delete(ctx context.Context, key string, etag *ETag) error {
	// ETag が指定されている場合は削除前に現在の ETag と照合する。
	if etag != nil {
		current, err := s.client.Get(ctx, s.etagKey(key))
		if err != nil {
			return NewComponentError(s.name, "Delete",
				fmt.Sprintf("failed to check etag for key %q", key), err)
		}
		if current != nil && *current != etag.Value {
			return &ETagMismatchError{Key: key, Expected: etag, Actual: &ETag{Value: *current}}
		}
	}
	if _, err := s.client.Delete(ctx, key); err != nil {
		return NewComponentError(s.name, "Delete",
			fmt.Sprintf("failed to delete key %q from Redis", key), err)
	}
	// ETag キーも合わせて削除する（エラーは無視する）。
	_, _ = s.client.Delete(ctx, s.etagKey(key))
	return nil
}

// BulkGet は複数のキーに対して Redis から状態エントリをまとめて取得する。
// いずれか一つでも取得に失敗した場合は即座にエラーを返す。
func (s *RedisStateStore) BulkGet(ctx context.Context, keys []string) ([]*StateEntry, error) {
	results := make([]*StateEntry, 0, len(keys))
	for _, key := range keys {
		entry, err := s.Get(ctx, key)
		if err != nil {
			return nil, fmt.Errorf("bulk get key %q: %w", key, err)
		}
		results = append(results, entry)
	}
	return results, nil
}

// BulkSet は複数の SetRequest を順番に処理して Redis に状態エントリをまとめて保存する。
// いずれか一つでも保存に失敗した場合は即座にエラーを返す。
func (s *RedisStateStore) BulkSet(ctx context.Context, requests []*SetRequest) ([]*ETag, error) {
	etags := make([]*ETag, 0, len(requests))
	for _, req := range requests {
		etag, err := s.Set(ctx, req)
		if err != nil {
			return nil, fmt.Errorf("bulk set key %q: %w", req.Key, err)
		}
		etags = append(etags, etag)
	}
	return etags, nil
}
