package usecase

import (
	"context"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/domain/repository"
)

// MockAuditLogRepository は AuditLogRepository のモック実装。
type MockAuditLogRepository struct {
	mock.Mock
}

func (m *MockAuditLogRepository) Create(ctx context.Context, log *model.AuditLog) error {
	args := m.Called(ctx, log)
	return args.Error(0)
}

func (m *MockAuditLogRepository) Search(ctx context.Context, params repository.AuditLogSearchParams) ([]*model.AuditLog, int, error) {
	args := m.Called(ctx, params)
	if args.Get(0) == nil {
		return nil, args.Int(1), args.Error(2)
	}
	return args.Get(0).([]*model.AuditLog), args.Int(1), args.Error(2)
}

// MockAuditEventPublisher は AuditEventPublisher のモック実装。
type MockAuditEventPublisher struct {
	mock.Mock
}

func (m *MockAuditEventPublisher) Publish(ctx context.Context, log *model.AuditLog) error {
	args := m.Called(ctx, log)
	return args.Error(0)
}

func TestRecordAuditLogUseCase_Execute_Success(t *testing.T) {
	mockRepo := new(MockAuditLogRepository)
	mockPublisher := new(MockAuditEventPublisher)
	uc := NewRecordAuditLogUseCase(mockRepo, mockPublisher)

	mockRepo.On("Create", mock.Anything, mock.AnythingOfType("*model.AuditLog")).Return(nil)
	mockPublisher.On("Publish", mock.Anything, mock.AnythingOfType("*model.AuditLog")).Return(nil)

	input := RecordAuditLogInput{
		EventType: "LOGIN_SUCCESS",
		UserID:    "user-uuid-1234",
		IPAddress: "192.168.1.100",
		UserAgent: "Mozilla/5.0",
		Resource:  "/api/v1/auth/token",
		Action:    "POST",
		Result:    "SUCCESS",
		Metadata: map[string]string{
			"client_id":  "react-spa",
			"grant_type": "authorization_code",
		},
	}

	output, err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.NotEmpty(t, output.ID)
	assert.False(t, output.RecordedAt.IsZero())
	mockRepo.AssertExpectations(t)
	mockPublisher.AssertExpectations(t)
}

func TestRecordAuditLogUseCase_Execute_DBError(t *testing.T) {
	mockRepo := new(MockAuditLogRepository)
	mockPublisher := new(MockAuditEventPublisher)
	uc := NewRecordAuditLogUseCase(mockRepo, mockPublisher)

	mockRepo.On("Create", mock.Anything, mock.AnythingOfType("*model.AuditLog")).
		Return(errors.New("database error"))

	input := RecordAuditLogInput{
		EventType: "LOGIN_SUCCESS",
		UserID:    "user-uuid-1234",
		Result:    "SUCCESS",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.Error(t, err)
	assert.Nil(t, output)
	mockRepo.AssertExpectations(t)
}

func TestRecordAuditLogUseCase_Execute_PublishErrorIgnored(t *testing.T) {
	mockRepo := new(MockAuditLogRepository)
	mockPublisher := new(MockAuditEventPublisher)
	uc := NewRecordAuditLogUseCase(mockRepo, mockPublisher)

	mockRepo.On("Create", mock.Anything, mock.AnythingOfType("*model.AuditLog")).Return(nil)
	mockPublisher.On("Publish", mock.Anything, mock.AnythingOfType("*model.AuditLog")).
		Return(errors.New("kafka error"))

	input := RecordAuditLogInput{
		EventType: "LOGIN_FAILURE",
		UserID:    "user-uuid-1234",
		Result:    "FAILURE",
	}

	output, err := uc.Execute(context.Background(), input)

	// Kafka エラーは無視されるので記録は成功
	assert.NoError(t, err)
	assert.NotNil(t, output)
	mockRepo.AssertExpectations(t)
	mockPublisher.AssertExpectations(t)
}

func TestRecordAuditLogUseCase_Execute_NilPublisher(t *testing.T) {
	mockRepo := new(MockAuditLogRepository)
	uc := NewRecordAuditLogUseCase(mockRepo, nil)

	mockRepo.On("Create", mock.Anything, mock.AnythingOfType("*model.AuditLog")).Return(nil)

	input := RecordAuditLogInput{
		EventType: "TOKEN_VALIDATE",
		UserID:    "user-uuid-1234",
		Result:    "SUCCESS",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	assert.NotNil(t, output)
	mockRepo.AssertExpectations(t)
}
