// 本ファイルは BFF DomainError の単体テスト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//
// テスト観点:
//   - Category → HTTPStatus マッピングが docs §「HTTP Status ↔ K1s0Error」表と一致
//   - Wrap() / New() の error chain が errors.Is / errors.As で解決される
//   - AsDomainError は nil / 非 DomainError を判定する

package errors

import (
	"errors"
	"testing"
)

func TestCategoryHTTPStatus(t *testing.T) {
	cases := []struct {
		cat  Category
		want int
	}{
		{CategoryValidation, 400},
		{CategoryUnauthorized, 401},
		{CategoryForbidden, 403},
		{CategoryNotFound, 404},
		{CategoryUpstream, 502},
		{CategoryInternal, 500},
		// 未知のカテゴリは Internal にフォールバック。
		{Category("UNKNOWN"), 500},
	}
	for _, c := range cases {
		got := c.cat.HTTPStatus()
		if got != c.want {
			t.Errorf("HTTPStatus(%q) = %d, want %d", c.cat, got, c.want)
		}
	}
}

func TestNewProducesDomainError(t *testing.T) {
	err := New(CategoryNotFound, "E-T3-BFF-X", "missing")
	if err.Category != CategoryNotFound {
		t.Errorf("Category = %q, want NOT_FOUND", err.Category)
	}
	if err.Code != "E-T3-BFF-X" {
		t.Errorf("Code = %q", err.Code)
	}
	if err.Message != "missing" {
		t.Errorf("Message = %q", err.Message)
	}
	if err.Cause != nil {
		t.Errorf("Cause should be nil for New, got %v", err.Cause)
	}
}

func TestWrapPreservesCause(t *testing.T) {
	cause := errors.New("upstream io error")
	err := Wrap(CategoryUpstream, "E-T3-BFF-Y", "cannot reach", cause)
	if err.Cause != cause {
		t.Errorf("Cause not preserved")
	}
	// Unwrap chain: errors.Is は cause まで届く。
	if !errors.Is(err, cause) {
		t.Errorf("errors.Is should match wrapped cause")
	}
}

func TestErrorStringFormatsBoth(t *testing.T) {
	// without cause
	a := New(CategoryValidation, "C1", "bad")
	if got := a.Error(); got != "C1 [VALIDATION] bad" {
		t.Errorf("without cause: %q", got)
	}
	// with cause
	b := Wrap(CategoryUpstream, "C2", "down", errors.New("503"))
	if got := b.Error(); got != "C2 [UPSTREAM] down: 503" {
		t.Errorf("with cause: %q", got)
	}
}

func TestAsDomainErrorClassifies(t *testing.T) {
	// nil
	if de, ok := AsDomainError(nil); ok || de != nil {
		t.Errorf("nil should return (nil, false), got (%v, %v)", de, ok)
	}
	// 非 DomainError
	if de, ok := AsDomainError(errors.New("plain")); ok || de != nil {
		t.Errorf("plain error should return (nil, false), got (%v, %v)", de, ok)
	}
	// DomainError
	src := New(CategoryForbidden, "C3", "denied")
	got, ok := AsDomainError(src)
	if !ok || got != src {
		t.Errorf("DomainError should be returned as-is")
	}
	// errors.As チェーンを通る wrap
	wrapped := Wrap(CategoryUpstream, "C4", "outer", src)
	if got2, ok2 := AsDomainError(wrapped); !ok2 || got2 == nil {
		t.Errorf("wrapped DomainError should be unwrapped")
	}
}
