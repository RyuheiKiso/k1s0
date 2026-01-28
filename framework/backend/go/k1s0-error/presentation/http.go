// Package presentation provides HTTP and gRPC error representations.
package presentation

import (
	"encoding/json"
)

// ErrorKind represents the classification of domain errors.
// This mirrors the ErrorKind in the parent package to avoid import cycles.
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

// HTTPStatus maps ErrorKind to HTTP status codes.
func HTTPStatus(kind ErrorKind) int {
	switch kind {
	case KindInvalidInput:
		return 400
	case KindNotFound:
		return 404
	case KindConflict:
		return 409
	case KindUnauthorized:
		return 401
	case KindForbidden:
		return 403
	case KindDependencyFailure:
		return 502
	case KindTransient:
		return 503
	case KindInvariantViolation:
		return 422
	case KindInternal:
		return 500
	default:
		return 500
	}
}

// HTTPTitle returns the HTTP status title for an ErrorKind.
func HTTPTitle(kind ErrorKind) string {
	switch kind {
	case KindInvalidInput:
		return "Bad Request"
	case KindNotFound:
		return "Not Found"
	case KindConflict:
		return "Conflict"
	case KindUnauthorized:
		return "Unauthorized"
	case KindForbidden:
		return "Forbidden"
	case KindDependencyFailure:
		return "Bad Gateway"
	case KindTransient:
		return "Service Unavailable"
	case KindInvariantViolation:
		return "Unprocessable Entity"
	case KindInternal:
		return "Internal Server Error"
	default:
		return "Internal Server Error"
	}
}

// ProblemDetails represents an RFC 7807 problem details response.
type ProblemDetails struct {
	// Type is a URI reference that identifies the problem type.
	Type string `json:"type,omitempty"`
	// Title is a short, human-readable summary of the problem type.
	Title string `json:"title"`
	// Status is the HTTP status code.
	Status int `json:"status"`
	// Detail is a human-readable explanation specific to this occurrence.
	Detail string `json:"detail"`
	// Instance is a URI reference that identifies the specific occurrence.
	Instance string `json:"instance,omitempty"`
	// ErrorCode is the operational error code.
	ErrorCode string `json:"error_code"`
	// TraceID is the distributed trace ID.
	TraceID string `json:"trace_id,omitempty"`
	// RequestID is the request ID.
	RequestID string `json:"request_id,omitempty"`
	// Hint is an optional user-facing hint.
	Hint string `json:"hint,omitempty"`
}

// HTTPError represents an HTTP error response.
type HTTPError struct {
	statusCode int
	problem    *ProblemDetails
}

// HTTPErrorInput represents the input for creating an HTTPError.
type HTTPErrorInput struct {
	Kind      ErrorKind
	Message   string
	ErrorCode string
	TraceID   string
	RequestID string
	Hint      string
}

// NewHTTPError creates a new HTTPError from the input.
func NewHTTPError(input *HTTPErrorInput) *HTTPError {
	problem := &ProblemDetails{
		Status:    HTTPStatus(input.Kind),
		Title:     HTTPTitle(input.Kind),
		Detail:    input.Message,
		ErrorCode: input.ErrorCode,
		TraceID:   input.TraceID,
		RequestID: input.RequestID,
		Hint:      input.Hint,
	}

	return &HTTPError{
		statusCode: problem.Status,
		problem:    problem,
	}
}

// StatusCode returns the HTTP status code.
func (e *HTTPError) StatusCode() int {
	return e.statusCode
}

// Problem returns the ProblemDetails.
func (e *HTTPError) Problem() *ProblemDetails {
	return e.problem
}

// TraceID returns the trace ID if set.
func (e *HTTPError) TraceID() string {
	return e.problem.TraceID
}

// RequestID returns the request ID if set.
func (e *HTTPError) RequestID() string {
	return e.problem.RequestID
}

// Error implements the error interface.
func (e *HTTPError) Error() string {
	return e.problem.Detail
}

// JSON returns the JSON representation of the error.
func (e *HTTPError) JSON() ([]byte, error) {
	return json.Marshal(e.problem)
}

// ContentType returns the content type for HTTP responses.
func (e *HTTPError) ContentType() string {
	return "application/problem+json"
}
