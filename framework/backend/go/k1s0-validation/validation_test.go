package k1s0validation

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestFieldError(t *testing.T) {
	err := NewFieldError("email", KindFormat, "email must be valid")
	err.WithTag("email").WithValue("invalid").WithParam("")

	if err.Field != "email" {
		t.Errorf("expected 'email', got '%s'", err.Field)
	}
	if err.Kind != KindFormat {
		t.Errorf("expected KindFormat, got '%s'", err.Kind)
	}
	if err.Tag != "email" {
		t.Errorf("expected 'email', got '%s'", err.Tag)
	}
	if err.Error() != "email must be valid" {
		t.Errorf("unexpected error message: %s", err.Error())
	}
}

func TestValidationErrors(t *testing.T) {
	errs := NewValidationErrors()
	if errs.HasErrors() {
		t.Error("expected no errors initially")
	}

	errs.AddError("name", KindRequired, "name is required")
	errs.AddError("email", KindFormat, "email is invalid")

	if !errs.HasErrors() {
		t.Error("expected errors after adding")
	}
	if errs.Len() != 2 {
		t.Errorf("expected 2 errors, got %d", errs.Len())
	}
}

func TestValidationErrorsFieldErrors(t *testing.T) {
	errs := NewValidationErrors()
	errs.AddError("name", KindRequired, "name is required")
	errs.AddError("name", KindLength, "name is too short")
	errs.AddError("email", KindFormat, "email is invalid")

	nameErrors := errs.FieldErrors("name")
	if len(nameErrors) != 2 {
		t.Errorf("expected 2 name errors, got %d", len(nameErrors))
	}

	emailErrors := errs.FieldErrors("email")
	if len(emailErrors) != 1 {
		t.Errorf("expected 1 email error, got %d", len(emailErrors))
	}
}

func TestValidationErrorsFields(t *testing.T) {
	errs := NewValidationErrors()
	errs.AddError("name", KindRequired, "name is required")
	errs.AddError("email", KindFormat, "email is invalid")

	fields := errs.Fields()
	if len(fields) != 2 {
		t.Errorf("expected 2 fields, got %d", len(fields))
	}
}

func TestValidationErrorsFirst(t *testing.T) {
	errs := NewValidationErrors()
	if errs.First() != nil {
		t.Error("expected nil for empty errors")
	}

	errs.AddError("name", KindRequired, "name is required")
	first := errs.First()
	if first == nil {
		t.Fatal("expected first error")
	}
	if first.Field != "name" {
		t.Errorf("expected 'name', got '%s'", first.Field)
	}
}

func TestValidationErrorsMerge(t *testing.T) {
	errs1 := NewValidationErrors()
	errs1.AddError("name", KindRequired, "name is required")

	errs2 := NewValidationErrors()
	errs2.AddError("email", KindFormat, "email is invalid")

	errs1.Merge(errs2)

	if errs1.Len() != 2 {
		t.Errorf("expected 2 errors after merge, got %d", errs1.Len())
	}
}

func TestValidationErrorsAsError(t *testing.T) {
	errs := NewValidationErrors()
	if errs.AsError() != nil {
		t.Error("expected nil for empty errors")
	}

	errs.AddError("name", KindRequired, "name is required")
	if errs.AsError() == nil {
		t.Error("expected error after adding")
	}
}

func TestValidationErrorsToMap(t *testing.T) {
	errs := NewValidationErrors()
	errs.AddError("name", KindRequired, "name is required")
	errs.AddError("name", KindLength, "name is too short")

	m := errs.ToMap()
	if len(m["name"]) != 2 {
		t.Errorf("expected 2 messages for name, got %d", len(m["name"]))
	}
}

func TestProblemDetails(t *testing.T) {
	errs := NewValidationErrors()
	errs.AddError("name", KindRequired, "name is required")
	errs.AddError("email", KindFormat, "email is invalid")

	problem := errs.ToProblemDetails()

	if problem.Status != 400 {
		t.Errorf("expected 400, got %d", problem.Status)
	}
	if problem.ErrorCode != "validation_error" {
		t.Errorf("expected 'validation_error', got '%s'", problem.ErrorCode)
	}
	if len(problem.Errors) != 2 {
		t.Errorf("expected 2 errors, got %d", len(problem.Errors))
	}

	jsonBytes, err := problem.JSON()
	if err != nil {
		t.Fatalf("failed to marshal: %v", err)
	}

	jsonStr := string(jsonBytes)
	if !strings.Contains(jsonStr, "name") {
		t.Error("expected 'name' in JSON")
	}
	if !strings.Contains(jsonStr, "email") {
		t.Error("expected 'email' in JSON")
	}
}

