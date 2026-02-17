package usecase

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/k1s0-platform/system-server-go-config/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-config/internal/domain/repository"
)

// MockConfigRepository は ConfigRepository のモック実装。
type MockConfigRepository struct {
	mock.Mock
}

func (m *MockConfigRepository) GetByKey(ctx context.Context, namespace, key string) (*model.ConfigEntry, error) {
	args := m.Called(ctx, namespace, key)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*model.ConfigEntry), args.Error(1)
}

func (m *MockConfigRepository) ListByNamespace(ctx context.Context, params repository.ConfigListParams) ([]*model.ConfigEntry, int, error) {
	args := m.Called(ctx, params)
	if args.Get(0) == nil {
		return nil, args.Int(1), args.Error(2)
	}
	return args.Get(0).([]*model.ConfigEntry), args.Int(1), args.Error(2)
}

func (m *MockConfigRepository) GetByServiceName(ctx context.Context, serviceName string) ([]*model.ConfigEntry, error) {
	args := m.Called(ctx, serviceName)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]*model.ConfigEntry), args.Error(1)
}

func (m *MockConfigRepository) Create(ctx context.Context, entry *model.ConfigEntry) error {
	args := m.Called(ctx, entry)
	return args.Error(0)
}

func (m *MockConfigRepository) Update(ctx context.Context, entry *model.ConfigEntry, expectedVersion int) error {
	args := m.Called(ctx, entry, expectedVersion)
	return args.Error(0)
}

func (m *MockConfigRepository) Delete(ctx context.Context, namespace, key string) error {
	args := m.Called(ctx, namespace, key)
	return args.Error(0)
}

// MockConfigChangeEventPublisher は ConfigChangeEventPublisher のモック実装。
type MockConfigChangeEventPublisher struct {
	mock.Mock
}

func (m *MockConfigChangeEventPublisher) Publish(ctx context.Context, log *model.ConfigChangeLog) error {
	args := m.Called(ctx, log)
	return args.Error(0)
}

func TestGetConfigUseCase_Execute_Success(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewGetConfigUseCase(mockRepo)

	now := time.Now().UTC()
	entry := &model.ConfigEntry{
		ID:          "entry-uuid-1234",
		Namespace:   "system.auth.database",
		Key:         "max_connections",
		ValueJSON:   json.RawMessage(`25`),
		Version:     3,
		Description: "認証サーバーの DB 最大接続数",
		CreatedBy:   "admin@example.com",
		UpdatedBy:   "admin@example.com",
		CreatedAt:   now,
		UpdatedAt:   now,
	}
	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "max_connections").Return(entry, nil)

	input := GetConfigInput{
		Namespace: "system.auth.database",
		Key:       "max_connections",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Equal(t, "system.auth.database", output.Namespace)
	assert.Equal(t, "max_connections", output.Key)
	assert.Equal(t, json.RawMessage(`25`), output.Value)
	assert.Equal(t, 3, output.Version)
	assert.Equal(t, "認証サーバーの DB 最大接続数", output.Description)
	assert.Equal(t, "admin@example.com", output.UpdatedBy)
	mockRepo.AssertExpectations(t)
}

func TestGetConfigUseCase_Execute_NotFound(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewGetConfigUseCase(mockRepo)

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "nonexistent").
		Return(nil, errors.New("config entry not found"))

	input := GetConfigInput{
		Namespace: "system.auth.database",
		Key:       "nonexistent",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.Error(t, err)
	assert.Nil(t, output)
	assert.Contains(t, err.Error(), "config entry not found")
	mockRepo.AssertExpectations(t)
}

func TestGetConfigUseCase_Execute_DBError(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewGetConfigUseCase(mockRepo)

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "max_connections").
		Return(nil, errors.New("database connection error"))

	input := GetConfigInput{
		Namespace: "system.auth.database",
		Key:       "max_connections",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.Error(t, err)
	assert.Nil(t, output)
	assert.Contains(t, err.Error(), "database connection error")
	mockRepo.AssertExpectations(t)
}
