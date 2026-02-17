package grpc

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/k1s0-platform/system-server-go-config/internal/usecase"
)

// GetConfigExecutor は GetConfigUseCase の実行インターフェース。
type GetConfigExecutor interface {
	Execute(ctx context.Context, input usecase.GetConfigInput) (*usecase.GetConfigOutput, error)
}

// ListConfigsExecutor は ListConfigsUseCase の実行インターフェース。
type ListConfigsExecutor interface {
	Execute(ctx context.Context, input usecase.ListConfigsInput) (*usecase.ListConfigsOutput, error)
}

// GetServiceConfigExecutor は GetServiceConfigUseCase の実行インターフェース。
type GetServiceConfigExecutor interface {
	Execute(ctx context.Context, input usecase.GetServiceConfigInput) (*usecase.GetServiceConfigOutput, error)
}

// UpdateConfigExecutor は UpdateConfigUseCase の実行インターフェース。
type UpdateConfigExecutor interface {
	Execute(ctx context.Context, input usecase.UpdateConfigInput) (*usecase.UpdateConfigOutput, error)
}

// DeleteConfigExecutor は DeleteConfigUseCase の実行インターフェース。
type DeleteConfigExecutor interface {
	Execute(ctx context.Context, input usecase.DeleteConfigInput) error
}

// ConfigGRPCService は gRPC ConfigService の実装。
type ConfigGRPCService struct {
	getConfigUC        GetConfigExecutor
	listConfigsUC      ListConfigsExecutor
	getServiceConfigUC GetServiceConfigExecutor
	updateConfigUC     UpdateConfigExecutor
	deleteConfigUC     DeleteConfigExecutor
}

// NewConfigGRPCService は ConfigGRPCService のコンストラクタ。
func NewConfigGRPCService(
	getConfigUC GetConfigExecutor,
	listConfigsUC ListConfigsExecutor,
	getServiceConfigUC GetServiceConfigExecutor,
	updateConfigUC UpdateConfigExecutor,
	deleteConfigUC DeleteConfigExecutor,
) *ConfigGRPCService {
	return &ConfigGRPCService{
		getConfigUC:        getConfigUC,
		listConfigsUC:      listConfigsUC,
		getServiceConfigUC: getServiceConfigUC,
		updateConfigUC:     updateConfigUC,
		deleteConfigUC:     deleteConfigUC,
	}
}

// GetConfig は指定された namespace と key の設定値を取得する。
func (s *ConfigGRPCService) GetConfig(ctx context.Context, req *GetConfigRequest) (*GetConfigResponse, error) {
	if req.Namespace == "" || req.Key == "" {
		return nil, fmt.Errorf("rpc error: code = InvalidArgument desc = namespace and key are required")
	}

	output, err := s.getConfigUC.Execute(ctx, usecase.GetConfigInput{
		Namespace: req.Namespace,
		Key:       req.Key,
	})
	if err != nil {
		if isNotFoundError(err) {
			return nil, fmt.Errorf("rpc error: code = NotFound desc = config not found: %s/%s", req.Namespace, req.Key)
		}
		return nil, fmt.Errorf("rpc error: code = Internal desc = %v", err)
	}

	return &GetConfigResponse{
		Entry: outputToEntry(output.Namespace, output.Key, output.Value, output.Description, output.Version, output.UpdatedAt),
	}, nil
}

// ListConfigs は namespace 内の設定値を一覧取得する。
func (s *ConfigGRPCService) ListConfigs(ctx context.Context, req *ListConfigsRequest) (*ListConfigsResponse, error) {
	input := usecase.ListConfigsInput{
		Namespace: req.Namespace,
		Page:      1,
		PageSize:  20,
	}
	if req.Pagination != nil {
		input.Page = int(req.Pagination.Page)
		input.PageSize = int(req.Pagination.PageSize)
	}
	if req.Search != "" {
		input.Search = req.Search
	}

	output, err := s.listConfigsUC.Execute(ctx, input)
	if err != nil {
		return nil, fmt.Errorf("rpc error: code = Internal desc = %v", err)
	}

	entries := make([]*PbConfigEntry, 0, len(output.Entries))
	for _, e := range output.Entries {
		entries = append(entries, outputToEntry(e.Namespace, e.Key, e.Value, e.Description, e.Version, e.UpdatedAt))
	}

	return &ListConfigsResponse{
		Entries: entries,
		Pagination: &PaginationResult{
			TotalCount: int32(output.TotalCount),
			Page:       int32(output.Page),
			PageSize:   int32(output.PageSize),
			HasNext:    output.HasNext,
		},
	}, nil
}

