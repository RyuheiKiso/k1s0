package dlq

import "fmt"

// DlqError は DLQ クライアントエラー。
type DlqError struct {
	Op         string
	StatusCode int
	Err        error
}

func (e *DlqError) Error() string {
	if e.StatusCode != 0 {
		return fmt.Sprintf("dlq %s: status %d: %v", e.Op, e.StatusCode, e.Err)
	}
	return fmt.Sprintf("dlq %s: %v", e.Op, e.Err)
}

func (e *DlqError) Unwrap() error {
	return e.Err
}
