package migration

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestBasicTable(t *testing.T) {
	tomlStr := `
[table]
name = "users"

[[table.columns]]
name = "id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "name"
type = "TEXT"
nullable = false

[[table.columns]]
name = "email"
type = "TEXT"
nullable = true
unique = true
`
	sql, err := TomlToCreateSQL(tomlStr)
	assert.NoError(t, err)
	assert.Contains(t, sql, "CREATE TABLE users")
	assert.Contains(t, sql, "id UUID")
	assert.Contains(t, sql, "name TEXT NOT NULL")
	assert.Contains(t, sql, "email TEXT UNIQUE")
	assert.Contains(t, sql, "PRIMARY KEY (id)")
}

func TestColumnWithDefault(t *testing.T) {
	tomlStr := `
[table]
name = "settings"

[[table.columns]]
name = "active"
type = "BOOLEAN"
nullable = false
default = "true"
`
	sql, err := TomlToCreateSQL(tomlStr)
	assert.NoError(t, err)
	assert.Contains(t, sql, "active BOOLEAN NOT NULL DEFAULT true")
}

func TestColumnWithReferences(t *testing.T) {
	tomlStr := `
[table]
name = "orders"

[[table.columns]]
name = "id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "user_id"
type = "UUID"
nullable = false
references = "users(id)"
`
	sql, err := TomlToCreateSQL(tomlStr)
	assert.NoError(t, err)
	assert.Contains(t, sql, "user_id UUID NOT NULL REFERENCES users(id)")
}

func TestInvalidToml(t *testing.T) {
	_, err := TomlToCreateSQL("not valid toml {{{}}}")
	assert.Error(t, err)
}

func TestCompositePrimaryKey(t *testing.T) {
	tomlStr := `
[table]
name = "order_items"

[[table.columns]]
name = "order_id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "item_id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "quantity"
type = "INT"
nullable = false
`
	sql, err := TomlToCreateSQL(tomlStr)
	assert.NoError(t, err)
	assert.Contains(t, sql, "PRIMARY KEY (order_id, item_id)")
}
