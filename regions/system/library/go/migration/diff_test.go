package migration

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func makeColumn(name, dataType string, nullable bool) Column {
	return Column{
		Name:     name,
		DataType: dataType,
		Nullable: nullable,
		Default:  nil,
	}
}

func makeTable(name string, columns []Column) Table {
	return Table{
		Name:    name,
		Columns: columns,
	}
}

// DiffSchemasが新しいテーブルの追加をDiffTableAddedとして検出することを確認する。
func TestTableAdded(t *testing.T) {
	old := &Schema{Tables: []Table{}}
	new := &Schema{Tables: []Table{
		makeTable("users", []Column{makeColumn("id", "UUID", false)}),
	}}
	diffs := DiffSchemas(old, new)
	assert.Equal(t, 1, len(diffs))
	assert.Equal(t, DiffTableAdded, diffs[0].Type)
	assert.Equal(t, "users", diffs[0].Table)
}

// DiffSchemasがテーブルの削除をDiffTableDroppedとして検出することを確認する。
func TestTableDropped(t *testing.T) {
	old := &Schema{Tables: []Table{
		makeTable("users", []Column{makeColumn("id", "UUID", false)}),
	}}
	new := &Schema{Tables: []Table{}}
	diffs := DiffSchemas(old, new)
	assert.Equal(t, 1, len(diffs))
	assert.Equal(t, DiffTableDropped, diffs[0].Type)
	assert.Equal(t, "users", diffs[0].Table)
}

// DiffSchemasがカラムの追加をDiffColumnAddedとして検出することを確認する。
func TestColumnAdded(t *testing.T) {
	old := &Schema{Tables: []Table{
		makeTable("users", []Column{makeColumn("id", "UUID", false)}),
	}}
	new := &Schema{Tables: []Table{
		makeTable("users", []Column{
			makeColumn("id", "UUID", false),
			makeColumn("email", "TEXT", true),
		}),
	}}
	diffs := DiffSchemas(old, new)
	assert.Equal(t, 1, len(diffs))
	assert.Equal(t, DiffColumnAdded, diffs[0].Type)
	assert.Equal(t, "users", diffs[0].Table)
	assert.Equal(t, "email", diffs[0].Column.Name)
}

// DiffSchemasがカラムの削除をDiffColumnDroppedとして検出することを確認する。
func TestColumnDropped(t *testing.T) {
	old := &Schema{Tables: []Table{
		makeTable("users", []Column{
			makeColumn("id", "UUID", false),
			makeColumn("email", "TEXT", true),
		}),
	}}
	new := &Schema{Tables: []Table{
		makeTable("users", []Column{makeColumn("id", "UUID", false)}),
	}}
	diffs := DiffSchemas(old, new)
	assert.Equal(t, 1, len(diffs))
	assert.Equal(t, DiffColumnDropped, diffs[0].Type)
	assert.Equal(t, "users", diffs[0].Table)
	assert.Equal(t, "email", diffs[0].ColumnName)
}

// DiffSchemasがカラムの型またはNullable変更をDiffColumnChangedとして検出することを確認する。
func TestColumnChanged(t *testing.T) {
	old := &Schema{Tables: []Table{
		makeTable("users", []Column{makeColumn("name", "TEXT", true)}),
	}}
	new := &Schema{Tables: []Table{
		makeTable("users", []Column{makeColumn("name", "VARCHAR", false)}),
	}}
	diffs := DiffSchemas(old, new)
	assert.Equal(t, 1, len(diffs))
	assert.Equal(t, DiffColumnChanged, diffs[0].Type)
	assert.Equal(t, "users", diffs[0].Table)
	assert.Equal(t, "name", diffs[0].ColumnName)
	assert.Equal(t, "TEXT", diffs[0].From.DataType)
	assert.Equal(t, "VARCHAR", diffs[0].To.DataType)
}

// 同一スキーマ同士のDiffSchemasが差分なしを返すことを確認する。
func TestNoChanges(t *testing.T) {
	schema := &Schema{Tables: []Table{
		makeTable("users", []Column{makeColumn("id", "UUID", false)}),
	}}
	diffs := DiffSchemas(schema, schema)
	assert.Empty(t, diffs)
}
