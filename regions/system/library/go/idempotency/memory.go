package idempotency

import (
	"context"
	"sync"
	"time"
)

// InMemoryIdempotencyStore はメモリ内ストア実装。
type InMemoryIdempotencyStore struct {
	mu   sync.RWMutex
	data map[string]*IdempotencyRecord
}

// NewInMemoryIdempotencyStore は新規メモリストアを生成する。
func NewInMemoryIdempotencyStore() *InMemoryIdempotencyStore {
	return &InMemoryIdempotencyStore{
		data: make(map[string]*IdempotencyRecord),
	}
}

func (s *InMemoryIdempotencyStore) cleanupExpiredLocked() {
	for key, record := range s.data {
		if record.IsExpired() {
			delete(s.data, key)
		}
	}
}

func (s *InMemoryIdempotencyStore) Get(_ context.Context, key string) (*IdempotencyRecord, error) {
	s.mu.Lock()
	s.cleanupExpiredLocked()
	record, ok := s.data[key]
	if !ok {
		s.mu.Unlock()
		return nil, nil
	}
	copy := *record
	s.mu.Unlock()
	return &copy, nil
}

func (s *InMemoryIdempotencyStore) Set(_ context.Context, key string, record *IdempotencyRecord) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.cleanupExpiredLocked()

	if _, ok := s.data[key]; ok {
		return NewDuplicateError(key)
	}

	copy := *record
	copy.Key = key
	if copy.CreatedAt.IsZero() {
		copy.CreatedAt = time.Now().UTC()
	}
	if copy.Status == "" {
		copy.Status = StatusPending
	}

	s.data[key] = &copy
	return nil
}

func (s *InMemoryIdempotencyStore) MarkCompleted(
	_ context.Context,
	key string,
	response []byte,
	statusCode int,
) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.cleanupExpiredLocked()

	record, ok := s.data[key]
	if !ok {
		return NewNotFoundError(key)
	}

	record.Status = StatusCompleted
	record.Response = append([]byte(nil), response...)
	record.StatusCode = statusCode
	record.Error = ""

	return nil
}

func (s *InMemoryIdempotencyStore) MarkFailed(_ context.Context, key string, err error) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.cleanupExpiredLocked()

	record, ok := s.data[key]
	if !ok {
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

	return nil
}
