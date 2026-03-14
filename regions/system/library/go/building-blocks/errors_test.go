package buildingblocks

import (
	"errors"
	"strings"
	"testing"
)

// NewComponentError がコンポーネント名・操作名・メッセージを持つエラーを正しく生成することを確認する。
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

// ComponentError の Error メソッドがコンポーネント名・操作名・メッセージを含む文字列を返すことを確認する。
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

// ComponentError の Error メソッドがラップされた内部エラーの内容も含むことを確認する。
func TestComponentErrorStringWithWrapped(t *testing.T) {
	inner := errors.New("timeout")
	err := NewComponentError("statestore", "Set", "operation failed", inner)
	s := err.Error()
	if !strings.Contains(s, "timeout") {
		t.Errorf("expected 'timeout' in error string, got %q", s)
	}
}

// ComponentError の Unwrap メソッドがラップされた内部エラーを返すことを確認する。
func TestComponentErrorUnwrap(t *testing.T) {
	inner := errors.New("underlying error")
	err := NewComponentError("binding", "Invoke", "failed", inner)
	if err.Unwrap() != inner {
		t.Errorf("Unwrap() did not return the wrapped error")
	}
}

// ComponentError の Unwrap メソッドが内部エラーなしの場合に nil を返すことを確認する。
func TestComponentErrorUnwrapNil(t *testing.T) {
	err := NewComponentError("binding", "Read", "no data", nil)
	if err.Unwrap() != nil {
		t.Errorf("expected Unwrap() to return nil, got %v", err.Unwrap())
	}
}

// ETagMismatchError の Error メソッドがキー・期待値・実際値を含む文字列を返すことを確認する。
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

// ETagMismatchError の Error メソッドが Expected と Actual が nil のとき "<nil>" を含む文字列を返すことを確認する。
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
