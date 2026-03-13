package migration

import (
	"fmt"
	"strings"

	"github.com/BurntSushi/toml"
)

type tableDef struct {
	Table tableSpec `toml:"table"`
}

type tableSpec struct {
	Name    string       `toml:"name"`
	Columns []columnSpec `toml:"columns"`
}

type columnSpec struct {
	Name       string  `toml:"name"`
	DataType   string  `toml:"type"`
	PrimaryKey bool    `toml:"primary_key"`
	Nullable   *bool   `toml:"nullable"` // pointer to distinguish unset (default true)
	Default    *string `toml:"default"`
	Unique     bool    `toml:"unique"`
	References *string `toml:"references"`
}

func (c columnSpec) isNullable() bool {
	if c.Nullable == nil {
		return true
	}
	return *c.Nullable
}

// TomlToCreateSQL converts a TOML table definition to CREATE TABLE SQL.
func TomlToCreateSQL(tomlStr string) (string, error) {
	var def tableDef
	if err := toml.Unmarshal([]byte(tomlStr), &def); err != nil {
		return "", fmt.Errorf("TOML parse error: %w", err)
	}

	var colDefs []string
	var primaryKeys []string

	for _, col := range def.Table.Columns {
		parts := []string{fmt.Sprintf("%s %s", col.Name, col.DataType)}

		if !col.isNullable() && !col.PrimaryKey {
			parts = append(parts, "NOT NULL")
		}

		if col.Unique {
			parts = append(parts, "UNIQUE")
		}

		if col.Default != nil {
			parts = append(parts, fmt.Sprintf("DEFAULT %s", *col.Default))
		}

		if col.References != nil {
			parts = append(parts, fmt.Sprintf("REFERENCES %s", *col.References))
		}

		if col.PrimaryKey {
			primaryKeys = append(primaryKeys, col.Name)
		}

		colDefs = append(colDefs, strings.Join(parts, " "))
	}

	if len(primaryKeys) > 0 {
		colDefs = append(colDefs, fmt.Sprintf("PRIMARY KEY (%s)", strings.Join(primaryKeys, ", ")))
	}

	return fmt.Sprintf("CREATE TABLE %s (\n  %s\n);", def.Table.Name, strings.Join(colDefs, ",\n  ")), nil
}
