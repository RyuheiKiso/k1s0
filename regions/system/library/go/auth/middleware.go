package auth

import (
	"context"
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
)

// contextKey はコンテキストに Claims を格納するためのキー型。
type contextKey string

const (
	// ClaimsContextKey はコンテキストに格納する Claims のキー。
	ClaimsContextKey contextKey = "auth_claims"
)

// AuthMiddleware は gin 用の JWT 認証ミドルウェアを返す。
// Authorization ヘッダーから Bearer トークンを取得し、JWKS 検証を行う。
// 検証成功時は Claims をコンテキストに格納する。
func AuthMiddleware(verifier *JWKSVerifier) gin.HandlerFunc {
	return func(c *gin.Context) {
		tokenString, err := extractBearerToken(c.Request)
		if err != nil {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error":   "SYS_AUTH_UNAUTHENTICATED",
				"message": "認証が必要です",
			})
			return
		}

		claims, err := verifier.VerifyToken(c.Request.Context(), tokenString)
		if err != nil {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error":   "SYS_AUTH_INVALID_TOKEN",
				"message": "トークンが無効です",
			})
			return
		}

		// Claims をコンテキストに格納
		ctx := context.WithValue(c.Request.Context(), ClaimsContextKey, claims)
		c.Request = c.Request.WithContext(ctx)
		c.Set(string(ClaimsContextKey), claims)

		c.Next()
	}
}

// RequireRole は指定ロールを必須とする gin ミドルウェアを返す。
// AuthMiddleware の後に使用すること。
func RequireRole(role string) gin.HandlerFunc {
	return func(c *gin.Context) {
		claims, ok := GetClaims(c)
		if !ok {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error":   "SYS_AUTH_UNAUTHENTICATED",
				"message": "認証が必要です",
			})
			return
		}

		if !HasRole(claims, role) {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
				"error":   "SYS_AUTH_FORBIDDEN",
				"message": "この操作を実行する権限がありません",
			})
			return
		}

		c.Next()
	}
}

// RequirePermission は指定リソース・アクションの権限を必須とする gin ミドルウェアを返す。
// AuthMiddleware の後に使用すること。
func RequirePermission(resource, action string) gin.HandlerFunc {
	return func(c *gin.Context) {
		claims, ok := GetClaims(c)
		if !ok {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error":   "SYS_AUTH_UNAUTHENTICATED",
				"message": "認証が必要です",
			})
			return
		}

		if !HasPermission(claims, resource, action) {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
				"error":   "SYS_AUTH_FORBIDDEN",
				"message": "この操作を実行する権限がありません",
			})
			return
		}

		c.Next()
	}
}

// RequireTierAccess は指定 Tier へのアクセスを必須とする gin ミドルウェアを返す。
// AuthMiddleware の後に使用すること。
func RequireTierAccess(tier string) gin.HandlerFunc {
	return func(c *gin.Context) {
		claims, ok := GetClaims(c)
		if !ok {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error":   "SYS_AUTH_UNAUTHENTICATED",
				"message": "認証が必要です",
			})
			return
		}

		if !HasTierAccess(claims, tier) {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
				"error":   "SYS_AUTH_TIER_FORBIDDEN",
				"message": "このTierへのアクセス権がありません",
			})
			return
		}

		c.Next()
	}
}

// GetClaims は gin.Context から Claims を取得する。
func GetClaims(c *gin.Context) (*Claims, bool) {
	val, exists := c.Get(string(ClaimsContextKey))
	if !exists {
		return nil, false
	}
	claims, ok := val.(*Claims)
	return claims, ok
}

// GetClaimsFromContext は context.Context から Claims を取得する。
func GetClaimsFromContext(ctx context.Context) (*Claims, bool) {
	val := ctx.Value(ClaimsContextKey)
	if val == nil {
		return nil, false
	}
	claims, ok := val.(*Claims)
	return claims, ok
}

// extractBearerToken は HTTP リクエストから Bearer トークンを取得する。
func extractBearerToken(r *http.Request) (string, error) {
	authHeader := r.Header.Get("Authorization")
	if authHeader == "" {
		return "", ErrMissingToken
	}

	parts := strings.SplitN(authHeader, " ", 2)
	if len(parts) != 2 || !strings.EqualFold(parts[0], "Bearer") {
		return "", ErrInvalidAuthHeader
	}

	token := strings.TrimSpace(parts[1])
	if token == "" {
		return "", ErrMissingToken
	}

	return token, nil
}

// エラー定義。
var (
	ErrMissingToken      = &AuthError{Code: "SYS_AUTH_MISSING_TOKEN", Message: "Authorization ヘッダーがありません"}
	ErrInvalidAuthHeader = &AuthError{Code: "SYS_AUTH_INVALID_HEADER", Message: "Authorization ヘッダーの形式が不正です"}
)

// AuthError は認証エラーを表す。
type AuthError struct {
	Code    string
	Message string
}

func (e *AuthError) Error() string {
	return e.Message
}
