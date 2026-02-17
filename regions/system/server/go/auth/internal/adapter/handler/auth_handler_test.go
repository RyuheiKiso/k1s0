package handler

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/domain/repository"
	"github.com/k1s0-platform/system-server-go-auth/internal/usecase"
)

// --- Mock implementations ---

type mockTokenVerifier struct {
	mock.Mock
}

func (m *mockTokenVerifier) VerifyToken(ctx context.Context, tokenString string) (*model.TokenClaims, error) {
	args := m.Called(ctx, tokenString)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*model.TokenClaims), args.Error(1)
}

type mockUserRepo struct {
	mock.Mock
}

func (m *mockUserRepo) GetUser(ctx context.Context, userID string) (*model.User, error) {
	args := m.Called(ctx, userID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*model.User), args.Error(1)
}

func (m *mockUserRepo) ListUsers(ctx context.Context, params repository.UserListParams) ([]*model.User, int, error) {
	args := m.Called(ctx, params)
	if args.Get(0) == nil {
		return nil, args.Int(1), args.Error(2)
	}
	return args.Get(0).([]*model.User), args.Int(1), args.Error(2)
}

func (m *mockUserRepo) GetUserRoles(ctx context.Context, userID string) ([]*model.Role, map[string][]*model.Role, error) {
	args := m.Called(ctx, userID)
	if args.Get(0) == nil {
		return nil, nil, args.Error(2)
	}
	return args.Get(0).([]*model.Role), args.Get(1).(map[string][]*model.Role), args.Error(2)
}

func (m *mockUserRepo) Healthy(ctx context.Context) error {
	args := m.Called(ctx)
	return args.Error(0)
}

// --- Helper ---

func setupAuthRouter(verifier usecase.TokenVerifier, userRepo repository.UserRepository) *gin.Engine {
	gin.SetMode(gin.TestMode)
	r := gin.New()

	jwtConfig := usecase.JWTConfig{
		Issuer:   "https://auth.k1s0.internal.example.com/realms/k1s0",
		Audience: "k1s0-api",
	}

	validateTokenUC := usecase.NewValidateTokenUseCase(verifier, jwtConfig)
	getUserUC := usecase.NewGetUserUseCase(userRepo)
	listUsersUC := usecase.NewListUsersUseCase(userRepo)

	h := NewAuthHandler(validateTokenUC, getUserUC, listUsersUC)
	h.RegisterRoutes(r)

	return r
}

// --- Tests ---