func TestProblemDetailsWithContext(t *testing.T) {
	errs := NewValidationErrors()
	errs.AddError("name", KindRequired, "name is required")

	problem := errs.ToProblemDetails().
		WithTraceID("trace-123").
		WithRequestID("req-456").
		WithInstance("/users/create")

	if problem.TraceID != "trace-123" {
		t.Errorf("expected 'trace-123', got '%s'", problem.TraceID)
	}
	if problem.RequestID != "req-456" {
		t.Errorf("expected 'req-456', got '%s'", problem.RequestID)
	}
	if problem.Instance != "/users/create" {
		t.Errorf("expected '/users/create', got '%s'", problem.Instance)
	}
}

func TestGRPCDetails(t *testing.T) {
	errs := NewValidationErrors()
	errs.AddError("name", KindRequired, "name is required")
	errs.AddError("email", KindFormat, "email is invalid")

	details := errs.ToGRPCDetails()

	if len(details.FieldViolations) != 2 {
		t.Errorf("expected 2 violations, got %d", len(details.FieldViolations))
	}

	if details.FieldViolations[0].Field != "name" {
		t.Errorf("expected 'name', got '%s'", details.FieldViolations[0].Field)
	}
}

func TestGRPCStatusCode(t *testing.T) {
	errs := NewValidationErrors()
	if errs.GRPCStatusCode() != 3 {
		t.Errorf("expected 3 (INVALID_ARGUMENT), got %d", errs.GRPCStatusCode())
	}
}

func TestGRPCMetadata(t *testing.T) {
	errs := NewValidationErrors()
	errs.AddError("name", KindRequired, "name is required")

	metadata := errs.ToGRPCMetadata()
	if metadata["error_code"] != "validation_error" {
		t.Errorf("expected 'validation_error', got '%s'", metadata["error_code"])
	}
}

func TestValidator(t *testing.T) {
	type User struct {
		Name  string `json:"name" validate:"required,min=1,max=100"`
		Email string `json:"email" validate:"required,email"`
		Age   int    `json:"age" validate:"gte=0,lte=150"`
	}

	v := New()

	// Valid user
	validUser := &User{Name: "John", Email: "john@example.com", Age: 30}
	errs := v.Validate(validUser)
	if errs != nil && errs.HasErrors() {
		t.Errorf("expected no errors for valid user, got %v", errs.Error())
	}

	// Invalid user
	invalidUser := &User{Name: "", Email: "invalid", Age: 200}
	errs = v.Validate(invalidUser)
	if errs == nil || !errs.HasErrors() {
		t.Fatal("expected errors for invalid user")
	}

	// Check specific errors
	if errs.Len() < 3 {
		t.Errorf("expected at least 3 errors, got %d", errs.Len())
	}

	// Should have error for name (required)
	nameErrs := errs.FieldErrors("name")
	if len(nameErrs) == 0 {
		t.Error("expected error for name")
	}

	// Should have error for email (format)
	emailErrs := errs.FieldErrors("email")
	if len(emailErrs) == 0 {
		t.Error("expected error for email")
	}

	// Should have error for age (range)
	ageErrs := errs.FieldErrors("age")
	if len(ageErrs) == 0 {
		t.Error("expected error for age")
	}
}

func TestValidatorJSONFieldNames(t *testing.T) {
	type Request struct {
		UserName string `json:"user_name" validate:"required"`
	}

	v := New()
	errs := v.Validate(&Request{UserName: ""})

	if errs == nil || !errs.HasErrors() {
		t.Fatal("expected error")
	}

	// Should use JSON tag name
	if errs.First().Field != "user_name" {
		t.Errorf("expected 'user_name', got '%s'", errs.First().Field)
	}
}

