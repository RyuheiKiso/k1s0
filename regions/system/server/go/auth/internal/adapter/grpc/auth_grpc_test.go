package grpc

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/usecase"
)

// --- Mock: ValidateTokenExecutor ---

type MockValidateTokenUC struct {
	mock.Mock
}

func (m *MockValidateTokenUC) Execute(ctx context.Context, token string) (*model.TokenClaims, error) {
	args := m.Called(ctx, token)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*model.TokenClaims), args.Error(1)
}

// --- Mock: GetUserExecutor ---

type MockGetUserUC struct {
	mock.Mock
}

func (m *MockGetUserUC) Execute(ctx context.Context, userID string) (*model.User, error) {
	args := m.Called(ctx, userID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*model.User), args.Error(1)
}

func (m *MockGetUserUC) GetUserRoles(ctx context.Context, userID string) ([]*model.Role, map[string][]*model.Role, error) {
	args := m.Called(ctx, userID)
	if args.Get(0) == nil {
		return nil, nil, args.Error(2)
	}
	return args.Get(0).([]*model.Role), args.Get(1).(map[string][]*model.Role), args.Error(2)
}

// --- Mock: ListUsersExecutor ---

type MockListUsersUC struct {
	mock.Mock
}

func (m *MockListUsersUC) Execute(ctx context.Context, input usecase.ListUsersInput) (*usecase.ListUsersOutput, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*usecase.ListUsersOutput), args.Error(1)
}

// --- TestValidateToken_Success ---

func TestValidateToken_Success(t *testing.T) {
	mockUC := new(MockValidateTokenUC)
	svc := &AuthGRPCService{
		validateTokenUC: mockUC,
	}

	expectedClaims := &model.TokenClaims{
		Sub:              "user-uuid-1234",
		Iss:              "https://auth.k1s0.internal.example.com/realms/k1s0",
		Aud:              "k1s0-api",
		Exp:              time.Now().Add(1 * time.Hour).Unix(),
		Iat:              time.Now().Unix(),
		Jti:              "token-uuid-5678",
		PreferredUsername: "taro.yamada",
		Email:            "taro.yamada@example.com",
		RealmAccess:      model.RealmAccess{Roles: []string{"user", "sys_auditor"}},
		ResourceAccess:   map[string]model.ClientRoles{},
		TierAccess:       []string{"system"},
	}

	mockUC.On("Execute", mock.Anything, "valid-token").Return(expectedClaims, nil)

	req := &ValidateTokenRequest{Token: "valid-token"}
	resp, err := svc.ValidateToken(context.Background(), req)

	assert.NoError(t, err)
	assert.True(t, resp.Valid)
	assert.Equal(t, "user-uuid-1234", resp.Claims.Sub)
	assert.Equal(t, "taro.yamada", resp.Claims.PreferredUsername)
	assert.Equal(t, "taro.yamada@example.com", resp.Claims.Email)
	assert.Equal(t, []string{"user", "sys_auditor"}, resp.Claims.RealmAccess.Roles)
	mockUC.AssertExpectations(t)
}

// --- TestValidateToken_InvalidToken ---

func TestValidateToken_InvalidToken(t *testing.T) {
	mockUC := new(MockValidateTokenUC)
	svc := &AuthGRPCService{
		validateTokenUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, "invalid-token").Return(nil, errors.New("invalid signature"))

	req := &ValidateTokenRequest{Token: "invalid-token"}
	resp, err := svc.ValidateToken(context.Background(), req)

	// gRPC のエラーではなく、valid=false で返す設計
	assert.NoError(t, err)
	assert.False(t, resp.Valid)
	assert.Contains(t, resp.ErrorMessage, "invalid signature")
	mockUC.AssertExpectations(t)
}

// --- TestValidateToken_EmptyToken ---

func TestValidateToken_EmptyToken(t *testing.T) {
	mockUC := new(MockValidateTokenUC)
	svc := &AuthGRPCService{
		validateTokenUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, "").Return(nil, usecase.ErrInvalidToken)

	req := &ValidateTokenRequest{Token: ""}
	resp, err := svc.ValidateToken(context.Background(), req)

	assert.NoError(t, err)
	assert.False(t, resp.Valid)
	assert.Contains(t, resp.ErrorMessage, "invalid token")
	mockUC.AssertExpectations(t)
}

// --- TestGetUser_Exists ---

func TestGetUser_Exists(t *testing.T) {
	mockUC := new(MockGetUserUC)
	svc := &AuthGRPCService{
		getUserUC: mockUC,
	}

	expectedUser := &model.User{
		ID:            "user-uuid-1234",
		Username:      "taro.yamada",
		Email:         "taro.yamada@example.com",
		FirstName:     "Taro",
		LastName:      "Yamada",
		Enabled:       true,
		EmailVerified: true,
		CreatedAt:     time.Date(2025, 1, 15, 10, 0, 0, 0, time.UTC),
		Attributes:    map[string][]string{"department": {"engineering"}},
	}

	mockUC.On("Execute", mock.Anything, "user-uuid-1234").Return(expectedUser, nil)

	req := &GetUserRequest{UserId: "user-uuid-1234"}
	resp, err := svc.GetUser(context.Background(), req)

	assert.NoError(t, err)
	assert.Equal(t, "user-uuid-1234", resp.User.Id)
	assert.Equal(t, "taro.yamada", resp.User.Username)
	assert.Equal(t, "taro.yamada@example.com", resp.User.Email)
	assert.Equal(t, "Taro", resp.User.FirstName)
	assert.Equal(t, "Yamada", resp.User.LastName)
	assert.True(t, resp.User.Enabled)
	assert.True(t, resp.User.EmailVerified)
	mockUC.AssertExpectations(t)
}

