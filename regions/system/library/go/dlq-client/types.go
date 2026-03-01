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
type ListDlqMessagesRequest struct {
	Topic    string `json:"topic"`
	Page     int    `json:"page"`
	PageSize int    `json:"page_size"`
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
