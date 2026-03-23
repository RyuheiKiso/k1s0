package handler

import (
	"net/http"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/middleware"
)

// errorBody は ADR-0005 準拠のエラーレスポンス内部構造。
// Rust サービスと共通のフォーマットにすることで、クライアント側の型チェックを統一する。
type errorBody struct {
	Code      string        `json:"code"`
	Message   string        `json:"message"`
	RequestID string        `json:"request_id"`
	// エラー詳細リスト（interface{} → any: Go 1.18+ 推奨エイリアスを使用する）
	Details   []any `json:"details"`
}

// errorResponse は ADR-0005 準拠のレスポンス外部構造。
type errorResponse struct {
	Error errorBody `json:"error"`
}

// respondError は ADR-0005 形式のエラーレスポンスを返す。
func respondError(c *gin.Context, status int, code string) {
	respondErrorWithMessage(c, status, code, code)
}

// respondErrorWithMessage はメッセージ付きの ADR-0005 形式エラーレスポンスを返す。
func respondErrorWithMessage(c *gin.Context, status int, code, message string) {
	payload := errorResponse{
		Error: errorBody{
			Code:      code,
			Message:   message,
			RequestID: middleware.GetRequestID(c),
			Details:   []any{},
		},
	}
	c.JSON(status, payload)
}

func respondBadRequest(c *gin.Context, code string) {
	respondError(c, http.StatusBadRequest, code)
}

// abortErrorWithMessage はリクエストを中断し ADR-0005 形式のエラーレスポンスを返す。
func abortErrorWithMessage(c *gin.Context, status int, code, message string) {
	payload := errorResponse{
		Error: errorBody{
			Code:      code,
			Message:   message,
			RequestID: middleware.GetRequestID(c),
			Details:   []any{},
		},
	}
	c.AbortWithStatusJSON(status, payload)
}
