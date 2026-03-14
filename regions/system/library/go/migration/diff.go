package migration

// SchemaDiffType represents the type of a schema difference.
type SchemaDiffType int

const (
	DiffTableAdded SchemaDiffType = iota
	DiffTableDropped
	DiffColumnAdded
	DiffColumnDropped
	DiffColumnChanged
)

// SchemaDiff represents a single difference between two schemas.
type SchemaDiff struct {
	Type       SchemaDiffType
	Table      string
	Column     *Column // for ColumnAdded
	ColumnName string  // for ColumnDropped/Changed
	From       *Column // for ColumnChanged
	To         *Column // for ColumnChanged
	AddedTable *Table  // for TableAdded
}

// DiffSchemas compares two schemas and returns the differences.
func DiffSchemas(old, new *Schema) []SchemaDiff {
	var diffs []SchemaDiff

	oldTables := make(map[string]*Table)
	for i := range old.Tables {
		oldTables[old.Tables[i].Name] = &old.Tables[i]
	}
	newTables := make(map[string]*Table)
	for i := range new.Tables {
		newTables[new.Tables[i].Name] = &new.Tables[i]
	}

	// Detect dropped tables
	for name := range oldTables {
		if _, ok := newTables[name]; !ok {
			diffs = append(diffs, SchemaDiff{
				Type:  DiffTableDropped,
				Table: name,
			})
		}
	}

	// Detect added tables and column changes
	for name, newTable := range newTables {
		oldTable, ok := oldTables[name]
		if !ok {
			diffs = append(diffs, SchemaDiff{
				Type:       DiffTableAdded,
				Table:      name,
				AddedTable: newTable,
			})
		} else {
			diffs = append(diffs, diffColumns(name, oldTable, newTable)...)
		}
	}

	return diffs
}

func diffColumns(table string, old, new *Table) []SchemaDiff {
	var diffs []SchemaDiff

	oldCols := make(map[string]*Column)
	for i := range old.Columns {
		oldCols[old.Columns[i].Name] = &old.Columns[i]
	}
	newCols := make(map[string]*Column)
	for i := range new.Columns {
		newCols[new.Columns[i].Name] = &new.Columns[i]
	}

	// Detect dropped columns
	for colName := range oldCols {
		if _, ok := newCols[colName]; !ok {
			diffs = append(diffs, SchemaDiff{
				Type:       DiffColumnDropped,
				Table:      table,
				ColumnName: colName,
			})
		}
	}

	// Detect added and changed columns
	for colName, newCol := range newCols {
		oldCol, ok := oldCols[colName]
		if !ok {
			diffs = append(diffs, SchemaDiff{
				Type:   DiffColumnAdded,
				Table:  table,
				Column: newCol,
			})
		} else if !columnsEqual(oldCol, newCol) {
			diffs = append(diffs, SchemaDiff{
				Type:       DiffColumnChanged,
				Table:      table,
				ColumnName: colName,
				From:       oldCol,
				To:         newCol,
			})
		}
	}

	return diffs
}

func columnsEqual(a, b *Column) bool {
	if a.Name != b.Name || a.DataType != b.DataType || a.Nullable != b.Nullable {
		return false
	}
	if a.Default == nil && b.Default == nil {
		return true
	}
	if a.Default == nil || b.Default == nil {
		return false
	}
	return *a.Default == *b.Default
}
