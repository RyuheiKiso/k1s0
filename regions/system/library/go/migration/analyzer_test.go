package migration

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestDetectDropTable(t *testing.T) {
	sql := "DROP TABLE users;"
	changes := DetectBreakingChanges(sql)
	assert.Equal(t, 1, len(changes))
	assert.Equal(t, TableDropped, changes[0].Type)
	assert.Equal(t, "users", changes[0].Table)
}

func TestDetectDropColumn(t *testing.T) {
	sql := "ALTER TABLE users DROP COLUMN email;"
	changes := DetectBreakingChanges(sql)
	assert.Equal(t, 1, len(changes))
	assert.Equal(t, ColumnDropped, changes[0].Type)
	assert.Equal(t, "users", changes[0].Table)
	assert.Equal(t, "email", changes[0].Column)
}

func TestDetectSetNotNull(t *testing.T) {
	sql := "ALTER TABLE users ALTER COLUMN email SET NOT NULL;"
	changes := DetectBreakingChanges(sql)
	assert.Equal(t, 1, len(changes))
	assert.Equal(t, NotNullAdded, changes[0].Type)
	assert.Equal(t, "users", changes[0].Table)
	assert.Equal(t, "email", changes[0].Column)
}

func TestDetectTypeChange(t *testing.T) {
	sql := "ALTER TABLE users ALTER COLUMN age SET DATA TYPE BIGINT;"
	changes := DetectBreakingChanges(sql)
	assert.Equal(t, 1, len(changes))
	assert.Equal(t, ColumnTypeChanged, changes[0].Type)
	assert.Equal(t, "users", changes[0].Table)
	assert.Equal(t, "age", changes[0].Column)
	assert.Equal(t, "BIGINT", changes[0].To)
}

func TestDetectRenameColumn(t *testing.T) {
	sql := "ALTER TABLE users RENAME COLUMN old_name TO new_name;"
	changes := DetectBreakingChanges(sql)
	assert.Equal(t, 1, len(changes))
	assert.Equal(t, ColumnRenamed, changes[0].Type)
	assert.Equal(t, "users", changes[0].Table)
	assert.Equal(t, "old_name", changes[0].From)
	assert.Equal(t, "new_name", changes[0].To)
}

func TestNoBreakingChanges(t *testing.T) {
	sql := "ALTER TABLE users ADD COLUMN email TEXT;"
	changes := DetectBreakingChanges(sql)
	assert.Empty(t, changes)
}

func TestDisplayFormatting(t *testing.T) {
	change := BreakingChange{
		Type:   ColumnDropped,
		Table:  "users",
		Column: "email",
	}
	assert.Equal(t, "Column users.email dropped", change.String())
}

func TestInvalidSQLReturnsEmpty(t *testing.T) {
	changes := DetectBreakingChanges("NOT VALID SQL AT ALL !!!")
	assert.Empty(t, changes)
}
