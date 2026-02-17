package persistence

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/jmoiron/sqlx"
	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/domain/repository"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newTestDB(t *testing.T) (*DB, sqlmock.Sqlmock) {
	t.Helper()
	mockDB, mock, err := sqlmock.New()
	require.NoError(t, err)
	t.Cleanup(func() { mockDB.Close() })

	db := sqlx.NewDb(mockDB, "postgres")
	return &DB{conn: db}, mock
}

func TestCreate_Success(t *testing.T) {
	db, mock := newTestDB(t)
	repo := NewAuditLogRepository(db)

	log := &model.AuditLog{
		ID:         "550e8400-e29b-41d4-a716-446655440000",
		EventType:  "LOGIN_SUCCESS",
		UserID:     "user-1",
		IPAddress:  "127.0.0.1",
		UserAgent:  "Mozilla/5.0",
		Resource:   "/api/v1/auth/token",
		Action:     "POST",
		Result:     "SUCCESS",
		Metadata:   map[string]string{"client_id": "react-spa"},
		RecordedAt: time.Date(2026, 2, 17, 10, 0, 0, 0, time.UTC),
	}

	metadataJSON, _ := json.Marshal(log.Metadata)

	mock.ExpectExec(`INSERT INTO audit_logs`).
		WithArgs(log.ID, log.EventType, log.UserID, log.IPAddress, log.UserAgent, log.Resource, log.Action, log.Result, metadataJSON, log.RecordedAt).
		WillReturnResult(sqlmock.NewResult(0, 1))

	err := repo.Create(context.Background(), log)
	assert.NoError(t, err)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearch_ByUserID(t *testing.T) {
	db, mock := newTestDB(t)
	repo := NewAuditLogRepository(db)

	now := time.Date(2026, 2, 17, 10, 0, 0, 0, time.UTC)
	metadataJSON := []byte(`{}`)

	// count クエリ
	countRows := sqlmock.NewRows([]string{"count"}).AddRow(1)
	mock.ExpectQuery(`SELECT COUNT\(\*\) FROM audit_logs WHERE user_id = \$1`).
		WithArgs("user-1").
		WillReturnRows(countRows)

	// data クエリ
	dataRows := sqlmock.NewRows([]string{"id", "event_type", "user_id", "ip_address", "user_agent", "resource", "action", "result", "metadata", "recorded_at"}).
		AddRow("550e8400-e29b-41d4-a716-446655440000", "LOGIN_SUCCESS", "user-1", "127.0.0.1", "test", "/api/v1/auth/token", "POST", "SUCCESS", metadataJSON, now)

	mock.ExpectQuery(`SELECT .+ FROM audit_logs WHERE user_id = \$1 ORDER BY recorded_at DESC`).
		WithArgs("user-1", 20, 0).
		WillReturnRows(dataRows)

	params := repository.AuditLogSearchParams{
		UserID:   "user-1",
		Page:     1,
		PageSize: 20,
	}

	logs, total, err := repo.Search(context.Background(), params)
	require.NoError(t, err)
	assert.Equal(t, 1, total)
	assert.Len(t, logs, 1)
	assert.Equal(t, "user-1", logs[0].UserID)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearch_ByEventType(t *testing.T) {
	db, mock := newTestDB(t)
	repo := NewAuditLogRepository(db)

	now := time.Date(2026, 2, 17, 10, 0, 0, 0, time.UTC)
	metadataJSON := []byte(`{}`)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(2)
	mock.ExpectQuery(`SELECT COUNT\(\*\) FROM audit_logs WHERE event_type = \$1`).
		WithArgs("TOKEN_VALIDATE").
		WillReturnRows(countRows)

	dataRows := sqlmock.NewRows([]string{"id", "event_type", "user_id", "ip_address", "user_agent", "resource", "action", "result", "metadata", "recorded_at"}).
		AddRow("id-1", "TOKEN_VALIDATE", "user-1", "10.0.0.1", "", "/api/v1/auth/token/validate", "POST", "SUCCESS", metadataJSON, now).
		AddRow("id-2", "TOKEN_VALIDATE", "user-2", "10.0.0.2", "", "/api/v1/auth/token/validate", "POST", "SUCCESS", metadataJSON, now)

	mock.ExpectQuery(`SELECT .+ FROM audit_logs WHERE event_type = \$1 ORDER BY recorded_at DESC`).
		WithArgs("TOKEN_VALIDATE", 20, 0).
		WillReturnRows(dataRows)

	params := repository.AuditLogSearchParams{
		EventType: "TOKEN_VALIDATE",
		Page:      1,
		PageSize:  20,
	}

	logs, total, err := repo.Search(context.Background(), params)
	require.NoError(t, err)
	assert.Equal(t, 2, total)
	assert.Len(t, logs, 2)
	assert.Equal(t, "TOKEN_VALIDATE", logs[0].EventType)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearch_ByDateRange(t *testing.T) {
	db, mock := newTestDB(t)
	repo := NewAuditLogRepository(db)

	from := time.Date(2026, 2, 1, 0, 0, 0, 0, time.UTC)
	to := time.Date(2026, 2, 28, 23, 59, 59, 0, time.UTC)
	now := time.Date(2026, 2, 15, 10, 0, 0, 0, time.UTC)
	metadataJSON := []byte(`{}`)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(1)
	mock.ExpectQuery(`SELECT COUNT\(\*\) FROM audit_logs WHERE recorded_at >= \$1 AND recorded_at <= \$2`).
		WithArgs(from, to).
		WillReturnRows(countRows)

	dataRows := sqlmock.NewRows([]string{"id", "event_type", "user_id", "ip_address", "user_agent", "resource", "action", "result", "metadata", "recorded_at"}).
		AddRow("id-1", "LOGIN_SUCCESS", "user-1", "127.0.0.1", "test", "/api/v1/auth/token", "POST", "SUCCESS", metadataJSON, now)

	mock.ExpectQuery(`SELECT .+ FROM audit_logs WHERE recorded_at >= \$1 AND recorded_at <= \$2 ORDER BY recorded_at DESC`).
		WithArgs(from, to, 20, 0).
		WillReturnRows(dataRows)

	params := repository.AuditLogSearchParams{
		From:     &from,
		To:       &to,
		Page:     1,
		PageSize: 20,
	}

	logs, total, err := repo.Search(context.Background(), params)
	require.NoError(t, err)
	assert.Equal(t, 1, total)
	assert.Len(t, logs, 1)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearch_Pagination(t *testing.T) {
	db, mock := newTestDB(t)
	repo := NewAuditLogRepository(db)

	now := time.Date(2026, 2, 17, 10, 0, 0, 0, time.UTC)
	metadataJSON := []byte(`{}`)

	// Page 2, PageSize 10 -> offset = 10
	countRows := sqlmock.NewRows([]string{"count"}).AddRow(25)
	mock.ExpectQuery(`SELECT COUNT\(\*\) FROM audit_logs$`).
		WillReturnRows(countRows)

	dataRows := sqlmock.NewRows([]string{"id", "event_type", "user_id", "ip_address", "user_agent", "resource", "action", "result", "metadata", "recorded_at"}).
		AddRow("id-11", "LOGIN_SUCCESS", "user-1", "127.0.0.1", "test", "/api/v1/auth/token", "POST", "SUCCESS", metadataJSON, now)

	mock.ExpectQuery(`SELECT .+ FROM audit_logs ORDER BY recorded_at DESC`).
		WithArgs(10, 10).
		WillReturnRows(dataRows)

	params := repository.AuditLogSearchParams{
		Page:     2,
		PageSize: 10,
	}

	logs, total, err := repo.Search(context.Background(), params)
	require.NoError(t, err)
	assert.Equal(t, 25, total)
	assert.Len(t, logs, 1)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearch_MultipleFilters(t *testing.T) {
	db, mock := newTestDB(t)
	repo := NewAuditLogRepository(db)

	from := time.Date(2026, 2, 1, 0, 0, 0, 0, time.UTC)
	now := time.Date(2026, 2, 15, 10, 0, 0, 0, time.UTC)
	metadataJSON := []byte(`{"client_id":"react-spa"}`)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(1)
	mock.ExpectQuery(`SELECT COUNT\(\*\) FROM audit_logs WHERE user_id = \$1 AND event_type = \$2 AND recorded_at >= \$3`).
		WithArgs("user-1", "LOGIN_SUCCESS", from).
		WillReturnRows(countRows)

	dataRows := sqlmock.NewRows([]string{"id", "event_type", "user_id", "ip_address", "user_agent", "resource", "action", "result", "metadata", "recorded_at"}).
		AddRow("id-1", "LOGIN_SUCCESS", "user-1", "127.0.0.1", "Mozilla/5.0", "/api/v1/auth/token", "POST", "SUCCESS", metadataJSON, now)

	mock.ExpectQuery(`SELECT .+ FROM audit_logs WHERE user_id = \$1 AND event_type = \$2 AND recorded_at >= \$3 ORDER BY recorded_at DESC`).
		WithArgs("user-1", "LOGIN_SUCCESS", from, 20, 0).
		WillReturnRows(dataRows)

	params := repository.AuditLogSearchParams{
		UserID:    "user-1",
		EventType: "LOGIN_SUCCESS",
		From:      &from,
		Page:      1,
		PageSize:  20,
	}

	logs, total, err := repo.Search(context.Background(), params)
	require.NoError(t, err)
	assert.Equal(t, 1, total)
	assert.Len(t, logs, 1)
	assert.Equal(t, "user-1", logs[0].UserID)
	assert.Equal(t, "LOGIN_SUCCESS", logs[0].EventType)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearch_NoResults(t *testing.T) {
	db, mock := newTestDB(t)
	repo := NewAuditLogRepository(db)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(0)
	mock.ExpectQuery(`SELECT COUNT\(\*\) FROM audit_logs WHERE user_id = \$1`).
		WithArgs("nonexistent-user").
		WillReturnRows(countRows)

	dataRows := sqlmock.NewRows([]string{"id", "event_type", "user_id", "ip_address", "user_agent", "resource", "action", "result", "metadata", "recorded_at"})

	mock.ExpectQuery(`SELECT .+ FROM audit_logs WHERE user_id = \$1 ORDER BY recorded_at DESC`).
		WithArgs("nonexistent-user", 20, 0).
		WillReturnRows(dataRows)

	params := repository.AuditLogSearchParams{
		UserID:   "nonexistent-user",
		Page:     1,
		PageSize: 20,
	}

	logs, total, err := repo.Search(context.Background(), params)
	require.NoError(t, err)
	assert.Equal(t, 0, total)
	assert.Empty(t, logs)
	assert.NoError(t, mock.ExpectationsWereMet())
}
