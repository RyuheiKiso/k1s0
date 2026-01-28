package k1s0validation

// GRPCBadRequest represents a gRPC BadRequest error detail.
// This is compatible with google.rpc.BadRequest from the googleapis/google/rpc/error_details.proto.
type GRPCBadRequest struct {
	// FieldViolations contains the field validation errors.
	FieldViolations []*GRPCFieldViolation `json:"field_violations,omitempty"`
}

// GRPCFieldViolation represents a single field violation in a BadRequest.
type GRPCFieldViolation struct {
	// Field is the field name. For nested messages, use dot notation.
	Field string `json:"field"`

	// Description is a human-readable error message.
	Description string `json:"description"`
}

// ToGRPCDetails converts ValidationErrors to gRPC BadRequest details.
func (v *ValidationErrors) ToGRPCDetails() *GRPCBadRequest {
	violations := make([]*GRPCFieldViolation, len(v.errors))
	for i, err := range v.errors {
		violations[i] = &GRPCFieldViolation{
			Field:       err.Field,
			Description: err.Message,
		}
	}

	return &GRPCBadRequest{
		FieldViolations: violations,
	}
}

// GRPCStatusCode returns the gRPC status code for validation errors.
func (v *ValidationErrors) GRPCStatusCode() int {
	// INVALID_ARGUMENT = 3
	return 3
}

// GRPCStatusMessage returns the status message for gRPC errors.
func (v *ValidationErrors) GRPCStatusMessage() string {
	if v.Len() == 0 {
		return "Validation succeeded"
	}
	return "Validation failed: " + v.Error()
}

// ToGRPCMetadata converts validation errors to gRPC metadata.
func (v *ValidationErrors) ToGRPCMetadata() map[string]string {
	metadata := make(map[string]string)
	metadata["error_code"] = "validation_error"
	metadata["error_count"] = string(rune('0' + v.Len()))

	// Add first few field names
	fields := v.Fields()
	if len(fields) > 0 {
		if len(fields) > 3 {
			fields = fields[:3]
		}
		metadata["error_fields"] = joinStrings(fields, ",")
	}

	return metadata
}

// joinStrings joins strings with a separator.
func joinStrings(strs []string, sep string) string {
	if len(strs) == 0 {
		return ""
	}
	result := strs[0]
	for _, s := range strs[1:] {
		result += sep + s
	}
	return result
}
