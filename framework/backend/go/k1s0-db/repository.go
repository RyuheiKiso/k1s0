package k1s0db

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"

	k1s0error "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error"
)

// BaseRepository provides common database operations.
type BaseRepository struct {
	pool      Pool
	tableName string
}

// NewBaseRepository creates a new BaseRepository.
func NewBaseRepository(pool Pool, tableName string) *BaseRepository {
	return &BaseRepository{
		pool:      pool,
		tableName: tableName,
	}
}

// Pool returns the connection pool.
func (r *BaseRepository) Pool() Pool {
	return r.pool
}

// TableName returns the table name.
func (r *BaseRepository) TableName() string {
	return r.tableName
}

// Querier returns the querier from context (transaction or pool).
func (r *BaseRepository) Querier(ctx context.Context) Querier {
	return QuerierFromContext(ctx, r.pool)
}

// Exec executes a query that doesn't return rows.
func (r *BaseRepository) Exec(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
	return r.Querier(ctx).Exec(ctx, sql, args...)
}

// Query executes a query that returns rows.
func (r *BaseRepository) Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error) {
	return r.Querier(ctx).Query(ctx, sql, args...)
}

// QueryRow executes a query that returns at most one row.
func (r *BaseRepository) QueryRow(ctx context.Context, sql string, args ...any) pgx.Row {
	return r.Querier(ctx).QueryRow(ctx, sql, args...)
}

// ExistsBy checks if a record exists matching the given condition.
func (r *BaseRepository) ExistsBy(ctx context.Context, condition string, args ...any) (bool, error) {
	sql := fmt.Sprintf("SELECT EXISTS(SELECT 1 FROM %s WHERE %s)", r.tableName, condition)
	var exists bool
	err := r.QueryRow(ctx, sql, args...).Scan(&exists)
	if err != nil {
		return false, fmt.Errorf("failed to check existence: %w", err)
	}
	return exists, nil
}

// CountBy counts records matching the given condition.
func (r *BaseRepository) CountBy(ctx context.Context, condition string, args ...any) (int64, error) {
	sql := fmt.Sprintf("SELECT COUNT(*) FROM %s WHERE %s", r.tableName, condition)
	var count int64
	err := r.QueryRow(ctx, sql, args...).Scan(&count)
	if err != nil {
		return 0, fmt.Errorf("failed to count: %w", err)
	}
	return count, nil
}

// Count counts all records in the table.
func (r *BaseRepository) Count(ctx context.Context) (int64, error) {
	sql := fmt.Sprintf("SELECT COUNT(*) FROM %s", r.tableName)
	var count int64
	err := r.QueryRow(ctx, sql).Scan(&count)
	if err != nil {
		return 0, fmt.Errorf("failed to count: %w", err)
	}
	return count, nil
}

// DeleteBy deletes records matching the given condition.
func (r *BaseRepository) DeleteBy(ctx context.Context, condition string, args ...any) (int64, error) {
	sql := fmt.Sprintf("DELETE FROM %s WHERE %s", r.tableName, condition)
	tag, err := r.Exec(ctx, sql, args...)
	if err != nil {
		return 0, fmt.Errorf("failed to delete: %w", err)
	}
	return tag.RowsAffected(), nil
}

// ConvertError converts database errors to domain errors.
func ConvertError(err error, resourceType, resourceID string) error {
	if err == nil {
		return nil
	}

	// Check for pgx no rows error
	if errors.Is(err, pgx.ErrNoRows) {
		return k1s0error.NotFound(resourceType, resourceID)
	}

	// Check for PostgreSQL errors
	var pgErr *pgconn.PgError
	if errors.As(err, &pgErr) {
		return convertPgError(pgErr, resourceType, resourceID)
	}

	// Default to internal error
	return k1s0error.Internal(err.Error())
}

