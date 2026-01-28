package k1s0validation

import (
	"strings"
)

// ValidationErrors represents a collection of field validation errors.
type ValidationErrors struct {
	errors []*FieldError
}

// NewValidationErrors creates a new empty ValidationErrors.
func NewValidationErrors() *ValidationErrors {
	return &ValidationErrors{
		errors: make([]*FieldError, 0),
	}
}

// Add adds a field error to the collection.
func (v *ValidationErrors) Add(err *FieldError) *ValidationErrors {
	v.errors = append(v.errors, err)
	return v
}

// AddError adds an error for a field with the given kind and message.
func (v *ValidationErrors) AddError(field string, kind FieldErrorKind, message string) *ValidationErrors {
	return v.Add(NewFieldError(field, kind, message))
}

// HasErrors returns true if there are any validation errors.
func (v *ValidationErrors) HasErrors() bool {
	return len(v.errors) > 0
}

// Len returns the number of validation errors.
func (v *ValidationErrors) Len() int {
	return len(v.errors)
}

// Errors returns the slice of field errors.
func (v *ValidationErrors) Errors() []*FieldError {
	return v.errors
}

// FieldErrors returns errors for a specific field.
func (v *ValidationErrors) FieldErrors(field string) []*FieldError {
	result := make([]*FieldError, 0)
	for _, err := range v.errors {
		if err.Field == field {
			result = append(result, err)
		}
	}
	return result
}

// Fields returns all field names that have errors.
func (v *ValidationErrors) Fields() []string {
	fieldSet := make(map[string]bool)
	for _, err := range v.errors {
		fieldSet[err.Field] = true
	}

	fields := make([]string, 0, len(fieldSet))
	for field := range fieldSet {
		fields = append(fields, field)
	}
	return fields
}

// Error implements the error interface.
func (v *ValidationErrors) Error() string {
	if len(v.errors) == 0 {
		return "no validation errors"
	}

	messages := make([]string, len(v.errors))
	for i, err := range v.errors {
		messages[i] = err.Message
	}
	return strings.Join(messages, "; ")
}

// First returns the first error, or nil if there are no errors.
func (v *ValidationErrors) First() *FieldError {
	if len(v.errors) == 0 {
		return nil
	}
	return v.errors[0]
}

// Merge adds all errors from another ValidationErrors.
func (v *ValidationErrors) Merge(other *ValidationErrors) *ValidationErrors {
	if other != nil {
		v.errors = append(v.errors, other.errors...)
	}
	return v
}

// AsError returns nil if there are no errors, otherwise returns self.
// This is useful for returning from validation functions.
func (v *ValidationErrors) AsError() error {
	if !v.HasErrors() {
		return nil
	}
	return v
}

// ToMap converts errors to a map of field -> messages.
func (v *ValidationErrors) ToMap() map[string][]string {
	result := make(map[string][]string)
	for _, err := range v.errors {
		result[err.Field] = append(result[err.Field], err.Message)
	}
	return result
}
