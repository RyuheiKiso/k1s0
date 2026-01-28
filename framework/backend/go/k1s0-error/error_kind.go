// Package k1s0error provides Clean Architecture error types for the k1s0 framework.
//
// This package implements a layered error handling approach:
//   - Domain layer: Transport-independent error types (no HTTP/gRPC concepts)
//   - Application layer: Error codes for operational identification
//   - Presentation layer: REST (problem+json) and gRPC (status + metadata) conversion
//
// # Error Classification
//
//	| Kind              | Description            | HTTP | gRPC             |
//	|-------------------|------------------------|------|------------------|
//	| InvalidInput      | Input validation error | 400  | INVALID_ARGUMENT |
//	| NotFound          | Resource not found     | 404  | NOT_FOUND        |
//	| Conflict          | Conflict (duplicate)   | 409  | ALREADY_EXISTS   |
//	| Unauthorized      | Authentication error   | 401  | UNAUTHENTICATED  |
//	| Forbidden         | Authorization error    | 403  | PERMISSION_DENIED|
//	| DependencyFailure | Dependency failure     | 502  | UNAVAILABLE      |
//	| Transient         | Transient error        | 503  | UNAVAILABLE      |
//	| Internal          | Internal error         | 500  | INTERNAL         |
package k1s0error

// ErrorKind represents the classification of domain errors.
// It is transport-independent (no HTTP/gRPC details).
type ErrorKind int

const (
	// KindInvalidInput represents input validation errors.
	KindInvalidInput ErrorKind = iota
	// KindNotFound represents resource not found errors.
	KindNotFound
	// KindConflict represents conflict errors (duplicates, optimistic lock failures).
	KindConflict
	// KindUnauthorized represents authentication errors.
	KindUnauthorized
	// KindForbidden represents authorization errors (insufficient permissions).
	KindForbidden
	// KindDependencyFailure represents external dependency failures.
	KindDependencyFailure
	// KindTransient represents transient errors (retryable).
	KindTransient
	// KindInvariantViolation represents business rule violations.
	KindInvariantViolation
	// KindInternal represents internal/unexpected errors.
	KindInternal
)

// String returns the string representation of the ErrorKind.
func (k ErrorKind) String() string {
	switch k {
	case KindInvalidInput:
		return "INVALID_INPUT"
	case KindNotFound:
		return "NOT_FOUND"
	case KindConflict:
		return "CONFLICT"
	case KindUnauthorized:
		return "UNAUTHORIZED"
	case KindForbidden:
		return "FORBIDDEN"
	case KindDependencyFailure:
		return "DEPENDENCY_FAILURE"
	case KindTransient:
		return "TRANSIENT"
	case KindInvariantViolation:
		return "INVARIANT_VIOLATION"
	case KindInternal:
		return "INTERNAL"
	default:
		return "UNKNOWN"
	}
}

// IsRetryable returns true if the error kind is retryable.
func (k ErrorKind) IsRetryable() bool {
	return k == KindTransient || k == KindDependencyFailure
}

// IsClientError returns true if the error is caused by the client.
func (k ErrorKind) IsClientError() bool {
	switch k {
	case KindInvalidInput, KindNotFound, KindConflict, KindUnauthorized, KindForbidden, KindInvariantViolation:
		return true
	default:
		return false
	}
}
