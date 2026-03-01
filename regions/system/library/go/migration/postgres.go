package migration

import (
	"context"
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"time"

	_ "github.com/lib/pq"
)

const createMigrationsTableSQL = `
CREATE TABLE IF NOT EXISTS _migrations (
    version    TEXT        PRIMARY KEY,
    name       TEXT        NOT NULL,
    checksum   TEXT        NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
`

// PostgresMigrationRunner implements MigrationRunner using PostgreSQL.
type PostgresMigrationRunner struct {
	db             *sql.DB
	config         MigrationConfig
	upMigrations   []migrationFile
	downMigrations map[string]migrationFile
}

// NewPostgresMigrationRunner creates a PostgresMigrationRunner that reads migration files from disk.
func NewPostgresMigrationRunner(db *sql.DB, config MigrationConfig) (*PostgresMigrationRunner, error) {
	r := &PostgresMigrationRunner{
		db:             db,
		config:         config,
		downMigrations: make(map[string]migrationFile),
	}

	if err := r.ensureTable(context.Background()); err != nil {
		return nil, err
	}

	if err := r.loadMigrations(); err != nil {
		return nil, err
	}

	return r, nil
}

func (r *PostgresMigrationRunner) ensureTable(ctx context.Context) error {
	_, err := r.db.ExecContext(ctx, createMigrationsTableSQL)
	if err != nil {
		return fmt.Errorf("failed to create migrations table: %w", err)
	}
	return nil
}

func (r *PostgresMigrationRunner) loadMigrations() error {
	info, err := os.Stat(r.config.MigrationsDir)
	if err != nil || !info.IsDir() {
		return fmt.Errorf("directory not found: %s", r.config.MigrationsDir)
	}

	entries, err := os.ReadDir(r.config.MigrationsDir)
	if err != nil {
		return fmt.Errorf("failed to read directory: %w", err)
	}

	var ups []migrationFile

	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		version, name, direction, ok := ParseFilename(entry.Name())
		if !ok {
			continue
		}
		content, err := os.ReadFile(filepath.Join(r.config.MigrationsDir, entry.Name()))
		if err != nil {
			return fmt.Errorf("failed to read file %s: %w", entry.Name(), err)
		}
		mf := migrationFile{
			Version:   version,
			Name:      name,
			Direction: direction,
			Content:   string(content),
		}
		switch direction {
		case DirectionUp:
			ups = append(ups, mf)
		case DirectionDown:
			r.downMigrations[version] = mf
		}
	}

	sort.Slice(ups, func(i, j int) bool {
		return ups[i].Version < ups[j].Version
	})

	r.upMigrations = ups
	return nil
}

func (r *PostgresMigrationRunner) appliedVersions(ctx context.Context) (map[string]bool, error) {
	rows, err := r.db.QueryContext(ctx, "SELECT version FROM _migrations")
	if err != nil {
		return nil, fmt.Errorf("failed to query applied migrations: %w", err)
	}
	defer rows.Close()

	applied := make(map[string]bool)
	for rows.Next() {
		var version string
		if err := rows.Scan(&version); err != nil {
			return nil, fmt.Errorf("failed to scan version: %w", err)
		}
		applied[version] = true
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("rows error: %w", err)
	}
	return applied, nil
}

// RunUp applies all pending up migrations in version order.
func (r *PostgresMigrationRunner) RunUp(ctx context.Context) (*MigrationReport, error) {
	start := time.Now()

	applied, err := r.appliedVersions(ctx)
	if err != nil {
		return nil, err
	}

	count := 0
	for _, mf := range r.upMigrations {
		if applied[mf.Version] {
			continue
		}

		tx, err := r.db.BeginTx(ctx, nil)
		if err != nil {
			return nil, fmt.Errorf("failed to begin transaction for %s: %w", mf.Version, err)
		}

		if _, err := tx.ExecContext(ctx, mf.Content); err != nil {
			tx.Rollback()
			return nil, fmt.Errorf("failed to execute migration %s: %w", mf.Version, err)
		}

		if _, err := tx.ExecContext(ctx,
			"INSERT INTO _migrations (version, name, checksum) VALUES ($1, $2, $3)",
			mf.Version, mf.Name, Checksum(mf.Content),
		); err != nil {
			tx.Rollback()
			return nil, fmt.Errorf("failed to record migration %s: %w", mf.Version, err)
		}

		if err := tx.Commit(); err != nil {
			return nil, fmt.Errorf("failed to commit migration %s: %w", mf.Version, err)
		}

		count++
	}

	return &MigrationReport{
		AppliedCount: count,
		Elapsed:      time.Since(start),
	}, nil
}

