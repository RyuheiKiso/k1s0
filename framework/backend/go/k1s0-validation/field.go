// Package k1s0validation provides input validation for the k1s0 framework.
//
// This package integrates with go-playground/validator and provides
// unified error representations for both REST and gRPC APIs.
//
// # Usage
//
//	type CreateUserRequest struct {
//	    Name  string `json:"name" validate:"required,min=1,max=100"`
//	    Email string `json:"email" validate:"required,email"`
//	    Age   int    `json:"age" validate:"gte=0,lte=150"`
//	}
//
//	v := k1s0validation.New()
//	req := &CreateUserRequest{Name: "", Email: "invalid"}
//
//	if err := v.Validate(req); err != nil {
//	    // For REST: convert to ProblemDetails
//	    problem := err.ToProblemDetails()
//
//	    // For gRPC: convert to BadRequest details
//	    details := err.ToGRPCDetails()
//	}
package k1s0validation

// FieldErrorKind represents the type of validation error.
type FieldErrorKind string

const (
	// KindRequired indicates a required field is missing.
	KindRequired FieldErrorKind = "required"
	// KindFormat indicates a format validation failed.
	KindFormat FieldErrorKind = "format"
	// KindRange indicates a value is out of range.
	KindRange FieldErrorKind = "range"
	// KindLength indicates a length constraint was violated.
	KindLength FieldErrorKind = "length"
	// KindPattern indicates a regex pattern didn't match.
	KindPattern FieldErrorKind = "pattern"
	// KindEnum indicates a value is not in the allowed set.
	KindEnum FieldErrorKind = "enum"
	// KindUnique indicates a uniqueness constraint was violated.
	KindUnique FieldErrorKind = "unique"
	// KindCustom indicates a custom validation failed.
	KindCustom FieldErrorKind = "custom"
)

// FieldError represents a validation error for a single field.
type FieldError struct {
	// Field is the name of the field that failed validation.
	Field string `json:"field"`

	// Kind is the type of validation error.
	Kind FieldErrorKind `json:"kind"`

	// Message is a human-readable error message.
	Message string `json:"message"`

	// Tag is the validation tag that failed (e.g., "required", "email").
	Tag string `json:"tag,omitempty"`

	// Value is the actual value that failed validation.
	Value interface{} `json:"value,omitempty"`

	// Param is the parameter of the validation rule (e.g., "10" for max=10).
	Param string `json:"param,omitempty"`
}

// NewFieldError creates a new FieldError.
func NewFieldError(field string, kind FieldErrorKind, message string) *FieldError {
	return &FieldError{
		Field:   field,
		Kind:    kind,
		Message: message,
	}
}

// WithTag sets the validation tag.
func (e *FieldError) WithTag(tag string) *FieldError {
	e.Tag = tag
	return e
}

// WithValue sets the actual value.
func (e *FieldError) WithValue(value interface{}) *FieldError {
	e.Value = value
	return e
}

// WithParam sets the validation parameter.
func (e *FieldError) WithParam(param string) *FieldError {
	e.Param = param
	return e
}

// Error implements the error interface.
func (e *FieldError) Error() string {
	return e.Message
}

// tagToKind maps validation tags to FieldErrorKind.
func tagToKind(tag string) FieldErrorKind {
	switch tag {
	case "required", "required_if", "required_unless", "required_with", "required_without":
		return KindRequired
	case "email", "url", "uri", "uuid", "uuid3", "uuid4", "uuid5", "ip", "ipv4", "ipv6", "mac", "hostname":
		return KindFormat
	case "eq", "ne", "gt", "gte", "lt", "lte", "min", "max":
		return KindRange
	case "len", "min_len", "max_len":
		return KindLength
	case "regexp", "alphanum", "alpha", "numeric", "lowercase", "uppercase":
		return KindPattern
	case "oneof", "eq_ignore_case":
		return KindEnum
	case "unique":
		return KindUnique
	default:
		return KindCustom
	}
}

// formatMessage creates a human-readable error message.
func formatMessage(field, tag, param string) string {
	switch tag {
	case "required":
		return field + " is required"
	case "email":
		return field + " must be a valid email address"
	case "url":
		return field + " must be a valid URL"
	case "uuid":
		return field + " must be a valid UUID"
	case "min":
		return field + " must be at least " + param
	case "max":
		return field + " must be at most " + param
	case "len":
		return field + " must have a length of " + param
	case "gte":
		return field + " must be greater than or equal to " + param
	case "lte":
		return field + " must be less than or equal to " + param
	case "oneof":
		return field + " must be one of: " + param
	default:
		return field + " failed validation: " + tag
	}
}
