package migration

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

// DROP TABLEのSQLからテーブル削除の破壊的変更が検出されることを確認する。
func TestDetectDropTable(t *testing.T) {
	sql := "DROP TABLE users;"
	changes := DetectBreakingChanges(sql)
	assert.Equal(t, 1, len(changes))
	assert.Equal(t, TableDropped, changes[0].Type)
	assert.Equal(t, "users", changes[0].Table)
}

// DROP COLUMNのSQLからカラム削除の破壊的変更が検出されることを確認する。
func TestDetectDropColumn(t *testing.T) {
	sql := "ALTER TABLE users DROP COLUMN email;"
	changes := DetectBreakingChanges(sql)
	assert.Equal(t, 1, len(changes))
	assert.Equal(t, ColumnDropped, changes[0].Type)
	assert.Equal(t, "users", changes[0].Table)
	assert.Equal(t, "email", changes[0].Column)
}

// SET NOT NULLのSQLからNOT NULL制約追加の破壊的変更が検出されることを確認する。
func TestDetectSetNotNull(t *testing.T) {
	sql := "ALTER TABLE users ALTER COLUMN email SET NOT NULL;"
	changes := DetectBreakingChanges(sql)
	assert.Equal(t, 1, len(changes))
	assert.Equal(t, NotNullAdded, changes[0].Type)
	assert.Equal(t, "users", changes[0].Table)
	assert.Equal(t, "email", changes[0].Column)
}

// SET DATA TYPEのSQLからカラム型変更の破壊的変更が検出されることを確認する。
func TestDetectTypeChange(t *testing.T) {
	sql := "ALTER TABLE users ALTER COLUMN age SET DATA TYPE BIGINT;"
	changes := DetectBreakingChanges(sql)
	assert.Equal(t, 1, len(changes))
	assert.Equal(t, ColumnTypeChanged, changes[0].Type)
	assert.Equal(t, "users", changes[0].Table)
	assert.Equal(t, "age", changes[0].Column)
	assert.Equal(t, "BIGINT", changes[0].To)
}

// RENAME COLUMNのSQLからカラム名変更の破壊的変更が検出されることを確認する。
func TestDetectRenameColumn(t *testing.T) {
	sql := "ALTER TABLE users RENAME COLUMN old_name TO new_name;"
	changes := DetectBreakingChanges(sql)
	assert.Equal(t, 1, len(changes))
	assert.Equal(t, ColumnRenamed, changes[0].Type)
	assert.Equal(t, "users", changes[0].Table)
	assert.Equal(t, "old_name", changes[0].From)
	assert.Equal(t, "new_name", changes[0].To)
}

// 非破壊的なSQLに対してDetectBreakingChangesが空のスライスを返すことを確認する。
func TestNoBreakingChanges(t *testing.T) {
	sql := "ALTER TABLE users ADD COLUMN email TEXT;"
	changes := DetectBreakingChanges(sql)
	assert.Empty(t, changes)
}

// BreakingChangeのStringメソッドが人間可読な表示文字列を返すことを確認する。
func TestDisplayFormatting(t *testing.T) {
	change := BreakingChange{
		Type:   ColumnDropped,
		Table:  "users",
		Column: "email",
	}
	assert.Equal(t, "Column users.email dropped", change.String())
}

// 不正なSQLを入力した場合にDetectBreakingChangesが空のスライスを返すことを確認する。
func TestInvalidSQLReturnsEmpty(t *testing.T) {
	changes := DetectBreakingChanges("NOT VALID SQL AT ALL !!!")
	assert.Empty(t, changes)
}
