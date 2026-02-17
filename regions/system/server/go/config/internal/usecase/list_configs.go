package usecase

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/k1s0-platform/system-server-go-config/internal/domain/repository"
)

// ListConfigsUseCase は設定値一覧取得ユースケース。
type ListConfigsUseCase struct {
	configRepo repository.ConfigRepository
}

// NewListConfigsUseCase は新しい ListConfigsUseCase を作成する。
func NewListConfigsUseCase(
	configRepo repository.ConfigRepository,
) *ListConfigsUseCase {
	return &ListConfigsUseCase{
		configRepo: configRepo,
	}
}

// ListConfigsInput は設定値一覧取得の入力パラメータ。
type ListConfigsInput struct {
	Namespace string `json:"namespace" validate:"required"`
	Search    string `json:"search"`
	Page      int    `json:"page"`
	PageSize  int    `json:"page_size"`
}

// ListConfigsEntry は設定値一覧の1エントリ。
type ListConfigsEntry struct {
	Namespace   string          `json:"namespace"`
	Key         string          `json:"key"`
	Value       json.RawMessage `json:"value"`
	Version     int             `json:"version"`
	Description string          `json:"description"`
	UpdatedBy   string          `json:"updated_by"`
	UpdatedAt   time.Time       `json:"updated_at"`
}

// ListConfigsOutput は設定値一覧取得の出力。
type ListConfigsOutput struct {
	Entries    []ListConfigsEntry `json:"entries"`
	TotalCount int                `json:"total_count"`
	Page       int                `json:"page"`
	PageSize   int                `json:"page_size"`
	HasNext    bool               `json:"has_next"`
}

// Execute は namespace 内の設定値を一覧取得する。
func (uc *ListConfigsUseCase) Execute(ctx context.Context, input ListConfigsInput) (*ListConfigsOutput, error) {
	page := input.Page
	if page < 1 {
		page = 1
	}
	pageSize := input.PageSize
	if pageSize < 1 {
		pageSize = 20
	}
	if pageSize > 100 {
		pageSize = 100
	}

	params := repository.ConfigListParams{
		Namespace: input.Namespace,
		Search:    input.Search,
		Page:      page,
		PageSize:  pageSize,
	}

	entries, totalCount, err := uc.configRepo.ListByNamespace(ctx, params)
	if err != nil {
		return nil, fmt.Errorf("failed to list config entries: %w", err)
	}

	outputEntries := make([]ListConfigsEntry, 0, len(entries))
	for _, e := range entries {
		outputEntries = append(outputEntries, ListConfigsEntry{
			Namespace:   e.Namespace,
			Key:         e.Key,
			Value:       e.ValueJSON,
			Version:     e.Version,
			Description: e.Description,
			UpdatedBy:   e.UpdatedBy,
			UpdatedAt:   e.UpdatedAt,
		})
	}

	hasNext := page*pageSize < totalCount

	return &ListConfigsOutput{
		Entries:    outputEntries,
		TotalCount: totalCount,
		Page:       page,
		PageSize:   pageSize,
		HasNext:    hasNext,
	}, nil
}
