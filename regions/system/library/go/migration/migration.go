package migration

import (
	"context"
	"crypto/sha256"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"time"
)

// MigrationRunner defines the interface for running database migrations.
type MigrationRunner interface {
	RunUp(ctx context.Context) (*MigrationReport, error)
	RunDown(ctx context.Context, steps int) (*MigrationReport, error)
	Status(ctx context.Context) ([]*MigrationStatus, error)
	Pending(ctx context.Context) ([]*PendingMigration, error)
}

// MigrationConfig holds the configuration for migration operations.
type MigrationConfig struct {
	MigrationsDir string
	DatabaseURL   string
	TableName     string
	Driver        string
}

// NewMigrationConfig creates a MigrationConfig with default values.
func NewMigrationConfig(migrationsDir, databaseURL string) MigrationConfig {
	return MigrationConfig{
		MigrationsDir: migrationsDir,
		DatabaseURL:   databaseURL,
		TableName:     "_migrations",
		Driver:        "postgres",
	}
}

// MigrationReport contains the result of a migration operation.
type MigrationReport struct {
	AppliedCount int
	Elapsed      time.Duration
	Errors       []error
}

// MigrationStatus represents the status of a single migration.
type MigrationStatus struct {
	Version   string
	Name      string
	AppliedAt *time.Time
	Checksum  string
}

// PendingMigration represents a migration that has not been applied.
type PendingMigration struct {
	Version string
	Name    string
}

// MigrationDirection represents up or down migration.
type MigrationDirection int

const (
	DirectionUp MigrationDirection = iota
	DirectionDown
)

type migrationFile struct {
	Version   string
	Name      string
	Direction MigrationDirection
	Content   string
}

// ParseFilename parses a migration filename into version, name, and direction.
func ParseFilename(filename string) (version, name string, direction MigrationDirection, ok bool) {
	if !strings.HasSuffix(filename, ".sql") {
		return "", "", 0, false
	}
	stem := strings.TrimSuffix(filename, ".sql")

	if strings.HasSuffix(stem, ".up") {
		direction = DirectionUp
		stem = strings.TrimSuffix(stem, ".up")
	} else if strings.HasSuffix(stem, ".down") {
		direction = DirectionDown
		stem = strings.TrimSuffix(stem, ".down")
	} else {
		return "", "", 0, false
	}

	idx := strings.Index(stem, "_")
	if idx <= 0 || idx >= len(stem)-1 {
		return "", "", 0, false
	}

	version = stem[:idx]
	name = stem[idx+1:]
	ok = true
	return
}

// Checksum computes a SHA-256 checksum of the given content.
func Checksum(content string) string {
	h := sha256.Sum256([]byte(content))
	return fmt.Sprintf("%x", h)
}

// InMemoryMigrationRunner implements MigrationRunner using in-memory state.
type InMemoryMigrationRunner struct {
	config         MigrationConfig
	upMigrations   []migrationFile
	downMigrations map[string]migrationFile
	applied        []*MigrationStatus
	mu             sync.Mutex
}

// NewInMemoryRunner creates an InMemoryMigrationRunner that reads migration files from disk.
func NewInMemoryRunner(cfg MigrationConfig) (*InMemoryMigrationRunner, error) {
	info, err := os.Stat(cfg.MigrationsDir)
	if err != nil || !info.IsDir() {
		return nil, fmt.Errorf("directory not found: %s", cfg.MigrationsDir)
	}

	entries, err := os.ReadDir(cfg.MigrationsDir)
	if err != nil {
		return nil, fmt.Errorf("failed to read directory: %w", err)
	}

	var ups []migrationFile
	downs := make(map[string]migrationFile)

	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		version, name, direction, ok := ParseFilename(entry.Name())
		if !ok {
			continue
		}
		content, err := os.ReadFile(filepath.Join(cfg.MigrationsDir, entry.Name()))
		if err != nil {
			return nil, fmt.Errorf("failed to read file %s: %w", entry.Name(), err)
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
			downs[version] = mf
		}
	}

	sort.Slice(ups, func(i, j int) bool {
		return ups[i].Version < ups[j].Version
	})

	return &InMemoryMigrationRunner{
		config:         cfg,
		upMigrations:   ups,
		downMigrations: downs,
		applied:        nil,
	}, nil
}

// NewInMemoryRunnerFromMigrations creates an InMemoryMigrationRunner from provided migration data.
func NewInMemoryRunnerFromMigrations(cfg MigrationConfig, ups []struct{ Version, Name, Content string }, downs []struct{ Version, Name, Content string }) *InMemoryMigrationRunner {
	var upFiles []migrationFile
	downFiles := make(map[string]migrationFile)

	for _, u := range ups {
		upFiles = append(upFiles, migrationFile{
			Version:   u.Version,
			Name:      u.Name,
			Direction: DirectionUp,
			Content:   u.Content,
		})
	}
	for _, d := range downs {
		downFiles[d.Version] = migrationFile{
			Version:   d.Version,
			Name:      d.Name,
			Direction: DirectionDown,
			Content:   d.Content,
		}
	}

	sort.Slice(upFiles, func(i, j int) bool {
		return upFiles[i].Version < upFiles[j].Version
	})

	return &InMemoryMigrationRunner{
		config:         cfg,
		upMigrations:   upFiles,
		downMigrations: downFiles,
		applied:        nil,
	}
}

func (r *InMemoryMigrationRunner) RunUp(_ context.Context) (*MigrationReport, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	start := time.Now()
	appliedSet := make(map[string]bool)
	for _, a := range r.applied {
		appliedSet[a.Version] = true
	}

	count := 0
	for _, mf := range r.upMigrations {
		if appliedSet[mf.Version] {
			continue
		}
		now := time.Now().UTC()
		r.applied = append(r.applied, &MigrationStatus{
			Version:   mf.Version,
			Name:      mf.Name,
			AppliedAt: &now,
			Checksum:  Checksum(mf.Content),
		})
		count++
	}

	return &MigrationReport{
		AppliedCount: count,
		Elapsed:      time.Since(start),
	}, nil
}

func (r *InMemoryMigrationRunner) RunDown(_ context.Context, steps int) (*MigrationReport, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	start := time.Now()
	count := 0

	for i := 0; i < steps; i++ {
		if len(r.applied) == 0 {
			break
		}
		r.applied = r.applied[:len(r.applied)-1]
		count++
	}

	return &MigrationReport{
		AppliedCount: count,
		Elapsed:      time.Since(start),
	}, nil
}

func (r *InMemoryMigrationRunner) Status(_ context.Context) ([]*MigrationStatus, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	appliedMap := make(map[string]*MigrationStatus)
	for _, a := range r.applied {
		appliedMap[a.Version] = a
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

func (r *InMemoryMigrationRunner) Pending(_ context.Context) ([]*PendingMigration, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	appliedSet := make(map[string]bool)
	for _, a := range r.applied {
		appliedSet[a.Version] = true
	}

	var pending []*PendingMigration
	for _, mf := range r.upMigrations {
		if !appliedSet[mf.Version] {
			pending = append(pending, &PendingMigration{
				Version: mf.Version,
				Name:    mf.Name,
			})
		}
	}

	return pending, nil
}
