package usecase

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/domain/repository"
)

// MockAuditLogRepo は AuditLogRepository のモック実装（search テスト用）。
type MockAuditLogRepo struct {
	mock.Mock
}

func (m *MockAuditLogRepo) Create(ctx context.Context, log *model.AuditLog) error {
	args := m.Called(ctx, log)
	return args.Error(0)
}

func (m *MockAuditLogRepo) Search(ctx context.Context, params repository.AuditLogSearchParams) ([]*model.AuditLog, int, error) {
	args := m.Called(ctx, params)
	if args.Get(0) == nil {
		return nil, args.Int(1), args.Error(2)
	}
	return args.Get(0).([]*model.AuditLog), args.Int(1), args.Error(2)
}

func TestSearchAuditLogsUseCase_Execute_Success(t *testing.T) {
	mockRepo := new(MockAuditLogRepo)
	uc := NewSearchAuditLogsUseCase(mockRepo)

	now := time.Now().UTC()
	logs := []*model.AuditLog{
		{
			ID:         "audit-1",
			EventType:  "LOGIN_SUCCESS",
			UserID:     "user-uuid-1234",
			IPAddress:  "192.168.1.100",
			Resource:   "/api/v1/auth/token",
			Action:     "POST",
			Result:     "SUCCESS",
			RecordedAt: now,
		},
	}

	mockRepo.On("Search", mock.Anything, repository.AuditLogSearchParams{
		Page:     1,
		PageSize: 50,
	}).Return(logs, 5000, nil)

	output, err := uc.Execute(context.Background(), SearchAuditLogsInput{})

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Len(t, output.Logs, 1)
	assert.Equal(t, 5000, output.TotalCount)
	assert.Equal(t, 1, output.Page)
	assert.Equal(t, 50, output.PageSize)
	assert.True(t, output.HasNext)
	mockRepo.AssertExpectations(t)
}

func TestSearchAuditLogsUseCase_Execute_DefaultValues(t *testing.T) {
	mockRepo := new(MockAuditLogRepo)
	uc := NewSearchAuditLogsUseCase(mockRepo)

	mockRepo.On("Search", mock.Anything, repository.AuditLogSearchParams{
		Page:     1,
		PageSize: 50,
	}).Return([]*model.AuditLog{}, 0, nil)

	output, err := uc.Execute(context.Background(), SearchAuditLogsInput{})

	assert.NoError(t, err)
	assert.Equal(t, 1, output.Page)
	assert.Equal(t, 50, output.PageSize)
	assert.False(t, output.HasNext)
	mockRepo.AssertExpectations(t)
}

func TestSearchAuditLogsUseCase_Execute_MaxPageSize(t *testing.T) {
	mockRepo := new(MockAuditLogRepo)
	uc := NewSearchAuditLogsUseCase(mockRepo)

	mockRepo.On("Search", mock.Anything, repository.AuditLogSearchParams{
		Page:     1,
		PageSize: 200,
	}).Return([]*model.AuditLog{}, 0, nil)

	output, err := uc.Execute(context.Background(), SearchAuditLogsInput{
		PageSize: 999, // 200 に制限される
	})

	assert.NoError(t, err)
	assert.Equal(t, 200, output.PageSize)
	mockRepo.AssertExpectations(t)
}

func TestSearchAuditLogsUseCase_Execute_WithFilters(t *testing.T) {
	mockRepo := new(MockAuditLogRepo)
	uc := NewSearchAuditLogsUseCase(mockRepo)

	from := time.Date(2026, 2, 1, 0, 0, 0, 0, time.UTC)
	to := time.Date(2026, 2, 17, 23, 59, 59, 0, time.UTC)

	mockRepo.On("Search", mock.Anything, repository.AuditLogSearchParams{
		UserID:    "user-uuid-1234",
		EventType: "LOGIN_SUCCESS",
		Result:    "SUCCESS",
		From:      &from,
		To:        &to,
		Page:      1,
		PageSize:  50,
	}).Return([]*model.AuditLog{
		{ID: "audit-1", EventType: "LOGIN_SUCCESS", UserID: "user-uuid-1234"},
	}, 1, nil)

	output, err := uc.Execute(context.Background(), SearchAuditLogsInput{
		UserID:    "user-uuid-1234",
		EventType: "LOGIN_SUCCESS",
		Result:    "SUCCESS",
		From:      &from,
		To:        &to,
	})

	assert.NoError(t, err)
	assert.Len(t, output.Logs, 1)
	assert.False(t, output.HasNext)
	mockRepo.AssertExpectations(t)
}

func TestSearchAuditLogsUseCase_Execute_RepoError(t *testing.T) {
	mockRepo := new(MockAuditLogRepo)
	uc := NewSearchAuditLogsUseCase(mockRepo)

	mockRepo.On("Search", mock.Anything, mock.Anything).
		Return(nil, 0, errors.New("database error"))

	output, err := uc.Execute(context.Background(), SearchAuditLogsInput{})

	assert.Error(t, err)
	assert.Nil(t, output)
	mockRepo.AssertExpectations(t)
}
