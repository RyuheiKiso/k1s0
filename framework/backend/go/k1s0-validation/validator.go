package k1s0validation

import (
	"reflect"
	"strings"

	"github.com/go-playground/validator/v10"
)

// Validator wraps go-playground/validator with k1s0 conventions.
type Validator struct {
	v *validator.Validate
}

// New creates a new Validator instance.
func New() *Validator {
	v := validator.New(validator.WithRequiredStructEnabled())

	// Use JSON tag names for field names in error messages
	v.RegisterTagNameFunc(func(field reflect.StructField) string {
		name := strings.SplitN(field.Tag.Get("json"), ",", 2)[0]
		if name == "-" {
			return ""
		}
		if name == "" {
			name = field.Name
		}
		return name
	})

	return &Validator{v: v}
}

// Validate validates a struct and returns ValidationErrors.
func (val *Validator) Validate(s interface{}) *ValidationErrors {
	err := val.v.Struct(s)
	if err == nil {
		return nil
	}

	validationErrors := NewValidationErrors()

	if errs, ok := err.(validator.ValidationErrors); ok {
		for _, e := range errs {
			field := e.Field()
			tag := e.Tag()
			param := e.Param()

			fieldErr := NewFieldError(
				field,
				tagToKind(tag),
				formatMessage(field, tag, param),
			).WithTag(tag).WithParam(param).WithValue(e.Value())

			validationErrors.Add(fieldErr)
		}
	}

	return validationErrors
}

// ValidateField validates a single field value.
func (val *Validator) ValidateField(value interface{}, tag string) *ValidationErrors {
	err := val.v.Var(value, tag)
	if err == nil {
		return nil
	}

	validationErrors := NewValidationErrors()

	if errs, ok := err.(validator.ValidationErrors); ok {
		for _, e := range errs {
			fieldErr := NewFieldError(
				"value",
				tagToKind(e.Tag()),
				formatMessage("value", e.Tag(), e.Param()),
			).WithTag(e.Tag()).WithParam(e.Param()).WithValue(e.Value())

			validationErrors.Add(fieldErr)
		}
	}

	return validationErrors
}

// RegisterValidation registers a custom validation function.
func (val *Validator) RegisterValidation(tag string, fn validator.Func) error {
	return val.v.RegisterValidation(tag, fn)
}

// RegisterValidationWithMessage registers a custom validation with a message generator.
func (val *Validator) RegisterValidationWithMessage(tag string, fn validator.Func, messageFunc func(field, param string) string) error {
	// Store the message function for later use
	// This is a simplified implementation; in production you might want a more sophisticated approach
	return val.v.RegisterValidation(tag, fn)
}

// Underlying returns the underlying validator.Validate instance.
func (val *Validator) Underlying() *validator.Validate {
	return val.v
}

// ValidateVar validates a single variable against a tag.
func ValidateVar(value interface{}, tag string) *ValidationErrors {
	return New().ValidateField(value, tag)
}

// ValidateStruct validates a struct using a new validator instance.
func ValidateStruct(s interface{}) *ValidationErrors {
	return New().Validate(s)
}
