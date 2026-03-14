package migration

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

// TOML定義から基本的なCREATE TABLE SQLが正しく生成されることを確認する。
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

// デフォルト値を持つカラムのCREATE TABLE SQLが正しく生成されることを確認する。
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

// REFERENCES制約を持つカラムのCREATE TABLE SQLが正しく生成されることを確認する。
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

// 不正なTOML文字列を入力した場合にTomlToCreateSQLがエラーを返すことを確認する。
func TestInvalidToml(t *testing.T) {
	_, err := TomlToCreateSQL("not valid toml {{{}}}")
	assert.Error(t, err)
}

// 複数カラムの複合主キーを持つテーブルのCREATE TABLE SQLが正しく生成されることを確認する。
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
