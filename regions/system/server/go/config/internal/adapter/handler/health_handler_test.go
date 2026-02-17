package handler

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

type mockHealthChecker struct {
	mock.Mock
}

func (m *mockHealthChecker) Healthy(ctx context.Context) error {
	args := m.Called(ctx)
	return args.Error(0)
}

func TestHealthzHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)
	r := gin.New()
	r.GET("/healthz", HealthzHandler())

	req := httptest.NewRequest(http.MethodGet, "/healthz", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var resp map[string]string
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "ok", resp["status"])
}

func TestReadyzHandler_AllReady(t *testing.T) {
	gin.SetMode(gin.TestMode)
	r := gin.New()

	mockDB := new(mockHealthChecker)
	mockKafka := new(mockHealthChecker)
	mockDB.On("Healthy", mock.Anything).Return(nil)
	mockKafka.On("Healthy", mock.Anything).Return(nil)

	r.GET("/readyz", ReadyzHandler(mockDB, mockKafka))

	req := httptest.NewRequest(http.MethodGet, "/readyz", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "ready", resp["status"])

	checks := resp["checks"].(map[string]interface{})
	assert.Equal(t, "ok", checks["database"])
	assert.Equal(t, "ok", checks["kafka"])
	mockDB.AssertExpectations(t)
	mockKafka.AssertExpectations(t)
}

func TestReadyzHandler_DatabaseDown(t *testing.T) {
	gin.SetMode(gin.TestMode)
	r := gin.New()

	mockDB := new(mockHealthChecker)
	mockKafka := new(mockHealthChecker)
	mockDB.On("Healthy", mock.Anything).Return(errors.New("connection refused"))
	mockKafka.On("Healthy", mock.Anything).Return(nil)

	r.GET("/readyz", ReadyzHandler(mockDB, mockKafka))

	req := httptest.NewRequest(http.MethodGet, "/readyz", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusServiceUnavailable, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "not ready", resp["status"])

	checks := resp["checks"].(map[string]interface{})
	assert.Equal(t, "error: connection refused", checks["database"])
	assert.Equal(t, "ok", checks["kafka"])
}

func TestReadyzHandler_KafkaDown(t *testing.T) {
	gin.SetMode(gin.TestMode)
	r := gin.New()

	mockDB := new(mockHealthChecker)
	mockKafka := new(mockHealthChecker)
	mockDB.On("Healthy", mock.Anything).Return(nil)
	mockKafka.On("Healthy", mock.Anything).Return(errors.New("connection timeout"))

	r.GET("/readyz", ReadyzHandler(mockDB, mockKafka))

	req := httptest.NewRequest(http.MethodGet, "/readyz", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusServiceUnavailable, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "not ready", resp["status"])

	checks := resp["checks"].(map[string]interface{})
	assert.Equal(t, "ok", checks["database"])
	assert.Equal(t, "error: connection timeout", checks["kafka"])
}

func TestReadyzHandler_NilCheckers(t *testing.T) {
	gin.SetMode(gin.TestMode)
	r := gin.New()

	r.GET("/readyz", ReadyzHandler(nil, nil))

	req := httptest.NewRequest(http.MethodGet, "/readyz", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "ready", resp["status"])
}

func TestReadyzHandler_BothDown(t *testing.T) {
	gin.SetMode(gin.TestMode)
	r := gin.New()

	mockDB := new(mockHealthChecker)
	mockKafka := new(mockHealthChecker)
	mockDB.On("Healthy", mock.Anything).Return(errors.New("connection refused"))
	mockKafka.On("Healthy", mock.Anything).Return(errors.New("connection timeout"))

	r.GET("/readyz", ReadyzHandler(mockDB, mockKafka))

	req := httptest.NewRequest(http.MethodGet, "/readyz", nil)
	w := httptest.NewRecorder()

	r.ServeHTTP(w, req)

	assert.Equal(t, http.StatusServiceUnavailable, w.Code)

	var resp map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &resp)
	assert.NoError(t, err)
	assert.Equal(t, "not ready", resp["status"])

	checks := resp["checks"].(map[string]interface{})
	assert.Equal(t, "error: connection refused", checks["database"])
	assert.Equal(t, "error: connection timeout", checks["kafka"])
}