// convertPgError converts PostgreSQL errors to domain errors.
func convertPgError(err *pgconn.PgError, resourceType, resourceID string) error {
	// See: https://www.postgresql.org/docs/current/errcodes-appendix.html
	switch err.Code {
	case "23505": // unique_violation
		return k1s0error.Duplicate(resourceType, extractConstraintField(err.ConstraintName))
	case "23503": // foreign_key_violation
		return k1s0error.Conflict(fmt.Sprintf("foreign key constraint violation: %s", err.ConstraintName))
	case "23502": // not_null_violation
		return k1s0error.InvalidInput(fmt.Sprintf("null value in column '%s'", err.ColumnName))
	case "23514": // check_violation
		return k1s0error.InvalidInput(fmt.Sprintf("check constraint violation: %s", err.ConstraintName))
	case "40001": // serialization_failure
		return k1s0error.Transient("concurrent modification detected, please retry")
	case "40P01": // deadlock_detected
		return k1s0error.Transient("deadlock detected, please retry")
	case "53300": // too_many_connections
		return k1s0error.DependencyFailure("database", "too many connections")
	case "57P01": // admin_shutdown
		return k1s0error.DependencyFailure("database", "server is shutting down")
	case "57P02": // crash_shutdown
		return k1s0error.DependencyFailure("database", "server crashed")
	case "57P03": // cannot_connect_now
		return k1s0error.DependencyFailure("database", "cannot connect now")
	default:
		return k1s0error.Internal(err.Message)
	}
}

// extractConstraintField extracts the field name from constraint name.
// Assumes constraint names follow pattern: {table}_{field}_key or {table}_{field}_fkey
func extractConstraintField(constraintName string) string {
	parts := strings.Split(constraintName, "_")
	if len(parts) >= 2 {
		// Remove the table prefix and suffix
		return strings.Join(parts[1:len(parts)-1], "_")
	}
	return constraintName
}

// Scanner is a function that scans a row into a value.
type Scanner[T any] func(pgx.Row) (T, error)

// CollectRows collects rows into a slice.
func CollectRows[T any](rows pgx.Rows, scanner func(pgx.CollectableRow) (T, error)) ([]T, error) {
	return pgx.CollectRows(rows, scanner)
}

// CollectOneRow collects a single row.
func CollectOneRow[T any](rows pgx.Rows, scanner func(pgx.CollectableRow) (T, error)) (T, error) {
	return pgx.CollectExactlyOneRow(rows, scanner)
}

// ScanRow scans a single value from a row.
func ScanRow[T any](row pgx.Row) (T, error) {
	var value T
	err := row.Scan(&value)
	return value, err
}

// Pagination holds pagination parameters.
type Pagination struct {
	// Limit is the maximum number of records to return.
	Limit int

	// Offset is the number of records to skip.
	Offset int
}

// NewPagination creates a new Pagination with defaults.
func NewPagination(limit, offset int) Pagination {
	if limit <= 0 {
		limit = 20
	}
	if limit > 100 {
		limit = 100
	}
	if offset < 0 {
		offset = 0
	}
	return Pagination{
		Limit:  limit,
		Offset: offset,
	}
}

// FromPage creates pagination from page number and page size.
func FromPage(page, pageSize int) Pagination {
	if page < 1 {
		page = 1
	}
	return NewPagination(pageSize, (page-1)*pageSize)
}

// SQL returns the LIMIT and OFFSET clause.
func (p Pagination) SQL() string {
	return fmt.Sprintf("LIMIT %d OFFSET %d", p.Limit, p.Offset)
}

// OrderDirection represents the sort direction.
type OrderDirection string

const (
	// ASC is ascending order.
	ASC OrderDirection = "ASC"

	// DESC is descending order.
	DESC OrderDirection = "DESC"
)

// OrderBy represents an order by clause.
type OrderBy struct {
	Column    string
	Direction OrderDirection
}

// SQL returns the ORDER BY clause.
func (o OrderBy) SQL() string {
	return fmt.Sprintf("%s %s", o.Column, o.Direction)
}

// OrderByList represents a list of order by clauses.
type OrderByList []OrderBy

// SQL returns the ORDER BY clause.
func (l OrderByList) SQL() string {
	if len(l) == 0 {
		return ""
	}
	parts := make([]string, len(l))
	for i, o := range l {
		parts[i] = o.SQL()
	}
	return "ORDER BY " + strings.Join(parts, ", ")
}
