// BFF サービス用の業務エラー型（E-T3-BFF-* 体系）。

// Package errors は tier3 BFF のドメインエラーを集約する。
package errors

import (
	"errors"
	"fmt"
)

// Category はエラーの大分類（HTTP status の根拠）。
type Category string

const (
	CategoryValidation   Category = "VALIDATION"
	CategoryUnauthorized Category = "UNAUTHORIZED"
	CategoryForbidden    Category = "FORBIDDEN"
	CategoryNotFound     Category = "NOT_FOUND"
	CategoryUpstream     Category = "UPSTREAM"
	CategoryInternal     Category = "INTERNAL"
)

// HTTPStatus はカテゴリから推奨 HTTP status を返す。
func (c Category) HTTPStatus() int {
	switch c {
	case CategoryValidation:
		return 400
	case CategoryUnauthorized:
		return 401
	case CategoryForbidden:
		return 403
	case CategoryNotFound:
		return 404
	case CategoryUpstream:
		return 502
	case CategoryInternal:
		return 500
	default:
		return 500
	}
}

// DomainError は BFF 業務エラーの基底型。
type DomainError struct {
	Category Category
	Code     string
	Message  string
	Cause    error
}

func (e *DomainError) Error() string {
	if e.Cause != nil {
		return fmt.Sprintf("%s [%s] %s: %v", e.Code, e.Category, e.Message, e.Cause)
	}
	return fmt.Sprintf("%s [%s] %s", e.Code, e.Category, e.Message)
}

func (e *DomainError) Unwrap() error { return e.Cause }

func New(cat Category, code, msg string) *DomainError {
	return &DomainError{Category: cat, Code: code, Message: msg}
}

func Wrap(cat Category, code, msg string, cause error) *DomainError {
	return &DomainError{Category: cat, Code: code, Message: msg, Cause: cause}
}

func AsDomainError(err error) (*DomainError, bool) {
	if err == nil {
		return nil, false
	}
	var de *DomainError
	if errors.As(err, &de) {
		return de, true
	}
	return nil, false
}
