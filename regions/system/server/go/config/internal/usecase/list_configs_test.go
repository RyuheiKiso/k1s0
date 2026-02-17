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

func TestListConfigsUseCase_Execute_Success(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewListConfigsUseCase(mockRepo)

	now := time.Now().UTC()
	entries := []*model.ConfigEntry{
		{
			ID:          "entry-1",
			Namespace:   "system.auth.database",
			Key:         "max_connections",
			ValueJSON:   json.RawMessage(`25`),
			Version:     3,
			Description: "DB 最大接続数",
			UpdatedBy:   "admin@example.com",
			UpdatedAt:   now,
		},
		{
			ID:          "entry-2",
			Namespace:   "system.auth.database",
			Key:         "ssl_mode",
			ValueJSON:   json.RawMessage(`"require"`),
			Version:     1,
			Description: "SSL 接続モード",
			UpdatedBy:   "admin@example.com",
			UpdatedAt:   now,
		},
	}

	mockRepo.On("ListByNamespace", mock.Anything, repository.ConfigListParams{
		Namespace: "system.auth.database",
		Page:      1,
		PageSize:  20,
	}).Return(entries, 42, nil)

	input := ListConfigsInput{
		Namespace: "system.auth.database",
		Page:      1,
		PageSize:  20,
	}

	output, err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Len(t, output.Entries, 2)
	assert.Equal(t, 42, output.TotalCount)
	assert.Equal(t, 1, output.Page)
	assert.Equal(t, 20, output.PageSize)
	assert.True(t, output.HasNext)
	assert.Equal(t, "max_connections", output.Entries[0].Key)
	assert.Equal(t, "ssl_mode", output.Entries[1].Key)
	mockRepo.AssertExpectations(t)
}

func TestListConfigsUseCase_Execute_EmptyResult(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewListConfigsUseCase(mockRepo)

	mockRepo.On("ListByNamespace", mock.Anything, repository.ConfigListParams{
		Namespace: "nonexistent.namespace",
		Page:      1,
		PageSize:  20,
	}).Return([]*model.ConfigEntry{}, 0, nil)

	input := ListConfigsInput{
		Namespace: "nonexistent.namespace",
		Page:      1,
		PageSize:  20,
	}

	output, err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Len(t, output.Entries, 0)
	assert.Equal(t, 0, output.TotalCount)
	assert.False(t, output.HasNext)
	mockRepo.AssertExpectations(t)
}

func TestListConfigsUseCase_Execute_WithSearch(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewListConfigsUseCase(mockRepo)

	now := time.Now().UTC()
	entries := []*model.ConfigEntry{
		{
			ID:          "entry-1",
			Namespace:   "system.auth.database",
			Key:         "max_connections",
			ValueJSON:   json.RawMessage(`25`),
			Version:     3,
			Description: "DB 最大接続数",
			UpdatedBy:   "admin@example.com",
			UpdatedAt:   now,
		},
	}

	mockRepo.On("ListByNamespace", mock.Anything, repository.ConfigListParams{
		Namespace: "system.auth.database",
		Search:    "max",
		Page:      1,
		PageSize:  20,
	}).Return(entries, 1, nil)

	input := ListConfigsInput{
		Namespace: "system.auth.database",
		Search:    "max",
		Page:      1,
		PageSize:  20,
	}

	output, err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Len(t, output.Entries, 1)
	assert.Equal(t, 1, output.TotalCount)
	assert.False(t, output.HasNext)
	mockRepo.AssertExpectations(t)
}

func TestListConfigsUseCase_Execute_DefaultPagination(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewListConfigsUseCase(mockRepo)

	mockRepo.On("ListByNamespace", mock.Anything, repository.ConfigListParams{
		Namespace: "system.auth.database",
		Page:      1,
		PageSize:  20,
	}).Return([]*model.ConfigEntry{}, 0, nil)

	// page=0, pageSize=0 の場合はデフォルト値が使われる
	input := ListConfigsInput{
		Namespace: "system.auth.database",
		Page:      0,
		PageSize:  0,
	}

	output, err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Equal(t, 1, output.Page)
	assert.Equal(t, 20, output.PageSize)
	mockRepo.AssertExpectations(t)
}

func TestListConfigsUseCase_Execute_MaxPageSize(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewListConfigsUseCase(mockRepo)

	mockRepo.On("ListByNamespace", mock.Anything, repository.ConfigListParams{
		Namespace: "system.auth.database",
		Page:      1,
		PageSize:  100,
	}).Return([]*model.ConfigEntry{}, 0, nil)

	// pageSize=200 の場合は 100 に制限される
	input := ListConfigsInput{
		Namespace: "system.auth.database",
		Page:      1,
		PageSize:  200,
	}

	output, err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Equal(t, 100, output.PageSize)
	mockRepo.AssertExpectations(t)
}

func TestListConfigsUseCase_Execute_DBError(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewListConfigsUseCase(mockRepo)

	mockRepo.On("ListByNamespace", mock.Anything, mock.Anything).
		Return(nil, 0, errors.New("database error"))

	input := ListConfigsInput{
		Namespace: "system.auth.database",
		Page:      1,
		PageSize:  20,
	}

	output, err := uc.Execute(context.Background(), input)

	assert.Error(t, err)
	assert.Nil(t, output)
	assert.Contains(t, err.Error(), "database error")
	mockRepo.AssertExpectations(t)
}
