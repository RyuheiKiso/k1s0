package k1s0error

import (
	"github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error/presentation"
)

// toPresKind converts ErrorKind to presentation.ErrorKind.
func toPresKind(kind ErrorKind) presentation.ErrorKind {
	return presentation.ErrorKind(kind)
}

// ToHTTPError converts an AppError to an HTTPError.
func (e *AppError) ToHTTPError() *presentation.HTTPError {
	input := &presentation.HTTPErrorInput{
		Kind:      toPresKind(e.Kind()),
		Message:   e.Message(),
		ErrorCode: e.ErrorCode().String(),
		Hint:      e.Hint(),
	}
	if e.context != nil {
		input.TraceID = e.context.TraceID
		input.RequestID = e.context.RequestID
	}
	return presentation.NewHTTPError(input)
}

// ToGRPCError converts an AppError to a GRPCError.
func (e *AppError) ToGRPCError() *presentation.GRPCError {
	input := &presentation.GRPCErrorInput{
		Kind:      toPresKind(e.Kind()),
		Message:   e.Message(),
		ErrorCode: e.ErrorCode().String(),
		Hint:      e.Hint(),
	}
	if e.context != nil {
		input.TraceID = e.context.TraceID
		input.RequestID = e.context.RequestID
	}
	return presentation.NewGRPCError(input)
}

// ToHTTPError converts a DomainError to an HTTPError.
func (e *DomainError) ToHTTPError() *presentation.HTTPError {
	appErr := FromDomainError(e)
	return appErr.ToHTTPError()
}

// ToGRPCError converts a DomainError to a GRPCError.
func (e *DomainError) ToGRPCError() *presentation.GRPCError {
	appErr := FromDomainError(e)
	return appErr.ToGRPCError()
}
