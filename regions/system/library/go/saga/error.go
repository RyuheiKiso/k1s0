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
