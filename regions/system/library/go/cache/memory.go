package cache

import (
	"context"
	"sync"
	"time"
)

type entry struct {
	value     string
	expiresAt *time.Time
}

func (e *entry) isExpired() bool {
	if e.expiresAt == nil {
		return false
	}
	return time.Now().After(*e.expiresAt)
}

// InMemoryCacheClient はメモリ内キャッシュの実装。
// バックグラウンドの sweeper goroutine で期限切れエントリを定期的にクリーンアップする。
type InMemoryCacheClient struct {
	mu    sync.RWMutex
	store map[string]*entry
	// stopSweeper は sweeper goroutine を停止するためのチャネル。
	stopSweeper chan struct{}
}

// デフォルトの sweeper 実行間隔（1分）。
const defaultSweepInterval = 1 * time.Minute

// NewInMemoryCacheClient は新しい InMemoryCacheClient を生成し、
// デフォルト間隔（1分）で期限切れエントリの sweeper を起動する。
func NewInMemoryCacheClient() *InMemoryCacheClient {
	return NewInMemoryCacheClientWithSweep(defaultSweepInterval)
}

// NewInMemoryCacheClientWithSweep は指定間隔で sweeper を起動する InMemoryCacheClient を生成する。
// interval が 0 以下の場合は sweeper を起動しない。
func NewInMemoryCacheClientWithSweep(interval time.Duration) *InMemoryCacheClient {
	c := &InMemoryCacheClient{
		store:       make(map[string]*entry),
		stopSweeper: make(chan struct{}),
	}
	if interval > 0 {
		go c.runSweeper(interval)
	}
	return c
}

// runSweeper はバックグラウンドで定期的に期限切れエントリを削除する。
func (c *InMemoryCacheClient) runSweeper(interval time.Duration) {
	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			c.sweep()
		case <-c.stopSweeper:
			return
		}
	}
}

// sweep は期限切れエントリをすべて削除する。
func (c *InMemoryCacheClient) sweep() {
	c.mu.Lock()
	defer c.mu.Unlock()

	for key, e := range c.store {
		if e.isExpired() {
			delete(c.store, key)
		}
	}
}

// Close は sweeper goroutine を停止する。
func (c *InMemoryCacheClient) Close() {
	close(c.stopSweeper)
}

func (c *InMemoryCacheClient) Get(_ context.Context, key string) (*string, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()

	e, ok := c.store[key]
	if !ok || e.isExpired() {
		return nil, nil
	}
	val := e.value
	return &val, nil
}

func (c *InMemoryCacheClient) Set(_ context.Context, key string, value string, ttl *time.Duration) error {
	c.mu.Lock()
	defer c.mu.Unlock()

	e := &entry{value: value}
	if ttl != nil {
		exp := time.Now().Add(*ttl)
		e.expiresAt = &exp
	}
	c.store[key] = e
	return nil
}

func (c *InMemoryCacheClient) Delete(_ context.Context, key string) (bool, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	_, ok := c.store[key]
	if ok {
		delete(c.store, key)
		return true, nil
	}
	return false, nil
}

func (c *InMemoryCacheClient) Exists(_ context.Context, key string) (bool, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()

	e, ok := c.store[key]
	if !ok || e.isExpired() {
		return false, nil
	}
	return true, nil
}

func (c *InMemoryCacheClient) SetNX(_ context.Context, key string, value string, ttl time.Duration) (bool, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	existing, ok := c.store[key]
	if ok && !existing.isExpired() {
		return false, nil
	}
	exp := time.Now().Add(ttl)
	c.store[key] = &entry{value: value, expiresAt: &exp}
	return true, nil
}

func (c *InMemoryCacheClient) Expire(_ context.Context, key string, ttl time.Duration) (bool, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	e, ok := c.store[key]
	if !ok || e.isExpired() {
		return false, nil
	}
	exp := time.Now().Add(ttl)
	e.expiresAt = &exp
	return true, nil
}
