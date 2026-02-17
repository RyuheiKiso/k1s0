package model

import (
	"encoding/json"
	"time"
)

// ConfigEntry は設定エントリを表す。
type ConfigEntry struct {
	ID          string          `json:"id" db:"id"`
	Namespace   string          `json:"namespace" db:"namespace"`
	Key         string          `json:"key" db:"key"`
	ValueJSON   json.RawMessage `json:"value" db:"value_json"`
	Version     int             `json:"version" db:"version"`
	Description string          `json:"description" db:"description"`
	CreatedBy   string          `json:"created_by" db:"created_by"`
	UpdatedBy   string          `json:"updated_by" db:"updated_by"`
	CreatedAt   time.Time       `json:"created_at" db:"created_at"`
	UpdatedAt   time.Time       `json:"updated_at" db:"updated_at"`
}
