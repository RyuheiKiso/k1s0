package k1s0error

import "fmt"

// DomainError represents a domain layer error.
// It is transport-independent and does not contain HTTP/gRPC details.
type DomainError struct {
	kind          ErrorKind
	message       string
	resourceType  string
	resourceID    string
	sourceMessage string
}

// NewDomainError creates a new DomainError with the given kind and message.
func NewDomainError(kind ErrorKind, message string) *DomainError {
	return &DomainError{
		kind:    kind,
		message: message,
	}
}

// InvalidInput creates a new invalid input error.
func InvalidInput(message string) *DomainError {
	return NewDomainError(KindInvalidInput, message)
}

// NotFound creates a new not found error for a specific resource.
func NotFound(resourceType, resourceID string) *DomainError {
	return &DomainError{
		kind:         KindNotFound,
		message:      fmt.Sprintf("%s '%s' not found", resourceType, resourceID),
		resourceType: resourceType,
		resourceID:   resourceID,
	}
}

// Conflict creates a new conflict error.
func Conflict(message string) *DomainError {
	return NewDomainError(KindConflict, message)
}

// Duplicate creates a new duplicate error for a specific field.
func Duplicate(resourceType, field string) *DomainError {
	return &DomainError{
		kind:         KindConflict,
		message:      fmt.Sprintf("%s %s already exists", resourceType, field),
		resourceType: resourceType,
	}
}

// Unauthorized creates a new authentication error.
func Unauthorized(message string) *DomainError {
	return NewDomainError(KindUnauthorized, message)
}

// Forbidden creates a new authorization error.
func Forbidden(message string) *DomainError {
	return NewDomainError(KindForbidden, message)
}

// DependencyFailure creates a new dependency failure error.
func DependencyFailure(dependency, message string) *DomainError {
	return &DomainError{
		kind:          KindDependencyFailure,
		message:       fmt.Sprintf("%s: %s", dependency, message),
		resourceType:  dependency,
		sourceMessage: message,
	}
}

// Transient creates a new transient (retryable) error.
func Transient(message string) *DomainError {
	return NewDomainError(KindTransient, message)
}

// InvariantViolation creates a new business rule violation error.
func InvariantViolation(message string) *DomainError {
	return NewDomainError(KindInvariantViolation, message)
}

// Internal creates a new internal error.
func Internal(message string) *DomainError {
	return NewDomainError(KindInternal, message)
}

// WithSource sets the source error message.
func (e *DomainError) WithSource(source string) *DomainError {
	e.sourceMessage = source
	return e
}

// WithResource sets the resource type and ID.
func (e *DomainError) WithResource(resourceType, resourceID string) *DomainError {
	e.resourceType = resourceType
	e.resourceID = resourceID
	return e
}

// Kind returns the error kind.
func (e *DomainError) Kind() ErrorKind {
	return e.kind
}

// Message returns the error message.
func (e *DomainError) Message() string {
	return e.message
}

// ResourceType returns the resource type if set.
func (e *DomainError) ResourceType() string {
	return e.resourceType
}

// ResourceID returns the resource ID if set.
func (e *DomainError) ResourceID() string {
	return e.resourceID
}

// SourceMessage returns the source error message if set.
func (e *DomainError) SourceMessage() string {
	return e.sourceMessage
}

// IsRetryable returns true if the error is retryable.
func (e *DomainError) IsRetryable() bool {
	return e.kind.IsRetryable()
}

// Error implements the error interface.
func (e *DomainError) Error() string {
	return e.message
}
