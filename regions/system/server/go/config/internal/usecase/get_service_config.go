package usecase

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/k1s0-platform/system-server-go-config/internal/domain/repository"
)

// GetServiceConfigUseCase はサービス向け設定一括取得ユースケース。
type GetServiceConfigUseCase struct {
	configRepo repository.ConfigRepository
}

// NewGetServiceConfigUseCase は新しい GetServiceConfigUseCase を作成する。
func NewGetServiceConfigUseCase(
	configRepo repository.ConfigRepository,
) *GetServiceConfigUseCase {
	return &GetServiceConfigUseCase{
		configRepo: configRepo,
	}
}

// GetServiceConfigInput はサービス向け設定取得の入力パラメータ。
type GetServiceConfigInput struct {
	ServiceName string `json:"service_name" validate:"required"`
}

// ServiceConfigEntry はサービス向け設定の1エントリ。
type ServiceConfigEntry struct {
	Namespace string          `json:"namespace"`
	Key       string          `json:"key"`
	Value     json.RawMessage `json:"value"`
}

// GetServiceConfigOutput はサービス向け設定取得の出力。
type GetServiceConfigOutput struct {
	ServiceName string               `json:"service_name"`
	Entries     []ServiceConfigEntry `json:"entries"`
}

// Execute はサービス名に対応する設定値を一括取得する。
func (uc *GetServiceConfigUseCase) Execute(ctx context.Context, input GetServiceConfigInput) (*GetServiceConfigOutput, error) {
	entries, err := uc.configRepo.GetByServiceName(ctx, input.ServiceName)
	if err != nil {
		return nil, fmt.Errorf("failed to get service config: %w", err)
	}

	if len(entries) == 0 {
		return nil, fmt.Errorf("service config not found: %s", input.ServiceName)
	}

	outputEntries := make([]ServiceConfigEntry, 0, len(entries))
	for _, e := range entries {
		outputEntries = append(outputEntries, ServiceConfigEntry{
			Namespace: e.Namespace,
			Key:       e.Key,
			Value:     e.ValueJSON,
		})
	}

	return &GetServiceConfigOutput{
		ServiceName: input.ServiceName,
		Entries:     outputEntries,
	}, nil
}
