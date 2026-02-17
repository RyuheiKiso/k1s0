package grpc

import "encoding/json"

// proto 生成コードが未生成のため、config_service.proto と common/types.proto に
// 対応する Go 構造体を手動定義する。buf generate 後にこのファイルは生成コードに置き換える。

// --- Common Types ---

// Pagination はページネーションパラメータ。
type Pagination struct {
	Page     int32 `json:"page"`
	PageSize int32 `json:"page_size"`
}

// PaginationResult はページネーション結果。
type PaginationResult struct {
	TotalCount int32 `json:"total_count"`
	Page       int32 `json:"page"`
	PageSize   int32 `json:"page_size"`
	HasNext    bool  `json:"has_next"`
}

// Timestamp は protobuf Timestamp 互換型。
type Timestamp struct {
	Seconds int64 `json:"seconds"`
	Nanos   int32 `json:"nanos"`
}

// --- ConfigEntry ---

// PbConfigEntry は proto の ConfigEntry に対応する構造体。
type PbConfigEntry struct {
	Namespace   string          `json:"namespace"`
	Key         string          `json:"key"`
	Value       json.RawMessage `json:"value"`
	Description string          `json:"description"`
	Version     int32           `json:"version"`
	CreatedAt   *Timestamp      `json:"created_at,omitempty"`
	UpdatedAt   *Timestamp      `json:"updated_at,omitempty"`
}

// --- GetConfig ---

// GetConfigRequest は設定値取得リクエスト。
type GetConfigRequest struct {
	Namespace string `json:"namespace"`
	Key       string `json:"key"`
}

// GetConfigResponse は設定値取得レスポンス。
type GetConfigResponse struct {
	Entry *PbConfigEntry `json:"entry"`
}

// --- ListConfigs ---

// ListConfigsRequest は設定値一覧取得リクエスト。
type ListConfigsRequest struct {
	Namespace  string      `json:"namespace"`
	Pagination *Pagination `json:"pagination,omitempty"`
	Search     string      `json:"search,omitempty"`
}

// ListConfigsResponse は設定値一覧取得レスポンス。
type ListConfigsResponse struct {
	Entries    []*PbConfigEntry  `json:"entries"`
	Pagination *PaginationResult `json:"pagination,omitempty"`
}

// --- GetServiceConfig ---

// GetServiceConfigRequest はサービス設定取得リクエスト。
type GetServiceConfigRequest struct {
	ServiceName string `json:"service_name"`
}

// GetServiceConfigResponse はサービス設定取得レスポンス。
type GetServiceConfigResponse struct {
	ServiceName string           `json:"service_name"`
	Entries     []*PbConfigEntry `json:"entries"`
}

// --- UpdateConfig ---

// UpdateConfigRequest は設定値更新リクエスト。
type UpdateConfigRequest struct {
	Namespace   string          `json:"namespace"`
	Key         string          `json:"key"`
	Value       json.RawMessage `json:"value"`
	Version     int32           `json:"version"`
	Description string          `json:"description,omitempty"`
	UpdatedBy   string          `json:"updated_by"`
}

// UpdateConfigResponse は設定値更新レスポンス。
type UpdateConfigResponse struct {
	Entry *PbConfigEntry `json:"entry"`
}

// --- DeleteConfig ---

// DeleteConfigRequest は設定値削除リクエスト。
type DeleteConfigRequest struct {
	Namespace string `json:"namespace"`
	Key       string `json:"key"`
	DeletedBy string `json:"deleted_by"`
}

// DeleteConfigResponse は設定値削除レスポンス。
type DeleteConfigResponse struct {
	Success bool `json:"success"`
}

// --- WatchConfig ---

// WatchConfigRequest は設定値監視リクエスト。
type WatchConfigRequest struct {
	Namespace string `json:"namespace"`
	Key       string `json:"key,omitempty"`
}

// WatchConfigResponse は設定値監視レスポンス。
type WatchConfigResponse struct {
	ChangeType string         `json:"change_type"`
	Entry      *PbConfigEntry `json:"entry"`
}
