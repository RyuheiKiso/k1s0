package eventstore_test

import (
	"context"
	"encoding/json"
	"testing"

	es "github.com/k1s0-platform/system-library-go-eventstore"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestAppendAndLoad(t *testing.T) {
	store := es.NewInMemoryEventStore()
	ctx := context.Background()
	sid := es.NewStreamId("order-123")

	event := es.NewEventEnvelope(sid, 0, "OrderCreated", json.RawMessage(`{"item":"widget"}`))
	version, err := store.Append(ctx, sid, []*es.EventEnvelope{event}, nil)
	require.NoError(t, err)
	assert.Equal(t, uint64(1), version)

	events, err := store.Load(ctx, sid)
	require.NoError(t, err)
	require.Len(t, events, 1)
	assert.Equal(t, "OrderCreated", events[0].EventType)
	assert.Equal(t, uint64(1), events[0].Version)
}

func TestAppend_VersionConflict(t *testing.T) {
	store := es.NewInMemoryEventStore()
	ctx := context.Background()
	sid := es.NewStreamId("order-123")

	event1 := es.NewEventEnvelope(sid, 0, "OrderCreated", json.RawMessage(`{}`))
	_, _ = store.Append(ctx, sid, []*es.EventEnvelope{event1}, nil)

	event2 := es.NewEventEnvelope(sid, 0, "OrderUpdated", json.RawMessage(`{}`))
	wrongVersion := uint64(0) // current is 1
	_, err := store.Append(ctx, sid, []*es.EventEnvelope{event2}, &wrongVersion)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "VERSION_CONFLICT")
}

func TestAppend_WithExpectedVersion(t *testing.T) {
	store := es.NewInMemoryEventStore()
	ctx := context.Background()
	sid := es.NewStreamId("order-123")

	event1 := es.NewEventEnvelope(sid, 0, "OrderCreated", json.RawMessage(`{}`))
	_, _ = store.Append(ctx, sid, []*es.EventEnvelope{event1}, nil)

	event2 := es.NewEventEnvelope(sid, 0, "OrderUpdated", json.RawMessage(`{}`))
	correctVersion := uint64(1)
	version, err := store.Append(ctx, sid, []*es.EventEnvelope{event2}, &correctVersion)
	require.NoError(t, err)
	assert.Equal(t, uint64(2), version)
}

func TestLoadFrom(t *testing.T) {
	store := es.NewInMemoryEventStore()
	ctx := context.Background()
	sid := es.NewStreamId("order-123")

	for i := 0; i < 5; i++ {
		event := es.NewEventEnvelope(sid, 0, "Event", json.RawMessage(`{}`))
		_, _ = store.Append(ctx, sid, []*es.EventEnvelope{event}, nil)
	}

	events, err := store.LoadFrom(ctx, sid, 3)
	require.NoError(t, err)
	assert.Len(t, events, 3)
	assert.Equal(t, uint64(3), events[0].Version)
}

func TestLoad_EmptyStream(t *testing.T) {
	store := es.NewInMemoryEventStore()
	ctx := context.Background()
	sid := es.NewStreamId("nonexistent")

	events, err := store.Load(ctx, sid)
	require.NoError(t, err)
	assert.Empty(t, events)
}

func TestExists(t *testing.T) {
	store := es.NewInMemoryEventStore()
	ctx := context.Background()
	sid := es.NewStreamId("order-123")

	exists, _ := store.Exists(ctx, sid)
	assert.False(t, exists)

	event := es.NewEventEnvelope(sid, 0, "OrderCreated", json.RawMessage(`{}`))
	_, _ = store.Append(ctx, sid, []*es.EventEnvelope{event}, nil)

	exists, _ = store.Exists(ctx, sid)
	assert.True(t, exists)
}

func TestCurrentVersion(t *testing.T) {
	store := es.NewInMemoryEventStore()
	ctx := context.Background()
	sid := es.NewStreamId("order-123")

	version, _ := store.CurrentVersion(ctx, sid)
	assert.Equal(t, uint64(0), version)

	event := es.NewEventEnvelope(sid, 0, "OrderCreated", json.RawMessage(`{}`))
	_, _ = store.Append(ctx, sid, []*es.EventEnvelope{event}, nil)

	version, _ = store.CurrentVersion(ctx, sid)
	assert.Equal(t, uint64(1), version)
}

func TestSnapshot_SaveAndLoad(t *testing.T) {
	store := es.NewInMemorySnapshotStore()
	ctx := context.Background()
	sid := es.NewStreamId("order-123")

	snap := &es.Snapshot{
		StreamID: sid.String(),
		Version:  5,
		State:    json.RawMessage(`{"total":100}`),
	}
	err := store.SaveSnapshot(ctx, snap)
	require.NoError(t, err)

	loaded, err := store.LoadSnapshot(ctx, sid)
	require.NoError(t, err)
	require.NotNil(t, loaded)
	assert.Equal(t, uint64(5), loaded.Version)
	assert.Equal(t, `{"total":100}`, string(loaded.State))
}

func TestSnapshot_LoadNotFound(t *testing.T) {
	store := es.NewInMemorySnapshotStore()
	ctx := context.Background()
	sid := es.NewStreamId("missing")

	loaded, err := store.LoadSnapshot(ctx, sid)
	require.NoError(t, err)
	assert.Nil(t, loaded)
}
