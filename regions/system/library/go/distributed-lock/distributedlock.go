package distributedlock

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"sync"
	"time"
)

// ErrAlreadyLocked はロック済みエラー。
var ErrAlreadyLocked = errors.New("既にロックされています")

// ErrTokenMismatch はトークン不一致エラー。
var ErrTokenMismatch = errors.New("トークンが一致しません")

// ErrLockNotFound はロック未発見エラー。
var ErrLockNotFound = errors.New("ロックが見つかりません")

// LockGuard はロックのガード。
type LockGuard struct {
	Key   string
	Token string
}

// DistributedLock は分散ロックのインターフェース。
type DistributedLock interface {
	Acquire(ctx context.Context, key string, ttl time.Duration) (*LockGuard, error)
	Release(ctx context.Context, guard *LockGuard) error
	IsLocked(ctx context.Context, key string) (bool, error)
}

type lockEntry struct {
	token     string
	expiresAt time.Time
}

// InMemoryLock はメモリ内の分散ロック実装。
type InMemoryLock struct {
	mu    sync.Mutex
	locks map[string]*lockEntry
}

// NewInMemoryLock は新しい InMemoryLock を生成する。
func NewInMemoryLock() *InMemoryLock {
	return &InMemoryLock{
		locks: make(map[string]*lockEntry),
	}
}

func (l *InMemoryLock) Acquire(_ context.Context, key string, ttl time.Duration) (*LockGuard, error) {
	l.mu.Lock()
	defer l.mu.Unlock()

	if entry, ok := l.locks[key]; ok && time.Now().Before(entry.expiresAt) {
		return nil, ErrAlreadyLocked
	}

	token := generateToken()
	l.locks[key] = &lockEntry{
		token:     token,
		expiresAt: time.Now().Add(ttl),
	}
	return &LockGuard{Key: key, Token: token}, nil
}

func (l *InMemoryLock) Release(_ context.Context, guard *LockGuard) error {
	l.mu.Lock()
	defer l.mu.Unlock()

	entry, ok := l.locks[guard.Key]
	if !ok {
		return ErrLockNotFound
	}
	if entry.token != guard.Token {
		return ErrTokenMismatch
	}
	delete(l.locks, guard.Key)
	return nil
}

func (l *InMemoryLock) IsLocked(_ context.Context, key string) (bool, error) {
	l.mu.Lock()
	defer l.mu.Unlock()

	entry, ok := l.locks[key]
	if !ok || time.Now().After(entry.expiresAt) {
		return false, nil
	}
	return true, nil
}

func generateToken() string {
	b := make([]byte, 16)
	_, _ = rand.Read(b)
	return hex.EncodeToString(b)
}
