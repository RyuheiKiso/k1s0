package k1s0error

import (
	"testing"
)

func TestErrorKindString(t *testing.T) {
	tests := []struct {
		kind     ErrorKind
		expected string
	}{
		{KindInvalidInput, "INVALID_INPUT"},
		{KindNotFound, "NOT_FOUND"},
		{KindConflict, "CONFLICT"},
		{KindUnauthorized, "UNAUTHORIZED"},
		{KindForbidden, "FORBIDDEN"},
		{KindDependencyFailure, "DEPENDENCY_FAILURE"},
		{KindTransient, "TRANSIENT"},
		{KindInvariantViolation, "INVARIANT_VIOLATION"},
		{KindInternal, "INTERNAL"},
	}

	for _, tt := range tests {
		t.Run(tt.expected, func(t *testing.T) {
			if got := tt.kind.String(); got != tt.expected {
				t.Errorf("ErrorKind.String() = %v, want %v", got, tt.expected)
			}
		})
	}
}

func TestErrorKindIsRetryable(t *testing.T) {
	if !KindTransient.IsRetryable() {
		t.Error("KindTransient should be retryable")
	}
	if !KindDependencyFailure.IsRetryable() {
		t.Error("KindDependencyFailure should be retryable")
	}
	if KindInvalidInput.IsRetryable() {
		t.Error("KindInvalidInput should not be retryable")
	}
	if KindNotFound.IsRetryable() {
		t.Error("KindNotFound should not be retryable")
	}
}

func TestErrorKindIsClientError(t *testing.T) {
	clientErrors := []ErrorKind{
		KindInvalidInput, KindNotFound, KindConflict,
		KindUnauthorized, KindForbidden, KindInvariantViolation,
	}
	for _, kind := range clientErrors {
		if !kind.IsClientError() {
			t.Errorf("%s should be a client error", kind)
		}
	}

	serverErrors := []ErrorKind{KindInternal, KindTransient, KindDependencyFailure}
	for _, kind := range serverErrors {
		if kind.IsClientError() {
			t.Errorf("%s should not be a client error", kind)
		}
	}
}

func TestDomainErrorInvalidInput(t *testing.T) {
	err := InvalidInput("name is required")
	if err.Kind() != KindInvalidInput {
		t.Errorf("expected KindInvalidInput, got %v", err.Kind())
	}
	if err.Message() != "name is required" {
		t.Errorf("expected 'name is required', got %v", err.Message())
	}
}

func TestDomainErrorNotFound(t *testing.T) {
	err := NotFound("User", "user-123")
	if err.Kind() != KindNotFound {
		t.Errorf("expected KindNotFound, got %v", err.Kind())
	}
	if err.ResourceType() != "User" {
		t.Errorf("expected 'User', got %v", err.ResourceType())
	}
	if err.ResourceID() != "user-123" {
		t.Errorf("expected 'user-123', got %v", err.ResourceID())
	}
}

func TestDomainErrorDuplicate(t *testing.T) {
	err := Duplicate("User", "email")
	if err.Kind() != KindConflict {
		t.Errorf("expected KindConflict, got %v", err.Kind())
	}
	if err.ResourceType() != "User" {
		t.Errorf("expected 'User', got %v", err.ResourceType())
	}
}

func TestDomainErrorDependencyFailure(t *testing.T) {
	err := DependencyFailure("PostgreSQL", "connection timeout")
	if err.Kind() != KindDependencyFailure {
		t.Errorf("expected KindDependencyFailure, got %v", err.Kind())
	}
	if !err.IsRetryable() {
		t.Error("DependencyFailure should be retryable")
	}
}

func TestDomainErrorWithSource(t *testing.T) {
	err := Internal("processing failed").WithSource("root cause: memory exhausted")
	if err.SourceMessage() != "root cause: memory exhausted" {
		t.Errorf("expected source message to be set")
	}
}

func TestDomainErrorWithResource(t *testing.T) {
	err := NewDomainError(KindInternal, "error").WithResource("Order", "order-456")
	if err.ResourceType() != "Order" {
		t.Errorf("expected 'Order', got %v", err.ResourceType())
	}
	if err.ResourceID() != "order-456" {
		t.Errorf("expected 'order-456', got %v", err.ResourceID())
	}
}

func TestErrorContextMerge(t *testing.T) {
	ctx1 := NewErrorContext().WithTraceID("trace-1").WithTenantID("tenant-1")
	ctx2 := NewErrorContext().WithTraceID("trace-2").WithRequestID("req-1")

	ctx1.Merge(ctx2)

	if ctx1.TraceID != "trace-2" {
		t.Errorf("expected trace-2, got %v", ctx1.TraceID)
	}
	if ctx1.TenantID != "tenant-1" {
		t.Errorf("expected tenant-1, got %v", ctx1.TenantID)
	}
	if ctx1.RequestID != "req-1" {
		t.Errorf("expected req-1, got %v", ctx1.RequestID)
	}
}

