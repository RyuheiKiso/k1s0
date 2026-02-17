package handler

import (
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-auth/internal/usecase"
)

// AuditHandler は監査ログ関連の REST ハンドラー。
type AuditHandler struct {
	recordAuditLogUC  *usecase.RecordAuditLogUseCase
	searchAuditLogsUC *usecase.SearchAuditLogsUseCase
}

// NewAuditHandler は新しい AuditHandler を作成する。
func NewAuditHandler(
	recordAuditLogUC *usecase.RecordAuditLogUseCase,
	searchAuditLogsUC *usecase.SearchAuditLogsUseCase,
) *AuditHandler {
	return &AuditHandler{
		recordAuditLogUC:  recordAuditLogUC,
		searchAuditLogsUC: searchAuditLogsUC,
	}
}

// RecordAuditLog は POST /api/v1/audit/logs のハンドラー。
func (h *AuditHandler) RecordAuditLog(c *gin.Context) {
	var input usecase.RecordAuditLogInput
	if err := c.ShouldBindJSON(&input); err != nil {
		WriteError(c, http.StatusBadRequest, "SYS_AUTH_VALIDATION_FAILED",
			"リクエストのバリデーションに失敗しました")
		return
	}

	output, err := h.recordAuditLogUC.Execute(c.Request.Context(), input)
	if err != nil {
		WriteError(c, http.StatusInternalServerError, "SYS_AUTH_INTERNAL_ERROR",
			"監査ログの記録に失敗しました")
		return
	}

	c.JSON(http.StatusCreated, gin.H{
		"id":          output.ID,
		"recorded_at": output.RecordedAt.Format(time.RFC3339Nano),
	})
}

// SearchAuditLogs は GET /api/v1/audit/logs のハンドラー。
func (h *AuditHandler) SearchAuditLogs(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "50"))

	input := usecase.SearchAuditLogsInput{
		Page:      page,
		PageSize:  pageSize,
		UserID:    c.Query("user_id"),
		EventType: c.Query("event_type"),
		Result:    c.Query("result"),
	}

	if fromStr := c.Query("from"); fromStr != "" {
		from, err := time.Parse(time.RFC3339, fromStr)
		if err == nil {
			input.From = &from
		}
	}
	if toStr := c.Query("to"); toStr != "" {
		to, err := time.Parse(time.RFC3339, toStr)
		if err == nil {
			input.To = &to
		}
	}

	output, err := h.searchAuditLogsUC.Execute(c.Request.Context(), input)
	if err != nil {
		WriteError(c, http.StatusInternalServerError, "SYS_AUTH_INTERNAL_ERROR",
			"監査ログの検索に失敗しました")
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"logs": output.Logs,
		"pagination": gin.H{
			"total_count": output.TotalCount,
			"page":        output.Page,
			"page_size":   output.PageSize,
			"has_next":    output.HasNext,
		},
	})
}

// RegisterRoutes は監査ログのルートを登録する。
func (h *AuditHandler) RegisterRoutes(r *gin.Engine) {
	audit := r.Group("/api/v1/audit")
	{
		audit.POST("/logs", h.RecordAuditLog)
		audit.GET("/logs", h.SearchAuditLogs)
	}
}
