// 本ファイルは tier1 facade の idempotency dedup cache。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「冪等性と再試行」:
//       状態変更を伴う API（State.Set / PubSub.Publish / Workflow.Start /
//       Secrets.Rotate / Binding.Send / Audit.Write）は冪等性キー（idempotency_key）を
//       受け付ける。同一キーでの再試行は副作用を重複させず、初回と同じレスポンスを
//       返すことを MUST とする。キー保管は Valkey で最短 24 時間（契約上は 1 時間で
//       十分とし、余裕を持たせる）。
//
// 実装範囲:
//   - 抽象 IdempotencyCache interface（in-memory 実装 + 将来の Valkey backend 想定）
//   - in-memory backend: TTL 付き sync map、24h 既定、idle eviction
//   - GenerateKey: tenant_id + rpc + idempotency_key を結合した安定キー
//   - GetOrCompute: cache hit 時は cached response を、miss 時は fn() 実行してキャッシュ
//
// 適用範囲（リリース時点 でリリース）:
//   - PubSub.Publish の handler 内 dedup（proto に既に idempotency_key がある）
//   - Secrets.Rotate の handler 内 dedup（同上）
//
// 残他 4 API（State.Set / Workflow.Start / Binding.Invoke / Audit.Record）の
// idempotency_key は proto 拡張 + SDK 4 言語再生成が前提のため別 PR。

package common

import (
	"context"
	"sync"
	"time"
)

// IdempotencyCache は (key) → (response) を TTL 付きで保持する抽象。
// production の multi-replica deploy では Valkey backend で共有する想定だが、
// release-initial では in-memory backend で 1 Pod 内 dedup を提供する。
type IdempotencyCache interface {
	// GetOrCompute は key の cache hit なら cached response を返し、miss なら
	// fn() を実行して結果を cache に保存しつつ返す。
	// fn が error を返した場合は cache せず error を伝搬する（再試行可能を保つ）。
	GetOrCompute(ctx context.Context, key string, fn func() (interface{}, error)) (interface{}, error)
}

// IdempotencyKey は tenant_id / rpc / client が渡した idempotency_key を結合して
// cache 衝突しない安定キーを生成する。空 idempotency_key は dedup 対象外を意味する
// ため空文字を返す（呼出側で「dedup しない」分岐に倒す）。
func IdempotencyKey(tenantID, rpc, clientKey string) string {
	if clientKey == "" {
		return ""
	}
	// 区切り文字に "|" を採用（tenant_id / rpc には UTF-8 任意文字が来うるが衝突は
	// 実用上無視できる。production で厳密性が必要なら HMAC 等に拡張可能）。
	return tenantID + "|" + rpc + "|" + clientKey
}

// inMemoryIdempotencyCache は sync.Map ベースの in-memory backend。
// production は Valkey backed cache に置き換える想定。
type inMemoryIdempotencyCache struct {
	ttl     time.Duration
	entries sync.Map // map[string]*idempotencyEntry
	// 同一 key への並行呼出を直列化する singleflight-like ロック群。
	locks sync.Map // map[string]*sync.Mutex
}

type idempotencyEntry struct {
	response interface{}
	expiresAt time.Time
}

// NewInMemoryIdempotencyCache は in-memory cache を生成する。
// ttl が 0 / 負値なら docs 既定の 24 時間を使う。
func NewInMemoryIdempotencyCache(ttl time.Duration) IdempotencyCache {
	if ttl <= 0 {
		ttl = 24 * time.Hour
	}
	c := &inMemoryIdempotencyCache{ttl: ttl}
	// idle entry の eviction goroutine（docs §「キー保管」最短 24 時間）。
	go c.evictionLoop()
	return c
}

// evictionLoop は 1/10 TTL ごとに expired entry を sweep する。
// process 終了で停止（goroutine リーク無し）。
func (c *inMemoryIdempotencyCache) evictionLoop() {
	interval := c.ttl / 10
	if interval < time.Minute {
		interval = time.Minute
	}
	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	for now := range ticker.C {
		c.entries.Range(func(k, v interface{}) bool {
			if e, ok := v.(*idempotencyEntry); ok && now.After(e.expiresAt) {
				c.entries.Delete(k)
				c.locks.Delete(k)
			}
			return true
		})
	}
}

// keyMutex は key 単位の lock（cache miss 時の並行 fn 実行を直列化する）。
func (c *inMemoryIdempotencyCache) keyMutex(key string) *sync.Mutex {
	if v, ok := c.locks.Load(key); ok {
		return v.(*sync.Mutex)
	}
	mu := &sync.Mutex{}
	actual, _ := c.locks.LoadOrStore(key, mu)
	return actual.(*sync.Mutex)
}

// GetOrCompute は cache hit なら即座に cached response を返す。
// miss + fn() success なら結果を cache し、TTL 付きで保持する。
// miss + fn() error なら cache せず error を伝搬（次回呼出時に再 fn 実行を許容）。
func (c *inMemoryIdempotencyCache) GetOrCompute(_ context.Context, key string, fn func() (interface{}, error)) (interface{}, error) {
	// 高速パス: cache hit で expired してないもの。
	if v, ok := c.entries.Load(key); ok {
		e := v.(*idempotencyEntry)
		if time.Now().Before(e.expiresAt) {
			return e.response, nil
		}
	}
	// slow path: 同一 key の並行実行を直列化する（同 idempotency_key で 2 回同時に
	// 来た場合、両方が fn を実行して副作用が発生するのを防ぐ）。
	mu := c.keyMutex(key)
	mu.Lock()
	defer mu.Unlock()
	// double-check（lock 取得中に他 goroutine が完了した場合）。
	if v, ok := c.entries.Load(key); ok {
		e := v.(*idempotencyEntry)
		if time.Now().Before(e.expiresAt) {
			return e.response, nil
		}
	}
	resp, err := fn()
	if err != nil {
		// error は cache しない（次回再試行で正常結果を得られる可能性を残す）。
		return nil, err
	}
	c.entries.Store(key, &idempotencyEntry{
		response:  resp,
		expiresAt: time.Now().Add(c.ttl),
	})
	return resp, nil
}
