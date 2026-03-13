package migration

import (
	"fmt"
	"regexp"
	"strings"
)

var (
	reCreateTable     = regexp.MustCompile(`(?i)^\s*CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(\S+)`)
	reCreateIndex     = regexp.MustCompile(`(?i)^\s*CREATE\s+(?:UNIQUE\s+)?INDEX\s+(?:IF\s+NOT\s+EXISTS\s+)?(\S+)`)
	reAlterAddColumn  = regexp.MustCompile(`(?i)^\s*ALTER\s+TABLE\s+(\S+)\s+ADD\s+COLUMN\s+(\S+)`)
	reAlterAddConstr  = regexp.MustCompile(`(?i)^\s*ALTER\s+TABLE\s+(\S+)\s+ADD\s+CONSTRAINT\s+(\S+)`)
)

// GenerateDownSQL generates DOWN migration SQL from UP SQL.
// Uses regex-based parsing (no external SQL parser dependency).
func GenerateDownSQL(upSQL string) (string, error) {
	if strings.TrimSpace(upSQL) == "" {
		return "", nil
	}

	parts := strings.Split(upSQL, ";")
	var downStatements []string

	for _, part := range parts {
		stmt := strings.TrimSpace(part)
		if stmt == "" {
			continue
		}

		if m := reCreateTable.FindStringSubmatch(stmt); m != nil {
			downStatements = append(downStatements, fmt.Sprintf("DROP TABLE IF EXISTS %s CASCADE;", m[1]))
		} else if m := reCreateIndex.FindStringSubmatch(stmt); m != nil {
			downStatements = append(downStatements, fmt.Sprintf("DROP INDEX IF EXISTS %s;", m[1]))
		} else if m := reAlterAddColumn.FindStringSubmatch(stmt); m != nil {
			downStatements = append(downStatements, fmt.Sprintf("ALTER TABLE %s DROP COLUMN %s;", m[1], m[2]))
		} else if m := reAlterAddConstr.FindStringSubmatch(stmt); m != nil {
			downStatements = append(downStatements, fmt.Sprintf("ALTER TABLE %s DROP CONSTRAINT %s;", m[1], m[2]))
		}
	}

	// Reverse order so drops happen in reverse dependency order
	for i, j := 0, len(downStatements)-1; i < j; i, j = i+1, j-1 {
		downStatements[i], downStatements[j] = downStatements[j], downStatements[i]
	}

	return strings.Join(downStatements, "\n"), nil
}
