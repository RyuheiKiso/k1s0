package k1s0db

import (
	"context"
	"errors"
	"fmt"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

// Tx represents a database transaction.
type Tx interface {
	// Exec executes a query that doesn't return rows.
	Exec(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error)

	// Query executes a query that returns rows.
	Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error)

	// QueryRow executes a query that returns at most one row.
	QueryRow(ctx context.Context, sql string, args ...any) pgx.Row

	// Commit commits the transaction.
	Commit(ctx context.Context) error

	// Rollback rolls back the transaction.
	Rollback(ctx context.Context) error
}

// TxOptions represents transaction options.
type TxOptions struct {
	// IsoLevel is the isolation level.
	IsoLevel pgx.TxIsoLevel

	// AccessMode is the access mode.
	AccessMode pgx.TxAccessMode

	// DeferrableMode is the deferrable mode.
	DeferrableMode pgx.TxDeferrableMode
}

// DefaultTxOptions returns default transaction options.
func DefaultTxOptions() TxOptions {
	return TxOptions{
		IsoLevel:   pgx.ReadCommitted,
		AccessMode: pgx.ReadWrite,
	}
}

// ReadOnlyTxOptions returns transaction options for read-only transactions.
func ReadOnlyTxOptions() TxOptions {
	return TxOptions{
		IsoLevel:   pgx.ReadCommitted,
		AccessMode: pgx.ReadOnly,
	}
}

// SerializableTxOptions returns transaction options for serializable transactions.
func SerializableTxOptions() TxOptions {
	return TxOptions{
		IsoLevel:   pgx.Serializable,
		AccessMode: pgx.ReadWrite,
	}
}

// toPgxOptions converts TxOptions to pgx.TxOptions.
func (o TxOptions) toPgxOptions() pgx.TxOptions {
	return pgx.TxOptions{
		IsoLevel:       o.IsoLevel,
		AccessMode:     o.AccessMode,
		DeferrableMode: o.DeferrableMode,
	}
}

// TxFunc is a function that executes within a transaction.
type TxFunc func(tx Tx) error

// TxManager manages database transactions.
type TxManager struct {
	pool Pool
}

// NewTxManager creates a new transaction manager.
//
// Example:
//
//	txManager := k1s0db.NewTxManager(pool)
//
//	err := txManager.RunInTx(ctx, func(tx k1s0db.Tx) error {
//	    _, err := tx.Exec(ctx, "INSERT INTO users (id, name) VALUES ($1, $2)", id, name)
//	    if err != nil {
//	        return err
//	    }
//	    _, err = tx.Exec(ctx, "INSERT INTO user_profiles (user_id) VALUES ($1)", id)
//	    return err
//	})
func NewTxManager(pool Pool) *TxManager {
	return &TxManager{pool: pool}
}

// RunInTx runs the given function within a transaction.
// If the function returns an error, the transaction is rolled back.
// Otherwise, the transaction is committed.
func (m *TxManager) RunInTx(ctx context.Context, fn TxFunc) error {
	return m.RunInTxWithOptions(ctx, DefaultTxOptions(), fn)
}

// RunInTxWithOptions runs the given function within a transaction with options.
func (m *TxManager) RunInTxWithOptions(ctx context.Context, opts TxOptions, fn TxFunc) error {
	tx, err := m.pool.BeginTx(ctx, opts.toPgxOptions())
	if err != nil {
		return fmt.Errorf("failed to begin transaction: %w", err)
	}

	// Wrap the pgx.Tx in our Tx interface
	wrappedTx := &txWrapper{tx: tx}

	// Execute the function
	err = fn(wrappedTx)
	if err != nil {
		// Rollback on error
		if rbErr := tx.Rollback(ctx); rbErr != nil {
			// If rollback also fails, return both errors
			return fmt.Errorf("transaction failed: %w (rollback also failed: %v)", err, rbErr)
		}
		return err
	}

	// Commit on success
	if err := tx.Commit(ctx); err != nil {
		return fmt.Errorf("failed to commit transaction: %w", err)
	}

	return nil
}

// RunInReadOnlyTx runs the given function within a read-only transaction.
func (m *TxManager) RunInReadOnlyTx(ctx context.Context, fn TxFunc) error {
	return m.RunInTxWithOptions(ctx, ReadOnlyTxOptions(), fn)
}

// RunInSerializableTx runs the given function within a serializable transaction.
func (m *TxManager) RunInSerializableTx(ctx context.Context, fn TxFunc) error {
	return m.RunInTxWithOptions(ctx, SerializableTxOptions(), fn)
}

// txWrapper wraps a pgx.Tx to implement the Tx interface.
type txWrapper struct {
	tx pgx.Tx
}

func (t *txWrapper) Exec(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
	return t.tx.Exec(ctx, sql, args...)
}

func (t *txWrapper) Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error) {
	return t.tx.Query(ctx, sql, args...)
}

func (t *txWrapper) QueryRow(ctx context.Context, sql string, args ...any) pgx.Row {
	return t.tx.QueryRow(ctx, sql, args...)
}

func (t *txWrapper) Commit(ctx context.Context) error {
	return t.tx.Commit(ctx)
}

func (t *txWrapper) Rollback(ctx context.Context) error {
	return t.tx.Rollback(ctx)
}

// txContextKey is the context key for transaction.
type txContextKey struct{}

// ContextWithTx returns a new context with the transaction.
func ContextWithTx(ctx context.Context, tx Tx) context.Context {
	return context.WithValue(ctx, txContextKey{}, tx)
}

// TxFromContext returns the transaction from the context.
// Returns nil if no transaction is found.
func TxFromContext(ctx context.Context) Tx {
	tx, _ := ctx.Value(txContextKey{}).(Tx)
	return tx
}

// HasTx returns true if the context has a transaction.
func HasTx(ctx context.Context) bool {
	return TxFromContext(ctx) != nil
}

// ErrNoActiveTransaction is returned when there is no active transaction.
var ErrNoActiveTransaction = errors.New("no active transaction")

// RequireTx returns the transaction from the context or an error if not found.
func RequireTx(ctx context.Context) (Tx, error) {
	tx := TxFromContext(ctx)
	if tx == nil {
		return nil, ErrNoActiveTransaction
	}
	return tx, nil
}

// Querier represents something that can execute queries (pool or transaction).
type Querier interface {
	Exec(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error)
	Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error)
	QueryRow(ctx context.Context, sql string, args ...any) pgx.Row
}

// QuerierFromContext returns the querier from the context.
// If a transaction is present, it returns the transaction.
// Otherwise, it returns the pool.
func QuerierFromContext(ctx context.Context, pool Pool) Querier {
	if tx := TxFromContext(ctx); tx != nil {
		return tx
	}
	return pool
}
