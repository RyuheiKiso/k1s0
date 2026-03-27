package saga

import "fmt"

// SagaError は Saga クライアントのエラー。
type SagaError struct {
	Op         string
	StatusCode int
	Err        error
}

func (e *SagaError) Error() string {
	if e.StatusCode > 0 {
		return fmt.Sprintf("saga %s: status %d: %v", e.Op, e.StatusCode, e.Err)
	}
	return fmt.Sprintf("saga %s: %v", e.Op, e.Err)
}

func (e *SagaError) Unwrap() error {
	return e.Err
}

// Err は SagaError の短縮エイリアス（L-3 監査対応: stutter 命名解消）。
// 注意: builtin error インターフェースとの混同を避けるため Err を使用する。
// 新しいコードでは saga.Err を使用すること。
type Err = SagaError
