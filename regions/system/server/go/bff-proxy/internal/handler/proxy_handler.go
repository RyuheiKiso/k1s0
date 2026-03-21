package handler

import (
	"log/slog"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"golang.org/x/sync/singleflight"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/middleware"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/upstream"
)

// ProxyHandler provides reverse proxy functionality that converts
// cookie-based sessions to bearer token authentication for upstream APIs.
type ProxyHandler struct {
	reverseProxy *upstream.ReverseProxy
	sessionStore session.Store
	oauthClient  *oauth.Client
	sessionTTL   time.Duration
	logger       *slog.Logger
	// G-03 対応: トークンリフレッシュの重複実行を防止するための singleflight グループ。
	// 同一セッション ID に対して複数の並行リクエストが同時にリフレッシュを試みる場合、
	// 最初の 1 件のみ実際に RefreshToken を呼び出し、他は結果を共有する。
	refreshGroup singleflight.Group
}

// NewProxyHandler creates a new reverse proxy handler targeting the upstream URL.
func NewProxyHandler(
	upstreamURL string,
	sessionStore session.Store,
	oauthClient *oauth.Client,
	sessionTTL time.Duration,
	timeout time.Duration,
	logger *slog.Logger,
) (*ProxyHandler, error) {
	reverseProxy, err := upstream.NewReverseProxy(upstreamURL, timeout)
	if err != nil {
		return nil, err
	}

	return &ProxyHandler{
		reverseProxy: reverseProxy,
		sessionStore: sessionStore,
		oauthClient:  oauthClient,
		sessionTTL:   sessionTTL,
		logger:       logger,
	}, nil
}

// Handle proxies requests to the upstream API after attaching the bearer token
// from the session. If the access token has expired, it attempts a silent refresh.
func (h *ProxyHandler) Handle(c *gin.Context) {
	sess, ok := middleware.GetSessionData(c)
	if !ok {
		abortErrorWithMessage(c, http.StatusUnauthorized, "BFF_PROXY_NO_SESSION", "セッションが見つかりません")
		return
	}

	sessionID, _ := middleware.GetSessionID(c)

	// SessionMiddleware が session_needs_refresh フラグを立てた場合のみ silent refresh を試みる。
	// フラグは「期限切れ かつ refresh token あり」の場合のみ middleware が設定する。
	needsRefresh, _ := c.Get(middleware.SessionNeedsRefreshKey)
	if needsRefresh != nil && needsRefresh.(bool) {
		// G-03 対応: singleflight でセッション単位のリフレッシュ重複を排除する。
		// 同一 sessionID に対して並行リクエストが殺到した場合、1 件のみ実際にリフレッシュし
		// 残りは同じ結果を共有する。これにより RefreshToken のレート制限エラーを防ぐ。
		type refreshResult struct {
			tokenResp *oauth.TokenResponse
		}
		val, err, _ := h.refreshGroup.Do(sessionID, func() (any, error) {
			resp, e := h.oauthClient.RefreshToken(c.Request.Context(), sess.RefreshToken)
			if e != nil {
				return nil, e
			}
			return &refreshResult{tokenResp: resp}, nil
		})
		if err != nil {
			h.logger.Warn("token refresh failed, session expired",
				slog.String("error", err.Error()),
			)
			abortErrorWithMessage(c, http.StatusUnauthorized, "BFF_PROXY_TOKEN_EXPIRED", "セッションが期限切れです。再認証してください")
			return
		}

		tokenResp := val.(*refreshResult).tokenResp

		// Update session with new tokens.
		sess.AccessToken = tokenResp.AccessToken
		if tokenResp.RefreshToken != "" {
			sess.RefreshToken = tokenResp.RefreshToken
		}
		if tokenResp.IDToken != "" {
			sess.IDToken = tokenResp.IDToken
		}
		sess.ExpiresAt = time.Now().Add(time.Duration(tokenResp.ExpiresIn) * time.Second).Unix()

		if err := h.sessionStore.Update(c.Request.Context(), sessionID, sess, h.sessionTTL); err != nil {
			h.logger.Error("failed to update session after refresh",
				slog.String("error", err.Error()),
			)
		}
	}

	// Inject Authorization header for upstream.
	c.Request.Header.Set("Authorization", "Bearer "+sess.AccessToken)

	// 相関ヘッダーをアップストリームに伝播する
	if cid, ok := c.Get(middleware.CorrelationIDKey); ok {
		// 型アサーションの安全化: comma-ok パターンで string 型を確認する（M-3）
		if cidStr, ok := cid.(string); ok {
			c.Request.Header.Set(middleware.HeaderCorrelationID, cidStr)
		}
	}
	if tid, ok := c.Get(middleware.TraceIDKey); ok {
		// 型アサーションの安全化: comma-ok パターンで string 型を確認する（M-3）
		if tidStr, ok := tid.(string); ok {
			c.Request.Header.Set(middleware.HeaderTraceID, tidStr)
		}
	}

	// Strip session cookie from upstream request.
	c.Request.Header.Del("Cookie")

	h.reverseProxy.ServeHTTP(c.Writer, c.Request)
}
