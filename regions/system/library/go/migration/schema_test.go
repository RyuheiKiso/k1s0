package migration

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestSchemaConstruction(t *testing.T) {
	defaultVal := "now()"
	schema := Schema{
		Tables: []Table{
			{
				Name: "users",
				Columns: []Column{
					{Name: "id", DataType: "UUID", Nullable: false, Default: nil},
					{Name: "created_at", DataType: "TIMESTAMP", Nullable: false, Default: &defaultVal},
				},
				Indexes: []Index{
					{Name: "idx_users_id", Table: "users", Columns: []string{"id"}, Unique: true},
				},
				Constraints: []Constraint{
					{Type: ConstraintPrimaryKey, Columns: []string{"id"}},
				},
			},
		},
	}

	assert.Equal(t, 1, len(schema.Tables))
	assert.Equal(t, "users", schema.Tables[0].Name)
	assert.Equal(t, 2, len(schema.Tables[0].Columns))
	assert.Equal(t, 1, len(schema.Tables[0].Indexes))
	assert.Equal(t, 1, len(schema.Tables[0].Constraints))
	assert.Equal(t, "now()", *schema.Tables[0].Columns[1].Default)
	assert.Nil(t, schema.Tables[0].Columns[0].Default)
}
