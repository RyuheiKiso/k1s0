package usecase

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"

	"github.com/k1s0-platform/system-server-go-config/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-config/internal/domain/repository"
)

// ErrVersionConflict はバージョン競合エラー。
var ErrVersionConflict = errors.New("version conflict")

// ConfigChangeEventPublisher は設定変更イベントの非同期配信インターフェース。
type ConfigChangeEventPublisher interface {
	Publish(ctx context.Context, log *model.ConfigChangeLog) error
}

// UpdateConfigUseCase は設定値更新ユースケース。
type UpdateConfigUseCase struct {
	configRepo repository.ConfigRepository
	publisher  ConfigChangeEventPublisher
}

// NewUpdateConfigUseCase は新しい UpdateConfigUseCase を作成する。
func NewUpdateConfigUseCase(
	configRepo repository.ConfigRepository,
	publisher ConfigChangeEventPublisher,
) *UpdateConfigUseCase {
	return &UpdateConfigUseCase{
		configRepo: configRepo,
		publisher:  publisher,
	}
}

// UpdateConfigInput は設定値更新の入力パラメータ。
type UpdateConfigInput struct {
	Namespace   string          `json:"namespace" validate:"required"`
	Key         string          `json:"key" validate:"required"`
	Value       json.RawMessage `json:"value" validate:"required"`
	Version     int             `json:"version" validate:"required"`
	Description string          `json:"description"`
	UpdatedBy   string          `json:"updated_by" validate:"required"`
}

// UpdateConfigOutput は設定値更新の出力。
type UpdateConfigOutput struct {
	Namespace   string          `json:"namespace"`
	Key         string          `json:"key"`
	Value       json.RawMessage `json:"value"`
	Version     int             `json:"version"`
	Description string          `json:"description"`
	UpdatedBy   string          `json:"updated_by"`
	UpdatedAt   time.Time       `json:"updated_at"`
}

// Execute は設定値を更新する。楽観的排他制御でバージョンを検証する。
func (uc *UpdateConfigUseCase) Execute(ctx context.Context, input UpdateConfigInput) (*UpdateConfigOutput, error) {
	// 既存エントリを取得
	existing, err := uc.configRepo.GetByKey(ctx, input.Namespace, input.Key)
	if err != nil {
		return nil, fmt.Errorf("failed to get config entry: %w", err)
	}

	// バージョン検証（楽観的排他制御）
	if existing.Version != input.Version {
		return nil, ErrVersionConflict
	}

	now := time.Now().UTC()
	oldValue := existing.ValueJSON
	oldVersion := existing.Version

	// エントリを更新
	existing.ValueJSON = input.Value
	existing.Version = existing.Version + 1
	existing.UpdatedBy = input.UpdatedBy
	existing.UpdatedAt = now
	if input.Description != "" {
		existing.Description = input.Description
	}

	if err := uc.configRepo.Update(ctx, existing, input.Version); err != nil {
		return nil, fmt.Errorf("failed to update config entry: %w", err)
	}

	// 変更ログを Kafka に非同期配信（エラーは無視して更新は成功とする）
	if uc.publisher != nil {
		changeLog := &model.ConfigChangeLog{
			ID:            uuid.New().String(),
			ConfigEntryID: existing.ID,
			Namespace:     existing.Namespace,
			Key:           existing.Key,
			OldValue:      oldValue,
			NewValue:      input.Value,
			OldVersion:    oldVersion,
			NewVersion:    existing.Version,
			ChangeType:    "UPDATED",
			ChangedBy:     input.UpdatedBy,
			ChangedAt:     now,
		}
		_ = uc.publisher.Publish(ctx, changeLog)
	}

	return &UpdateConfigOutput{
		Namespace:   existing.Namespace,
		Key:         existing.Key,
		Value:       existing.ValueJSON,
		Version:     existing.Version,
		Description: existing.Description,
		UpdatedBy:   existing.UpdatedBy,
		UpdatedAt:   existing.UpdatedAt,
	}, nil
}
