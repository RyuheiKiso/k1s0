package handler

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/k1s0-platform/system-server-go-config/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-config/internal/domain/repository"
	"github.com/k1s0-platform/system-server-go-config/internal/usecase"
)

// --- Mock implementations ---

type mockConfigRepo struct {
	mock.Mock
}

func (m *mockConfigRepo) GetByKey(ctx context.Context, namespace, key string) (*model.ConfigEntry, error) {
	args := m.Called(ctx, namespace, key)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*model.ConfigEntry), args.Error(1)
}

func (m *mockConfigRepo) ListByNamespace(ctx context.Context, params repository.ConfigListParams) ([]*model.ConfigEntry, int, error) {
	args := m.Called(ctx, params)
	if args.Get(0) == nil {
		return nil, args.Int(1), args.Error(2)
	}
	return args.Get(0).([]*model.ConfigEntry), args.Int(1), args.Error(2)
}

func (m *mockConfigRepo) GetByServiceName(ctx context.Context, serviceName string) ([]*model.ConfigEntry, error) {
	args := m.Called(ctx, serviceName)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]*model.ConfigEntry), args.Error(1)
}

func (m *mockConfigRepo) Create(ctx context.Context, entry *model.ConfigEntry) error {
	args := m.Called(ctx, entry)
	return args.Error(0)
}

func (m *mockConfigRepo) Update(ctx context.Context, entry *model.ConfigEntry, expectedVersion int) error {
	args := m.Called(ctx, entry, expectedVersion)
	return args.Error(0)
}

func (m *mockConfigRepo) Delete(ctx context.Context, namespace, key string) error {
	args := m.Called(ctx, namespace, key)
	return args.Error(0)
}

type mockConfigPublisher struct {
	mock.Mock
}

func (m *mockConfigPublisher) Publish(ctx context.Context, log *model.ConfigChangeLog) error {
	args := m.Called(ctx, log)
	return args.Error(0)
}

// --- Helper ---

func setupConfigRouter(repo repository.ConfigRepository, publisher usecase.ConfigChangeEventPublisher) *gin.Engine {
	gin.SetMode(gin.TestMode)
	r := gin.New()

	getConfigUC := usecase.NewGetConfigUseCase(repo)
	listConfigsUC := usecase.NewListConfigsUseCase(repo)
	updateConfigUC := usecase.NewUpdateConfigUseCase(repo, publisher)
	deleteConfigUC := usecase.NewDeleteConfigUseCase(repo, publisher)
	getServiceConfigUC := usecase.NewGetServiceConfigUseCase(repo)

	h := NewConfigHandler(getConfigUC, listConfigsUC, updateConfigUC, deleteConfigUC, getServiceConfigUC)
	h.RegisterRoutes(r)

	return r
}

// --- Tests ---

func TestGetConfig_Success(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

	now := time.Now().UTC()
	entry := &model.ConfigEntry{
		ID:          "entry-uuid-1234",
		Namespace:   "system.auth.database",
		Key:         "max_connections",
		ValueJSON:   json.RawMessage(`25`),
		Version:     3,
		Description: "認証サーバーの DB 最大接続数",
		UpdatedBy:   "admin@example.com",
		UpdatedAt:   now,
	}
	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "max_connections").Return(entry, nil)

	r := setupConfigRouter(mockRepo, mockPub)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/config/system.auth.database/max_connections", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "system.auth.database", resp["namespace"])
	assert.Equal(t, "max_connections", resp["key"])
	assert.Equal(t, float64(25), resp["value"])
	assert.Equal(t, float64(3), resp["version"])
	assert.Equal(t, "認証サーバーの DB 最大接続数", resp["description"])
	assert.Equal(t, "admin@example.com", resp["updated_by"])
	mockRepo.AssertExpectations(t)
}

func TestGetConfig_NotFound(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "nonexistent").
		Return(nil, errors.New("config entry not found"))

	r := setupConfigRouter(mockRepo, mockPub)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/config/system.auth.database/nonexistent", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusNotFound, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_CONFIG_KEY_NOT_FOUND", resp.Error.Code)
	mockRepo.AssertExpectations(t)
}

func TestListConfigs_Success(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

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

	r := setupConfigRouter(mockRepo, mockPub)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/config/system.auth.database?page=1&page_size=20", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)

	entriesResp := resp["entries"].([]interface{})
	assert.Len(t, entriesResp, 2)

	pagination := resp["pagination"].(map[string]interface{})
	assert.Equal(t, float64(42), pagination["total_count"])
	assert.Equal(t, float64(1), pagination["page"])
	assert.Equal(t, float64(20), pagination["page_size"])
	assert.Equal(t, true, pagination["has_next"])
	mockRepo.AssertExpectations(t)
}

func TestListConfigs_InternalError(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

	mockRepo.On("ListByNamespace", mock.Anything, mock.Anything).
		Return(nil, 0, errors.New("database error"))

	r := setupConfigRouter(mockRepo, mockPub)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/config/system.auth.database", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusInternalServerError, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_CONFIG_INTERNAL_ERROR", resp.Error.Code)
	mockRepo.AssertExpectations(t)
}

