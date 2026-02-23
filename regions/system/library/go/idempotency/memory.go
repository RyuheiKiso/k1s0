package idempotency

import (
	"context"
	"sync"
	"time"
)

// InMemoryIdempotencyStore はメモリ内べき等ストアの実装。
type InMemoryIdempotencyStore struct {
	mu   sync.RWMutex
	data map[string]*IdempotencyRecord
}

// NewInMemoryIdempotencyStore は新しい InMemoryIdempotencyStore を生成する。
func NewInMemoryIdempotencyStore() *InMemoryIdempotencyStore {
	return &InMemoryIdempotencyStore{
		data: make(map[string]*IdempotencyRecord),
	}
}

func (s *InMemoryIdempotencyStore) cleanupExpired() {
	for key, record := range s.data {
		if record.IsExpired() {
			delete(s.data, key)
		}
	}
}

func (s *InMemoryIdempotencyStore) Get(_ context.Context, key string) (*IdempotencyRecord, error) {
	s.mu.Lock()
	s.cleanupExpired()
	s.mu.Unlock()

	s.mu.RLock()
	defer s.mu.RUnlock()

	record, ok := s.data[key]
	if !ok {
		return nil, nil
	}
	// レコードのコピーを返す
	copy := *record
	return &copy, nil
}

func (s *InMemoryIdempotencyStore) Insert(_ context.Context, record *IdempotencyRecord) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.cleanupExpired()

	if _, ok := s.data[record.Key]; ok {
		return NewDuplicateError(record.Key)
	}
	// コピーを格納
	copy := *record
	s.data[record.Key] = &copy
	return nil
}

func (s *InMemoryIdempotencyStore) Update(_ context.Context, key string, status IdempotencyStatus, responseBody *string, responseStatus *int) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	record, ok := s.data[key]
	if !ok {
		return NewNotFoundError(key)
	}
	record.Status = status
	record.ResponseBody = responseBody
	record.ResponseStatus = responseStatus
	now := time.Now()
	record.CompletedAt = &now
	return nil
}

func (s *InMemoryIdempotencyStore) Delete(_ context.Context, key string) (bool, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	_, ok := s.data[key]
	if ok {
		delete(s.data, key)
		return true, nil
	}
	return false, nil
}
