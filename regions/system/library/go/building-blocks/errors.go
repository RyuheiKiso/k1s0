package buildingblocks

import "fmt"

// ComponentError represents errors from building block operations.
type ComponentError struct {
	Component string
	Operation string
	Message   string
	Err       error
}

func (e *ComponentError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("[%s] %s: %s: %v", e.Component, e.Operation, e.Message, e.Err)
	}
	return fmt.Sprintf("[%s] %s: %s", e.Component, e.Operation, e.Message)
}

func (e *ComponentError) Unwrap() error {
	return e.Err
}

// ETagMismatchError indicates an optimistic concurrency conflict.
type ETagMismatchError struct {
	Key      string
	Expected *ETag
	Actual   *ETag
}

func (e *ETagMismatchError) Error() string {
	expected := "<nil>"
	actual := "<nil>"
	if e.Expected != nil {
		expected = e.Expected.Value
	}
	if e.Actual != nil {
		actual = e.Actual.Value
	}
	return fmt.Sprintf("etag mismatch for key %q: expected %q, got %q", e.Key, expected, actual)
}

// NewComponentError creates a new ComponentError.
func NewComponentError(component, operation, message string, err error) *ComponentError {
	return &ComponentError{Component: component, Operation: operation, Message: message, Err: err}
}
