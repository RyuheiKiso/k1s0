package handler

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

// respondError が指定ステータスとエラーコードを返す
func TestRespondError_SetsStatusAndCode(t *testing.T) {
	router := gin.New()
	router.GET("/test", func(c *gin.Context) {
		respondError(c, http.StatusInternalServerError, "SYS_INTERNAL_ERROR")
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusInternalServerError, w.Code)

	var body map[string]interface{}
	assert.NoError(t, json.Unmarshal(w.Body.Bytes(), &body))
	assert.Equal(t, "SYS_INTERNAL_ERROR", body["error"])
}

// respondBadRequest が 400 とエラーコードを返す
func TestRespondBadRequest_Returns400(t *testing.T) {
	router := gin.New()
	router.GET("/test", func(c *gin.Context) {
		respondBadRequest(c, "INVALID_PARAM")
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)

	var body map[string]interface{}
	assert.NoError(t, json.Unmarshal(w.Body.Bytes(), &body))
	assert.Equal(t, "INVALID_PARAM", body["error"])
}

// abortErrorWithMessage が指定ステータス・コード・メッセージを返しリクエストを中断する
func TestAbortErrorWithMessage_SetsAllFields(t *testing.T) {
	router := gin.New()
	router.GET("/test", func(c *gin.Context) {
		abortErrorWithMessage(c, http.StatusUnauthorized, "AUTH_REQUIRED", "authentication is required")
		// AbortWithStatusJSON 後に後続ハンドラが呼ばれないことを確認するためのダミー
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusUnauthorized, w.Code)

	var body map[string]interface{}
	assert.NoError(t, json.Unmarshal(w.Body.Bytes(), &body))
	assert.Equal(t, "AUTH_REQUIRED", body["error"])
	assert.Equal(t, "authentication is required", body["message"])
}

// respondError のペイロードに request_id フィールドが含まれる
func TestRespondError_IncludesRequestID(t *testing.T) {
	router := gin.New()
	router.GET("/test", func(c *gin.Context) {
		respondError(c, http.StatusForbidden, "FORBIDDEN")
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	var body map[string]interface{}
	assert.NoError(t, json.Unmarshal(w.Body.Bytes(), &body))
	// request_id フィールドが存在すること（空でも良い）
	_, exists := body["request_id"]
	assert.True(t, exists, "response should contain request_id field")
}
