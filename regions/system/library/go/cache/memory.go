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
type InMemoryCacheClient struct {
	mu    sync.RWMutex
	store map[string]*entry
}

// NewInMemoryCacheClient は新しい InMemoryCacheClient を生成する。
func NewInMemoryCacheClient() *InMemoryCacheClient {
	return &InMemoryCacheClient{
		store: make(map[string]*entry),
	}
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
