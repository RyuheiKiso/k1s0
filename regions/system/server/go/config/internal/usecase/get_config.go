package usecase

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/k1s0-platform/system-server-go-config/internal/domain/repository"
)

// GetConfigUseCase は設定値取得ユースケース。
type GetConfigUseCase struct {
	configRepo repository.ConfigRepository
}

// NewGetConfigUseCase は新しい GetConfigUseCase を作成する。
func NewGetConfigUseCase(
	configRepo repository.ConfigRepository,
) *GetConfigUseCase {
	return &GetConfigUseCase{
		configRepo: configRepo,
	}
}

// GetConfigInput は設定値取得の入力パラメータ。
type GetConfigInput struct {
	Namespace string `json:"namespace" validate:"required"`
	Key       string `json:"key" validate:"required"`
}

// GetConfigOutput は設定値取得の出力。
type GetConfigOutput struct {
	Namespace   string          `json:"namespace"`
	Key         string          `json:"key"`
	Value       json.RawMessage `json:"value"`
	Version     int             `json:"version"`
	Description string          `json:"description"`
	UpdatedBy   string          `json:"updated_by"`
	UpdatedAt   time.Time       `json:"updated_at"`
}

// Execute は指定された namespace と key の設定値を取得する。
func (uc *GetConfigUseCase) Execute(ctx context.Context, input GetConfigInput) (*GetConfigOutput, error) {
	entry, err := uc.configRepo.GetByKey(ctx, input.Namespace, input.Key)
	if err != nil {
		return nil, fmt.Errorf("failed to get config entry: %w", err)
	}

	return &GetConfigOutput{
		Namespace:   entry.Namespace,
		Key:         entry.Key,
		Value:       entry.ValueJSON,
		Version:     entry.Version,
		Description: entry.Description,
		UpdatedBy:   entry.UpdatedBy,
		UpdatedAt:   entry.UpdatedAt,
	}, nil
}
