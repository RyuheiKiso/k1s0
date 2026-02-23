package eventstore

import (
	"context"
	"sync"
)

// InMemoryEventStore はメモリ内イベントストアの実装。
type InMemoryEventStore struct {
	mu      sync.RWMutex
	streams map[string][]*EventEnvelope
}

// NewInMemoryEventStore は新しい InMemoryEventStore を生成する。
func NewInMemoryEventStore() *InMemoryEventStore {
	return &InMemoryEventStore{
		streams: make(map[string][]*EventEnvelope),
	}
}

func (s *InMemoryEventStore) Append(_ context.Context, streamID StreamId, events []*EventEnvelope, expectedVersion *uint64) (uint64, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	key := streamID.String()
	stream := s.streams[key]

	var currentVersion uint64
	if len(stream) > 0 {
		currentVersion = stream[len(stream)-1].Version
	}

	if expectedVersion != nil && *expectedVersion != currentVersion {
		return 0, NewVersionConflictError(*expectedVersion, currentVersion)
	}

	version := currentVersion
	for _, event := range events {
		version++
		// イベントのコピーを作成
		copy := *event
		copy.Version = version
		copy.StreamID = key
		stream = append(stream, &copy)
	}
	s.streams[key] = stream
	return version, nil
}

func (s *InMemoryEventStore) Load(_ context.Context, streamID StreamId) ([]*EventEnvelope, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	stream, ok := s.streams[streamID.String()]
	if !ok {
		return []*EventEnvelope{}, nil
	}
	result := make([]*EventEnvelope, len(stream))
	for i, e := range stream {
		copy := *e
		result[i] = &copy
	}
	return result, nil
}

func (s *InMemoryEventStore) LoadFrom(_ context.Context, streamID StreamId, fromVersion uint64) ([]*EventEnvelope, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	stream, ok := s.streams[streamID.String()]
	if !ok {
		return []*EventEnvelope{}, nil
	}
	var result []*EventEnvelope
	for _, e := range stream {
		if e.Version >= fromVersion {
			copy := *e
			result = append(result, &copy)
		}
	}
	if result == nil {
		return []*EventEnvelope{}, nil
	}
	return result, nil
}

func (s *InMemoryEventStore) Exists(_ context.Context, streamID StreamId) (bool, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	_, ok := s.streams[streamID.String()]
	return ok, nil
}

func (s *InMemoryEventStore) CurrentVersion(_ context.Context, streamID StreamId) (uint64, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	stream, ok := s.streams[streamID.String()]
	if !ok || len(stream) == 0 {
		return 0, nil
	}
	return stream[len(stream)-1].Version, nil
}

// InMemorySnapshotStore はメモリ内スナップショットストアの実装。
type InMemorySnapshotStore struct {
	mu        sync.RWMutex
	snapshots map[string]*Snapshot
}

// NewInMemorySnapshotStore は新しい InMemorySnapshotStore を生成する。
func NewInMemorySnapshotStore() *InMemorySnapshotStore {
	return &InMemorySnapshotStore{
		snapshots: make(map[string]*Snapshot),
	}
}

func (s *InMemorySnapshotStore) SaveSnapshot(_ context.Context, snapshot *Snapshot) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	copy := *snapshot
	s.snapshots[snapshot.StreamID] = &copy
	return nil
}

func (s *InMemorySnapshotStore) LoadSnapshot(_ context.Context, streamID StreamId) (*Snapshot, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	snap, ok := s.snapshots[streamID.String()]
	if !ok {
		return nil, nil
	}
	copy := *snap
	return &copy, nil
}
