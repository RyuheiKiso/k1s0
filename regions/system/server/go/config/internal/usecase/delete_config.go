package usecase

import (
	"context"
	"fmt"
	"time"

	"github.com/google/uuid"

	"github.com/k1s0-platform/system-server-go-config/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-config/internal/domain/repository"
)

// DeleteConfigUseCase は設定値削除ユースケース。
type DeleteConfigUseCase struct {
	configRepo repository.ConfigRepository
	publisher  ConfigChangeEventPublisher
}

// NewDeleteConfigUseCase は新しい DeleteConfigUseCase を作成する。
func NewDeleteConfigUseCase(
	configRepo repository.ConfigRepository,
	publisher ConfigChangeEventPublisher,
) *DeleteConfigUseCase {
	return &DeleteConfigUseCase{
		configRepo: configRepo,
		publisher:  publisher,
	}
}

// DeleteConfigInput は設定値削除の入力パラメータ。
type DeleteConfigInput struct {
	Namespace string `json:"namespace" validate:"required"`
	Key       string `json:"key" validate:"required"`
	DeletedBy string `json:"deleted_by" validate:"required"`
}

// Execute は設定値を削除する。
func (uc *DeleteConfigUseCase) Execute(ctx context.Context, input DeleteConfigInput) error {
	// 既存エントリを取得（変更ログ用）
	existing, err := uc.configRepo.GetByKey(ctx, input.Namespace, input.Key)
	if err != nil {
		return fmt.Errorf("failed to get config entry: %w", err)
	}

	// エントリを削除
	if err := uc.configRepo.Delete(ctx, input.Namespace, input.Key); err != nil {
		return fmt.Errorf("failed to delete config entry: %w", err)
	}

	// 変更ログを Kafka に非同期配信（エラーは無視して削除は成功とする）
	if uc.publisher != nil {
		now := time.Now().UTC()
		changeLog := &model.ConfigChangeLog{
			ID:            uuid.New().String(),
			ConfigEntryID: existing.ID,
			Namespace:     existing.Namespace,
			Key:           existing.Key,
			OldValue:      existing.ValueJSON,
			NewValue:      nil,
			OldVersion:    existing.Version,
			NewVersion:    0,
			ChangeType:    "DELETED",
			ChangedBy:     input.DeletedBy,
			ChangedAt:     now,
		}
		_ = uc.publisher.Publish(ctx, changeLog)
	}

	return nil
}
