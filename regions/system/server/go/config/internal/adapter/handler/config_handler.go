package handler

import (
	"encoding/json"
	"errors"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-config/internal/usecase"
)

// ConfigHandler は設定管理関連の REST ハンドラー。
type ConfigHandler struct {
	getConfigUC        *usecase.GetConfigUseCase
	listConfigsUC      *usecase.ListConfigsUseCase
	updateConfigUC     *usecase.UpdateConfigUseCase
	deleteConfigUC     *usecase.DeleteConfigUseCase
	getServiceConfigUC *usecase.GetServiceConfigUseCase
}

// NewConfigHandler は新しい ConfigHandler を作成する。
func NewConfigHandler(
	getConfigUC *usecase.GetConfigUseCase,
	listConfigsUC *usecase.ListConfigsUseCase,
	updateConfigUC *usecase.UpdateConfigUseCase,
	deleteConfigUC *usecase.DeleteConfigUseCase,
	getServiceConfigUC *usecase.GetServiceConfigUseCase,
) *ConfigHandler {
	return &ConfigHandler{
		getConfigUC:        getConfigUC,
		listConfigsUC:      listConfigsUC,
		updateConfigUC:     updateConfigUC,
		deleteConfigUC:     deleteConfigUC,
		getServiceConfigUC: getServiceConfigUC,
	}
}

// GetConfig は GET /api/v1/config/:namespace/:key のハンドラー。
func (h *ConfigHandler) GetConfig(c *gin.Context) {
	namespace := c.Param("namespace")
	key := c.Param("key")

	input := usecase.GetConfigInput{
		Namespace: namespace,
		Key:       key,
	}

	output, err := h.getConfigUC.Execute(c.Request.Context(), input)
	if err != nil {
		WriteError(c, http.StatusNotFound, "SYS_CONFIG_KEY_NOT_FOUND",
			"指定された設定キーが見つかりません")
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"namespace":   output.Namespace,
		"key":         output.Key,
		"value":       output.Value,
		"version":     output.Version,
		"description": output.Description,
		"updated_by":  output.UpdatedBy,
		"updated_at":  output.UpdatedAt,
	})
}

// ListConfigs は GET /api/v1/config/:namespace のハンドラー。
func (h *ConfigHandler) ListConfigs(c *gin.Context) {
	namespace := c.Param("namespace")
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	search := c.Query("search")

	input := usecase.ListConfigsInput{
		Namespace: namespace,
		Search:    search,
		Page:      page,
		PageSize:  pageSize,
	}

	output, err := h.listConfigsUC.Execute(c.Request.Context(), input)
	if err != nil {
		WriteError(c, http.StatusInternalServerError, "SYS_CONFIG_INTERNAL_ERROR",
			"設定値一覧の取得に失敗しました")
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"entries": output.Entries,
		"pagination": gin.H{
			"total_count": output.TotalCount,
			"page":        output.Page,
			"page_size":   output.PageSize,
			"has_next":    output.HasNext,
		},
	})
}

// UpdateConfig は PUT /api/v1/config/:namespace/:key のハンドラー。
func (h *ConfigHandler) UpdateConfig(c *gin.Context) {
	namespace := c.Param("namespace")
	key := c.Param("key")

	var req struct {
		Value       json.RawMessage `json:"value" binding:"required"`
		Version     int             `json:"version" binding:"required"`
		Description string          `json:"description"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		WriteError(c, http.StatusBadRequest, "SYS_CONFIG_VALIDATION_FAILED",
			"リクエストのバリデーションに失敗しました")
		return
	}

	// TODO: 認証情報から取得する（現在はヘッダーから取得）
	updatedBy := c.GetHeader("X-User-Email")
	if updatedBy == "" {
		updatedBy = "unknown"
	}

	input := usecase.UpdateConfigInput{
		Namespace:   namespace,
		Key:         key,
		Value:       req.Value,
		Version:     req.Version,
		Description: req.Description,
		UpdatedBy:   updatedBy,
	}

	output, err := h.updateConfigUC.Execute(c.Request.Context(), input)
	if err != nil {
		if errors.Is(err, usecase.ErrVersionConflict) {
			WriteError(c, http.StatusConflict, "SYS_CONFIG_VERSION_CONFLICT",
				"設定値が他のユーザーによって更新されています。最新のバージョンを取得してください")
			return
		}
		WriteError(c, http.StatusInternalServerError, "SYS_CONFIG_INTERNAL_ERROR",
			"設定値の更新に失敗しました")
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"namespace":   output.Namespace,
		"key":         output.Key,
		"value":       output.Value,
		"version":     output.Version,
		"description": output.Description,
		"updated_by":  output.UpdatedBy,
		"updated_at":  output.UpdatedAt,
	})
}

// DeleteConfig は DELETE /api/v1/config/:namespace/:key のハンドラー。
func (h *ConfigHandler) DeleteConfig(c *gin.Context) {
	namespace := c.Param("namespace")
	key := c.Param("key")

	// TODO: 認証情報から取得する（現在はヘッダーから取得）
	deletedBy := c.GetHeader("X-User-Email")
	if deletedBy == "" {
		deletedBy = "unknown"
	}

	input := usecase.DeleteConfigInput{
		Namespace: namespace,
		Key:       key,
		DeletedBy: deletedBy,
	}

	err := h.deleteConfigUC.Execute(c.Request.Context(), input)
	if err != nil {
		if isNotFoundError(err) {
			WriteError(c, http.StatusNotFound, "SYS_CONFIG_KEY_NOT_FOUND",
				"指定された設定キーが見つかりません")
			return
		}
		WriteError(c, http.StatusInternalServerError, "SYS_CONFIG_INTERNAL_ERROR",
			"設定値の削除に失敗しました")
		return
	}

	c.Status(http.StatusNoContent)
}

// GetServiceConfig は GET /api/v1/config/services/:service_name のハンドラー。
func (h *ConfigHandler) GetServiceConfig(c *gin.Context) {
	serviceName := c.Param("service_name")

	input := usecase.GetServiceConfigInput{
		ServiceName: serviceName,
	}

	output, err := h.getServiceConfigUC.Execute(c.Request.Context(), input)
	if err != nil {
		WriteError(c, http.StatusNotFound, "SYS_CONFIG_SERVICE_NOT_FOUND",
			"指定されたサービスの設定が見つかりません")
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"service_name": output.ServiceName,
		"entries":      output.Entries,
	})
}

// RegisterRoutes はルートを登録する。
func (h *ConfigHandler) RegisterRoutes(r *gin.Engine) {
	v1 := r.Group("/api/v1")

	config := v1.Group("/config")
	{
		// サービス向け設定一括取得（:namespace より先にマッチさせる）
		config.GET("/services/:service_name", h.GetServiceConfig)

		// 設定値の CRUD
		config.GET("/:namespace/:key", h.GetConfig)
		config.GET("/:namespace", h.ListConfigs)
		config.PUT("/:namespace/:key", h.UpdateConfig)
		config.DELETE("/:namespace/:key", h.DeleteConfig)
	}
}

// isNotFoundError はエラーが「見つからない」系かどうかを判定する。
func isNotFoundError(err error) bool {
	if err == nil {
		return false
	}
	errMsg := err.Error()
	return contains(errMsg, "not found")
}

// contains は文字列に部分文字列が含まれるか判定する。
func contains(s, substr string) bool {
	return len(s) >= len(substr) && searchString(s, substr)
}

func searchString(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
