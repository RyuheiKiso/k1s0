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

func TestListUsersUseCase_Execute_Success(t *testing.T) {
	mockRepo := new(MockUserRepository)
	uc := NewListUsersUseCase(mockRepo)

	users := []*model.User{
		{ID: "user-1", Username: "user1", Email: "user1@example.com", Enabled: true},
		{ID: "user-2", Username: "user2", Email: "user2@example.com", Enabled: true},
	}

	mockRepo.On("ListUsers", mock.Anything, repository.UserListParams{
		Page:     1,
		PageSize: 20,
		Search:   "",
		Enabled:  nil,
	}).Return(users, 150, nil)

	output, err := uc.Execute(context.Background(), ListUsersInput{
		Page:     1,
		PageSize: 20,
	})

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Len(t, output.Users, 2)
	assert.Equal(t, 150, output.TotalCount)
	assert.Equal(t, 1, output.Page)
	assert.Equal(t, 20, output.PageSize)
	assert.True(t, output.HasNext)
	mockRepo.AssertExpectations(t)
}

func TestListUsersUseCase_Execute_DefaultValues(t *testing.T) {
	mockRepo := new(MockUserRepository)
	uc := NewListUsersUseCase(mockRepo)

	mockRepo.On("ListUsers", mock.Anything, repository.UserListParams{
		Page:     1,
		PageSize: 20,
	}).Return([]*model.User{}, 0, nil)

	output, err := uc.Execute(context.Background(), ListUsersInput{})

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Equal(t, 1, output.Page)
	assert.Equal(t, 20, output.PageSize)
	assert.False(t, output.HasNext)
	mockRepo.AssertExpectations(t)
}

func TestListUsersUseCase_Execute_MaxPageSize(t *testing.T) {
	mockRepo := new(MockUserRepository)
	uc := NewListUsersUseCase(mockRepo)

	mockRepo.On("ListUsers", mock.Anything, repository.UserListParams{
		Page:     1,
		PageSize: 100,
	}).Return([]*model.User{}, 0, nil)

	output, err := uc.Execute(context.Background(), ListUsersInput{
		Page:     1,
		PageSize: 500, // 100 に制限される
	})

	assert.NoError(t, err)
	assert.NotNil(t, output)
	assert.Equal(t, 100, output.PageSize)
	mockRepo.AssertExpectations(t)
}

func TestListUsersUseCase_Execute_HasNextFalse(t *testing.T) {
	mockRepo := new(MockUserRepository)
	uc := NewListUsersUseCase(mockRepo)

	users := []*model.User{
		{ID: "user-1", Username: "user1"},
	}

	mockRepo.On("ListUsers", mock.Anything, repository.UserListParams{
		Page:     1,
		PageSize: 20,
	}).Return(users, 1, nil)

	output, err := uc.Execute(context.Background(), ListUsersInput{
		Page:     1,
		PageSize: 20,
	})

	assert.NoError(t, err)
	assert.False(t, output.HasNext)
	mockRepo.AssertExpectations(t)
}

func TestListUsersUseCase_Execute_WithSearch(t *testing.T) {
	mockRepo := new(MockUserRepository)
	uc := NewListUsersUseCase(mockRepo)

	enabled := true
	mockRepo.On("ListUsers", mock.Anything, repository.UserListParams{
		Page:     1,
		PageSize: 20,
		Search:   "taro",
		Enabled:  &enabled,
	}).Return([]*model.User{
		{ID: "user-1", Username: "taro.yamada"},
	}, 1, nil)

	output, err := uc.Execute(context.Background(), ListUsersInput{
		Page:     1,
		PageSize: 20,
		Search:   "taro",
		Enabled:  &enabled,
	})

	assert.NoError(t, err)
	assert.Len(t, output.Users, 1)
	mockRepo.AssertExpectations(t)
}

func TestListUsersUseCase_Execute_RepoError(t *testing.T) {
	mockRepo := new(MockUserRepository)
	uc := NewListUsersUseCase(mockRepo)

	mockRepo.On("ListUsers", mock.Anything, mock.Anything).
		Return(nil, 0, errors.New("database error"))

	output, err := uc.Execute(context.Background(), ListUsersInput{})

	assert.Error(t, err)
	assert.Nil(t, output)
	mockRepo.AssertExpectations(t)
}