func TestValidateToken_Success(t *testing.T) {
	mockVerifier := new(mockTokenVerifier)
	mockRepo := new(mockUserRepo)

	expectedClaims := &model.TokenClaims{
		Sub:              "user-uuid-1234",
		Iss:              "https://auth.k1s0.internal.example.com/realms/k1s0",
		Aud:              "k1s0-api",
		Exp:              1710000900,
		Iat:              1710000000,
		Jti:              "token-uuid-5678",
		PreferredUsername: "taro.yamada",
		Email:            "taro.yamada@example.com",
		RealmAccess: model.RealmAccess{
			Roles: []string{"user", "order_manager"},
		},
	}
	mockVerifier.On("VerifyToken", mock.Anything, "valid-token").Return(expectedClaims, nil)

	r := setupAuthRouter(mockVerifier, mockRepo)

	body, _ := json.Marshal(map[string]string{"token": "valid-token"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/token/validate", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, true, resp["valid"])
	assert.NotNil(t, resp["claims"])

	claims := resp["claims"].(map[string]interface{})
	assert.Equal(t, "user-uuid-1234", claims["sub"])
	assert.Equal(t, "taro.yamada", claims["preferred_username"])
	mockVerifier.AssertExpectations(t)
}

func TestValidateToken_InvalidToken(t *testing.T) {
	mockVerifier := new(mockTokenVerifier)
	mockRepo := new(mockUserRepo)

	mockVerifier.On("VerifyToken", mock.Anything, "bad-token").
		Return(nil, errors.New("signature verification failed"))

	r := setupAuthRouter(mockVerifier, mockRepo)

	body, _ := json.Marshal(map[string]string{"token": "bad-token"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/token/validate", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusUnauthorized, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_AUTH_TOKEN_INVALID", resp.Error.Code)
	mockVerifier.AssertExpectations(t)
}

func TestValidateToken_MissingToken(t *testing.T) {
	mockVerifier := new(mockTokenVerifier)
	mockRepo := new(mockUserRepo)

	r := setupAuthRouter(mockVerifier, mockRepo)

	body, _ := json.Marshal(map[string]string{})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/token/validate", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_AUTH_VALIDATION_FAILED", resp.Error.Code)
}

func TestIntrospectToken_Active(t *testing.T) {
	mockVerifier := new(mockTokenVerifier)
	mockRepo := new(mockUserRepo)

	expectedClaims := &model.TokenClaims{
		Sub:              "user-uuid-1234",
		Iss:              "https://auth.k1s0.internal.example.com/realms/k1s0",
		Aud:              "k1s0-api",
		Exp:              1710000900,
		Iat:              1710000000,
		Azp:              "react-spa",
		Typ:              "Bearer",
		Scope:            "openid profile email",
		PreferredUsername: "taro.yamada",
		RealmAccess: model.RealmAccess{
			Roles: []string{"user", "order_manager"},
		},
	}
	mockVerifier.On("VerifyToken", mock.Anything, "valid-token").Return(expectedClaims, nil)

	r := setupAuthRouter(mockVerifier, mockRepo)

	body, _ := json.Marshal(map[string]string{
		"token":           "valid-token",
		"token_type_hint": "access_token",
	})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/token/introspect", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, true, resp["active"])
	assert.Equal(t, "user-uuid-1234", resp["sub"])
	assert.Equal(t, "react-spa", resp["client_id"])
	assert.Equal(t, "taro.yamada", resp["username"])
	mockVerifier.AssertExpectations(t)
}

func TestIntrospectToken_Inactive(t *testing.T) {
	mockVerifier := new(mockTokenVerifier)
	mockRepo := new(mockUserRepo)

	mockVerifier.On("VerifyToken", mock.Anything, "expired-token").
		Return(nil, errors.New("token expired"))

	r := setupAuthRouter(mockVerifier, mockRepo)

	body, _ := json.Marshal(map[string]string{"token": "expired-token"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/token/introspect", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, false, resp["active"])
	mockVerifier.AssertExpectations(t)
}

func TestGetUser_Success(t *testing.T) {
	mockVerifier := new(mockTokenVerifier)
	mockRepo := new(mockUserRepo)

	expectedUser := &model.User{
		ID:        "user-uuid-1234",
		Username:  "taro.yamada",
		Email:     "taro.yamada@example.com",
		FirstName: "太郎",
		LastName:  "山田",
		Enabled:   true,
	}
	mockRepo.On("GetUser", mock.Anything, "user-uuid-1234").Return(expectedUser, nil)

	r := setupAuthRouter(mockVerifier, mockRepo)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/users/user-uuid-1234", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var user model.User
	err := json.Unmarshal(w.Body.Bytes(), &user)
	assert.NoError(t, err)
	assert.Equal(t, "user-uuid-1234", user.ID)
	assert.Equal(t, "taro.yamada", user.Username)
	mockRepo.AssertExpectations(t)
}

func TestGetUser_NotFound(t *testing.T) {
	mockVerifier := new(mockTokenVerifier)
	mockRepo := new(mockUserRepo)

	mockRepo.On("GetUser", mock.Anything, "nonexistent").
		Return(nil, errors.New("user not found"))

	r := setupAuthRouter(mockVerifier, mockRepo)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/users/nonexistent", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusNotFound, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_AUTH_USER_NOT_FOUND", resp.Error.Code)
	mockRepo.AssertExpectations(t)
}

func TestListUsers_Success(t *testing.T) {
	mockVerifier := new(mockTokenVerifier)
	mockRepo := new(mockUserRepo)

	users := []*model.User{
		{ID: "user-1", Username: "user1", Email: "user1@example.com", Enabled: true},
		{ID: "user-2", Username: "user2", Email: "user2@example.com", Enabled: true},
	}
	mockRepo.On("ListUsers", mock.Anything, repository.UserListParams{
		Page:     1,
		PageSize: 20,
	}).Return(users, 150, nil)

	r := setupAuthRouter(mockVerifier, mockRepo)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/users?page=1&page_size=20", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)

	usersResp := resp["users"].([]interface{})
	assert.Len(t, usersResp, 2)

	pagination := resp["pagination"].(map[string]interface{})
	assert.Equal(t, float64(150), pagination["total_count"])
	assert.Equal(t, float64(1), pagination["page"])
	assert.Equal(t, float64(20), pagination["page_size"])
	assert.Equal(t, true, pagination["has_next"])
	mockRepo.AssertExpectations(t)
}
