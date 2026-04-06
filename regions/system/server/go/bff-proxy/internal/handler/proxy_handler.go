// proxy_handler.go はクッキーベースのセッションをベアラートークン認証に変換し、
// アップストリーム API へのリバースプロキシ機能を提供する HTTP ハンドラー。
// ビジネスロジック（セッション検証・トークンリフレッシュ）は ProxyUseCase に委譲し、
// このハンドラーは HTTP コンテキストの操作と上流転送のみを担当する。
package handler

import (
	"log/slog"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/middleware"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/port"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/upstream"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/usecase"
)

// ProxyHandler はセッション検証とアップストリーム転送を担う HTTP ハンドラー。
// ビジネスロジックは ProxyUseCase に委譲し、ハンドラーは HTTP 変換のみを担当する。
type ProxyHandler struct {
	// reverseProxy はアップストリーム API へのリクエスト転送を担う。
	reverseProxy *upstream.ReverseProxy
	// proxyUseCase はセッション検証・トークンリフレッシュのビジネスロジックを提供する。
	proxyUseCase *usecase.ProxyUseCase
}

// NewProxyHandler はアップストリーム URL を対象とした新しいリバースプロキシハンドラーを生成する。
// oauthClient は port.OAuthClient インターフェース型で受け取り、nil も許容する（リフレッシュ不要時）。
// allowedHosts は設定ファイル由来の静的アップストリームホスト名のセット。
// nil を渡した場合はすべてのターゲットに SSRF チェックを適用する。
func NewProxyHandler(
	upstreamURL string,
	sessionStore port.SessionStore,
	oauthClient port.OAuthClient,
	sessionTTL time.Duration,
	timeout time.Duration,
	logger *slog.Logger,
	allowedHosts map[string]bool,
) (*ProxyHandler, error) {
	// allowedHosts を渡すことで、設定ファイル由来の静的アップストリームは SSRF チェックをバイパスする
	reverseProxy, err := upstream.NewReverseProxy(upstreamURL, timeout, allowedHosts)
	if err != nil {
		return nil, err
	}

	// ProxyUseCase にビジネスロジック（セッション検証・リフレッシュ）を委譲する
	proxyUseCase := usecase.NewProxyUseCase(oauthClient, sessionStore, sessionTTL, logger)

	return &ProxyHandler{
		reverseProxy: reverseProxy,
		proxyUseCase: proxyUseCase,
	}, nil
}

// Handle はセッションからベアラートークンを付加してアップストリーム API へリクエストをプロキシする。
// アクセストークンが期限切れの場合は ProxyUseCase 経由でサイレントリフレッシュを試みる。
func (h *ProxyHandler) Handle(c *gin.Context) {
	sess, ok := middleware.GetSessionData(c)
	if !ok {
		abortErrorWithMessage(c, http.StatusUnauthorized, "BFF_PROXY_NO_SESSION", "セッションが見つかりません")
		return
	}

	// GetSessionID はセッション ID を gin コンテキストから取得する。
	// 取得失敗時（SessionMiddleware がセットしていない場合）は空文字列になり
	// singleflight キーが衝突するため、明示的に 401 を返して処理を中断する。
	sessionID, ok := middleware.GetSessionID(c)
	if !ok || sessionID == "" {
		abortErrorWithMessage(c, http.StatusUnauthorized, "BFF_PROXY_NO_SESSION_ID", "セッション ID が取得できません")
		return
	}

	// SessionMiddleware が session_needs_refresh フラグを立てた場合のみ silent refresh を試みる。
	// フラグは「期限切れ かつ refresh token あり」の場合のみ middleware が設定する。
	needsRefresh, _ := c.Get(middleware.SessionNeedsRefreshKey)
	// 型アサーションの安全化: comma-ok パターンで bool 型を確認する（H-009）
	refresh, _ := needsRefresh.(bool)

	// ProxyUseCase にセッション検証・リフレッシュのビジネスロジックを委譲する
	out, err := h.proxyUseCase.PrepareProxy(c.Request.Context(), usecase.PrepareProxyInput{
		SessionData:  sess,
		SessionID:    sessionID,
		NeedsRefresh: refresh,
	})
	if err != nil {
		abortErrorWithMessage(c, http.StatusUnauthorized, "BFF_PROXY_TOKEN_EXPIRED", "セッションが期限切れです。再認証してください")
		return
	}

	// アップストリーム向けに Authorization ヘッダーを付加する
	c.Request.Header.Set("Authorization", "Bearer "+out.AccessToken)

	// MEDIUM-GO-001 監査対応: セッションから tenant_id を取得し、X-Tenant-ID ヘッダーとして上流に転送する。
	// テナント分離を実現するため、各マイクロサービスはこのヘッダーを使用してリクエストをフィルタリングする。
	// TenantID が空の場合（Keycloak に tenant_id クレームが未設定の場合）はヘッダーを設定しない。
	if sess.TenantID != "" {
		c.Request.Header.Set("X-Tenant-ID", sess.TenantID)
	}

	// H-10 監査対応: トークンリフレッシュが発生した場合、新しい CSRF トークンをレスポンスヘッダーに設定する。
	// クライアントはこのヘッダーを検出して保持している CSRF トークンを更新する必要がある。
	if out.TokenRefreshed && out.CSRFToken != "" {
		c.Header("X-CSRF-Token", out.CSRFToken)
	}

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

	// アップストリームへのリクエストからセッション Cookie を除去する
	c.Request.Header.Del("Cookie")

	h.reverseProxy.ServeHTTP(c.Writer, c.Request)
}