func TestValidateField(t *testing.T) {
	v := New()

	// Valid email
	errs := v.ValidateField("test@example.com", "required,email")
	if errs != nil && errs.HasErrors() {
		t.Errorf("expected no errors for valid email, got %v", errs.Error())
	}

	// Invalid email
	errs = v.ValidateField("invalid", "required,email")
	if errs == nil || !errs.HasErrors() {
		t.Error("expected error for invalid email")
	}
}

func TestValidateVar(t *testing.T) {
	errs := ValidateVar("", "required")
	if errs == nil || !errs.HasErrors() {
		t.Error("expected error for empty required field")
	}
}

func TestValidateStruct(t *testing.T) {
	type Data struct {
		Value int `json:"value" validate:"gte=10"`
	}

	errs := ValidateStruct(&Data{Value: 5})
	if errs == nil || !errs.HasErrors() {
		t.Error("expected error for value less than 10")
	}
}

func TestTagToKind(t *testing.T) {
	tests := []struct {
		tag      string
		expected FieldErrorKind
	}{
		{"required", KindRequired},
		{"email", KindFormat},
		{"url", KindFormat},
		{"min", KindRange},
		{"max", KindRange},
		{"len", KindLength},
		{"alphanum", KindPattern},
		{"oneof", KindEnum},
		{"unique", KindUnique},
		{"custom_tag", KindCustom},
	}

	for _, tt := range tests {
		t.Run(tt.tag, func(t *testing.T) {
			kind := tagToKind(tt.tag)
			if kind != tt.expected {
				t.Errorf("expected %s, got %s", tt.expected, kind)
			}
		})
	}
}

func TestFormatMessage(t *testing.T) {
	tests := []struct {
		field    string
		tag      string
		param    string
		contains string
	}{
		{"name", "required", "", "required"},
		{"email", "email", "", "valid email"},
		{"url", "url", "", "valid URL"},
		{"age", "min", "18", "at least 18"},
		{"age", "max", "100", "at most 100"},
		{"status", "oneof", "active inactive", "one of: active inactive"},
	}

	for _, tt := range tests {
		t.Run(tt.tag, func(t *testing.T) {
			msg := formatMessage(tt.field, tt.tag, tt.param)
			if !strings.Contains(msg, tt.contains) {
				t.Errorf("expected message to contain '%s', got '%s'", tt.contains, msg)
			}
		})
	}
}

func TestValidationErrorsErrorMessage(t *testing.T) {
	errs := NewValidationErrors()
	if errs.Error() != "no validation errors" {
		t.Errorf("unexpected error message: %s", errs.Error())
	}

	errs.AddError("name", KindRequired, "name is required")
	if !strings.Contains(errs.Error(), "name is required") {
		t.Errorf("unexpected error message: %s", errs.Error())
	}

	errs.AddError("email", KindFormat, "email is invalid")
	if !strings.Contains(errs.Error(), ";") {
		t.Errorf("expected semicolon separator in error message: %s", errs.Error())
	}
}

func TestProblemDetailsSingleError(t *testing.T) {
	errs := NewValidationErrors()
	errs.AddError("name", KindRequired, "name is required")

	problem := errs.ToProblemDetails()

	// Single error should use the error message as detail
	if problem.Detail != "name is required" {
		t.Errorf("expected 'name is required', got '%s'", problem.Detail)
	}
}

func TestProblemDetailsContentType(t *testing.T) {
	errs := NewValidationErrors()
	problem := errs.ToProblemDetails()

	if problem.ContentType() != "application/problem+json" {
		t.Errorf("expected 'application/problem+json', got '%s'", problem.ContentType())
	}
}

func TestJSONMarshal(t *testing.T) {
	errs := NewValidationErrors()
	errs.AddError("name", KindRequired, "name is required")

	problem := errs.ToProblemDetails()
	jsonBytes, err := json.Marshal(problem)
	if err != nil {
		t.Fatalf("failed to marshal: %v", err)
	}

	var unmarshaled map[string]interface{}
	if err := json.Unmarshal(jsonBytes, &unmarshaled); err != nil {
		t.Fatalf("failed to unmarshal: %v", err)
	}

	if unmarshaled["status"].(float64) != 400 {
		t.Errorf("expected status 400")
	}
}
