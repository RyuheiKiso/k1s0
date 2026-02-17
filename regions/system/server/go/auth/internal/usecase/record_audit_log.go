package usecase

import (
	"context"
	"time"

	"github.com/google/uuid"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/domain/repository"
)

// AuditEventPublisher は監査イベントの非同期配信インターフェース。
type AuditEventPublisher interface {
	Publish(ctx context.Context, log *model.AuditLog) error
}

// RecordAuditLogUseCase は監査ログ記録ユースケース。
type RecordAuditLogUseCase struct {
	auditRepo repository.AuditLogRepository
	publisher AuditEventPublisher
}

// NewRecordAuditLogUseCase は新しい RecordAuditLogUseCase を作成する。
func NewRecordAuditLogUseCase(
	auditRepo repository.AuditLogRepository,
	publisher AuditEventPublisher,
) *RecordAuditLogUseCase {
	return &RecordAuditLogUseCase{
		auditRepo: auditRepo,
		publisher: publisher,
	}
}

// RecordAuditLogInput は監査ログ記録の入力パラメータ。
type RecordAuditLogInput struct {
	EventType string            `json:"event_type" validate:"required"`
	UserID    string            `json:"user_id" validate:"required"`
	IPAddress string            `json:"ip_address"`
	UserAgent string            `json:"user_agent"`
	Resource  string            `json:"resource"`
	Action    string            `json:"action"`
	Result    string            `json:"result" validate:"required,oneof=SUCCESS FAILURE"`
	Metadata  map[string]string `json:"metadata"`
}

// RecordAuditLogOutput は監査ログ記録の出力。
type RecordAuditLogOutput struct {
	ID         string    `json:"id"`
	RecordedAt time.Time `json:"recorded_at"`
}

// Execute は監査ログエントリを記録する。
func (uc *RecordAuditLogUseCase) Execute(ctx context.Context, input RecordAuditLogInput) (*RecordAuditLogOutput, error) {
	now := time.Now().UTC()
	auditLog := &model.AuditLog{
		ID:         uuid.New().String(),
		EventType:  input.EventType,
		UserID:     input.UserID,
		IPAddress:  input.IPAddress,
		UserAgent:  input.UserAgent,
		Resource:   input.Resource,
		Action:     input.Action,
		Result:     input.Result,
		Metadata:   input.Metadata,
		RecordedAt: now,
	}

	// DB に保存
	if err := uc.auditRepo.Create(ctx, auditLog); err != nil {
		return nil, err
	}

	// Kafka に非同期配信（エラーは無視して記録は成功とする）
	if uc.publisher != nil {
		_ = uc.publisher.Publish(ctx, auditLog)
	}

	return &RecordAuditLogOutput{
		ID:         auditLog.ID,
		RecordedAt: now,
	}, nil
}
