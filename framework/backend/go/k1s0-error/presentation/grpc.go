package presentation

// GRPCStatusCode represents gRPC status codes.
type GRPCStatusCode int

const (
	// GRPCCodeOK indicates success.
	GRPCCodeOK GRPCStatusCode = 0
	// GRPCCodeCancelled indicates the operation was cancelled.
	GRPCCodeCancelled GRPCStatusCode = 1
	// GRPCCodeUnknown indicates an unknown error.
	GRPCCodeUnknown GRPCStatusCode = 2
	// GRPCCodeInvalidArgument indicates invalid argument.
	GRPCCodeInvalidArgument GRPCStatusCode = 3
	// GRPCCodeDeadlineExceeded indicates deadline exceeded.
	GRPCCodeDeadlineExceeded GRPCStatusCode = 4
	// GRPCCodeNotFound indicates not found.
	GRPCCodeNotFound GRPCStatusCode = 5
	// GRPCCodeAlreadyExists indicates already exists.
	GRPCCodeAlreadyExists GRPCStatusCode = 6
	// GRPCCodePermissionDenied indicates permission denied.
	GRPCCodePermissionDenied GRPCStatusCode = 7
	// GRPCCodeResourceExhausted indicates resource exhausted.
	GRPCCodeResourceExhausted GRPCStatusCode = 8
	// GRPCCodeFailedPrecondition indicates failed precondition.
	GRPCCodeFailedPrecondition GRPCStatusCode = 9
	// GRPCCodeAborted indicates aborted.
	GRPCCodeAborted GRPCStatusCode = 10
	// GRPCCodeOutOfRange indicates out of range.
	GRPCCodeOutOfRange GRPCStatusCode = 11
	// GRPCCodeUnimplemented indicates unimplemented.
	GRPCCodeUnimplemented GRPCStatusCode = 12
	// GRPCCodeInternal indicates internal error.
	GRPCCodeInternal GRPCStatusCode = 13
	// GRPCCodeUnavailable indicates service unavailable.
	GRPCCodeUnavailable GRPCStatusCode = 14
	// GRPCCodeDataLoss indicates data loss.
	GRPCCodeDataLoss GRPCStatusCode = 15
	// GRPCCodeUnauthenticated indicates unauthenticated.
	GRPCCodeUnauthenticated GRPCStatusCode = 16
)

// String returns the string representation of the gRPC status code.
func (c GRPCStatusCode) String() string {
	switch c {
	case GRPCCodeOK:
		return "OK"
	case GRPCCodeCancelled:
		return "CANCELLED"
	case GRPCCodeUnknown:
		return "UNKNOWN"
	case GRPCCodeInvalidArgument:
		return "INVALID_ARGUMENT"
	case GRPCCodeDeadlineExceeded:
		return "DEADLINE_EXCEEDED"
	case GRPCCodeNotFound:
		return "NOT_FOUND"
	case GRPCCodeAlreadyExists:
		return "ALREADY_EXISTS"
	case GRPCCodePermissionDenied:
		return "PERMISSION_DENIED"
	case GRPCCodeResourceExhausted:
		return "RESOURCE_EXHAUSTED"
	case GRPCCodeFailedPrecondition:
		return "FAILED_PRECONDITION"
	case GRPCCodeAborted:
		return "ABORTED"
	case GRPCCodeOutOfRange:
		return "OUT_OF_RANGE"
	case GRPCCodeUnimplemented:
		return "UNIMPLEMENTED"
	case GRPCCodeInternal:
		return "INTERNAL"
	case GRPCCodeUnavailable:
		return "UNAVAILABLE"
	case GRPCCodeDataLoss:
		return "DATA_LOSS"
	case GRPCCodeUnauthenticated:
		return "UNAUTHENTICATED"
	default:
		return "UNKNOWN"
	}
}

// GRPCStatus maps ErrorKind to gRPC status codes.
func GRPCStatus(kind ErrorKind) GRPCStatusCode {
	switch kind {
	case KindInvalidInput:
		return GRPCCodeInvalidArgument
	case KindNotFound:
		return GRPCCodeNotFound
	case KindConflict:
		return GRPCCodeAlreadyExists
	case KindUnauthorized:
		return GRPCCodeUnauthenticated
	case KindForbidden:
		return GRPCCodePermissionDenied
	case KindDependencyFailure:
		return GRPCCodeUnavailable
	case KindTransient:
		return GRPCCodeUnavailable
	case KindInvariantViolation:
		return GRPCCodeFailedPrecondition
	case KindInternal:
		return GRPCCodeInternal
	default:
		return GRPCCodeInternal
	}
}

// GRPCError represents a gRPC error response.
type GRPCError struct {
	statusCode GRPCStatusCode
	message    string
	errorCode  string
	traceID    string
	requestID  string
	metadata   map[string]string
}

// GRPCErrorInput represents the input for creating a GRPCError.
type GRPCErrorInput struct {
	Kind      ErrorKind
	Message   string
	ErrorCode string
	TraceID   string
	RequestID string
	Hint      string
}

// NewGRPCError creates a new GRPCError from the input.
func NewGRPCError(input *GRPCErrorInput) *GRPCError {
	grpcErr := &GRPCError{
		statusCode: GRPCStatus(input.Kind),
		message:    input.Message,
		errorCode:  input.ErrorCode,
		traceID:    input.TraceID,
		requestID:  input.RequestID,
		metadata:   make(map[string]string),
	}

	// Add standard metadata
	grpcErr.metadata["error_code"] = grpcErr.errorCode
	if grpcErr.traceID != "" {
		grpcErr.metadata["trace_id"] = grpcErr.traceID
	}
	if grpcErr.requestID != "" {
		grpcErr.metadata["request_id"] = grpcErr.requestID
	}
	if input.Hint != "" {
		grpcErr.metadata["hint"] = input.Hint
	}

	return grpcErr
}

// StatusCode returns the gRPC status code.
func (e *GRPCError) StatusCode() GRPCStatusCode {
	return e.statusCode
}

// Message returns the error message.
func (e *GRPCError) Message() string {
	return e.message
}

// ErrorCode returns the error code.
func (e *GRPCError) ErrorCode() string {
	return e.errorCode
}

// TraceID returns the trace ID if set.
func (e *GRPCError) TraceID() string {
	return e.traceID
}

// RequestID returns the request ID if set.
func (e *GRPCError) RequestID() string {
	return e.requestID
}

// Metadata returns the error metadata for gRPC trailers.
func (e *GRPCError) Metadata() map[string]string {
	return e.metadata
}

// Error implements the error interface.
func (e *GRPCError) Error() string {
	return e.message
}
