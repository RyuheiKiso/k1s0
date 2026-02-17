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
)

func TestUpdateConfigUseCase_Execute_Success(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	mockPublisher := new(MockConfigChangeEventPublisher)
	uc := NewUpdateConfigUseCase(mockRepo, mockPublisher)

	now := time.Now().UTC()
	existing := &model.ConfigEntry{
		ID:          "entry-uuid-1234",
		Namespace:   "system.auth.database",
		Key:         "max_connections",
		ValueJSON:   json.RawMessage(`25`),
		Version:     3,
		Description: "認証サーバーの DB 最大接続数",
		CreatedBy:   "admin@example.com",
		UpdatedBy:   "admin@example.com",
		CreatedAt:   now.Add(-24 * time.Hour),
		UpdatedAt:   now.Add(-1 * time.Hour),
	}

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "max_connections").Return(existing, nil)
	mockRepo.On("Update", mock.Anything, mock.AnythingOfType("*model.ConfigEntry"), 3).Return(nil)
	mockPublisher.On("Publish", mock.Anything, mock.AnythingOfType("*model.ConfigChangeLog")).Return(nil)

	input := UpdateConfigInput{
		Namespace:   "system.auth.database",
		Key:         "max_connections",
		Value:       json.RawMessage(`50`),
		Version:     3,
		Description: "認証サーバーの DB 最大接続数（増設）",
		UpdatedBy:   "operator@example.com",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Equal(t, "system.auth.database", output.Namespace)
	assert.Equal(t, "max_connections", output.Key)
	assert.Equal(t, json.RawMessage(`50`), output.Value)
	assert.Equal(t, 4, output.Version)
	assert.Equal(t, "認証サーバーの DB 最大接続数（増設）", output.Description)
	assert.Equal(t, "operator@example.com", output.UpdatedBy)
	mockRepo.AssertExpectations(t)
	mockPublisher.AssertExpectations(t)
}

func TestUpdateConfigUseCase_Execute_VersionConflict(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	mockPublisher := new(MockConfigChangeEventPublisher)
	uc := NewUpdateConfigUseCase(mockRepo, mockPublisher)

	now := time.Now().UTC()
	existing := &model.ConfigEntry{
		ID:        "entry-uuid-1234",
		Namespace: "system.auth.database",
		Key:       "max_connections",
		ValueJSON: json.RawMessage(`25`),
		Version:   4, // 既にバージョン4に更新されている
		UpdatedBy: "admin@example.com",
		UpdatedAt: now,
	}

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "max_connections").Return(existing, nil)

	input := UpdateConfigInput{
		Namespace: "system.auth.database",
		Key:       "max_connections",
		Value:     json.RawMessage(`50`),
		Version:   3, // 古いバージョンを指定
		UpdatedBy: "operator@example.com",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.Error(t, err)
	assert.Nil(t, output)
	assert.True(t, errors.Is(err, ErrVersionConflict))
	mockRepo.AssertExpectations(t)
}

func TestUpdateConfigUseCase_Execute_NotFound(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	mockPublisher := new(MockConfigChangeEventPublisher)
	uc := NewUpdateConfigUseCase(mockRepo, mockPublisher)

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "nonexistent").
		Return(nil, errors.New("config entry not found"))

	input := UpdateConfigInput{
		Namespace: "system.auth.database",
		Key:       "nonexistent",
		Value:     json.RawMessage(`50`),
		Version:   1,
		UpdatedBy: "operator@example.com",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.Error(t, err)
	assert.Nil(t, output)
	assert.Contains(t, err.Error(), "config entry not found")
	mockRepo.AssertExpectations(t)
}

func TestUpdateConfigUseCase_Execute_DBUpdateError(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	mockPublisher := new(MockConfigChangeEventPublisher)
	uc := NewUpdateConfigUseCase(mockRepo, mockPublisher)

	now := time.Now().UTC()
	existing := &model.ConfigEntry{
		ID:        "entry-uuid-1234",
		Namespace: "system.auth.database",
		Key:       "max_connections",
		ValueJSON: json.RawMessage(`25`),
		Version:   3,
		UpdatedBy: "admin@example.com",
		UpdatedAt: now,
	}

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "max_connections").Return(existing, nil)
	mockRepo.On("Update", mock.Anything, mock.AnythingOfType("*model.ConfigEntry"), 3).
		Return(errors.New("database error"))

	input := UpdateConfigInput{
		Namespace: "system.auth.database",
		Key:       "max_connections",
		Value:     json.RawMessage(`50`),
		Version:   3,
		UpdatedBy: "operator@example.com",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.Error(t, err)
	assert.Nil(t, output)
	assert.Contains(t, err.Error(), "database error")
	mockRepo.AssertExpectations(t)
}

func TestUpdateConfigUseCase_Execute_PublishErrorIgnored(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	mockPublisher := new(MockConfigChangeEventPublisher)
	uc := NewUpdateConfigUseCase(mockRepo, mockPublisher)

	now := time.Now().UTC()
	existing := &model.ConfigEntry{
		ID:          "entry-uuid-1234",
		Namespace:   "system.auth.database",
		Key:         "max_connections",
		ValueJSON:   json.RawMessage(`25`),
		Version:     3,
		Description: "DB 最大接続数",
		UpdatedBy:   "admin@example.com",
		UpdatedAt:   now,
	}

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "max_connections").Return(existing, nil)
	mockRepo.On("Update", mock.Anything, mock.AnythingOfType("*model.ConfigEntry"), 3).Return(nil)
	mockPublisher.On("Publish", mock.Anything, mock.AnythingOfType("*model.ConfigChangeLog")).
		Return(errors.New("kafka error"))

	input := UpdateConfigInput{
		Namespace: "system.auth.database",
		Key:       "max_connections",
		Value:     json.RawMessage(`50`),
		Version:   3,
		UpdatedBy: "operator@example.com",
	}

	output, err := uc.Execute(context.Background(), input)

	// Kafka エラーは無視されるので更新は成功
	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Equal(t, 4, output.Version)
	mockRepo.AssertExpectations(t)
	mockPublisher.AssertExpectations(t)
}

func TestUpdateConfigUseCase_Execute_NilPublisher(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewUpdateConfigUseCase(mockRepo, nil)

	now := time.Now().UTC()
	existing := &model.ConfigEntry{
		ID:          "entry-uuid-1234",
		Namespace:   "system.auth.database",
		Key:         "max_connections",
		ValueJSON:   json.RawMessage(`25`),
		Version:     3,
		Description: "DB 最大接続数",
		UpdatedBy:   "admin@example.com",
		UpdatedAt:   now,
	}

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "max_connections").Return(existing, nil)
	mockRepo.On("Update", mock.Anything, mock.AnythingOfType("*model.ConfigEntry"), 3).Return(nil)

	input := UpdateConfigInput{
		Namespace: "system.auth.database",
		Key:       "max_connections",
		Value:     json.RawMessage(`50`),
		Version:   3,
		UpdatedBy: "operator@example.com",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	assert.NotNil(t, output)
	mockRepo.AssertExpectations(t)
}
