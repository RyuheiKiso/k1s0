package k1s0error

// ErrorCode represents an operational error code.
// Format: {service_name}.{category}.{reason}
// Examples: auth.invalid_credentials, user.not_found, db.conflict
type ErrorCode string

// NewErrorCode creates a new ErrorCode.
func NewErrorCode(code string) ErrorCode {
	return ErrorCode(code)
}

// String returns the string representation of the error code.
func (c ErrorCode) String() string {
	return string(c)
}

// Predefined error codes for common scenarios.
const (
	CodeValidationError   ErrorCode = "validation_error"
	CodeNotFound          ErrorCode = "not_found"
	CodeConflict          ErrorCode = "conflict"
	CodeUnauthenticated   ErrorCode = "unauthenticated"
	CodePermissionDenied  ErrorCode = "permission_denied"
	CodeDependencyFailure ErrorCode = "dependency_failure"
	CodeTransient         ErrorCode = "transient_error"
	CodeInternal          ErrorCode = "internal_error"
)

// DefaultCodeForKind returns the default error code for a given ErrorKind.
func DefaultCodeForKind(kind ErrorKind) ErrorCode {
	switch kind {
	case KindInvalidInput:
		return CodeValidationError
	case KindNotFound:
		return CodeNotFound
	case KindConflict:
		return CodeConflict
	case KindUnauthorized:
		return CodeUnauthenticated
	case KindForbidden:
		return CodePermissionDenied
	case KindDependencyFailure:
		return CodeDependencyFailure
	case KindTransient:
		return CodeTransient
	case KindInvariantViolation:
		return CodeValidationError
	case KindInternal:
		return CodeInternal
	default:
		return CodeInternal
	}
}
