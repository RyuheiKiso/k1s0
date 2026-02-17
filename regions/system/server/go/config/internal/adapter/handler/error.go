package handler

import (
	"github.com/gin-gonic/gin"
)

// ErrorResponse は統一エラーレスポンス（API設計.md D-007 準拠）。
type ErrorResponse struct {
	Error ErrorDetail `json:"error"`
}

// ErrorDetail はエラーの詳細情報。
type ErrorDetail struct {
	Code      string   `json:"code"`
	Message   string   `json:"message"`
	RequestID string   `json:"request_id"`
	Details   []string `json:"details"`
}

// WriteError は統一フォーマットのエラーレスポンスを書き込む。
func WriteError(c *gin.Context, statusCode int, code string, message string) {
	requestID, _ := c.Get("request_id")
	reqID, _ := requestID.(string)

	c.JSON(statusCode, ErrorResponse{
		Error: ErrorDetail{
			Code:      code,
			Message:   message,
			RequestID: reqID,
			Details:   []string{},
		},
	})
}
