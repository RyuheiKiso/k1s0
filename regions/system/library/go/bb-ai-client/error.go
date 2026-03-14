package bbaiclient

import "fmt"

// APIError は AI ゲートウェイから返された HTTP エラーを表す。
type APIError struct {
	StatusCode int
	Message    string
}

// Error は error インターフェースを実装する。
func (e *APIError) Error() string {
	if e.Message != "" {
		return fmt.Sprintf("bbaiclient: API error %d: %s", e.StatusCode, e.Message)
	}
	return fmt.Sprintf("bbaiclient: API error %d", e.StatusCode)
}
