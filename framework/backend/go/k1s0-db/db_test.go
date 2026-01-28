package k1s0db

import (
	"context"
	"errors"
	"os"
	"testing"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

// =============================================================================
// Config Tests
// =============================================================================

func TestDBConfig_Validate_Success(t *testing.T) {
	config := DefaultDBConfig()
	config.Password = "secret"

	err := config.Validate()
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
}

func TestDBConfig_Validate_MissingHost(t *testing.T) {
	config := DefaultDBConfig()
	config.Host = ""
	config.Password = "secret"

	err := config.Validate()
	if err == nil {
		t.Error("expected error for missing host")
	}
}

func TestDBConfig_Validate_MissingPassword(t *testing.T) {
	config := DefaultDBConfig()

	err := config.Validate()
	if err == nil {
		t.Error("expected error for missing password")
	}
}

func TestDBConfig_Validate_InvalidPort(t *testing.T) {
	config := DefaultDBConfig()
	config.Port = 0
	config.Password = "secret"

	err := config.Validate()
	if err == nil {
		t.Error("expected error for invalid port")
	}
}

func TestDBConfig_GetPassword_Direct(t *testing.T) {
	config := &DBConfig{Password: "direct_secret"}

	password, err := config.GetPassword()
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if password != "direct_secret" {
		t.Errorf("expected 'direct_secret', got '%s'", password)
	}
}

func TestDBConfig_GetPassword_FromFile(t *testing.T) {
	// Create temp file with password
	tmpFile, err := os.CreateTemp("", "password")
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(tmpFile.Name())

	if _, err := tmpFile.WriteString("file_secret\n"); err != nil {
		t.Fatal(err)
	}
	tmpFile.Close()

	config := &DBConfig{PasswordFile: tmpFile.Name()}

	password, err := config.GetPassword()
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if password != "file_secret" {
		t.Errorf("expected 'file_secret', got '%s'", password)
	}
}

func TestDBConfigBuilder(t *testing.T) {
	config, err := NewDBConfigBuilder().
		Host("myhost").
		Port(5433).
		Database("mydb").
		User("myuser").
		Password("mypassword").
		SSLMode("require").
		MaxConns(50).
		MinConns(10).
		Build()

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if config.Host != "myhost" {
		t.Errorf("expected host 'myhost', got '%s'", config.Host)
	}
	if config.Port != 5433 {
		t.Errorf("expected port 5433, got %d", config.Port)
	}
	if config.Pool.MaxConns != 50 {
		t.Errorf("expected max conns 50, got %d", config.Pool.MaxConns)
	}
}

func TestPoolConfig_Validate(t *testing.T) {
	config := &PoolConfig{
		MaxConns:          0,
		MinConns:          -1,
		MaxConnLifetime:   0,
		MaxConnIdleTime:   0,
		HealthCheckPeriod: 0,
	}

	validated := config.Validate()

	if validated.MaxConns != 25 {
		t.Errorf("expected MaxConns 25, got %d", validated.MaxConns)
	}
	if validated.MinConns != 5 {
		t.Errorf("expected MinConns 5, got %d", validated.MinConns)
	}
	if validated.MaxConnLifetime != time.Hour {
		t.Errorf("expected MaxConnLifetime 1h, got %v", validated.MaxConnLifetime)
	}
}

// =============================================================================
// Transaction Context Tests
// =============================================================================

func TestContextWithTx(t *testing.T) {
	ctx := context.Background()

	// Initially no transaction
	if HasTx(ctx) {
		t.Error("expected no transaction in initial context")
	}

	// Add mock transaction
	mockTx := &mockTx{}
	ctxWithTx := ContextWithTx(ctx, mockTx)

	if !HasTx(ctxWithTx) {
		t.Error("expected transaction in context")
	}

	tx := TxFromContext(ctxWithTx)
	if tx != mockTx {
		t.Error("expected to get back the same transaction")
	}
}

func TestRequireTx(t *testing.T) {
	ctx := context.Background()

	_, err := RequireTx(ctx)
	if !errors.Is(err, ErrNoActiveTransaction) {
		t.Error("expected ErrNoActiveTransaction")
	}

	mockTx := &mockTx{}
	ctxWithTx := ContextWithTx(ctx, mockTx)

	tx, err := RequireTx(ctxWithTx)
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if tx != mockTx {
		t.Error("expected to get back the same transaction")
	}
}

// =============================================================================
// Transaction Options Tests
// =============================================================================

func TestTxOptions(t *testing.T) {
	opts := DefaultTxOptions()
	if opts.IsoLevel != pgx.ReadCommitted {
		t.Error("expected ReadCommitted isolation level")
	}
	if opts.AccessMode != pgx.ReadWrite {
		t.Error("expected ReadWrite access mode")
	}

	readOnlyOpts := ReadOnlyTxOptions()
	if readOnlyOpts.AccessMode != pgx.ReadOnly {
		t.Error("expected ReadOnly access mode")
	}

	serializableOpts := SerializableTxOptions()
	if serializableOpts.IsoLevel != pgx.Serializable {
		t.Error("expected Serializable isolation level")
	}
}

// =============================================================================
// Repository Tests
// =============================================================================

func TestPagination(t *testing.T) {
	p := NewPagination(10, 20)
	if p.Limit != 10 {
		t.Errorf("expected limit 10, got %d", p.Limit)
	}
	if p.Offset != 20 {
		t.Errorf("expected offset 20, got %d", p.Offset)
	}

	sql := p.SQL()
	expected := "LIMIT 10 OFFSET 20"
	if sql != expected {
		t.Errorf("expected '%s', got '%s'", expected, sql)
	}
}

func TestPagination_Defaults(t *testing.T) {
	p := NewPagination(-1, -5)
	if p.Limit != 20 {
		t.Errorf("expected default limit 20, got %d", p.Limit)
	}
	if p.Offset != 0 {
		t.Errorf("expected offset 0, got %d", p.Offset)
	}
}

func TestPagination_MaxLimit(t *testing.T) {
	p := NewPagination(1000, 0)
	if p.Limit != 100 {
		t.Errorf("expected max limit 100, got %d", p.Limit)
	}
}

func TestFromPage(t *testing.T) {
	p := FromPage(3, 25)
	if p.Limit != 25 {
		t.Errorf("expected limit 25, got %d", p.Limit)
	}
	if p.Offset != 50 {
		t.Errorf("expected offset 50 (page 3 * size 25 - 25), got %d", p.Offset)
	}
}

func TestFromPage_FirstPage(t *testing.T) {
	p := FromPage(1, 10)
	if p.Offset != 0 {
		t.Errorf("expected offset 0 for first page, got %d", p.Offset)
	}
}

func TestFromPage_InvalidPage(t *testing.T) {
	p := FromPage(0, 10)
	if p.Offset != 0 {
		t.Errorf("expected offset 0 for invalid page, got %d", p.Offset)
	}
}

func TestOrderBy(t *testing.T) {
	order := OrderBy{Column: "created_at", Direction: DESC}
	expected := "created_at DESC"
	if order.SQL() != expected {
		t.Errorf("expected '%s', got '%s'", expected, order.SQL())
	}
}

func TestOrderByList(t *testing.T) {
	list := OrderByList{
		{Column: "created_at", Direction: DESC},
		{Column: "name", Direction: ASC},
	}
	expected := "ORDER BY created_at DESC, name ASC"
	if list.SQL() != expected {
		t.Errorf("expected '%s', got '%s'", expected, list.SQL())
	}
}

func TestOrderByList_Empty(t *testing.T) {
	list := OrderByList{}
	if list.SQL() != "" {
		t.Errorf("expected empty string, got '%s'", list.SQL())
	}
}

// =============================================================================
// Error Conversion Tests
// =============================================================================

func TestConvertError_NoRows(t *testing.T) {
	err := ConvertError(pgx.ErrNoRows, "User", "123")
	if err == nil {
		t.Fatal("expected error")
	}
	if err.Error() != "User '123' not found" {
		t.Errorf("unexpected error message: %s", err.Error())
	}
}

func TestConvertError_Nil(t *testing.T) {
	err := ConvertError(nil, "User", "123")
	if err != nil {
		t.Errorf("expected nil, got %v", err)
	}
}

func TestConvertError_UniqueViolation(t *testing.T) {
	pgErr := &pgconn.PgError{
		Code:           "23505",
		ConstraintName: "users_email_key",
	}
	err := ConvertError(pgErr, "User", "123")
	if err == nil {
		t.Fatal("expected error")
	}
	// Should be a duplicate error
	if err.Error() != "User email already exists" {
		t.Errorf("unexpected error message: %s", err.Error())
	}
}

func TestConvertError_ForeignKeyViolation(t *testing.T) {
	pgErr := &pgconn.PgError{
		Code:           "23503",
		ConstraintName: "orders_user_id_fkey",
	}
	err := ConvertError(pgErr, "Order", "456")
	if err == nil {
		t.Fatal("expected error")
	}
}

func TestConvertError_DeadlockDetected(t *testing.T) {
	pgErr := &pgconn.PgError{
		Code:    "40P01",
		Message: "deadlock detected",
	}
	err := ConvertError(pgErr, "User", "123")
	if err == nil {
		t.Fatal("expected error")
	}
	// Should be a transient error (retryable)
	if err.Error() != "deadlock detected, please retry" {
		t.Errorf("unexpected error message: %s", err.Error())
	}
}

// =============================================================================
// Migration Tests
// =============================================================================

func TestMigration_Checksum(t *testing.T) {
	checksum1 := calculateChecksum("CREATE TABLE users (id INT);")
	checksum2 := calculateChecksum("CREATE TABLE users (id INT);")
	checksum3 := calculateChecksum("CREATE TABLE users (id BIGINT);")

	if checksum1 != checksum2 {
		t.Error("expected same checksum for same content")
	}
	if checksum1 == checksum3 {
		t.Error("expected different checksum for different content")
	}
}

// =============================================================================
// Mock Types for Testing
// =============================================================================

type mockTx struct{}

func (m *mockTx) Exec(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
	return pgconn.NewCommandTag(""), nil
}

func (m *mockTx) Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error) {
	return nil, nil
}

func (m *mockTx) QueryRow(ctx context.Context, sql string, args ...any) pgx.Row {
	return nil
}

func (m *mockTx) Commit(ctx context.Context) error {
	return nil
}

func (m *mockTx) Rollback(ctx context.Context) error {
	return nil
}
