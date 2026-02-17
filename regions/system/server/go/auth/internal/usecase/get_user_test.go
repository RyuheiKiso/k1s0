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

// MockUserRepository は UserRepository のモック実装。
type MockUserRepository struct {
	mock.Mock
}

func (m *MockUserRepository) GetUser(ctx context.Context, userID string) (*model.User, error) {
	args := m.Called(ctx, userID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*model.User), args.Error(1)
}

func (m *MockUserRepository) ListUsers(ctx context.Context, params repository.UserListParams) ([]*model.User, int, error) {
	args := m.Called(ctx, params)
	if args.Get(0) == nil {
		return nil, args.Int(1), args.Error(2)
	}
	return args.Get(0).([]*model.User), args.Int(1), args.Error(2)
}

func (m *MockUserRepository) GetUserRoles(ctx context.Context, userID string) ([]*model.Role, map[string][]*model.Role, error) {
	args := m.Called(ctx, userID)
	if args.Get(0) == nil {
		return nil, nil, args.Error(2)
	}
	return args.Get(0).([]*model.Role), args.Get(1).(map[string][]*model.Role), args.Error(2)
}

func (m *MockUserRepository) Healthy(ctx context.Context) error {
	args := m.Called(ctx)
	return args.Error(0)
}

func TestGetUserUseCase_Execute_Success(t *testing.T) {
	mockRepo := new(MockUserRepository)
	uc := NewGetUserUseCase(mockRepo)

	expectedUser := &model.User{
		ID:            "user-uuid-1234",
		Username:      "taro.yamada",
		Email:         "taro.yamada@example.com",
		FirstName:     "太郎",
		LastName:      "山田",
		Enabled:       true,
		EmailVerified: true,
		CreatedAt:     time.Date(2024, 1, 15, 9, 30, 0, 0, time.UTC),
		Attributes: map[string][]string{
			"department":  {"engineering"},
			"employee_id": {"EMP001"},
		},
	}

	mockRepo.On("GetUser", mock.Anything, "user-uuid-1234").Return(expectedUser, nil)

	user, err := uc.Execute(context.Background(), "user-uuid-1234")

	assert.NoError(t, err)
	assert.NotNil(t, user)
	assert.Equal(t, "user-uuid-1234", user.ID)
	assert.Equal(t, "taro.yamada", user.Username)
	assert.Equal(t, "taro.yamada@example.com", user.Email)
	assert.Equal(t, "太郎", user.FirstName)
	assert.Equal(t, "山田", user.LastName)
	assert.True(t, user.Enabled)
	mockRepo.AssertExpectations(t)
}

func TestGetUserUseCase_Execute_EmptyID(t *testing.T) {
	mockRepo := new(MockUserRepository)
	uc := NewGetUserUseCase(mockRepo)

	user, err := uc.Execute(context.Background(), "")

	assert.ErrorIs(t, err, ErrUserNotFound)
	assert.Nil(t, user)
}

func TestGetUserUseCase_Execute_NotFound(t *testing.T) {
	mockRepo := new(MockUserRepository)
	uc := NewGetUserUseCase(mockRepo)

	mockRepo.On("GetUser", mock.Anything, "nonexistent").
		Return(nil, errors.New("user not found"))

	user, err := uc.Execute(context.Background(), "nonexistent")

	assert.Error(t, err)
	assert.Nil(t, user)
	mockRepo.AssertExpectations(t)
}
