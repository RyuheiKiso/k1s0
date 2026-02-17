package usecase

import (
	"context"
	"time"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/domain/repository"
)

// SearchAuditLogsUseCase は監査ログ検索ユースケース。
type SearchAuditLogsUseCase struct {
	auditRepo repository.AuditLogRepository
}

// NewSearchAuditLogsUseCase は新しい SearchAuditLogsUseCase を作成する。
func NewSearchAuditLogsUseCase(auditRepo repository.AuditLogRepository) *SearchAuditLogsUseCase {
	return &SearchAuditLogsUseCase{
		auditRepo: auditRepo,
	}
}

// SearchAuditLogsInput は監査ログ検索の入力パラメータ。
type SearchAuditLogsInput struct {
	Page      int
	PageSize  int
	UserID    string
	EventType string
	From      *time.Time
	To        *time.Time
	Result    string
}

// SearchAuditLogsOutput は監査ログ検索の出力。
type SearchAuditLogsOutput struct {
	Logs       []*model.AuditLog
	TotalCount int
	Page       int
	PageSize   int
	HasNext    bool
}

// Execute は監査ログを検索する。
func (uc *SearchAuditLogsUseCase) Execute(ctx context.Context, input SearchAuditLogsInput) (*SearchAuditLogsOutput, error) {
	// デフォルト値の設定
	if input.Page <= 0 {
		input.Page = 1
	}
	if input.PageSize <= 0 {
		input.PageSize = 50
	}
	if input.PageSize > 200 {
		input.PageSize = 200
	}

	params := repository.AuditLogSearchParams{
		UserID:    input.UserID,
		EventType: input.EventType,
		Result:    input.Result,
		From:      input.From,
		To:        input.To,
		Page:      input.Page,
		PageSize:  input.PageSize,
	}

	logs, totalCount, err := uc.auditRepo.Search(ctx, params)
	if err != nil {
		return nil, err
	}

	hasNext := input.Page*input.PageSize < totalCount

	return &SearchAuditLogsOutput{
		Logs:       logs,
		TotalCount: totalCount,
		Page:       input.Page,
		PageSize:   input.PageSize,
		HasNext:    hasNext,
	}, nil
}
