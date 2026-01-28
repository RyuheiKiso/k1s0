package k1s0db

import (
	"context"
	"crypto/sha256"
	"embed"
	"encoding/hex"
	"fmt"
	"io/fs"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

// Migration represents a database migration.
type Migration struct {
	// Version is the migration version (usually timestamp-based).
	Version string

	// Name is the human-readable name.
	Name string

	// UpSQL is the SQL to apply the migration.
	UpSQL string

	// DownSQL is the SQL to rollback the migration.
	DownSQL string

	// Checksum is the SHA256 checksum of the migration.
	Checksum string
}

// MigrationRecord represents a migration record in the database.
type MigrationRecord struct {
	Version   string
	Name      string
	Checksum  string
	AppliedAt time.Time
}

// MigrationRunner runs database migrations.
type MigrationRunner struct {
	pool       Pool
	tableName  string
	migrations []Migration
}

// NewMigrationRunner creates a new MigrationRunner.
//
// Example:
//
//	//go:embed migrations/*.sql
//	var migrationsFS embed.FS
//
//	runner, err := k1s0db.NewMigrationRunner(pool, k1s0db.WithMigrationsFS(migrationsFS, "migrations"))
//	if err != nil {
//	    log.Fatal(err)
//	}
//
//	err = runner.Up(ctx)
func NewMigrationRunner(pool Pool, opts ...MigrationOption) (*MigrationRunner, error) {
	r := &MigrationRunner{
		pool:       pool,
		tableName:  "schema_migrations",
		migrations: make([]Migration, 0),
	}

	for _, opt := range opts {
		if err := opt(r); err != nil {
			return nil, err
		}
	}

	return r, nil
}

// MigrationOption is a function that configures the MigrationRunner.
type MigrationOption func(*MigrationRunner) error

// WithTableName sets the migration table name.
func WithTableName(name string) MigrationOption {
	return func(r *MigrationRunner) error {
		r.tableName = name
		return nil
	}
}

// WithMigrations adds migrations directly.
func WithMigrations(migrations ...Migration) MigrationOption {
	return func(r *MigrationRunner) error {
		for i := range migrations {
			migrations[i].Checksum = calculateChecksum(migrations[i].UpSQL)
		}
		r.migrations = append(r.migrations, migrations...)
		return nil
	}
}

// WithMigrationsFS adds migrations from an embedded filesystem.
// Migration files should be named: {version}_{name}.up.sql and {version}_{name}.down.sql
func WithMigrationsFS(fsys embed.FS, dir string) MigrationOption {
	return func(r *MigrationRunner) error {
		migrations := make(map[string]*Migration)

		err := fs.WalkDir(fsys, dir, func(path string, d fs.DirEntry, err error) error {
			if err != nil {
				return err
			}
			if d.IsDir() {
				return nil
			}

			name := filepath.Base(path)
			if !strings.HasSuffix(name, ".sql") {
				return nil
			}

			// Parse filename: {version}_{name}.{up|down}.sql
			parts := strings.SplitN(name, "_", 2)
			if len(parts) != 2 {
				return nil
			}

			version := parts[0]
			rest := parts[1]

			var isUp bool
			var migrationName string
			if strings.HasSuffix(rest, ".up.sql") {
				isUp = true
				migrationName = strings.TrimSuffix(rest, ".up.sql")
			} else if strings.HasSuffix(rest, ".down.sql") {
				isUp = false
				migrationName = strings.TrimSuffix(rest, ".down.sql")
			} else {
				return nil
			}

			content, err := fs.ReadFile(fsys, path)
			if err != nil {
				return fmt.Errorf("failed to read migration file %s: %w", path, err)
			}

			key := version + "_" + migrationName
			if migrations[key] == nil {
				migrations[key] = &Migration{
					Version: version,
					Name:    migrationName,
				}
			}

			if isUp {
				migrations[key].UpSQL = string(content)
				migrations[key].Checksum = calculateChecksum(string(content))
			} else {
				migrations[key].DownSQL = string(content)
			}

			return nil
		})
		if err != nil {
			return err
		}

		// Convert map to sorted slice
		for _, m := range migrations {
			r.migrations = append(r.migrations, *m)
		}
		sort.Slice(r.migrations, func(i, j int) bool {
			return r.migrations[i].Version < r.migrations[j].Version
		})

		return nil
	}
}

// calculateChecksum calculates SHA256 checksum of content.
func calculateChecksum(content string) string {
	hash := sha256.Sum256([]byte(content))
	return hex.EncodeToString(hash[:])
}

// Init initializes the migration table.
func (r *MigrationRunner) Init(ctx context.Context) error {
	sql := fmt.Sprintf(`
		CREATE TABLE IF NOT EXISTS %s (
			version VARCHAR(255) PRIMARY KEY,
			name VARCHAR(255) NOT NULL,
			checksum VARCHAR(64) NOT NULL,
			applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
		)
	`, r.tableName)

	_, err := r.pool.Exec(ctx, sql)
	if err != nil {
		return fmt.Errorf("failed to create migration table: %w", err)
	}

	return nil
}

// Applied returns the list of applied migrations.
func (r *MigrationRunner) Applied(ctx context.Context) ([]MigrationRecord, error) {
	if err := r.Init(ctx); err != nil {
		return nil, err
	}

	sql := fmt.Sprintf(`
		SELECT version, name, checksum, applied_at
		FROM %s
		ORDER BY version ASC
	`, r.tableName)

	rows, err := r.pool.Query(ctx, sql)
	if err != nil {
		return nil, fmt.Errorf("failed to query applied migrations: %w", err)
	}
	defer rows.Close()

	var records []MigrationRecord
	for rows.Next() {
		var record MigrationRecord
		if err := rows.Scan(&record.Version, &record.Name, &record.Checksum, &record.AppliedAt); err != nil {
			return nil, fmt.Errorf("failed to scan migration record: %w", err)
		}
		records = append(records, record)
	}

	return records, nil
}

// Pending returns the list of pending migrations.
func (r *MigrationRunner) Pending(ctx context.Context) ([]Migration, error) {
	applied, err := r.Applied(ctx)
	if err != nil {
		return nil, err
	}

	appliedVersions := make(map[string]bool)
	for _, m := range applied {
		appliedVersions[m.Version] = true
	}

	var pending []Migration
	for _, m := range r.migrations {
		if !appliedVersions[m.Version] {
			pending = append(pending, m)
		}
	}

	return pending, nil
}

// Up applies all pending migrations.
func (r *MigrationRunner) Up(ctx context.Context) error {
	pending, err := r.Pending(ctx)
	if err != nil {
		return err
	}

	txManager := NewTxManager(r.pool)

	for _, m := range pending {
		err := txManager.RunInTx(ctx, func(tx Tx) error {
			// Apply migration
			if _, err := tx.Exec(ctx, m.UpSQL); err != nil {
				return fmt.Errorf("failed to apply migration %s: %w", m.Version, err)
			}

			// Record migration
			sql := fmt.Sprintf(`
				INSERT INTO %s (version, name, checksum, applied_at)
				VALUES ($1, $2, $3, $4)
			`, r.tableName)

			if _, err := tx.Exec(ctx, sql, m.Version, m.Name, m.Checksum, time.Now()); err != nil {
				return fmt.Errorf("failed to record migration %s: %w", m.Version, err)
			}

			return nil
		})
		if err != nil {
			return err
		}
	}

	return nil
}

// UpTo applies migrations up to and including the given version.
func (r *MigrationRunner) UpTo(ctx context.Context, targetVersion string) error {
	pending, err := r.Pending(ctx)
	if err != nil {
		return err
	}

	txManager := NewTxManager(r.pool)

	for _, m := range pending {
		if m.Version > targetVersion {
			break
		}

		err := txManager.RunInTx(ctx, func(tx Tx) error {
			if _, err := tx.Exec(ctx, m.UpSQL); err != nil {
				return fmt.Errorf("failed to apply migration %s: %w", m.Version, err)
			}

			sql := fmt.Sprintf(`
				INSERT INTO %s (version, name, checksum, applied_at)
				VALUES ($1, $2, $3, $4)
			`, r.tableName)

			if _, err := tx.Exec(ctx, sql, m.Version, m.Name, m.Checksum, time.Now()); err != nil {
				return fmt.Errorf("failed to record migration %s: %w", m.Version, err)
			}

			return nil
		})
		if err != nil {
			return err
		}
	}

	return nil
}

// Down rolls back the last applied migration.
func (r *MigrationRunner) Down(ctx context.Context) error {
	applied, err := r.Applied(ctx)
	if err != nil {
		return err
	}

	if len(applied) == 0 {
		return nil
	}

	last := applied[len(applied)-1]

	// Find the migration
	var migration *Migration
	for i := range r.migrations {
		if r.migrations[i].Version == last.Version {
			migration = &r.migrations[i]
			break
		}
	}

	if migration == nil {
		return fmt.Errorf("migration %s not found in source", last.Version)
	}

	if migration.DownSQL == "" {
		return fmt.Errorf("migration %s has no down SQL", last.Version)
	}

	txManager := NewTxManager(r.pool)

	return txManager.RunInTx(ctx, func(tx Tx) error {
		if _, err := tx.Exec(ctx, migration.DownSQL); err != nil {
			return fmt.Errorf("failed to rollback migration %s: %w", migration.Version, err)
		}

		sql := fmt.Sprintf("DELETE FROM %s WHERE version = $1", r.tableName)
		if _, err := tx.Exec(ctx, sql, migration.Version); err != nil {
			return fmt.Errorf("failed to delete migration record %s: %w", migration.Version, err)
		}

		return nil
	})
}

// DownTo rolls back migrations down to (but not including) the given version.
func (r *MigrationRunner) DownTo(ctx context.Context, targetVersion string) error {
	for {
		applied, err := r.Applied(ctx)
		if err != nil {
			return err
		}

		if len(applied) == 0 {
			break
		}

		last := applied[len(applied)-1]
		if last.Version <= targetVersion {
			break
		}

		if err := r.Down(ctx); err != nil {
			return err
		}
	}

	return nil
}

// Status returns the current migration status.
func (r *MigrationRunner) Status(ctx context.Context) (*MigrationStatus, error) {
	applied, err := r.Applied(ctx)
	if err != nil {
		return nil, err
	}

	pending, err := r.Pending(ctx)
	if err != nil {
		return nil, err
	}

	var currentVersion string
	if len(applied) > 0 {
		currentVersion = applied[len(applied)-1].Version
	}

	return &MigrationStatus{
		CurrentVersion: currentVersion,
		Applied:        applied,
		Pending:        pending,
	}, nil
}

// MigrationStatus holds the current migration status.
type MigrationStatus struct {
	CurrentVersion string
	Applied        []MigrationRecord
	Pending        []Migration
}

// Verify checks that applied migrations match their checksums.
func (r *MigrationRunner) Verify(ctx context.Context) error {
	applied, err := r.Applied(ctx)
	if err != nil {
		return err
	}

	// Build migration map
	migrationMap := make(map[string]Migration)
	for _, m := range r.migrations {
		migrationMap[m.Version] = m
	}

	for _, record := range applied {
		migration, ok := migrationMap[record.Version]
		if !ok {
			return fmt.Errorf("applied migration %s not found in source", record.Version)
		}
		if migration.Checksum != record.Checksum {
			return fmt.Errorf("checksum mismatch for migration %s: expected %s, got %s",
				record.Version, migration.Checksum, record.Checksum)
		}
	}

	return nil
}
