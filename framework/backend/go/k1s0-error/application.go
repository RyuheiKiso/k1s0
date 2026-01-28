package k1s0error

import "fmt"

// AppError represents an application layer error.
// It wraps a DomainError and adds an error code and context for correlation.
type AppError struct {
	domainError *DomainError
	errorCode   ErrorCode
	context     *ErrorContext
	hint        string
}

// NewAppError creates a new AppError from a DomainError with an explicit error code.
func NewAppError(domainError *DomainError, errorCode ErrorCode) *AppError {
	return &AppError{
		domainError: domainError,
		errorCode:   errorCode,
		context:     NewErrorContext(),
	}
}

// FromDomainError creates an AppError from a DomainError with an automatic error code.
func FromDomainError(domainError *DomainError) *AppError {
	return &AppError{
		domainError: domainError,
		errorCode:   DefaultCodeForKind(domainError.Kind()),
		context:     NewErrorContext(),
	}
}

// WithTraceID sets the trace ID and returns the error.
func (e *AppError) WithTraceID(traceID string) *AppError {
	e.context.WithTraceID(traceID)
	return e
}

// WithRequestID sets the request ID and returns the error.
func (e *AppError) WithRequestID(requestID string) *AppError {
	e.context.WithRequestID(requestID)
	return e
}

// WithContext sets the error context and returns the error.
func (e *AppError) WithContext(ctx *ErrorContext) *AppError {
	e.context.Merge(ctx)
	return e
}

// WithHint sets a hint message for the user and returns the error.
func (e *AppError) WithHint(hint string) *AppError {
	e.hint = hint
	return e
}

// Kind returns the error kind.
func (e *AppError) Kind() ErrorKind {
	return e.domainError.Kind()
}

// ErrorCode returns the error code.
func (e *AppError) ErrorCode() ErrorCode {
	return e.errorCode
}

// Message returns the error message.
func (e *AppError) Message() string {
	return e.domainError.Message()
}

// DomainError returns the underlying domain error.
func (e *AppError) DomainError() *DomainError {
	return e.domainError
}

// Context returns the error context.
func (e *AppError) Context() *ErrorContext {
	return e.context
}

// Hint returns the hint message if set.
func (e *AppError) Hint() string {
	return e.hint
}

// IsRetryable returns true if the error is retryable.
func (e *AppError) IsRetryable() bool {
	return e.domainError.IsRetryable()
}

// Error implements the error interface.
func (e *AppError) Error() string {
	return fmt.Sprintf("[%s] %s", e.errorCode, e.domainError.Error())
}

// Unwrap returns the underlying domain error for errors.Is/As support.
func (e *AppError) Unwrap() error {
	return e.domainError
}
