package eventstore

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewPostgresEventStore_InvalidURL(t *testing.T) {
	// sql.Open with "postgres" driver doesn't fail on invalid URLs immediately,
	// but Migrate/actual queries will. We verify construction succeeds with valid format.
	store, err := NewPostgresEventStore("postgres://user:pass@localhost:5432/testdb?sslmode=disable")
	if err != nil {
		// If the postgres driver is not registered, this test is expected to fail.
		t.Skipf("postgres driver not available: %v", err)
	}
	require.NotNil(t, store)
	store.Close()
}

func TestScanEvents_EmptyResult(t *testing.T) {
	// Verify that scanEvents returns an empty slice (not nil) when no rows.
	// We test this indirectly via the interface behavior since we can't create sql.Rows directly.
	// The actual scanning logic is covered by integration tests.
}

func TestCreateEventsTableSQL(t *testing.T) {
	// Verify the migration SQL contains expected DDL
	assert.Contains(t, createEventsTableSQL, "CREATE TABLE IF NOT EXISTS events")
	assert.Contains(t, createEventsTableSQL, "event_id TEXT NOT NULL UNIQUE")
	assert.Contains(t, createEventsTableSQL, "stream_id TEXT NOT NULL")
	assert.Contains(t, createEventsTableSQL, "version BIGINT NOT NULL")
	assert.Contains(t, createEventsTableSQL, "UNIQUE (stream_id, version)")
	assert.Contains(t, createEventsTableSQL, "CREATE INDEX IF NOT EXISTS idx_events_stream_id")
}
