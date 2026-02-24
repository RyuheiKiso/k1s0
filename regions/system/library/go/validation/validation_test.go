package validation_test

import (
	"testing"
	"time"

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
	assert.Equal(t, "INVALID_EMAIL", ve.Code)
}

func TestValidatePagination_Valid(t *testing.T) {
	v := validation.NewDefaultValidator()
	assert.NoError(t, v.ValidatePagination(1, 10))
	assert.NoError(t, v.ValidatePagination(1, 1))
	assert.NoError(t, v.ValidatePagination(1, 100))
	assert.NoError(t, v.ValidatePagination(999, 50))
}

func TestValidatePagination_Invalid(t *testing.T) {
	v := validation.NewDefaultValidator()
	assert.Error(t, v.ValidatePagination(0, 10))  // page < 1
	assert.Error(t, v.ValidatePagination(1, 0))   // perPage < 1
	assert.Error(t, v.ValidatePagination(1, 101)) // perPage > 100
	assert.Error(t, v.ValidatePagination(-1, 50)) // negative page
}

func TestValidatePagination_ErrorCode(t *testing.T) {
	v := validation.NewDefaultValidator()
	err := v.ValidatePagination(0, 10)
	require.Error(t, err)
	var ve *validation.ValidationError
	require.ErrorAs(t, err, &ve)
	assert.Equal(t, "INVALID_PAGE", ve.Code)

	err = v.ValidatePagination(1, 0)
	require.Error(t, err)
	require.ErrorAs(t, err, &ve)
	assert.Equal(t, "INVALID_PER_PAGE", ve.Code)
}

func TestValidateDateRange_Valid(t *testing.T) {
	v := validation.NewDefaultValidator()
	start := time.Date(2024, 1, 1, 0, 0, 0, 0, time.UTC)
	end := time.Date(2024, 12, 31, 23, 59, 59, 0, time.UTC)
	assert.NoError(t, v.ValidateDateRange(start, end))
}

func TestValidateDateRange_Equal(t *testing.T) {
	v := validation.NewDefaultValidator()
	dt := time.Date(2024, 6, 15, 12, 0, 0, 0, time.UTC)
	assert.NoError(t, v.ValidateDateRange(dt, dt))
}

func TestValidateDateRange_Invalid(t *testing.T) {
	v := validation.NewDefaultValidator()
	start := time.Date(2024, 12, 31, 23, 59, 59, 0, time.UTC)
	end := time.Date(2024, 1, 1, 0, 0, 0, 0, time.UTC)
	err := v.ValidateDateRange(start, end)
	require.Error(t, err)
	var ve *validation.ValidationError
	require.ErrorAs(t, err, &ve)
	assert.Equal(t, "INVALID_DATE_RANGE", ve.Code)
}

func TestValidationErrors_Collection(t *testing.T) {
	errors := validation.NewValidationErrors()
	assert.False(t, errors.HasErrors())
	assert.Empty(t, errors.GetErrors())

	errors.Add(&validation.ValidationError{Field: "email", Message: "invalid", Code: "INVALID_EMAIL"})
	errors.Add(&validation.ValidationError{Field: "page", Message: "invalid", Code: "INVALID_PAGE"})

	assert.True(t, errors.HasErrors())
	assert.Len(t, errors.GetErrors(), 2)
	assert.Equal(t, "INVALID_EMAIL", errors.GetErrors()[0].Code)
	assert.Equal(t, "INVALID_PAGE", errors.GetErrors()[1].Code)
}