// RunDown rolls back the last N applied migrations.
func (r *PostgresMigrationRunner) RunDown(ctx context.Context, steps int) (*MigrationReport, error) {
	start := time.Now()

	rows, err := r.db.QueryContext(ctx, "SELECT version FROM _migrations ORDER BY version DESC LIMIT $1", steps)
	if err != nil {
		return nil, fmt.Errorf("failed to query applied migrations: %w", err)
	}
	defer rows.Close()

	var versions []string
	for rows.Next() {
		var v string
		if err := rows.Scan(&v); err != nil {
			return nil, fmt.Errorf("failed to scan version: %w", err)
		}
		versions = append(versions, v)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("rows error: %w", err)
	}

	count := 0
	for _, version := range versions {
		mf, ok := r.downMigrations[version]
		if !ok {
			return nil, fmt.Errorf("no down migration found for version %s", version)
		}

		tx, err := r.db.BeginTx(ctx, nil)
		if err != nil {
			return nil, fmt.Errorf("failed to begin transaction for %s: %w", version, err)
		}

		if _, err := tx.ExecContext(ctx, mf.Content); err != nil {
			tx.Rollback()
			return nil, fmt.Errorf("failed to execute down migration %s: %w", version, err)
		}

		if _, err := tx.ExecContext(ctx, "DELETE FROM _migrations WHERE version = $1", version); err != nil {
			tx.Rollback()
			return nil, fmt.Errorf("failed to remove migration record %s: %w", version, err)
		}

		if err := tx.Commit(); err != nil {
			return nil, fmt.Errorf("failed to commit down migration %s: %w", version, err)
		}

		count++
	}

	return &MigrationReport{
		AppliedCount: count,
		Elapsed:      time.Since(start),
	}, nil
}

// Status returns the status of all known migrations.
func (r *PostgresMigrationRunner) Status(ctx context.Context) ([]*MigrationStatus, error) {
	rows, err := r.db.QueryContext(ctx, "SELECT version, name, checksum, applied_at FROM _migrations")
	if err != nil {
		return nil, fmt.Errorf("failed to query applied migrations: %w", err)
	}
	defer rows.Close()

	appliedMap := make(map[string]*MigrationStatus)
	for rows.Next() {
		var s MigrationStatus
		var appliedAt time.Time
		if err := rows.Scan(&s.Version, &s.Name, &s.Checksum, &appliedAt); err != nil {
			return nil, fmt.Errorf("failed to scan migration status: %w", err)
		}
		t := appliedAt.UTC()
		s.AppliedAt = &t
		appliedMap[s.Version] = &s
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("rows error: %w", err)
	}

	var statuses []*MigrationStatus
	for _, mf := range r.upMigrations {
		checksum := Checksum(mf.Content)
		if applied, ok := appliedMap[mf.Version]; ok {
			statuses = append(statuses, &MigrationStatus{
				Version:   mf.Version,
				Name:      mf.Name,
				AppliedAt: applied.AppliedAt,
				Checksum:  checksum,
			})
		} else {
			statuses = append(statuses, &MigrationStatus{
				Version:  mf.Version,
				Name:     mf.Name,
				Checksum: checksum,
			})
		}
	}

	return statuses, nil
}

// Pending returns migrations that have not been applied yet.
func (r *PostgresMigrationRunner) Pending(ctx context.Context) ([]*PendingMigration, error) {
	applied, err := r.appliedVersions(ctx)
	if err != nil {
		return nil, err
	}

	var pending []*PendingMigration
	for _, mf := range r.upMigrations {
		if !applied[mf.Version] {
			pending = append(pending, &PendingMigration{
				Version: mf.Version,
				Name:    mf.Name,
			})
		}
	}

	return pending, nil
}
