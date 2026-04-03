package middleware

import (
	"crypto/subtle"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

const (
	// DefaultCSRFHeader は CSRF トークンに使用するデフォルトのヘッダー名。
	DefaultCSRFHeader = "X-CSRF-Token"

	// CSRFTokenTTL は CSRF トークンの有効期間（H-12 監査対応、MED-9 監査対応）。
	// セッション TTL のデフォルト値は 30 分（config.yaml の session.ttl で変更可能）。
	// CSRF トークンも同じく 30 分に固定し、セッション有効中でも定期的に再発行する。
	// これによりトークン窃取時の悪用可能時間を最小化する。
	CSRFTokenTTL = 30 * time.Minute
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

		// CSRF トークンの有効期間（30分 TTL）を検証する（H-12 監査対応）。
		// FE-MED-004: CSRFTokenCreatedAt が 0 の場合は旧バージョンセッションとして TTL チェックをスキップする
		// TODO: マイグレーション完了後（全セッションが CSRFTokenCreatedAt を持つようになった後）このフォールバックを削除する
		if sess.CSRFTokenCreatedAt > 0 {
			csrfAge := time.Since(time.Unix(sess.CSRFTokenCreatedAt, 0))
			if csrfAge > CSRFTokenTTL {
				c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
					"error":      "BFF_CSRF_EXPIRED",
					"message":    "CSRF token expired",
					"request_id": GetRequestID(c),
				})
				return
			}
		}

		c.Next()
	}
}
