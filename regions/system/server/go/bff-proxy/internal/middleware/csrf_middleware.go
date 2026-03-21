package middleware

import (
	"crypto/subtle"
	"net/http"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

const (
	// DefaultCSRFHeader は CSRF トークンに使用するデフォルトのヘッダー名。
	DefaultCSRFHeader = "X-CSRF-Token"
)

// CSRFMiddleware はリクエストヘッダーの CSRF トークンをセッションに紐付いたトークンと照合する。
// 状態変更メソッド（POST, PUT, PATCH, DELETE）のみに適用される。
// SessionMiddleware がチェーン上で先に実行されている場合は gin.Context からセッションを取得し、
// 冗長なストアへの問い合わせを回避する。コンテキストにセッションがない場合はフォールバックとして
// ストアから直接取得する。
func CSRFMiddleware(store session.Store, headerName string, sessionCookie string) gin.HandlerFunc {
	if headerName == "" {
		headerName = DefaultCSRFHeader
	}

	return func(c *gin.Context) {
		// 安全なメソッド（GET, HEAD, OPTIONS）は CSRF チェックを免除する。
		if c.Request.Method == http.MethodGet ||
			c.Request.Method == http.MethodHead ||
			c.Request.Method == http.MethodOptions {
			c.Next()
			return
		}

		// SessionMiddleware がセットしたセッションをコンテキストから取得する。
		// これにより、SessionMiddleware の後に実行される場合は冗長な store.Get() を回避できる。
		sess, ok := GetSessionData(c)
		if !ok {
			// フォールバック: SessionMiddleware が未実行の場合はストアから直接取得する。
			sessionID, err := c.Cookie(sessionCookie)
			if err != nil || sessionID == "" {
				c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
					"error":      "BFF_CSRF_NO_SESSION",
					"message":    "Session not found",
					"request_id": GetRequestID(c),
				})
				return
			}

			sess, err = store.Get(c.Request.Context(), sessionID)
			if err != nil || sess == nil {
				c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
					"error":      "BFF_CSRF_INVALID_SESSION",
					"message":    "Invalid session",
					"request_id": GetRequestID(c),
				})
				return
			}
		}

		csrfHeader := c.GetHeader(headerName)
		// タイミング攻撃を防止するため、定数時間比較を使用する。
		// 通常の文字列比較（==）は一致しない最初の文字で早期リターンするため、
		// 応答時間の差からトークンの内容を推測される可能性がある。
		if csrfHeader == "" || subtle.ConstantTimeCompare([]byte(csrfHeader), []byte(sess.CSRFToken)) != 1 {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
				"error":      "BFF_CSRF_MISMATCH",
				"message":    "CSRF token mismatch",
				"request_id": GetRequestID(c),
			})
			return
		}

		c.Next()
	}
}
