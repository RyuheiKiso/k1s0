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

func TestDeleteConfigUseCase_Execute_Success(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	mockPublisher := new(MockConfigChangeEventPublisher)
	uc := NewDeleteConfigUseCase(mockRepo, mockPublisher)

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
	mockRepo.On("Delete", mock.Anything, "system.auth.database", "max_connections").Return(nil)
	mockPublisher.On("Publish", mock.Anything, mock.AnythingOfType("*model.ConfigChangeLog")).Return(nil)

	input := DeleteConfigInput{
		Namespace: "system.auth.database",
		Key:       "max_connections",
		DeletedBy: "admin@example.com",
	}

	err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	mockRepo.AssertExpectations(t)
	mockPublisher.AssertExpectations(t)
}

func TestDeleteConfigUseCase_Execute_NotFound(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	mockPublisher := new(MockConfigChangeEventPublisher)
	uc := NewDeleteConfigUseCase(mockRepo, mockPublisher)

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "nonexistent").
		Return(nil, errors.New("config entry not found"))

	input := DeleteConfigInput{
		Namespace: "system.auth.database",
		Key:       "nonexistent",
		DeletedBy: "admin@example.com",
	}

	err := uc.Execute(context.Background(), input)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "config entry not found")
	mockRepo.AssertExpectations(t)
}

func TestDeleteConfigUseCase_Execute_DBDeleteError(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	mockPublisher := new(MockConfigChangeEventPublisher)
	uc := NewDeleteConfigUseCase(mockRepo, mockPublisher)

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
	mockRepo.On("Delete", mock.Anything, "system.auth.database", "max_connections").
		Return(errors.New("database error"))

	input := DeleteConfigInput{
		Namespace: "system.auth.database",
		Key:       "max_connections",
		DeletedBy: "admin@example.com",
	}

	err := uc.Execute(context.Background(), input)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "database error")
	mockRepo.AssertExpectations(t)
}

func TestDeleteConfigUseCase_Execute_PublishErrorIgnored(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	mockPublisher := new(MockConfigChangeEventPublisher)
	uc := NewDeleteConfigUseCase(mockRepo, mockPublisher)

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
	mockRepo.On("Delete", mock.Anything, "system.auth.database", "max_connections").Return(nil)
	mockPublisher.On("Publish", mock.Anything, mock.AnythingOfType("*model.ConfigChangeLog")).
		Return(errors.New("kafka error"))

	input := DeleteConfigInput{
		Namespace: "system.auth.database",
		Key:       "max_connections",
		DeletedBy: "admin@example.com",
	}

	err := uc.Execute(context.Background(), input)

	// Kafka エラーは無視されるので削除は成功
	assert.NoError(t, err)
	mockRepo.AssertExpectations(t)
	mockPublisher.AssertExpectations(t)
}

func TestDeleteConfigUseCase_Execute_NilPublisher(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewDeleteConfigUseCase(mockRepo, nil)

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
	mockRepo.On("Delete", mock.Anything, "system.auth.database", "max_connections").Return(nil)

	input := DeleteConfigInput{
		Namespace: "system.auth.database",
		Key:       "max_connections",
		DeletedBy: "admin@example.com",
	}

	err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	mockRepo.AssertExpectations(t)
}
