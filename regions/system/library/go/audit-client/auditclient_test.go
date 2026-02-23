package auditclient_test

import (
	"context"
	"testing"
	"time"

	"github.com/k1s0-platform/system-library-go-audit-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newEvent(id, action string) auditclient.AuditEvent {
	return auditclient.AuditEvent{
		ID:           id,
		TenantID:     "tenant-1",
		ActorID:      "actor-1",
		Action:       action,
		ResourceType: "document",
		ResourceID:   "doc-1",
		Timestamp:    time.Now(),
	}
}

func TestRecord_And_Flush(t *testing.T) {
	c := auditclient.NewBufferedClient()
	ctx := context.Background()

	err := c.Record(ctx, newEvent("1", "create"))
	require.NoError(t, err)
	err = c.Record(ctx, newEvent("2", "update"))
	require.NoError(t, err)

	events, err := c.Flush(ctx)
	require.NoError(t, err)
	assert.Len(t, events, 2)
	assert.Equal(t, "1", events[0].ID)
	assert.Equal(t, "2", events[1].ID)
}

func TestFlush_ClearsBuffer(t *testing.T) {
	c := auditclient.NewBufferedClient()
	ctx := context.Background()

	_ = c.Record(ctx, newEvent("1", "create"))
	_, _ = c.Flush(ctx)

	events, err := c.Flush(ctx)
	require.NoError(t, err)
	assert.Empty(t, events)
}

func TestFlush_EmptyBuffer(t *testing.T) {
	c := auditclient.NewBufferedClient()
	events, err := c.Flush(context.Background())
	require.NoError(t, err)
	assert.Empty(t, events)
}

func TestRecord_PreservesFields(t *testing.T) {
	c := auditclient.NewBufferedClient()
	ctx := context.Background()
	event := auditclient.AuditEvent{
		ID:           "evt-1",
		TenantID:     "t-1",
		ActorID:      "a-1",
		Action:       "delete",
		ResourceType: "file",
		ResourceID:   "f-1",
		Timestamp:    time.Date(2025, 1, 1, 0, 0, 0, 0, time.UTC),
	}
	_ = c.Record(ctx, event)
	events, _ := c.Flush(ctx)
	assert.Equal(t, event, events[0])
}
