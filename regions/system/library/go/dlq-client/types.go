package dlq

import "time"

// DlqStatus は DLQ メッセージのステータス。
type DlqStatus string

const (
	DlqStatusPending  DlqStatus = "PENDING"
	DlqStatusRetrying DlqStatus = "RETRYING"
	DlqStatusResolved DlqStatus = "RESOLVED"
	DlqStatusDead     DlqStatus = "DEAD"
)

// DlqMessage は DLQ メッセージ。
type DlqMessage struct {
	ID            string     `json:"id"`
	OriginalTopic string     `json:"original_topic"`
	ErrorMessage  string     `json:"error_message"`
	RetryCount    int        `json:"retry_count"`
	MaxRetries    int        `json:"max_retries"`
	Payload       any        `json:"payload"`
	Status        DlqStatus  `json:"status"`
	CreatedAt     time.Time  `json:"created_at"`
	UpdatedAt     *time.Time `json:"updated_at,omitempty"`
	LastRetryAt   *time.Time `json:"last_retry_at"`
}

// ListDlqMessagesRequest は DLQ メッセージ一覧取得リクエスト。
// PageSize は 1〜1000 の範囲で必須。過大なページサイズによるメモリ枯渇を防止する（M-01 監査対応）。
type ListDlqMessagesRequest struct {
	Topic    string `json:"topic"`
	Page     int    `json:"page"     validate:"min=1"`
	PageSize int    `json:"page_size" validate:"min=1,max=1000"`
}

// ListDlqMessagesResponse は DLQ メッセージ一覧取得レスポンス。
type ListDlqMessagesResponse struct {
	Messages []DlqMessage `json:"messages"`
	Total    int          `json:"total"`
	Page     int          `json:"page"`
}

// RetryDlqMessageResponse は DLQ メッセージ再処理レスポンス。
type RetryDlqMessageResponse struct {
	MessageID string    `json:"message_id"`
	Status    DlqStatus `json:"status"`
}

// ---------------------------------------------------------------------------
// L-3 監査対応: Go 命名規約準拠の短縮型エイリアス（stutter 命名解消）
// 新しいコードでは dlq.Status / Message を使用すること。
// ---------------------------------------------------------------------------

// Status は DlqStatus の短縮エイリアス。
type Status = DlqStatus

// Message は DlqMessage の短縮エイリアス。
type Message = DlqMessage
