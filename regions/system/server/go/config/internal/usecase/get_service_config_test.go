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

func TestGetServiceConfigUseCase_Execute_Success(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewGetServiceConfigUseCase(mockRepo)

	now := time.Now().UTC()
	entries := []*model.ConfigEntry{
		{
			ID:        "entry-1",
			Namespace: "system.auth.database",
			Key:       "max_connections",
			ValueJSON: json.RawMessage(`25`),
			Version:   3,
			UpdatedBy: "admin@example.com",
			UpdatedAt: now,
		},
		{
			ID:        "entry-2",
			Namespace: "system.auth.database",
			Key:       "ssl_mode",
			ValueJSON: json.RawMessage(`"require"`),
			Version:   1,
			UpdatedBy: "admin@example.com",
			UpdatedAt: now,
		},
		{
			ID:        "entry-3",
			Namespace: "system.auth.jwt",
			Key:       "issuer",
			ValueJSON: json.RawMessage(`"https://auth.k1s0.internal.example.com/realms/k1s0"`),
			Version:   1,
			UpdatedBy: "admin@example.com",
			UpdatedAt: now,
		},
	}

	mockRepo.On("GetByServiceName", mock.Anything, "auth-server").Return(entries, nil)

	input := GetServiceConfigInput{
		ServiceName: "auth-server",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Equal(t, "auth-server", output.ServiceName)
	assert.Len(t, output.Entries, 3)
	assert.Equal(t, "max_connections", output.Entries[0].Key)
	assert.Equal(t, "ssl_mode", output.Entries[1].Key)
	assert.Equal(t, "issuer", output.Entries[2].Key)
	mockRepo.AssertExpectations(t)
}

func TestGetServiceConfigUseCase_Execute_ServiceNotFound(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewGetServiceConfigUseCase(mockRepo)

	mockRepo.On("GetByServiceName", mock.Anything, "nonexistent-service").
		Return([]*model.ConfigEntry{}, nil)

	input := GetServiceConfigInput{
		ServiceName: "nonexistent-service",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.Error(t, err)
	assert.Nil(t, output)
	assert.Contains(t, err.Error(), "service config not found")
	mockRepo.AssertExpectations(t)
}

func TestGetServiceConfigUseCase_Execute_DBError(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewGetServiceConfigUseCase(mockRepo)

	mockRepo.On("GetByServiceName", mock.Anything, "auth-server").
		Return(nil, errors.New("database error"))

	input := GetServiceConfigInput{
		ServiceName: "auth-server",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.Error(t, err)
	assert.Nil(t, output)
	assert.Contains(t, err.Error(), "database error")
	mockRepo.AssertExpectations(t)
}

func TestGetServiceConfigUseCase_Execute_SingleEntry(t *testing.T) {
	mockRepo := new(MockConfigRepository)
	uc := NewGetServiceConfigUseCase(mockRepo)

	now := time.Now().UTC()
	entries := []*model.ConfigEntry{
		{
			ID:        "entry-1",
			Namespace: "system.simple",
			Key:       "timeout",
			ValueJSON: json.RawMessage(`30`),
			Version:   1,
			UpdatedBy: "admin@example.com",
			UpdatedAt: now,
		},
	}

	mockRepo.On("GetByServiceName", mock.Anything, "simple-service").Return(entries, nil)

	input := GetServiceConfigInput{
		ServiceName: "simple-service",
	}

	output, err := uc.Execute(context.Background(), input)

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Equal(t, "simple-service", output.ServiceName)
	assert.Len(t, output.Entries, 1)
	assert.Equal(t, "timeout", output.Entries[0].Key)
	assert.Equal(t, json.RawMessage(`30`), output.Entries[0].Value)
	mockRepo.AssertExpectations(t)
}
