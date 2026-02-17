package handler

import (
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-auth/internal/usecase"
)

// AuthHandler は認証関連の REST ハンドラー。
type AuthHandler struct {
	validateTokenUC  *usecase.ValidateTokenUseCase
	getUserUC        *usecase.GetUserUseCase
	listUsersUC      *usecase.ListUsersUseCase
	checkPermissionUC *usecase.CheckPermissionUseCase
}

// NewAuthHandler は新しい AuthHandler を作成する。
func NewAuthHandler(
	validateTokenUC *usecase.ValidateTokenUseCase,
	getUserUC *usecase.GetUserUseCase,
	listUsersUC *usecase.ListUsersUseCase,
	checkPermissionUC *usecase.CheckPermissionUseCase,
) *AuthHandler {
	return &AuthHandler{
		validateTokenUC:  validateTokenUC,
		getUserUC:        getUserUC,
		listUsersUC:      listUsersUC,
		checkPermissionUC: checkPermissionUC,
	}
}

// ValidateToken は POST /api/v1/auth/token/validate のハンドラー。
func (h *AuthHandler) ValidateToken(c *gin.Context) {
	var req struct {
		Token string `json:"token" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		WriteError(c, http.StatusBadRequest, "SYS_AUTH_VALIDATION_FAILED",
			"リクエストのバリデーションに失敗しました")
		return
	}

	claims, err := h.validateTokenUC.Execute(c.Request.Context(), req.Token)
	if err != nil {
		WriteError(c, http.StatusUnauthorized, "SYS_AUTH_TOKEN_INVALID",
			"トークンの検証に失敗しました")
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"valid":  true,
		"claims": claims,
	})
}

// IntrospectToken は POST /api/v1/auth/token/introspect のハンドラー。
func (h *AuthHandler) IntrospectToken(c *gin.Context) {
	var req struct {
		Token         string `json:"token" binding:"required"`
		TokenTypeHint string `json:"token_type_hint"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		WriteError(c, http.StatusBadRequest, "SYS_AUTH_VALIDATION_FAILED",
			"リクエストのバリデーションに失敗しました")
		return
	}

	claims, err := h.validateTokenUC.Execute(c.Request.Context(), req.Token)
	if err != nil {
		// RFC 7662: 無効なトークンでも 200 を返す
		c.JSON(http.StatusOK, gin.H{
			"active": false,
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"active":       true,
		"sub":          claims.Sub,
		"client_id":    claims.Azp,
		"username":     claims.PreferredUsername,
		"token_type":   claims.Typ,
		"exp":          claims.Exp,
		"iat":          claims.Iat,
		"scope":        claims.Scope,
		"realm_access": claims.RealmAccess,
	})
}

// GetUser は GET /api/v1/users/:id のハンドラー。
func (h *AuthHandler) GetUser(c *gin.Context) {
	userID := c.Param("id")

	user, err := h.getUserUC.Execute(c.Request.Context(), userID)
	if err != nil {
		WriteError(c, http.StatusNotFound, "SYS_AUTH_USER_NOT_FOUND",
			"指定されたユーザーが見つかりません")
		return
	}

	c.JSON(http.StatusOK, user)
}

// ListUsers は GET /api/v1/users のハンドラー。
func (h *AuthHandler) ListUsers(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	search := c.Query("search")

	input := usecase.ListUsersInput{
		Page:     page,
		PageSize: pageSize,
		Search:   search,
	}

	if enabledStr := c.Query("enabled"); enabledStr != "" {
		enabled := enabledStr == "true"
		input.Enabled = &enabled
	}

	output, err := h.listUsersUC.Execute(c.Request.Context(), input)
	if err != nil {
		WriteError(c, http.StatusInternalServerError, "SYS_AUTH_INTERNAL_ERROR",
			"ユーザー一覧の取得に失敗しました")
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"users": output.Users,
		"pagination": gin.H{
			"total_count": output.TotalCount,
			"page":        output.Page,
			"page_size":   output.PageSize,
			"has_next":    output.HasNext,
		},
	})
}

// GetUserRoles は GET /api/v1/users/:id/roles のハンドラー。
// RESTHandler に委譲する設計のためここではスタブ実装。
func (h *AuthHandler) GetUserRoles(c *gin.Context) {
	c.JSON(http.StatusNotImplemented, gin.H{
		"error": "not implemented",
	})
}

// CheckPermission は POST /api/v1/auth/permissions/check のハンドラー。
func (h *AuthHandler) CheckPermission(c *gin.Context) {
	var input usecase.CheckPermissionInput
	if err := c.ShouldBindJSON(&input); err != nil {
		WriteError(c, http.StatusBadRequest, "SYS_AUTH_VALIDATION_FAILED",
			"リクエストのバリデーションに失敗しました")
		return
	}
	output := h.checkPermissionUC.Execute(input)
	c.JSON(http.StatusOK, output)
}

// RegisterRoutes はルートを登録する。
func (h *AuthHandler) RegisterRoutes(r *gin.Engine) {
	v1 := r.Group("/api/v1")

	// 公開エンドポイント（認可不要）
	auth := v1.Group("/auth")
	{
		auth.POST("/token/validate", h.ValidateToken)
		auth.POST("/token/introspect", h.IntrospectToken)
		auth.POST("/permissions/check", h.CheckPermission)
	}

	// ユーザー管理
	users := v1.Group("/users")
	{
		users.GET("", h.ListUsers)
		users.GET("/:id", h.GetUser)
		users.GET("/:id/roles", h.GetUserRoles)
	}
}
