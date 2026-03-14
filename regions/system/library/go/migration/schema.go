package migration

// Schema represents a database schema.
type Schema struct {
	Tables []Table
}

// Table represents a database table.
type Table struct {
	Name        string
	Columns     []Column
	Indexes     []Index
	Constraints []Constraint
}

// Column represents a database column.
type Column struct {
	Name     string
	DataType string
	Nullable bool
	Default  *string // nil if no default
}

// Index represents a database index.
type Index struct {
	Name    string
	Table   string
	Columns []string
	Unique  bool
}

// ConstraintType represents the type of a database constraint.
type ConstraintType int

const (
	ConstraintPrimaryKey ConstraintType = iota
	ConstraintForeignKey
	ConstraintUnique
	ConstraintCheck
)

// Constraint represents a database constraint.
type Constraint struct {
	Type       ConstraintType
	Columns    []string
	RefTable   string   // for FK
	RefColumns []string // for FK
	Expression string   // for Check
}
