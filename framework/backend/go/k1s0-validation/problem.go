package k1s0validation

import "encoding/json"

// ProblemDetails represents an RFC 7807 problem details response for validation errors.
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

	// Errors contains the individual field validation errors.
	Errors []FieldErrorResponse `json:"errors,omitempty"`
}

// FieldErrorResponse is the JSON representation of a field error.
type FieldErrorResponse struct {
	Field   string `json:"field"`
	Message string `json:"message"`
	Code    string `json:"code,omitempty"`
}

// ToProblemDetails converts ValidationErrors to RFC 7807 ProblemDetails.
func (v *ValidationErrors) ToProblemDetails() *ProblemDetails {
	fieldErrors := make([]FieldErrorResponse, len(v.errors))
	for i, err := range v.errors {
		fieldErrors[i] = FieldErrorResponse{
			Field:   err.Field,
			Message: err.Message,
			Code:    string(err.Kind),
		}
	}

	detail := "Validation failed"
	if v.Len() == 1 {
		detail = v.errors[0].Message
	} else if v.Len() > 1 {
		detail = "Multiple validation errors occurred"
	}

	return &ProblemDetails{
		Type:      "https://k1s0.dev/problems/validation-error",
		Title:     "Validation Error",
		Status:    400,
		Detail:    detail,
		ErrorCode: "validation_error",
		Errors:    fieldErrors,
	}
}

// WithTraceID sets the trace ID on the problem details.
func (p *ProblemDetails) WithTraceID(traceID string) *ProblemDetails {
	p.TraceID = traceID
	return p
}

// WithRequestID sets the request ID on the problem details.
func (p *ProblemDetails) WithRequestID(requestID string) *ProblemDetails {
	p.RequestID = requestID
	return p
}

// WithInstance sets the instance URI on the problem details.
func (p *ProblemDetails) WithInstance(instance string) *ProblemDetails {
	p.Instance = instance
	return p
}

// JSON returns the JSON representation.
func (p *ProblemDetails) JSON() ([]byte, error) {
	return json.Marshal(p)
}

// ContentType returns the content type for the response.
func (p *ProblemDetails) ContentType() string {
	return "application/problem+json"
}
