package buildingblocks

import (
	"errors"
	"strings"
	"testing"
)

func TestNewComponentError(t *testing.T) {
	err := NewComponentError("statestore", "Get", "key not found", nil)
	if err.Component != "statestore" {
		t.Errorf("expected Component 'statestore', got %q", err.Component)
	}
	if err.Operation != "Get" {
		t.Errorf("expected Operation 'Get', got %q", err.Operation)
	}
	if err.Message != "key not found" {
		t.Errorf("expected Message 'key not found', got %q", err.Message)
	}
}

func TestComponentErrorString(t *testing.T) {
	err := NewComponentError("pubsub", "Publish", "connection failed", nil)
	s := err.Error()
	if !strings.Contains(s, "[pubsub]") {
		t.Errorf("expected '[pubsub]' in error string, got %q", s)
	}
	if !strings.Contains(s, "Publish") {
		t.Errorf("expected 'Publish' in error string, got %q", s)
	}
	if !strings.Contains(s, "connection failed") {
		t.Errorf("expected 'connection failed' in error string, got %q", s)
	}
}

func TestComponentErrorStringWithWrapped(t *testing.T) {
	inner := errors.New("timeout")
	err := NewComponentError("statestore", "Set", "operation failed", inner)
	s := err.Error()
	if !strings.Contains(s, "timeout") {
		t.Errorf("expected 'timeout' in error string, got %q", s)
	}
}

func TestComponentErrorUnwrap(t *testing.T) {
	inner := errors.New("underlying error")
	err := NewComponentError("binding", "Invoke", "failed", inner)
	if err.Unwrap() != inner {
		t.Errorf("Unwrap() did not return the wrapped error")
	}
}

func TestComponentErrorUnwrapNil(t *testing.T) {
	err := NewComponentError("binding", "Read", "no data", nil)
	if err.Unwrap() != nil {
		t.Errorf("expected Unwrap() to return nil, got %v", err.Unwrap())
	}
}

func TestETagMismatchError(t *testing.T) {
	err := &ETagMismatchError{
		Key:      "user:1",
		Expected: &ETag{Value: "abc"},
		Actual:   &ETag{Value: "def"},
	}
	s := err.Error()
	if !strings.Contains(s, "user:1") {
		t.Errorf("expected 'user:1' in error string, got %q", s)
	}
	if !strings.Contains(s, "abc") {
		t.Errorf("expected 'abc' in error string, got %q", s)
	}
	if !strings.Contains(s, "def") {
		t.Errorf("expected 'def' in error string, got %q", s)
	}
}

func TestETagMismatchErrorNilETags(t *testing.T) {
	err := &ETagMismatchError{
		Key:      "key",
		Expected: nil,
		Actual:   nil,
	}
	s := err.Error()
	if !strings.Contains(s, "<nil>") {
		t.Errorf("expected '<nil>' in error string, got %q", s)
	}
}
