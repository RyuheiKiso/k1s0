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

// Err は DlqError の短縮エイリアス（L-3 監査対応: stutter 命名解消）。
// 注意: builtin error との混同を避けるため Err を使用する。
// 新しいコードでは dlq.Err を使用すること。
type Err = DlqError
