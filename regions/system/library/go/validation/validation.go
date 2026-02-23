package validation

import (
	"fmt"
	"net/url"
	"regexp"
)

// Validator は入力値検証のインターフェース。
type Validator interface {
	ValidateEmail(email string) error
	ValidateUUID(id string) error
	ValidateURL(rawURL string) error
	ValidateTenantID(tenantID string) error
}

// ValidationError は検証エラー。
type ValidationError struct {
	Field   string
	Message string
}

func (e *ValidationError) Error() string {
	return fmt.Sprintf("%s: %s", e.Field, e.Message)
}

var (
	emailRegex    = regexp.MustCompile(`^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$`)
	uuidRegex     = regexp.MustCompile(`^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$`)
	tenantIDRegex = regexp.MustCompile(`^[a-z0-9\-]{3,63}$`)
)

// DefaultValidator はデフォルト実装。
type DefaultValidator struct{}

// NewDefaultValidator は新しい DefaultValidator を生成する。
func NewDefaultValidator() *DefaultValidator {
	return &DefaultValidator{}
}

func (v *DefaultValidator) ValidateEmail(email string) error {
	if !emailRegex.MatchString(email) {
		return &ValidationError{Field: "email", Message: "無効なメールアドレスです"}
	}
	return nil
}

func (v *DefaultValidator) ValidateUUID(id string) error {
	if !uuidRegex.MatchString(id) {
		return &ValidationError{Field: "uuid", Message: "無効なUUID v4です"}
	}
	return nil
}

func (v *DefaultValidator) ValidateURL(rawURL string) error {
	u, err := url.Parse(rawURL)
	if err != nil || u.Scheme == "" || u.Host == "" {
		return &ValidationError{Field: "url", Message: "無効なURLです"}
	}
	if u.Scheme != "http" && u.Scheme != "https" {
		return &ValidationError{Field: "url", Message: "スキームはhttpまたはhttpsである必要があります"}
	}
	return nil
}

func (v *DefaultValidator) ValidateTenantID(tenantID string) error {
	if !tenantIDRegex.MatchString(tenantID) {
		return &ValidationError{Field: "tenant_id", Message: "テナントIDは3-63文字の小文字英数字とハイフンのみ使用可能です"}
	}
	return nil
}
