package validation

import (
	"fmt"
	"net/url"
	"regexp"
	"time"
)

// Validator は入力値検証のインターフェース。
type Validator interface {
	ValidateEmail(email string) error
	ValidateUUID(id string) error
	ValidateURL(rawURL string) error
	ValidateTenantID(tenantID string) error
	ValidatePagination(page, perPage int) error
	ValidateDateRange(startDate, endDate time.Time) error
}

// ValidationError は検証エラー。
type ValidationError struct {
	Field   string
	Message string
	Code    string
}

func (e *ValidationError) Error() string {
	return fmt.Sprintf("%s: %s", e.Field, e.Message)
}

// ValidationErrors は複数のValidationErrorを保持するコレクション。
type ValidationErrors struct {
	errors []*ValidationError
}

// NewValidationErrors は新しいValidationErrorsを生成する。
func NewValidationErrors() *ValidationErrors {
	return &ValidationErrors{}
}

// HasErrors はエラーが存在するかを返す。
func (ve *ValidationErrors) HasErrors() bool {
	return len(ve.errors) > 0
}

// GetErrors は全てのエラーを返す。
func (ve *ValidationErrors) GetErrors() []*ValidationError {
	return ve.errors
}

// Add はエラーをコレクションに追加する。
func (ve *ValidationErrors) Add(err *ValidationError) {
	ve.errors = append(ve.errors, err)
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
		return &ValidationError{Field: "email", Message: "無効なメールアドレスです", Code: "INVALID_EMAIL"}
	}
	return nil
}

func (v *DefaultValidator) ValidateUUID(id string) error {
	if !uuidRegex.MatchString(id) {
		return &ValidationError{Field: "uuid", Message: "無効なUUID v4です", Code: "INVALID_UUID"}
	}
	return nil
}

func (v *DefaultValidator) ValidateURL(rawURL string) error {
	u, err := url.Parse(rawURL)
	if err != nil || u.Scheme == "" || u.Host == "" {
		return &ValidationError{Field: "url", Message: "無効なURLです", Code: "INVALID_URL"}
	}
	if u.Scheme != "http" && u.Scheme != "https" {
		return &ValidationError{Field: "url", Message: "スキームはhttpまたはhttpsである必要があります", Code: "INVALID_URL"}
	}
	return nil
}

func (v *DefaultValidator) ValidateTenantID(tenantID string) error {
	if !tenantIDRegex.MatchString(tenantID) {
		return &ValidationError{Field: "tenant_id", Message: "テナントIDは3-63文字の小文字英数字とハイフンのみ使用可能です", Code: "INVALID_TENANT_ID"}
	}
	return nil
}

func (v *DefaultValidator) ValidatePagination(page, perPage int) error {
	if page < 1 {
		return &ValidationError{Field: "page", Message: fmt.Sprintf("pageは1以上である必要があります: %d", page), Code: "INVALID_PAGE"}
	}
	if perPage < 1 || perPage > 100 {
		return &ValidationError{Field: "per_page", Message: fmt.Sprintf("per_pageは1-100の範囲である必要があります: %d", perPage), Code: "INVALID_PER_PAGE"}
	}
	return nil
}

func (v *DefaultValidator) ValidateDateRange(startDate, endDate time.Time) error {
	if startDate.After(endDate) {
		return &ValidationError{
			Field:   "date_range",
			Message: fmt.Sprintf("開始日(%s)は終了日(%s)以前である必要があります", startDate.Format(time.RFC3339), endDate.Format(time.RFC3339)),
			Code:    "INVALID_DATE_RANGE",
		}
	}
	return nil
}