func TestUpdateConfig_Success(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

	now := time.Now().UTC()
	existing := &model.ConfigEntry{
		ID:          "entry-uuid-1234",
		Namespace:   "system.auth.database",
		Key:         "max_connections",
		ValueJSON:   json.RawMessage(`25`),
		Version:     3,
		Description: "認証サーバーの DB 最大接続数",
		UpdatedBy:   "admin@example.com",
		UpdatedAt:   now,
	}

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "max_connections").Return(existing, nil)
	mockRepo.On("Update", mock.Anything, mock.AnythingOfType("*model.ConfigEntry"), 3).Return(nil)
	mockPub.On("Publish", mock.Anything, mock.AnythingOfType("*model.ConfigChangeLog")).Return(nil)

	r := setupConfigRouter(mockRepo, mockPub)

	body, _ := json.Marshal(map[string]interface{}{
		"value":       50,
		"version":     3,
		"description": "認証サーバーの DB 最大接続数（増設）",
	})
	req := httptest.NewRequest(http.MethodPut, "/api/v1/config/system.auth.database/max_connections", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-User-Email", "operator@example.com")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "system.auth.database", resp["namespace"])
	assert.Equal(t, "max_connections", resp["key"])
	assert.Equal(t, float64(4), resp["version"])
	mockRepo.AssertExpectations(t)
	mockPub.AssertExpectations(t)
}

func TestUpdateConfig_ValidationFailed(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

	r := setupConfigRouter(mockRepo, mockPub)

	// 不正なJSON
	req := httptest.NewRequest(http.MethodPut, "/api/v1/config/system.auth.database/max_connections",
		bytes.NewReader([]byte("invalid")))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_CONFIG_VALIDATION_FAILED", resp.Error.Code)
}

func TestUpdateConfig_VersionConflict(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

	now := time.Now().UTC()
	existing := &model.ConfigEntry{
		ID:        "entry-uuid-1234",
		Namespace: "system.auth.database",
		Key:       "max_connections",
		ValueJSON: json.RawMessage(`25`),
		Version:   4, // 既にバージョン4
		UpdatedBy: "admin@example.com",
		UpdatedAt: now,
	}

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "max_connections").Return(existing, nil)

	r := setupConfigRouter(mockRepo, mockPub)

	body, _ := json.Marshal(map[string]interface{}{
		"value":   50,
		"version": 3, // 古いバージョン
	})
	req := httptest.NewRequest(http.MethodPut, "/api/v1/config/system.auth.database/max_connections", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusConflict, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_CONFIG_VERSION_CONFLICT", resp.Error.Code)
	mockRepo.AssertExpectations(t)
}

func TestUpdateConfig_InternalError(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

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

	r := setupConfigRouter(mockRepo, mockPub)

	body, _ := json.Marshal(map[string]interface{}{
		"value":   50,
		"version": 3,
	})
	req := httptest.NewRequest(http.MethodPut, "/api/v1/config/system.auth.database/max_connections", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusInternalServerError, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_CONFIG_INTERNAL_ERROR", resp.Error.Code)
	mockRepo.AssertExpectations(t)
}

func TestDeleteConfig_Success(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

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
	mockPub.On("Publish", mock.Anything, mock.AnythingOfType("*model.ConfigChangeLog")).Return(nil)

	r := setupConfigRouter(mockRepo, mockPub)

	req := httptest.NewRequest(http.MethodDelete, "/api/v1/config/system.auth.database/max_connections", nil)
	req.Header.Set("X-User-Email", "admin@example.com")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusNoContent, w.Code)
	mockRepo.AssertExpectations(t)
	mockPub.AssertExpectations(t)
}

func TestDeleteConfig_NotFound(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

	mockRepo.On("GetByKey", mock.Anything, "system.auth.database", "nonexistent").
		Return(nil, errors.New("config entry not found"))

	r := setupConfigRouter(mockRepo, mockPub)

	req := httptest.NewRequest(http.MethodDelete, "/api/v1/config/system.auth.database/nonexistent", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusNotFound, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_CONFIG_KEY_NOT_FOUND", resp.Error.Code)
	mockRepo.AssertExpectations(t)
}

func TestDeleteConfig_InternalError(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

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

	r := setupConfigRouter(mockRepo, mockPub)

	req := httptest.NewRequest(http.MethodDelete, "/api/v1/config/system.auth.database/max_connections", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusInternalServerError, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_CONFIG_INTERNAL_ERROR", resp.Error.Code)
	mockRepo.AssertExpectations(t)
}

func TestGetServiceConfig_Success(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

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
			Namespace: "system.auth.jwt",
			Key:       "issuer",
			ValueJSON: json.RawMessage(`"https://auth.k1s0.internal.example.com/realms/k1s0"`),
			Version:   1,
			UpdatedBy: "admin@example.com",
			UpdatedAt: now,
		},
	}

	mockRepo.On("GetByServiceName", mock.Anything, "auth-server").Return(entries, nil)

	r := setupConfigRouter(mockRepo, mockPub)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/config/services/auth-server", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "auth-server", resp["service_name"])

	entriesResp := resp["entries"].([]interface{})
	assert.Len(t, entriesResp, 2)
	mockRepo.AssertExpectations(t)
}

func TestGetServiceConfig_NotFound(t *testing.T) {
	mockRepo := new(mockConfigRepo)
	mockPub := new(mockConfigPublisher)

	mockRepo.On("GetByServiceName", mock.Anything, "nonexistent-service").
		Return([]*model.ConfigEntry{}, nil)

	r := setupConfigRouter(mockRepo, mockPub)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/config/services/nonexistent-service", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusNotFound, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_CONFIG_SERVICE_NOT_FOUND", resp.Error.Code)
	mockRepo.AssertExpectations(t)
}
