package handler

import (
	"log/slog"
	"net/http"
	"net/http/httputil"
	"net/url"
	"time"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/middleware"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

// ProxyHandler provides reverse proxy functionality that converts
// cookie-based sessions to bearer token authentication for upstream APIs.
type ProxyHandler struct {
	upstream     *url.URL
	proxy        *httputil.ReverseProxy
	sessionStore session.Store
	oauthClient  *oauth.Client
	sessionTTL   time.Duration
	logger       *slog.Logger
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
	target, err := url.Parse(upstreamURL)
	if err != nil {
		return nil, err
	}

	proxy := httputil.NewSingleHostReverseProxy(target)
	proxy.Transport = &http.Transport{
		ResponseHeaderTimeout: timeout,
	}

	return &ProxyHandler{
		upstream:     target,
		proxy:        proxy,
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
		c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
			"error":   "BFF_PROXY_NO_SESSION",
			"message": "Session not found",
		})
		return
	}

	sessionID, _ := middleware.GetSessionID(c)

	// Try silent token refresh if expired.
	if sess.IsExpired() && sess.RefreshToken != "" {
		tokenResp, err := h.oauthClient.RefreshToken(c.Request.Context(), sess.RefreshToken)
		if err != nil {
			h.logger.Warn("token refresh failed, session expired",
				slog.String("error", err.Error()),
			)
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error":   "BFF_PROXY_TOKEN_EXPIRED",
				"message": "Session expired, please re-authenticate",
			})
			return
		}

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

	// Propagate correlation headers.
	if cid, ok := c.Get(middleware.CorrelationIDKey); ok {
		c.Request.Header.Set(middleware.HeaderCorrelationID, cid.(string))
	}
	if tid, ok := c.Get(middleware.TraceIDKey); ok {
		c.Request.Header.Set(middleware.HeaderTraceID, tid.(string))
	}

	// Strip session cookie from upstream request.
	c.Request.Header.Del("Cookie")

	h.proxy.ServeHTTP(c.Writer, c.Request)
}
