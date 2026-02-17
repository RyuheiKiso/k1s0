package model

import (
	"encoding/json"
	"time"
)

// ConfigChangeLog は設定変更ログエントリを表す。
type ConfigChangeLog struct {
	ID            string          `json:"id" db:"id"`
	ConfigEntryID string          `json:"config_entry_id" db:"config_entry_id"`
	Namespace     string          `json:"namespace" db:"namespace"`
	Key           string          `json:"key" db:"key"`
	OldValue      json.RawMessage `json:"old_value" db:"old_value"`
	NewValue      json.RawMessage `json:"new_value" db:"new_value"`
	OldVersion    int             `json:"old_version" db:"old_version"`
	NewVersion    int             `json:"new_version" db:"new_version"`
	ChangeType    string          `json:"change_type" db:"change_type"`
	ChangedBy     string          `json:"changed_by" db:"changed_by"`
	ChangedAt     time.Time       `json:"changed_at" db:"changed_at"`
}
