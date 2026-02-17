package model

import "time"

// AuditLog は監査ログエントリを表す。
type AuditLog struct {
	ID         string            `json:"id" db:"id"`
	EventType  string            `json:"event_type" db:"event_type"`
	UserID     string            `json:"user_id" db:"user_id"`
	IPAddress  string            `json:"ip_address" db:"ip_address"`
	UserAgent  string            `json:"user_agent" db:"user_agent"`
	Resource   string            `json:"resource" db:"resource"`
	Action     string            `json:"action" db:"action"`
	Result     string            `json:"result" db:"result"`
	Metadata   map[string]string `json:"metadata" db:"metadata"`
	RecordedAt time.Time         `json:"recorded_at" db:"recorded_at"`
}