func TestErrorContextClone(t *testing.T) {
	ctx := NewErrorContext().
		WithTraceID("trace-1").
		WithRequestID("req-1").
		WithExtra("key", "value")

	clone := ctx.Clone()

	if clone.TraceID != ctx.TraceID {
		t.Errorf("expected trace ID to be cloned")
	}
	if clone.Extra["key"] != ctx.Extra["key"] {
		t.Errorf("expected extra to be cloned")
	}

	// Modify clone and verify original is unchanged
	clone.TraceID = "modified"
	if ctx.TraceID == "modified" {
		t.Error("original should not be modified")
	}
}

func TestAppErrorFromDomainError(t *testing.T) {
	domainErr := NotFound("User", "user-123")
	appErr := NewAppError(domainErr, NewErrorCode("USER_NOT_FOUND"))

	if appErr.Kind() != KindNotFound {
		t.Errorf("expected KindNotFound, got %v", appErr.Kind())
	}
	if appErr.ErrorCode() != "USER_NOT_FOUND" {
		t.Errorf("expected USER_NOT_FOUND, got %v", appErr.ErrorCode())
	}
}

func TestAppErrorFromDomainErrorAuto(t *testing.T) {
	tests := []struct {
		domainErr    *DomainError
		expectedCode ErrorCode
	}{
		{InvalidInput("test"), CodeValidationError},
		{NotFound("X", "1"), CodeNotFound},
		{Conflict("test"), CodeConflict},
		{Unauthorized("test"), CodeUnauthenticated},
		{Forbidden("test"), CodePermissionDenied},
		{DependencyFailure("db", "err"), CodeDependencyFailure},
		{Transient("test"), CodeTransient},
		{InvariantViolation("test"), CodeValidationError},
		{Internal("test"), CodeInternal},
	}

	for _, tt := range tests {
		t.Run(tt.expectedCode.String(), func(t *testing.T) {
			appErr := FromDomainError(tt.domainErr)
			if appErr.ErrorCode() != tt.expectedCode {
				t.Errorf("expected %s, got %s", tt.expectedCode, appErr.ErrorCode())
			}
		})
	}
}

func TestAppErrorWithContext(t *testing.T) {
	domainErr := Internal("test")
	appErr := FromDomainError(domainErr).
		WithTraceID("trace-123").
		WithRequestID("req-456")

	ctx := appErr.Context()
	if ctx.TraceID != "trace-123" {
		t.Errorf("expected trace-123, got %v", ctx.TraceID)
	}
	if ctx.RequestID != "req-456" {
		t.Errorf("expected req-456, got %v", ctx.RequestID)
	}
}

func TestAppErrorWithHint(t *testing.T) {
	domainErr := NotFound("User", "user-123")
	appErr := FromDomainError(domainErr).WithHint("Please verify the user ID")

	if appErr.Hint() != "Please verify the user ID" {
		t.Errorf("expected hint to be set")
	}
}

func TestAppErrorDisplay(t *testing.T) {
	domainErr := NotFound("User", "user-123")
	appErr := NewAppError(domainErr, NewErrorCode("USER_NOT_FOUND"))

	display := appErr.Error()
	if display != "[USER_NOT_FOUND] User 'user-123' not found" {
		t.Errorf("unexpected display: %v", display)
	}
}

func TestAppErrorUnwrap(t *testing.T) {
	domainErr := Internal("test")
	appErr := FromDomainError(domainErr)

	unwrapped := appErr.Unwrap()
	if unwrapped != domainErr {
		t.Error("Unwrap should return the domain error")
	}
}

func TestFullFlow(t *testing.T) {
	// Domain layer: transport-independent
	domainErr := NotFound("User", "user-123")
	if domainErr.Kind() != KindNotFound {
		t.Errorf("expected KindNotFound")
	}

	// Application layer: add error code
	appErr := NewAppError(domainErr, NewErrorCode("USER_NOT_FOUND"))
	if appErr.ErrorCode() != "USER_NOT_FOUND" {
		t.Errorf("expected USER_NOT_FOUND")
	}

	// Presentation layer: HTTP conversion
	httpErr := appErr.ToHTTPError()
	if httpErr.StatusCode() != 404 {
		t.Errorf("expected 404, got %d", httpErr.StatusCode())
	}

	// Presentation layer: gRPC conversion
	grpcErr := appErr.ToGRPCError()
	if grpcErr.StatusCode().String() != "NOT_FOUND" {
		t.Errorf("expected NOT_FOUND, got %s", grpcErr.StatusCode())
	}
}

func TestWithContextFlow(t *testing.T) {
	domainErr := Internal("database connection error")
	appErr := FromDomainError(domainErr).
		WithTraceID("trace-abc123").
		WithRequestID("req-xyz789")

	httpErr := appErr.ToHTTPError()
	if httpErr.TraceID() != "trace-abc123" {
		t.Errorf("expected trace ID to be set")
	}
	if httpErr.RequestID() != "req-xyz789" {
		t.Errorf("expected request ID to be set")
	}
}
