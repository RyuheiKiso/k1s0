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

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/domain/repository"
	"github.com/k1s0-platform/system-server-go-auth/internal/usecase"
)

// --- Mock implementations ---

type mockAuditLogRepo struct {
	mock.Mock
}

func (m *mockAuditLogRepo) Create(ctx context.Context, log *model.AuditLog) error {
	args := m.Called(ctx, log)
	return args.Error(0)
}

func (m *mockAuditLogRepo) Search(ctx context.Context, params repository.AuditLogSearchParams) ([]*model.AuditLog, int, error) {
	args := m.Called(ctx, params)
	if args.Get(0) == nil {
		return nil, args.Int(1), args.Error(2)
	}
	return args.Get(0).([]*model.AuditLog), args.Int(1), args.Error(2)
}

type mockPublisher struct {
	mock.Mock
}

func (m *mockPublisher) Publish(ctx context.Context, log *model.AuditLog) error {
	args := m.Called(ctx, log)
	return args.Error(0)
}

// --- Helper ---

func setupAuditRouter(auditRepo repository.AuditLogRepository, publisher usecase.AuditEventPublisher) *gin.Engine {
	gin.SetMode(gin.TestMode)
	r := gin.New()

	recordAuditLogUC := usecase.NewRecordAuditLogUseCase(auditRepo, publisher)
	searchAuditLogsUC := usecase.NewSearchAuditLogsUseCase(auditRepo)

	h := NewAuditHandler(recordAuditLogUC, searchAuditLogsUC)
	h.RegisterRoutes(r)

	return r
}

// --- Tests ---

func TestRecordAuditLog_Success(t *testing.T) {
	mockRepo := new(mockAuditLogRepo)
	mockPub := new(mockPublisher)

	mockRepo.On("Create", mock.Anything, mock.AnythingOfType("*model.AuditLog")).Return(nil)
	mockPub.On("Publish", mock.Anything, mock.AnythingOfType("*model.AuditLog")).Return(nil)

	r := setupAuditRouter(mockRepo, mockPub)

	body, _ := json.Marshal(map[string]interface{}{
		"event_type": "LOGIN_SUCCESS",
		"user_id":    "user-uuid-1234",
		"ip_address": "192.168.1.100",
		"user_agent": "Mozilla/5.0",
		"resource":   "/api/v1/auth/token",
		"action":     "POST",
		"result":     "SUCCESS",
		"metadata": map[string]string{
			"client_id":  "react-spa",
			"grant_type": "authorization_code",
		},
	})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/audit/logs", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusCreated, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.NotEmpty(t, resp["id"])
	assert.NotEmpty(t, resp["recorded_at"])
	mockRepo.AssertExpectations(t)
	mockPub.AssertExpectations(t)
}

func TestRecordAuditLog_BadRequest(t *testing.T) {
	mockRepo := new(mockAuditLogRepo)
	mockPub := new(mockPublisher)

	r := setupAuditRouter(mockRepo, mockPub)

	// 不正なJSON
	req := httptest.NewRequest(http.MethodPost, "/api/v1/audit/logs", bytes.NewReader([]byte("invalid")))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)
}

func TestRecordAuditLog_InternalError(t *testing.T) {
	mockRepo := new(mockAuditLogRepo)
	mockPub := new(mockPublisher)

	mockRepo.On("Create", mock.Anything, mock.AnythingOfType("*model.AuditLog")).
		Return(errors.New("database error"))

	r := setupAuditRouter(mockRepo, mockPub)

	body, _ := json.Marshal(map[string]interface{}{
		"event_type": "LOGIN_SUCCESS",
		"user_id":    "user-uuid-1234",
		"result":     "SUCCESS",
	})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/audit/logs", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusInternalServerError, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_AUTH_INTERNAL_ERROR", resp.Error.Code)
	mockRepo.AssertExpectations(t)
}

func TestSearchAuditLogs_Success(t *testing.T) {
	mockRepo := new(mockAuditLogRepo)
	mockPub := new(mockPublisher)

	now := time.Now().UTC()
	logs := []*model.AuditLog{
		{
			ID:         "audit-uuid-5678",
			EventType:  "LOGIN_SUCCESS",
			UserID:     "user-uuid-1234",
			IPAddress:  "192.168.1.100",
			Resource:   "/api/v1/auth/token",
			Action:     "POST",
			Result:     "SUCCESS",
			RecordedAt: now,
			Metadata: map[string]string{
				"client_id": "react-spa",
			},
		},
	}

	mockRepo.On("Search", mock.Anything, repository.AuditLogSearchParams{
		Page:     1,
		PageSize: 50,
	}).Return(logs, 5000, nil)

	r := setupAuditRouter(mockRepo, mockPub)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/audit/logs", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)

	logsResp := resp["logs"].([]interface{})
	assert.Len(t, logsResp, 1)

	pagination := resp["pagination"].(map[string]interface{})
	assert.Equal(t, float64(5000), pagination["total_count"])
	assert.Equal(t, true, pagination["has_next"])
	mockRepo.AssertExpectations(t)
}

func TestSearchAuditLogs_WithFilters(t *testing.T) {
	mockRepo := new(mockAuditLogRepo)
	mockPub := new(mockPublisher)

	mockRepo.On("Search", mock.Anything, mock.MatchedBy(func(params repository.AuditLogSearchParams) bool {
		return params.UserID == "user-uuid-1234" &&
			params.EventType == "LOGIN_SUCCESS" &&
			params.Result == "SUCCESS"
	})).Return([]*model.AuditLog{}, 0, nil)

	r := setupAuditRouter(mockRepo, mockPub)

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/audit/logs?user_id=user-uuid-1234&event_type=LOGIN_SUCCESS&result=SUCCESS", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
	mockRepo.AssertExpectations(t)
}

func TestSearchAuditLogs_InternalError(t *testing.T) {
	mockRepo := new(mockAuditLogRepo)
	mockPub := new(mockPublisher)

	mockRepo.On("Search", mock.Anything, mock.Anything).
		Return(nil, 0, errors.New("database error"))

	r := setupAuditRouter(mockRepo, mockPub)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/audit/logs", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusInternalServerError, w.Code)

	var resp ErrorResponse
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "SYS_AUTH_INTERNAL_ERROR", resp.Error.Code)
	mockRepo.AssertExpectations(t)
}