// GetServiceConfig はサービス名に対応する設定値を一括取得する。
func (s *ConfigGRPCService) GetServiceConfig(ctx context.Context, req *GetServiceConfigRequest) (*GetServiceConfigResponse, error) {
	if req.ServiceName == "" {
		return nil, fmt.Errorf("rpc error: code = InvalidArgument desc = service_name is required")
	}

	output, err := s.getServiceConfigUC.Execute(ctx, usecase.GetServiceConfigInput{
		ServiceName: req.ServiceName,
	})
	if err != nil {
		if isNotFoundError(err) {
			return nil, fmt.Errorf("rpc error: code = NotFound desc = service config not found: %s", req.ServiceName)
		}
		return nil, fmt.Errorf("rpc error: code = Internal desc = %v", err)
	}

	entries := make([]*PbConfigEntry, 0, len(output.Entries))
	for _, e := range output.Entries {
		entries = append(entries, &PbConfigEntry{
			Namespace: e.Namespace,
			Key:       e.Key,
			Value:     e.Value,
		})
	}

	return &GetServiceConfigResponse{
		ServiceName: output.ServiceName,
		Entries:     entries,
	}, nil
}

// UpdateConfig は設定値を更新する。
func (s *ConfigGRPCService) UpdateConfig(ctx context.Context, req *UpdateConfigRequest) (*UpdateConfigResponse, error) {
	if req.Namespace == "" || req.Key == "" || req.UpdatedBy == "" || req.Value == nil {
		return nil, fmt.Errorf("rpc error: code = InvalidArgument desc = namespace, key, value, and updated_by are required")
	}

	output, err := s.updateConfigUC.Execute(ctx, usecase.UpdateConfigInput{
		Namespace:   req.Namespace,
		Key:         req.Key,
		Value:       req.Value,
		Version:     int(req.Version),
		Description: req.Description,
		UpdatedBy:   req.UpdatedBy,
	})
	if err != nil {
		if errors.Is(err, usecase.ErrVersionConflict) {
			return nil, fmt.Errorf("rpc error: code = Aborted desc = version conflict")
		}
		if isNotFoundError(err) {
			return nil, fmt.Errorf("rpc error: code = NotFound desc = config not found: %s/%s", req.Namespace, req.Key)
		}
		return nil, fmt.Errorf("rpc error: code = Internal desc = %v", err)
	}

	return &UpdateConfigResponse{
		Entry: outputToEntry(output.Namespace, output.Key, output.Value, output.Description, output.Version, output.UpdatedAt),
	}, nil
}

// DeleteConfig は設定値を削除する。
func (s *ConfigGRPCService) DeleteConfig(ctx context.Context, req *DeleteConfigRequest) (*DeleteConfigResponse, error) {
	if req.Namespace == "" || req.Key == "" || req.DeletedBy == "" {
		return nil, fmt.Errorf("rpc error: code = InvalidArgument desc = namespace, key, and deleted_by are required")
	}

	err := s.deleteConfigUC.Execute(ctx, usecase.DeleteConfigInput{
		Namespace: req.Namespace,
		Key:       req.Key,
		DeletedBy: req.DeletedBy,
	})
	if err != nil {
		if isNotFoundError(err) {
			return nil, fmt.Errorf("rpc error: code = NotFound desc = config not found: %s/%s", req.Namespace, req.Key)
		}
		return nil, fmt.Errorf("rpc error: code = Internal desc = %v", err)
	}

	return &DeleteConfigResponse{Success: true}, nil
}

// WatchConfig は設定値の変更を監視する（ストリーミング RPC）。
// 現時点では Unimplemented を返す。
func (s *ConfigGRPCService) WatchConfig(ctx context.Context, req *WatchConfigRequest) (*WatchConfigResponse, error) {
	return nil, fmt.Errorf("rpc error: code = Unimplemented desc = WatchConfig is not yet implemented")
}

// --- ヘルパー関数 ---

func isNotFoundError(err error) bool {
	return strings.Contains(err.Error(), "not found")
}

func outputToEntry(namespace, key string, value []byte, description string, version int, updatedAt time.Time) *PbConfigEntry {
	return &PbConfigEntry{
		Namespace:   namespace,
		Key:         key,
		Value:       value,
		Description: description,
		Version:     int32(version),
		UpdatedAt:   timeToTimestamp(updatedAt),
	}
}

func timeToTimestamp(t time.Time) *Timestamp {
	return &Timestamp{
		Seconds: t.Unix(),
		Nanos:   int32(t.Nanosecond()),
	}
}
