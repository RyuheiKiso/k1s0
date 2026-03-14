package migration

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

// CREATE TABLEのUP SQLからDROP TABLEのDOWN SQLが生成されることを確認する。
func TestCreateTableGeneratesDrop(t *testing.T) {
	up := "CREATE TABLE users (id UUID PRIMARY KEY, name TEXT NOT NULL);"
	down, err := GenerateDownSQL(up)
	assert.NoError(t, err)
	assert.Contains(t, down, "DROP TABLE IF EXISTS users CASCADE;")
}

// ADD COLUMNのUP SQLからDROP COLUMNのDOWN SQLが生成されることを確認する。
func TestAddColumnGeneratesDropColumn(t *testing.T) {
	up := "ALTER TABLE users ADD COLUMN email TEXT;"
	down, err := GenerateDownSQL(up)
	assert.NoError(t, err)
	assert.Contains(t, down, "ALTER TABLE users DROP COLUMN email;")
}

// CREATE INDEXのUP SQLからDROP INDEXのDOWN SQLが生成されることを確認する。
func TestCreateIndexGeneratesDropIndex(t *testing.T) {
	up := "CREATE INDEX idx_users_name ON users (name);"
	down, err := GenerateDownSQL(up)
	assert.NoError(t, err)
	assert.Contains(t, down, "DROP INDEX IF EXISTS idx_users_name;")
}

// CREATE UNIQUE INDEXのUP SQLからDROP INDEXのDOWN SQLが生成されることを確認する。
func TestCreateUniqueIndexGeneratesDropIndex(t *testing.T) {
	up := "CREATE UNIQUE INDEX idx_users_email ON users (email);"
	down, err := GenerateDownSQL(up)
	assert.NoError(t, err)
	assert.Contains(t, down, "DROP INDEX IF EXISTS idx_users_email;")
}

// 複数のSQLステートメントを逆順にしたDOWN SQLが生成されることを確認する。
func TestMultipleStatementsReversed(t *testing.T) {
	up := "CREATE TABLE users (id UUID PRIMARY KEY);\nCREATE INDEX idx_users_id ON users (id);"
	down, err := GenerateDownSQL(up)
	assert.NoError(t, err)
	lines := strings.Split(strings.TrimSpace(down), "\n")
	assert.Equal(t, 2, len(lines))
	assert.Contains(t, lines[0], "DROP INDEX")
	assert.Contains(t, lines[1], "DROP TABLE")
}

// 空のSQLを入力した場合にGenerateDownSQLが空文字列を返すことを確認する。
func TestEmptySQL(t *testing.T) {
	down, err := GenerateDownSQL("")
	assert.NoError(t, err)
	assert.Empty(t, down)
}
