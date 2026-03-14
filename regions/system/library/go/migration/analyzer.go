package migration

import (
	"fmt"
	"regexp"
	"strings"
)

// BreakingChangeType represents the type of a breaking change.
type BreakingChangeType int

const (
	ColumnDropped BreakingChangeType = iota
	ColumnTypeChanged
	TableDropped
	NotNullAdded
	ColumnRenamed
)

// BreakingChange represents a detected breaking change in a migration.
type BreakingChange struct {
	Type   BreakingChangeType
	Table  string
	Column string
	From   string // for type change/rename
	To     string // for type change/rename
}

// String returns a human-readable description of the breaking change.
func (b BreakingChange) String() string {
	switch b.Type {
	case ColumnDropped:
		return fmt.Sprintf("Column %s.%s dropped", b.Table, b.Column)
	case ColumnTypeChanged:
		return fmt.Sprintf("Column %s.%s type changed from %s to %s", b.Table, b.Column, b.From, b.To)
	case TableDropped:
		return fmt.Sprintf("Table %s dropped", b.Table)
	case NotNullAdded:
		return fmt.Sprintf("NOT NULL added to %s.%s", b.Table, b.Column)
	case ColumnRenamed:
		return fmt.Sprintf("Column %s.%s renamed to %s", b.Table, b.From, b.To)
	default:
		return "unknown breaking change"
	}
}

var (
	reDropTable    = regexp.MustCompile(`(?i)^\s*DROP\s+TABLE\s+(?:IF\s+EXISTS\s+)?(\S+)`)
	reDropColumn   = regexp.MustCompile(`(?i)^\s*ALTER\s+TABLE\s+(\S+)\s+DROP\s+COLUMN\s+(\S+)`)
	reSetNotNull   = regexp.MustCompile(`(?i)^\s*ALTER\s+TABLE\s+(\S+)\s+ALTER\s+COLUMN\s+(\S+)\s+SET\s+NOT\s+NULL`)
	reTypeChange   = regexp.MustCompile(`(?i)^\s*ALTER\s+TABLE\s+(\S+)\s+ALTER\s+COLUMN\s+(\S+)\s+(?:SET\s+DATA\s+)?TYPE\s+(\S+)`)
	reRenameColumn = regexp.MustCompile(`(?i)^\s*ALTER\s+TABLE\s+(\S+)\s+RENAME\s+COLUMN\s+(\S+)\s+TO\s+(\S+)`)
)

// DetectBreakingChanges analyzes SQL and returns any breaking changes found.
func DetectBreakingChanges(sql string) []BreakingChange {
	if strings.TrimSpace(sql) == "" {
		return nil
	}

	parts := strings.Split(sql, ";")
	var changes []BreakingChange

	for _, part := range parts {
		stmt := strings.TrimSpace(part)
		if stmt == "" {
			continue
		}

		if m := reDropTable.FindStringSubmatch(stmt); m != nil {
			changes = append(changes, BreakingChange{
				Type:  TableDropped,
				Table: m[1],
			})
		} else if m := reDropColumn.FindStringSubmatch(stmt); m != nil {
			changes = append(changes, BreakingChange{
				Type:   ColumnDropped,
				Table:  m[1],
				Column: m[2],
			})
		} else if m := reSetNotNull.FindStringSubmatch(stmt); m != nil {
			changes = append(changes, BreakingChange{
				Type:   NotNullAdded,
				Table:  m[1],
				Column: m[2],
			})
		} else if m := reTypeChange.FindStringSubmatch(stmt); m != nil {
			changes = append(changes, BreakingChange{
				Type:   ColumnTypeChanged,
				Table:  m[1],
				Column: m[2],
				From:   "unknown",
				To:     m[3],
			})
		} else if m := reRenameColumn.FindStringSubmatch(stmt); m != nil {
			changes = append(changes, BreakingChange{
				Type:  ColumnRenamed,
				Table: m[1],
				From:  m[2],
				To:    m[3],
			})
		}
	}

	return changes
}
