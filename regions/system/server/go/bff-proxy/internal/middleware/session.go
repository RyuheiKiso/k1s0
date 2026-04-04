package middleware

import (
	"log/slog"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"

	bffmetrics "github.com/k1s0-platform/system-server-go-bff-proxy/internal/metrics"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/util"
)

var (
	// sessionTouchFailuresTotal はスライディングウィンドウの TTL 延長失敗数を記録する（M-012）。
	// Redis 障害や接続断を検出するためのアラートに利用する。
	sessionTouchFailuresTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "bff_session_touch_failures_total",
		Help: "Total number of session TTL touch failures (sliding window).",
	})
)

const (
	// SessionDataKey is the gin context key where SessionData is stored.
	SessionDataKey = "bff_session"

	// SessionIDKey is the gin context key where the session ID is stored.
	SessionIDKey = "bff_session_id"

	// SessionNeedsRefreshKey は期限切れセッションで refresh token がある場合に
	// handler への silent refresh 指示として設定する gin context キー。
	SessionNeedsRefreshKey = "session_needs_refresh"
)

// SessionMiddleware extracts a session from the cookie, validates it,
// and stores it in the gin context. Optionally applies sliding TTL.
func SessionMiddleware(store session.Store, cookieName string, ttl time.Duration, sliding bool) gin.HandlerFunc {
	return func(c *gin.Context) {
		sessionID, err := c.Cookie(cookieName)
		if err != nil || sessionID == "" {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error":      "BFF_SESSION_MISSING",
				"message":    "Session cookie not found",
				"request_id": GetRequestID(c),
			})
			return
		}

		sess, err := store.Get(c.Request.Context(), sessionID)
		if err != nil || sess == nil {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error":      "BFF_SESSION_INVALID",
				"message":    "Session expired or invalid",
				"request_id": GetRequestID(c),
			})
			return
		}

		// M-17 監査対応: セッションの絶対有効期限を確認する。
		// スライディングウィンドウで TTL が延長され続けても、絶対有効期限を超えたセッションは強制無効化する。
		// refresh token があっても絶対期限超過の場合は再認証を要求する。
		if sess.IsAbsoluteExpired() {
			// セッション削除失敗はユーザーの認証には影響しないが、Redisのゴミデータ蓄積を防ぐためログに記録する
			if err := store.Delete(c.Request.Context(), sessionID); err != nil {
				slog.Warn("セッション削除に失敗しました", "session_id", util.MaskSessionID(sessionID), "error", err)
				// L-003 監査対応: 絶対期限切れセッションの削除失敗をメトリクスに記録する
				bffmetrics.SessionDeleteErrorsTotal.WithLabelValues("session_expired").Inc()
			}
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error":      "BFF_SESSION_ABSOLUTE_EXPIRED",
				"message":    "Session has exceeded maximum lifetime, re-authentication required",
				"request_id": GetRequestID(c),
			})
			return
		}

		// アクセストークンの有効期限を確認する。
		// refresh token がある場合は handler 側で silent refresh を試みるため、
		// フラグを立てて handler に通す。refresh token がない場合は即 401 を返す。
		if sess.IsExpired() {
			if sess.RefreshToken == "" {
				c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
					"error":      "BFF_SESSION_EXPIRED",
					"message":    "Session token has expired",
					"request_id": GetRequestID(c),
				})
				return
			}
			// refresh token がある場合は handler に refresh 可能フラグを伝える
			c.Set(SessionNeedsRefreshKey, true)
		}

		c.Set(SessionDataKey, sess)
		c.Set(SessionIDKey, sessionID)

		// スライディングウィンドウ: リクエストごとに TTL を延長する
		if sliding && ttl > 0 {
			if err := store.Touch(c.Request.Context(), sessionID, ttl); err != nil {
				// L-5 監査対応: セッション ID は先頭 8 文字のみログに出力してマスクする
				slog.Warn("セッション TTL 延長に失敗", "session_id", util.MaskSessionID(sessionID), "error", err)
				// Touch 失敗をメトリクスに記録する（M-012）
				// 高頻度で発生する場合は Redis 障害を示す可能性があるため、アラート設定を推奨する
				sessionTouchFailuresTotal.Inc()
			}
		}

		c.Next()
	}
}

// GetSessionData retrieves SessionData from the gin context.
func GetSessionData(c *gin.Context) (*session.SessionData, bool) {
	val, exists := c.Get(SessionDataKey)
	if !exists {
		return nil, false
	}
	sess, ok := val.(*session.SessionData)
	return sess, ok
}

// GetSessionID retrieves the session ID from the gin context.
func GetSessionID(c *gin.Context) (string, bool) {
	val, exists := c.Get(SessionIDKey)
	if !exists {
		return "", false
	}
	id, ok := val.(string)
	return id, ok
}

