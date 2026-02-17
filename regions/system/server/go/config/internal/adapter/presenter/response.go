package presenter

import (
	"encoding/json"
	"time"
)

// ConfigEntryResponse は設定エントリの API レスポンス。
type ConfigEntryResponse struct {
	Namespace   string          `json:"namespace"`
	Key         string          `json:"key"`
	Value       json.RawMessage `json:"value"`
	Version     int             `json:"version"`
	Description string          `json:"description"`
	UpdatedBy   string          `json:"updated_by"`
	UpdatedAt   time.Time       `json:"updated_at"`
}

// PaginationResponse はページネーション情報のレスポンス。
type PaginationResponse struct {
	TotalCount int  `json:"total_count"`
	Page       int  `json:"page"`
	PageSize   int  `json:"page_size"`
	HasNext    bool `json:"has_next"`
}

// ListConfigsResponse は設定値一覧の API レスポンス。
type ListConfigsResponse struct {
	Entries    []ConfigEntryResponse `json:"entries"`
	Pagination PaginationResponse    `json:"pagination"`
}

// ServiceConfigEntryResponse はサービス向け設定エントリの API レスポンス。
type ServiceConfigEntryResponse struct {
	Namespace string          `json:"namespace"`
	Key       string          `json:"key"`
	Value     json.RawMessage `json:"value"`
}

// ServiceConfigResponse はサービス向け設定の API レスポンス。
type ServiceConfigResponse struct {
	ServiceName string                       `json:"service_name"`
	Entries     []ServiceConfigEntryResponse `json:"entries"`
}
