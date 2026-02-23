package validation_test

import (
	"testing"

	"github.com/k1s0-platform/system-library-go-validation"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestValidateEmail_Valid(t *testing.T) {
	v := validation.NewDefaultValidator()
	assert.NoError(t, v.ValidateEmail("user@example.com"))
	assert.NoError(t, v.ValidateEmail("test.user+tag@sub.domain.co.jp"))
}

func TestValidateEmail_Invalid(t *testing.T) {
	v := validation.NewDefaultValidator()
	assert.Error(t, v.ValidateEmail(""))
	assert.Error(t, v.ValidateEmail("not-an-email"))
	assert.Error(t, v.ValidateEmail("@example.com"))
}

func TestValidateUUID_Valid(t *testing.T) {
	v := validation.NewDefaultValidator()
	assert.NoError(t, v.ValidateUUID("550e8400-e29b-41d4-a716-446655440000"))
	assert.NoError(t, v.ValidateUUID("6ba7b810-9dad-41d1-80b4-00c04fd430c8"))
}

func TestValidateUUID_Invalid(t *testing.T) {
	v := validation.NewDefaultValidator()
	assert.Error(t, v.ValidateUUID(""))
	assert.Error(t, v.ValidateUUID("not-a-uuid"))
	assert.Error(t, v.ValidateUUID("550e8400-e29b-31d4-a716-446655440000")) // v3, not v4
}

func TestValidateURL_Valid(t *testing.T) {
	v := validation.NewDefaultValidator()
	assert.NoError(t, v.ValidateURL("https://example.com"))
	assert.NoError(t, v.ValidateURL("http://localhost:8080/path"))
}

func TestValidateURL_Invalid(t *testing.T) {
	v := validation.NewDefaultValidator()
	assert.Error(t, v.ValidateURL(""))
	assert.Error(t, v.ValidateURL("ftp://example.com"))
	assert.Error(t, v.ValidateURL("not-a-url"))
}

func TestValidateTenantID_Valid(t *testing.T) {
	v := validation.NewDefaultValidator()
	assert.NoError(t, v.ValidateTenantID("abc"))
	assert.NoError(t, v.ValidateTenantID("my-tenant-123"))
}

func TestValidateTenantID_Invalid(t *testing.T) {
	v := validation.NewDefaultValidator()
	assert.Error(t, v.ValidateTenantID("ab"))                          // too short
	assert.Error(t, v.ValidateTenantID("ABC"))                         // uppercase
	assert.Error(t, v.ValidateTenantID("a_b"))                         // underscore
}

func TestValidationError_Message(t *testing.T) {
	v := validation.NewDefaultValidator()
	err := v.ValidateEmail("bad")
	require.Error(t, err)
	var ve *validation.ValidationError
	require.ErrorAs(t, err, &ve)
	assert.Equal(t, "email", ve.Field)
}