// --- TestGetUser_NotFound ---

func TestGetUser_NotFound(t *testing.T) {
	mockUC := new(MockGetUserUC)
	svc := &AuthGRPCService{
		getUserUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, "nonexistent").Return(nil, usecase.ErrUserNotFound)

	req := &GetUserRequest{UserId: "nonexistent"}
	resp, err := svc.GetUser(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, resp)
	// gRPC status code NOT_FOUND が期待される
	assert.Contains(t, err.Error(), "not found")
}

// --- TestListUsers_WithPagination ---

func TestListUsers_WithPagination(t *testing.T) {
	mockUC := new(MockListUsersUC)
	svc := &AuthGRPCService{
		listUsersUC: mockUC,
	}

	mockUC.On("Execute", mock.Anything, mock.MatchedBy(func(input usecase.ListUsersInput) bool {
		return input.Page == 2 && input.PageSize == 10
	})).Return(&usecase.ListUsersOutput{
		Users: []*model.User{
			{
				ID:       "user-1",
				Username: "taro.yamada",
				Email:    "taro@example.com",
				Enabled:  true,
			},
		},
		TotalCount: 25,
		Page:       2,
		PageSize:   10,
		HasNext:    true,
	}, nil)

	req := &ListUsersRequest{
		Pagination: &Pagination{Page: 2, PageSize: 10},
	}
	resp, err := svc.ListUsers(context.Background(), req)

	assert.NoError(t, err)
	assert.Len(t, resp.Users, 1)
	assert.Equal(t, "user-1", resp.Users[0].Id)
	assert.Equal(t, int32(25), resp.Pagination.TotalCount)
	assert.Equal(t, int32(2), resp.Pagination.Page)
	assert.Equal(t, int32(10), resp.Pagination.PageSize)
	assert.True(t, resp.Pagination.HasNext)
	mockUC.AssertExpectations(t)
}

// --- TestGetUserRoles_Exists ---

func TestGetUserRoles_Exists(t *testing.T) {
	mockUC := new(MockGetUserUC)
	svc := &AuthGRPCService{
		getUserUC: mockUC,
	}

	realmRoles := []*model.Role{
		{ID: "role-1", Name: "user", Description: "General user"},
		{ID: "role-2", Name: "sys_admin", Description: "System admin"},
	}
	clientRoles := map[string][]*model.Role{
		"order-service": {
			{ID: "role-3", Name: "read", Description: "Read access"},
		},
	}

	mockUC.On("GetUserRoles", mock.Anything, "user-uuid-1234").Return(realmRoles, clientRoles, nil)

	req := &GetUserRolesRequest{UserId: "user-uuid-1234"}
	resp, err := svc.GetUserRoles(context.Background(), req)

	assert.NoError(t, err)
	assert.Equal(t, "user-uuid-1234", resp.UserId)
	assert.Len(t, resp.RealmRoles, 2)
	assert.Equal(t, "user", resp.RealmRoles[0].Name)
	assert.Equal(t, "sys_admin", resp.RealmRoles[1].Name)
	assert.Len(t, resp.ClientRoles["order-service"].Roles, 1)
	assert.Equal(t, "read", resp.ClientRoles["order-service"].Roles[0].Name)
	mockUC.AssertExpectations(t)
}

// --- TestCheckPermission_Allowed ---

func TestCheckPermission_Allowed(t *testing.T) {
	svc := &AuthGRPCService{}

	req := &CheckPermissionRequest{
		UserId:     "user-uuid-1234",
		Permission: "admin",
		Resource:   "users",
		Roles:      []string{"sys_admin"},
	}

	resp, err := svc.CheckPermission(context.Background(), req)

	assert.NoError(t, err)
	assert.True(t, resp.Allowed)
	assert.Empty(t, resp.Reason)
}

// --- TestCheckPermission_Denied ---

func TestCheckPermission_Denied(t *testing.T) {
	svc := &AuthGRPCService{}

	req := &CheckPermissionRequest{
		UserId:     "user-uuid-1234",
		Permission: "admin",
		Resource:   "users",
		Roles:      []string{"user"},
	}

	resp, err := svc.CheckPermission(context.Background(), req)

	assert.NoError(t, err)
	assert.False(t, resp.Allowed)
	assert.NotEmpty(t, resp.Reason)
	assert.Contains(t, resp.Reason, "insufficient permissions")
}
