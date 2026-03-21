package handler

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

// ADR-0005 形式のレスポンスから error オブジェクトを取り出すヘルパー。
func extractErrorBody(t *testing.T, body map[string]interface{}) map[string]interface{} {
	t.Helper()
	errObj, ok := body["error"].(map[string]interface{})
	assert.True(t, ok, "body.error should be an object")
	return errObj
}

// respondError が ADR-0005 形式で指定ステータスとエラーコードを返す
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
	errObj := extractErrorBody(t, body)
	assert.Equal(t, "SYS_INTERNAL_ERROR", errObj["code"])
}

// respondBadRequest が ADR-0005 形式で 400 とエラーコードを返す
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
	errObj := extractErrorBody(t, body)
	assert.Equal(t, "INVALID_PARAM", errObj["code"])
}

// abortErrorWithMessage が ADR-0005 形式で指定ステータス・コード・メッセージを返しリクエストを中断する
func TestAbortErrorWithMessage_SetsAllFields(t *testing.T) {
	router := gin.New()
	router.GET("/test", func(c *gin.Context) {
		abortErrorWithMessage(c, http.StatusUnauthorized, "AUTH_REQUIRED", "authentication is required")
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusUnauthorized, w.Code)

	var body map[string]interface{}
	assert.NoError(t, json.Unmarshal(w.Body.Bytes(), &body))
	errObj := extractErrorBody(t, body)
	assert.Equal(t, "AUTH_REQUIRED", errObj["code"])
	assert.Equal(t, "authentication is required", errObj["message"])
}

// respondError のペイロードに request_id フィールドが error オブジェクト内に含まれる
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
	errObj := extractErrorBody(t, body)
	// request_id フィールドが error オブジェクト内に存在すること
	_, exists := errObj["request_id"]
	assert.True(t, exists, "error object should contain request_id field")
}

// respondError のレスポンスに details フィールドが空配列で含まれる
func TestRespondError_IncludesEmptyDetails(t *testing.T) {
	router := gin.New()
	router.GET("/test", func(c *gin.Context) {
		respondError(c, http.StatusBadRequest, "SOME_ERROR")
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	var body map[string]interface{}
	assert.NoError(t, json.Unmarshal(w.Body.Bytes(), &body))
	errObj := extractErrorBody(t, body)
	details, exists := errObj["details"]
	assert.True(t, exists, "error object should contain details field")
	assert.IsType(t, []interface{}{}, details)
}
